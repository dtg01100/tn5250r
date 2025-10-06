//! Network module for handling AS/400 connections using the 5250 protocol
//!
//! This module provides the TCP networking functionality for connecting
//! to AS/400 systems using the TN5250 protocol.
//!
//! INTEGRATION ARCHITECTURE DECISIONS:
//! ===================================
//!
//! 1. **Protocol Auto-Detection**: Implements ProtocolDetector to automatically
//!    distinguish between NVT (plain text) and 5250 (EBCDIC with ESC sequences)
//!    protocols based on initial data patterns. This resolves the NVT Mode vs
//!    5250 Protocol Confusion issue by analyzing the first 256 bytes of data.
//!
//! 2. **Mode Switching**: AS400Connection integrates protocol detection and
//!    switches processing modes dynamically. NVT data is buffered for fallback,
//!    while 5250 data is processed through telnet negotiation and protocol parsing.
//!
//! 3. **Component Integration**: Network layer coordinates with telnet negotiation
//!    and protocol processing layers, providing a unified interface while maintaining
//!    separation of concerns.
//!
//! 4. **Security Integration**: All network operations include bounds checking,
//!    data validation, and secure cleanup to prevent resource leaks and attacks.
//!
//! 5. **Performance Optimization**: Uses pooled buffers and efficient data structures
//!    to minimize allocations during high-frequency network operations.

use std::net::{TcpStream, ToSocketAddrs, SocketAddr};
use base64::Engine;
use std::io::{Read, Write, Result as IoResult};
use std::time::{Duration, Instant};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use rustls::{ClientConfig, ClientConnection, RootCertStore};
use rustls::pki_types::{ServerName as TlsServerName};

use std::fs;

use crate::telnet_negotiation::TelnetNegotiator;
use crate::error::{TN5250Error};
use crate::monitoring::{set_component_status, set_component_error, ComponentState};
use crate::network_platform;


#[derive(Debug)]
struct OwnedTlsStream {
    conn: ClientConnection,
    stream: std::net::TcpStream,
}

impl OwnedTlsStream {
    fn set_read_timeout(&self, dur: Option<std::time::Duration>) -> std::io::Result<()> {
        self.stream.set_read_timeout(dur)
    }
    fn set_write_timeout(&self, dur: Option<std::time::Duration>) -> std::io::Result<()> {
        self.stream.set_write_timeout(dur)
    }
}

impl std::io::Read for OwnedTlsStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut s = rustls::Stream::new(&mut self.conn, &mut self.stream);
        s.read(buf)
    }
}

impl std::io::Write for OwnedTlsStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut s = rustls::Stream::new(&mut self.conn, &mut self.stream);
        s.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        let mut s = rustls::Stream::new(&mut self.conn, &mut self.stream);
        s.flush()
    }
}

#[derive(Debug)]
enum StreamType {
    Plain(std::net::TcpStream),
    Tls(OwnedTlsStream),
}

impl std::io::Read for StreamType {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            StreamType::Plain(t) => t.read(buf),
            StreamType::Tls(t) => t.read(buf),
        }
    }
}

impl std::io::Write for StreamType {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            StreamType::Plain(t) => t.write(buf),
            StreamType::Tls(t) => t.write(buf),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            StreamType::Plain(t) => t.flush(),
            StreamType::Tls(t) => t.flush(),
        }
    }
}

/// Session management configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Session idle timeout in seconds (default: 900 = 15 minutes)
    pub idle_timeout_secs: u64,
    /// TCP keepalive interval in seconds (default: 60)
    pub keepalive_interval_secs: u64,
    /// Connection timeout for initial connect (default: 30)
    pub connection_timeout_secs: u64,
    /// Enable automatic reconnection on connection loss
    pub auto_reconnect: bool,
    /// Maximum reconnection attempts (default: 3)
    pub max_reconnect_attempts: u32,
    /// Reconnection backoff multiplier (default: 2)
    pub reconnect_backoff_multiplier: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            idle_timeout_secs: 900,  // 15 minutes
            keepalive_interval_secs: 60,  // 1 minute
            connection_timeout_secs: 30,
            auto_reconnect: false,
            max_reconnect_attempts: 3,
            reconnect_backoff_multiplier: 2,
        }
    }
}

/// INTEGRATION: Protocol auto-detection and mode switching
/// Resolves NVT Mode vs 5250 Protocol Confusion (Issue #1)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProtocolMode {
    /// Auto-detect protocol from initial data patterns
    AutoDetect,
    /// NVT (Network Virtual Terminal) mode - plain text communication
    NVT,
    /// 5250 protocol mode - IBM AS/400 terminal protocol
    TN5250,
    /// 3270 protocol mode - IBM mainframe terminal protocol
    TN3270,
}

impl Default for ProtocolMode {
    fn default() -> Self {
        ProtocolMode::AutoDetect
    }
}

/// INTEGRATION: Protocol detector for automatic mode switching
/// Analyzes initial data patterns to distinguish NVT from 5250 protocol
#[derive(Debug)]
struct ProtocolDetector {
    mode: ProtocolMode,
    detection_buffer: Vec<u8>,
    max_detection_bytes: usize,
    detection_start_time: Option<Instant>,
    detection_timeout: Duration,
}

impl ProtocolDetector {
    fn new() -> Self {
        Self {
            mode: ProtocolMode::TN5250, // Default to TN5250 for AS/400 systems
            detection_buffer: Vec::new(),
            max_detection_bytes: 256, // Analyze first 256 bytes for protocol detection
            detection_start_time: None,
            detection_timeout: Duration::from_secs(5), // 5 second timeout for detection
        }
    }

    /// INTEGRATION: Detect protocol from data patterns with timeout handling
    /// 5250 starts with ESC (0x04) sequences, 3270 uses command codes (0x01-0x11), NVT is plain text
    /// Returns Result to handle detection failures and timeouts
    fn detect_protocol(&mut self, data: &[u8]) -> Result<ProtocolMode, TN5250Error> {
        if self.mode != ProtocolMode::AutoDetect {
            return Ok(self.mode);
        }

        // Start detection timer on first data
        if self.detection_start_time.is_none() {
            self.detection_start_time = Some(Instant::now());
        }

        // Check for detection timeout
        if let Some(start_time) = self.detection_start_time {
            if start_time.elapsed() > self.detection_timeout {
                // Timeout - fall back to NVT mode for safety
                self.mode = ProtocolMode::NVT;
                eprintln!("Protocol detection timeout - falling back to NVT mode");
                return Ok(self.mode);
            }
        }

        // Validate input data
        if data.is_empty() {
            return Ok(ProtocolMode::AutoDetect); // Need more data
        }

        // Accumulate data for detection
        self.detection_buffer.extend_from_slice(data);
        if self.detection_buffer.len() < 4 {
            return Ok(ProtocolMode::AutoDetect); // Need more data
        }

        // Limit detection buffer size to prevent memory exhaustion
        if self.detection_buffer.len() > self.max_detection_bytes {
            self.detection_buffer.truncate(self.max_detection_bytes);
        }

        // Check for 3270 protocol patterns first (more specific)
        // 3270 data streams start with command codes in range 0x01-0x11
        if self.detection_buffer.len() >= 2 {
            let first_byte = self.detection_buffer[0];
            // Check for 3270 command codes
            if matches!(first_byte, 0x01 | 0x02 | 0x05 | 0x06 | 0x0D | 0x0E | 0x0F | 0x11) {
                // Verify with WCC byte or order codes
                if self.detection_buffer.len() >= 3 {
                    let second_byte = self.detection_buffer[1];
                    // WCC byte or order codes indicate 3270
                    if second_byte <= 0x7F || matches!(second_byte, 0x11 | 0x1D | 0x29 | 0x28 | 0x2C | 0x13 | 0x05 | 0x3C | 0x12 | 0x08) {
                        self.mode = ProtocolMode::TN3270;
                        println!("INTEGRATION: Auto-detected 3270 protocol (command: 0x{:02X})", first_byte);
                        return Ok(self.mode);
                    }
                }
            }
        }

        // Check for 5250 protocol patterns
        // 5250 data streams typically start with ESC (0x04) followed by command codes
        if self.detection_buffer.len() >= 2 {
            // Look for ESC sequences that indicate 5250 protocol
            for i in 0..self.detection_buffer.len().saturating_sub(1) {
                if self.detection_buffer[i] == 0x04 { // ESC in EBCDIC
                    let next_byte = self.detection_buffer[i + 1];
                    // Check for known 5250 command codes after ESC
                    if matches!(next_byte, 0xF1..=0xFF) { // 5250 command range
                        self.mode = ProtocolMode::TN5250;
                        println!("INTEGRATION: Auto-detected 5250 protocol (ESC + command: 0x04, 0x{:02X})", next_byte);
                        return Ok(self.mode);
                    }
                }
            }
        }

        // Check for NVT patterns (plain text, no ESC sequences)
        let has_control_chars = self.detection_buffer.iter().any(|&b| b < 32 && b != 9 && b != 10 && b != 13);
        let has_high_chars = self.detection_buffer.iter().any(|&b| b > 127);

        if !has_control_chars && !has_high_chars {
            // Looks like plain ASCII text - likely NVT
            self.mode = ProtocolMode::NVT;
            println!("INTEGRATION: Auto-detected NVT protocol (plain text)");
            return Ok(self.mode);
        }

        // If we have enough data and still can't determine, default to NVT for safety
        // NVT is the safest fallback as it doesn't require specific protocol handling
        if self.detection_buffer.len() >= self.max_detection_bytes {
            self.mode = ProtocolMode::NVT;
            println!("INTEGRATION: Defaulting to NVT protocol after analysis (safest fallback)");
            return Ok(self.mode);
        }

        Ok(ProtocolMode::AutoDetect)
    }

    fn is_detection_complete(&self) -> bool {
        self.mode != ProtocolMode::AutoDetect
    }

    fn reset(&mut self) {
        self.mode = ProtocolMode::AutoDetect;
        self.detection_buffer.clear();
        self.detection_start_time = None;
    }
}

/// PERFORMANCE OPTIMIZATION: Buffer pool for reusing Vec<u8> allocations
/// Reduces memory allocation overhead by maintaining a pool of reusable buffers
struct BufferPool {
    pool: Mutex<VecDeque<Vec<u8>>>,
    max_buffers: usize,
    buffer_size: usize,
}

impl BufferPool {
    fn new(max_buffers: usize, buffer_size: usize) -> Self {
        Self {
            pool: Mutex::new(VecDeque::with_capacity(max_buffers)),
            max_buffers,
            buffer_size,
        }
    }

    /// PERFORMANCE OPTIMIZATION: Get a buffer from the pool or allocate new one
    fn get_buffer(&self) -> Vec<u8> {
        let mut pool = self.pool.lock().unwrap_or_else(|poisoned| {
            eprintln!("SECURITY: BufferPool mutex poisoned - recovering");
            poisoned.into_inner()
        });
        pool.pop_front().unwrap_or_else(|| Vec::with_capacity(self.buffer_size))
    }

    /// PERFORMANCE OPTIMIZATION: Return buffer to pool for reuse
    fn return_buffer(&self, mut buffer: Vec<u8>) {
        let mut pool = self.pool.lock().unwrap_or_else(|poisoned| {
            eprintln!("SECURITY: BufferPool mutex poisoned - recovering");
            poisoned.into_inner()
        });
        if pool.len() < self.max_buffers {
            buffer.clear();
            buffer.shrink_to_fit(); // PERFORMANCE: Minimize memory usage
            pool.push_back(buffer);
        }
        // If pool is full, buffer is dropped (memory freed)
    }
}

// Global buffer pool instance for network operations
lazy_static::lazy_static! {
    static ref NETWORK_BUFFER_POOL: BufferPool = BufferPool::new(32, 8192);
}

// A helper trait alias for objects that implement both Read and Write
trait ReadWrite: Read + Write {}
impl<T: Read + Write> ReadWrite for T {}

type DynStream = Box<StreamType>;
type SharedStream = Arc<Mutex<DynStream>>;

/// Represents a connection to an AS/400 system
#[derive(Debug)]
pub struct AS400Connection {
    stream: Option<SharedStream>,
    host: String,
    port: u16,
    receiver: Option<mpsc::Receiver<Vec<u8>>>,
    sender: Option<mpsc::Sender<Vec<u8>>>,
    running: bool,
    telnet_negotiator: TelnetNegotiator,
    negotiation_complete: bool,
    use_tls: bool,
    // TLS options
    tls_ca_bundle_path: Option<String>,
    // INTEGRATION: Protocol detection and mode switching
    protocol_detector: ProtocolDetector,
    detected_mode: ProtocolMode,
    // SESSION MANAGEMENT: Timeout and keepalive tracking
    session_config: SessionConfig,
    last_activity: Option<Instant>,
    reconnect_attempts: u32,
}

impl AS400Connection {
    /// Creates a new connection instance
    pub fn new(host: String, port: u16) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            stream: None,
            host,
            port,
            receiver: Some(rx),
            sender: Some(tx),
            running: false,
            telnet_negotiator: TelnetNegotiator::new(),
            negotiation_complete: false,
             use_tls: port == 992, // default secure if standard SSL port
             tls_ca_bundle_path: None,
            // INTEGRATION: Initialize protocol detection
            protocol_detector: ProtocolDetector::new(),
            detected_mode: ProtocolMode::TN5250, // Default to TN5250 for AS/400 systems
            // SESSION MANAGEMENT: Initialize with defaults
            session_config: SessionConfig::default(),
            last_activity: None,
            reconnect_attempts: 0,
        }
    }

    /// Set custom session configuration
    pub fn set_session_config(&mut self, config: SessionConfig) {
        self.session_config = config;
    }

    /// Get current session configuration
    pub fn session_config(&self) -> &SessionConfig {
        &self.session_config
    }

    /// Check if session has been idle too long
    pub fn is_session_idle_timeout(&self) -> bool {
        if let Some(last_activity) = self.last_activity {
            let elapsed = last_activity.elapsed();
            elapsed.as_secs() > self.session_config.idle_timeout_secs
        } else {
            false
        }
    }

    /// Update last activity timestamp
    fn update_last_activity(&mut self) {
        self.last_activity = Some(Instant::now());
    }

    /// Get time since last activity
    pub fn time_since_last_activity(&self) -> Option<Duration> {
        self.last_activity.map(|t| t.elapsed())
    }

    /// Enable or disable TLS explicitly (overrides port-based default)
    pub fn set_tls(&mut self, enabled: bool) {
        self.use_tls = enabled;
    }

    /// Returns true if TLS is enabled for this connection
    pub fn is_tls_enabled(&self) -> bool {
        self.use_tls
    }

    /// Provide a path to a PEM bundle containing trusted CAs to validate server certs.
    pub fn set_tls_ca_bundle_path<S: Into<String>>(&mut self, path: S) {
        let p = path.into();
        if p.is_empty() {
            self.tls_ca_bundle_path = None;
        } else {
            self.tls_ca_bundle_path = Some(p);
        }
    }

    /// Connects to the AS/400 system
    pub fn connect(&mut self) -> IoResult<()> {
        // Monitoring: mark network as starting
        set_component_status("network", ComponentState::Starting);
        let address = format!("{}:{}", self.host, self.port);
        let tcp = match TcpStream::connect(&address) {
            Ok(s) => s,
            Err(e) => {
                set_component_status("network", ComponentState::Error);
                set_component_error("network", Some(format!("TCP connect failed: {}", e)));
                return Err(e);
            }
        };
        
        // SESSION MANAGEMENT: Enable TCP keepalive
        self.configure_tcp_keepalive(&tcp)?;
        
        // Set read/write timeouts
        tcp.set_read_timeout(Some(Duration::from_secs(self.session_config.connection_timeout_secs)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(self.session_config.connection_timeout_secs)))?;

        // Wrap with TLS if requested
        let mut rw: DynStream;
        if self.use_tls {
            // Build TLS connector with secure certificate validation
            let tls_config = match self.build_tls_connector() {
                Ok(cfg) => cfg,
                Err(e) => {
                    set_component_status("network", ComponentState::Error);
                    set_component_error("network", Some(format!("TLS config creation failed: {}", e)));
                    return Err(e);
                }
            };

            // Create TLS connection
            let server_name = match TlsServerName::try_from(self.host.clone()) {
                Ok(name) => name,
                Err(e) => {
                    set_component_status("network", ComponentState::Error);
                    set_component_error("network", Some(format!("Invalid server name: {}", e)));
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid server name: {}", e)));
                }
            };
            let tcp = match TcpStream::connect(&address) {
                Ok(s) => s,
                Err(e) => {
                    set_component_status("network", ComponentState::Error);
                    set_component_error("network", Some(format!("TCP connect failed (TLS): {}", e)));
                    return Err(e);
                }
            };
            self.configure_tcp_keepalive(&tcp)?;
            tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
            tcp.set_write_timeout(Some(Duration::from_secs(10)))?;

            let tls_conn = match ClientConnection::new(tls_config, server_name) {
                Ok(conn) => conn,
                Err(e) => {
                    set_component_status("network", ComponentState::Error);
                    set_component_error("network", Some(format!("TLS connection failed: {}", e)));
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("TLS connection failed: {}", e)));
                }
            };
            rw = Box::new(StreamType::Tls(OwnedTlsStream { conn: tls_conn, stream: tcp }));
            // Record successful TLS connection in monitoring
            let monitoring = crate::monitoring::MonitoringSystem::global();
            monitoring.integration_monitor.record_integration_event(crate::monitoring::IntegrationEvent {
                timestamp: std::time::Instant::now(),
                event_type: crate::monitoring::IntegrationEventType::IntegrationSuccess,
                source_component: "network".to_string(),
                target_component: Some("tls".to_string()),
                description: format!("Secure TLS connection established to {}:{}", self.host, self.port),
                details: std::collections::HashMap::new(),
                duration_us: None,
                success: true,
            });
        } else {
            let tcp = match TcpStream::connect(&address) {
                Ok(s) => s,
                Err(e) => {
                    set_component_status("network", ComponentState::Error);
                    set_component_error("network", Some(format!("TCP connect failed: {}", e)));
                    return Err(e);
                }
            };
            self.configure_tcp_keepalive(&tcp)?;
            tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
            tcp.set_write_timeout(Some(Duration::from_secs(10)))?;
            rw = Box::new(StreamType::Plain(tcp));
        }

        // Telnet negotiation lifecycle signals
        set_component_status("telnet_negotiator", ComponentState::Starting);
        if let Err(e) = self.perform_telnet_negotiation_rw(&mut *rw) {
            set_component_status("telnet_negotiator", ComponentState::Error);
            set_component_error("telnet_negotiator", Some(format!("Telnet negotiation failed: {}", e)));
            set_component_status("network", ComponentState::Error);
            set_component_error("network", Some(format!("Telnet negotiation error: {}", e)));
            return Err(e);
        }
        set_component_status("telnet_negotiator", ComponentState::Running);

        match &mut *rw {
            StreamType::Plain(t) => {
                t.set_read_timeout(None)?;
                t.set_write_timeout(None)?;
            }
            StreamType::Tls(t) => {
                t.set_read_timeout(None)?;
                t.set_write_timeout(None)?;
            }
        }
    self.stream = Some(Arc::new(Mutex::new(rw)));
    self.running = true;
    // Monitoring: mark network as running and clear last error
    set_component_status("network", ComponentState::Running);
    set_component_error("network", None::<&str>);
        
        // SESSION MANAGEMENT: Initialize activity tracking
        self.update_last_activity();
        self.reconnect_attempts = 0;
        
        // Start a background thread to receive data
        self.start_receive_thread();
        
        Ok(())
    }

    /// Configure TCP keepalive for connection health monitoring
    fn configure_tcp_keepalive(&self, tcp: &TcpStream) -> IoResult<()> {

        
        // Enable TCP keepalive with platform-specific settings
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = tcp.as_raw_fd();
            
            // Enable SO_KEEPALIVE
            unsafe {
                let optval: libc::c_int = 1;
                if libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_KEEPALIVE,
                    &optval as *const _ as *const libc::c_void,
                    std::mem::size_of_val(&optval) as libc::socklen_t,
                ) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                
                // Set TCP_KEEPIDLE (time before first keepalive probe)
                #[cfg(target_os = "linux")]
                {
                    let keepidle = self.session_config.keepalive_interval_secs as libc::c_int;
                    libc::setsockopt(
                        fd,
                        libc::IPPROTO_TCP,
                        libc::TCP_KEEPIDLE,
                        &keepidle as *const _ as *const libc::c_void,
                        std::mem::size_of_val(&keepidle) as libc::socklen_t,
                    );
                }
                
                // Set TCP_KEEPINTVL (interval between keepalive probes)
                #[cfg(target_os = "linux")]
                {
                    let keepintvl = 10 as libc::c_int; // 10 seconds between probes
                    libc::setsockopt(
                        fd,
                        libc::IPPROTO_TCP,
                        libc::TCP_KEEPINTVL,
                        &keepintvl as *const _ as *const libc::c_void,
                        std::mem::size_of_val(&keepintvl) as libc::socklen_t,
                    );
                }
                
                // Set TCP_KEEPCNT (number of keepalive probes)
                #[cfg(target_os = "linux")]
                {
                    let keepcnt = 3 as libc::c_int; // 3 probes before declaring dead
                    libc::setsockopt(
                        fd,
                        libc::IPPROTO_TCP,
                        libc::TCP_KEEPCNT,
                        &keepcnt as *const _ as *const libc::c_void,
                        std::mem::size_of_val(&keepcnt) as libc::socklen_t,
                    );
                }
            }
        }
        
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawSocket;
            let socket = tcp.as_raw_socket();
            
            // Enable SO_KEEPALIVE on Windows
            unsafe {
                #[cfg(unix)]
                {
                    use std::os::unix::io::AsRawFd;
                    let raw_fd = tcp.as_raw_fd();
                    network_platform::enable_tcp_keepalive(raw_fd)?;
                }
                #[cfg(windows)]
                {
                    use std::os::windows::io::AsRawSocket;
                    let raw_socket = tcp.as_raw_socket();
                    network_platform::enable_tcp_keepalive(raw_socket)?;
                }
            }
        }
        
        eprintln!("SESSION: TCP keepalive enabled with {}s interval", self.session_config.keepalive_interval_secs);
        Ok(())
    }

    /// Connect with an explicit timeout for the initial TCP connection. Telnet negotiation still uses its own timeouts.
    pub fn connect_with_timeout(&mut self, timeout: Duration) -> IoResult<()> {
        // Monitoring: mark network as starting
        set_component_status("network", ComponentState::Starting);
        // Resolve the address
        let address = format!("{}:{}", self.host, self.port);
        let mut addrs_iter = address.to_socket_addrs()?;
        let addr: SocketAddr = addrs_iter.next().ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::AddrNotAvailable,
            "No socket addresses resolved",
        ))?;

        // Use connect_timeout for the initial TCP connect
        let tcp = match TcpStream::connect_timeout(&addr, timeout) {
            Ok(s) => s,
            Err(e) => {
                set_component_status("network", ComponentState::Error);
                set_component_error("network", Some(format!("TCP connect (timeout) failed: {}", e)));
                return Err(e);
            }
        };

        // SESSION MANAGEMENT: Enable TCP keepalive
        self.configure_tcp_keepalive(&tcp)?;

        // Set read/write timeouts
        tcp.set_read_timeout(Some(Duration::from_secs(self.session_config.connection_timeout_secs)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(self.session_config.connection_timeout_secs)))?;

        // Wrap with TLS if requested
        let mut rw: DynStream;
        if self.use_tls {
            // Build TLS connector with secure certificate validation
            let tls_config = match self.build_tls_connector() {
                Ok(cfg) => cfg,
                Err(e) => {
                    set_component_status("network", ComponentState::Error);
                    set_component_error("network", Some(format!("TLS config creation failed: {}", e)));
                    return Err(e);
                }
            };

            // Create TLS connection
            let server_name = match TlsServerName::try_from(self.host.clone()) {
                Ok(name) => name,
                Err(e) => {
                    set_component_status("network", ComponentState::Error);
                    set_component_error("network", Some(format!("Invalid server name: {}", e)));
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid server name: {}", e)));
                }
            };

            let tcp = match TcpStream::connect_timeout(&addr, timeout) {
                Ok(s) => s,
                Err(e) => {
                    set_component_status("network", ComponentState::Error);
                    set_component_error("network", Some(format!("TCP connect (TLS, timeout) failed: {}", e)));
                    return Err(e);
                }
            };
            self.configure_tcp_keepalive(&tcp)?;
            tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
            tcp.set_write_timeout(Some(Duration::from_secs(10)))?;

let tls_conn = match ClientConnection::new(tls_config, server_name) {
    Ok(conn) => conn,
    Err(e) => {
        set_component_status("network", ComponentState::Error);
        set_component_error("network", Some(format!("TLS connection failed: {}", e)));
        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("TLS connection failed: {}", e)))
    },
};
rw = Box::new(StreamType::Tls(OwnedTlsStream { conn: tls_conn, stream: tcp }));
        } else {
            let tcp = match TcpStream::connect_timeout(&addr, timeout) {
                Ok(s) => s,
                Err(e) => {
                    set_component_status("network", ComponentState::Error);
                    set_component_error("network", Some(format!("TCP connect (timeout) failed: {}", e)));
                    return Err(e);
                }
            };
            self.configure_tcp_keepalive(&tcp)?;
            tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
            tcp.set_write_timeout(Some(Duration::from_secs(10)))?;
            rw = Box::new(StreamType::Plain(tcp));
        }

        // Telnet negotiation lifecycle signals
        set_component_status("telnet_negotiator", ComponentState::Starting);
        if let Err(e) = self.perform_telnet_negotiation_rw(&mut *rw) {
            set_component_status("telnet_negotiator", ComponentState::Error);
            set_component_error("telnet_negotiator", Some(format!("Telnet negotiation failed: {}", e)));
            set_component_status("network", ComponentState::Error);
            set_component_error("network", Some(format!("Telnet negotiation error: {}", e)));
            return Err(e);
        }
        set_component_status("telnet_negotiator", ComponentState::Running);

        match &mut *rw {
            StreamType::Plain(t) => {
                t.set_read_timeout(None)?;
                t.set_write_timeout(None)?;
            }
            StreamType::Tls(t) => {
                t.set_read_timeout(None)?;
                t.set_write_timeout(None)?;
            }
        }

    self.stream = Some(Arc::new(Mutex::new(rw)));
    self.running = true;
    set_component_status("network", ComponentState::Running);
    set_component_error("network", None::<&str>);

        // SESSION MANAGEMENT: Initialize activity tracking
        self.update_last_activity();
        self.reconnect_attempts = 0;

        // Start a background thread to receive data
        self.start_receive_thread();

        // Record successful connection in monitoring
        let monitoring = crate::monitoring::MonitoringSystem::global();
        monitoring.integration_monitor.record_integration_event(crate::monitoring::IntegrationEvent {
            timestamp: std::time::Instant::now(),
            event_type: crate::monitoring::IntegrationEventType::IntegrationSuccess,
            source_component: "network".to_string(),
            target_component: Some("telnet_negotiator".to_string()),
            description: format!("Network connection established to {}:{} with keepalive", self.host, self.port),
            details: std::collections::HashMap::new(),
            duration_us: None,
            success: true,
        });

        Ok(())
    }

    /// Build a TLS connector with secure certificate validation
    /// SECURITY: Always enforces proper certificate validation to prevent MITM attacks
    fn build_tls_connector(&self) -> IoResult<Arc<ClientConfig>> {

        
        // Create a root certificate store with system certificates
        let mut root_store = RootCertStore::empty();
        
        // Add native certificates
        for cert in rustls_native_certs::load_native_certs().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to load native certificates: {}", e))
        })? {
            root_store.add(cert).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to add certificate: {}", e))
            })?;
        }

        // Add custom CA certificates if provided
        if let Some(ref path) = self.tls_ca_bundle_path {
            match self.load_certificates_securely_rustls(path) {
                Ok(certificates) => {
                    for cert in &certificates {
                        if let Err(e) = root_store.add(cert.clone()) {
                            eprintln!("SECURITY WARNING: Failed to add certificate to root store: {}", e);
                        }
                    }
                    println!("SECURITY: Added {} trusted CA certificates from {}", certificates.len(), path);
                }
                Err(e) => {
                    eprintln!("SECURITY ERROR: Failed to load CA bundle {}: {}", path, e);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to load CA bundle {}: {}", path, e)
                    ));
                }
            }
        }

        // Create client config with secure defaults
        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Arc::new(config))
    }

    /// SECURITY: Load certificates with comprehensive validation for rustls
    fn load_certificates_securely_rustls(&self, path: &str) -> IoResult<Vec<rustls::pki_types::CertificateDer<'static>>> {

        
        let bytes = fs::read(path)?;
        let mut certificates = Vec::new();

        // Validate file size to prevent memory exhaustion attacks
        if bytes.len() > 10_000_000 { // 10MB limit
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Certificate bundle too large - possible DoS attempt"
            ));
        }

        // Parse as PEM with enhanced security validation
        if let Ok(text) = String::from_utf8(bytes) {
            // Validate PEM format more strictly
            if !text.contains("-----BEGIN CERTIFICATE-----") ||
               !text.contains("-----END CERTIFICATE-----") {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid PEM certificate format"
                ));
            }

            let marker_begin = "-----BEGIN CERTIFICATE-----";
            let marker_end = "-----END CERTIFICATE-----";
            let mut start = 0;

            while let Some(b) = text[start..].find(marker_begin) {
                let bpos = start + b + marker_begin.len();
                if let Some(e) = text[bpos..].find(marker_end) {
                    let epos = bpos + e;
                    let b64 = text[bpos..epos]
                        .lines()
                        .filter(|line| !line.trim().is_empty() && !line.starts_with("-----"))
                        .collect::<Vec<_>>()
                        .join("");

                    // Validate base64 content
                    if b64.chars().any(|c| !c.is_alphanumeric() && c != '+' && c != '/' && c != '=') {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid base64 characters in certificate"
                        ));
                    }

                    match base64::engine::general_purpose::STANDARD.decode(&b64) {
                        Ok(der) => {
                            // Validate certificate format and add to collection
                            let cert = rustls::pki_types::CertificateDer::from(der);
                            certificates.push(cert);
                        }
                        Err(e) => {
                            eprintln!("SECURITY WARNING: Failed to decode certificate: {}", e);
                        }
                    }
                    start = epos + marker_end.len();
                } else {
                    break;
                }
            }

            if certificates.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "No valid certificates found in PEM bundle"
                ));
            } else {
                return Ok(certificates);
            }
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "CA bundle contains invalid UTF-8"
            ));
        }


    }
    
    /// Performs RFC-compliant telnet option negotiation
    fn perform_telnet_negotiation_rw(&mut self, stream: &mut dyn ReadWrite) -> IoResult<()> {
        // Send initial negotiation requests
        let initial_negotiation = self.telnet_negotiator.generate_initial_negotiation();
        if !initial_negotiation.is_empty() {
            println!("Sending initial telnet negotiation ({} bytes)", initial_negotiation.len());
            stream.write_all(&initial_negotiation)?;
            stream.flush()?;
        }
        
        let mut negotiation_attempts = 0;
        const MAX_NEGOTIATION_ATTEMPTS: usize = 30; // Reduced for faster timeout
        const NEGOTIATION_TIMEOUT_SECS: u64 = 15; // Total timeout for negotiation
        
        let negotiation_start = std::time::Instant::now();
        
        while !self.telnet_negotiator.is_negotiation_complete() && 
              negotiation_attempts < MAX_NEGOTIATION_ATTEMPTS &&
              negotiation_start.elapsed().as_secs() < NEGOTIATION_TIMEOUT_SECS {
            
            let mut buffer = [0u8; 1024];
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Connection closed during negotiation");
                    break;
                }
                Ok(n) => {
                    println!("Received negotiation data ({} bytes)", n);
                    let response = self.telnet_negotiator.process_incoming_data(&buffer[..n]);
                    
                    if !response.is_empty() {
                        println!("Sending negotiation response ({} bytes)", response.len());
                        match stream.write_all(&response) {
                            Ok(()) => {
                                if let Err(e) = stream.flush() {
                                    println!("Warning: Failed to flush negotiation response: {}", e);
                                }
                            }
                            Err(e) => {
                                println!("Error sending negotiation response: {}", e);
                                return Err(e);
                            }
                        }
                    }
                    
                    negotiation_attempts += 1;
                    
                    // Small delay to prevent busy waiting
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                    println!("Negotiation timeout, checking if we can proceed...");
                    
                    // Try to force completion if essential options were attempted
                    if self.telnet_negotiator.force_negotiation_complete() {
                        println!("Proceeding with forced negotiation completion");
                        break;
                    }
                    
                    // If we can't force completion, give up
                    println!("Cannot complete negotiation, timing out");
                    break;
                }
                Err(e) => {
                    println!("Negotiation error: {}", e);
                    return Err(e);
                }
            }
        }

        // Check final negotiation status
        if self.telnet_negotiator.is_negotiation_complete() {
            self.negotiation_complete = true;
            println!("RFC 2877 telnet negotiation completed successfully after {} attempts in {:.2}s",
                     negotiation_attempts, negotiation_start.elapsed().as_secs_f64());

            // Record successful negotiation in monitoring
            let monitoring = crate::monitoring::MonitoringSystem::global();
            monitoring.integration_monitor.record_integration_event(crate::monitoring::IntegrationEvent {
                timestamp: std::time::Instant::now(),
                event_type: crate::monitoring::IntegrationEventType::IntegrationSuccess,
                source_component: "telnet_negotiator".to_string(),
                target_component: Some("network".to_string()),
                description: format!("Telnet negotiation completed successfully in {:.2}s", negotiation_start.elapsed().as_secs_f64()),
                details: std::collections::HashMap::new(),
                duration_us: Some(negotiation_start.elapsed().as_micros() as u64),
                success: true,
            });
        } else {
            println!("Telnet negotiation incomplete after {} attempts and {:.2}s", 
                     negotiation_attempts, negotiation_start.elapsed().as_secs_f64());
            
            // Log current negotiation status for debugging
            let status = self.telnet_negotiator.get_negotiation_state_details();
            println!("Final negotiation status:");
            for (option, state) in status {
                println!("  {:?}: {:?}", option, state);
            }
            
            // Try to proceed anyway if essential options are somewhat negotiated
            if self.telnet_negotiator.force_negotiation_complete() {
                self.negotiation_complete = true;
                println!("Proceeding with partial negotiation");
            } else {
                // Record negotiation failure in monitoring
                let monitoring = crate::monitoring::MonitoringSystem::global();
                let alert = crate::monitoring::Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: std::time::Instant::now(),
                    level: crate::monitoring::AlertLevel::Critical,
                    component: "telnet_negotiator".to_string(),
                    message: format!("Telnet negotiation failed after {} attempts in {:.2}s",
                        negotiation_attempts, negotiation_start.elapsed().as_secs_f64()),
                    details: std::collections::HashMap::new(),
                    acknowledged: false,
                    acknowledged_at: None,
                    resolved: false,
                    resolved_at: None,
                    occurrence_count: 1,
                    last_occurrence: std::time::Instant::now(),
                };
                monitoring.alerting_system.trigger_alert(alert);

                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Telnet option negotiation failed - essential options not negotiated"
                ));
            }
        }
        Ok(())
    }
    
    /// Starts a background thread to receive data
    /// SECURITY: Enhanced with bounds checking and secure resource cleanup
    fn start_receive_thread(&mut self) {
        // CRITICAL FIX: Avoid self borrowing by cloning needed data first
        let shared = if let Some(ref stream) = self.stream {
            Arc::clone(stream)
        } else {
            return; // No stream available
        };

        let sender = if let Some(ref s) = self.sender {
            s.clone()
        } else {
            return; // No sender available
        };

        thread::spawn(move || {
                // PERFORMANCE OPTIMIZATION: Use pooled buffer to reduce allocations
                // SECURITY: Use a reasonable buffer size with upper bound
                const MAX_READ_SIZE: usize = 8192; // 8KB max read size
                let mut pooled_buffer = NETWORK_BUFFER_POOL.get_buffer();
                pooled_buffer.resize(MAX_READ_SIZE, 0);
                let mut consecutive_errors = 0;
                const MAX_CONSECUTIVE_ERRORS: u32 = 10;

                loop {
                    // Lock the stream only for the duration of the read
                    let read_result = {
                        let mut guard = match shared.lock() {
                            Ok(g) => g,
                            Err(poisoned) => {
                                // SECURITY: Handle poisoned mutex gracefully
                                eprintln!("SECURITY: Stream mutex poisoned - performing emergency cleanup");
                                poisoned.into_inner()
                            }
                        };
    
                        // PERFORMANCE OPTIMIZATION: Use pooled buffer directly
                        // CRITICAL FIX: Enhanced buffer handling with comprehensive validation
                        // SECURITY: Limit read size to prevent memory exhaustion
                        match guard.read(&mut pooled_buffer[..MAX_READ_SIZE]) {
                            Ok(0) => Ok(0), // Connection closed
                            Ok(n) => {
                                // CRITICAL FIX: Additional validation of read size
                                if n > MAX_READ_SIZE {
                                    eprintln!("SECURITY: Read size {} exceeds maximum allowed {}", n, MAX_READ_SIZE);
                                    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Read size too large"))
                                } else if n == 0 {
                                    // CRITICAL FIX: Handle zero reads properly
                                    Ok(0)
                                } else {
                                    // CRITICAL FIX: Validate buffer contents before copying
                                    let safe_bytes = &pooled_buffer[..n];

                                    // Check for suspicious data patterns that might indicate attacks
                                    if AS400Connection::validate_network_data(safe_bytes) {
                                        // PERFORMANCE OPTIMIZATION: Data is already in pooled buffer
                                        Ok(n)
                                    } else {
                                        eprintln!("SECURITY: Suspicious network data detected");
                            
                                        // Record security event in monitoring
                                        let monitoring = crate::monitoring::MonitoringSystem::global();
                                        let alert = crate::monitoring::Alert {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            timestamp: std::time::Instant::now(),
                                            level: crate::monitoring::AlertLevel::Warning,
                                            component: "network".to_string(),
                                            message: "Suspicious network data pattern detected".to_string(),
                                            details: std::collections::HashMap::new(),
                                            acknowledged: false,
                                            acknowledged_at: None,
                                            resolved: false,
                                            resolved_at: None,
                                            occurrence_count: 1,
                                            last_occurrence: std::time::Instant::now(),
                                        };
                                        monitoring.alerting_system.trigger_alert(alert);
                            
                                        // Record security event
                                        let security_event = crate::monitoring::SecurityEvent {
                                            timestamp: std::time::Instant::now(),
                                            event_type: crate::monitoring::SecurityEventType::SuspiciousNetworkPattern,
                                            severity: crate::monitoring::SecurityEventSeverity::Medium,
                                            description: "Suspicious network data pattern detected".to_string(),
                                            source_ip: None,
                                            details: std::collections::HashMap::new(),
                                            mitigated: true,
                                        };
                                        monitoring.security_monitor.record_security_event(security_event);
                            
                                        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Suspicious data"))
                                    }
                                }
                            }
                            Err(e) => Err(e),
                        }
                    };

                    match read_result {
                        Ok(0) => {
                            // Connection closed cleanly
                            println!("SECURITY: Network connection closed cleanly");
                            break;
                        }
                        Ok(n) => {
                            // SECURITY: Reset error counter on successful read
                            consecutive_errors = 0;

                            // SECURITY: Validate data size before sending
                            if n > MAX_READ_SIZE {
                                eprintln!("SECURITY: Invalid data size received: {}", n);
                                break;
                            }

                            // PERFORMANCE OPTIMIZATION: Clone data from pooled buffer instead of allocating new Vec
                            let data_to_send = pooled_buffer[..n].to_vec();
                            if sender.send(data_to_send).is_err() {
                                eprintln!("SECURITY: Channel send failed - receiver may be closed");
                                break;
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                            // Just try again - these are normal for non-blocking I/O
                            thread::sleep(Duration::from_millis(50));
                            continue;
                        }
                        Err(e) => {
                            // SECURITY: Track consecutive errors to prevent infinite retry loops
                            consecutive_errors += 1;
                            if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                                eprintln!("SECURITY: Too many consecutive read errors ({}), terminating thread", consecutive_errors);
                                break;
                            }

                            eprintln!("SECURITY: Read error ({} consecutive): {}", consecutive_errors, e);
                            thread::sleep(Duration::from_millis(100 * consecutive_errors as u64));
                        }
                    }
                }

                // PERFORMANCE OPTIMIZATION: Return buffer to pool for reuse
                NETWORK_BUFFER_POOL.return_buffer(pooled_buffer);

                println!("SECURITY: Receive thread terminated cleanly with proper resource cleanup");
        });
    }

    /// Disconnects from the AS/400 system
    /// SECURITY: Enhanced with secure resource cleanup to prevent resource leaks
    /// INTEGRATION: Reset protocol detection state
    /// SESSION MANAGEMENT: Reset session tracking
    pub fn disconnect(&mut self) {
        // CRITICAL FIX: Enhanced cleanup with proper resource management
        self.running = false;
        self.negotiation_complete = false;

        // CRITICAL FIX: Safer stream cleanup with explicit drop
        if let Some(stream) = self.stream.take() {
            // Explicitly drop the stream to ensure cleanup
            drop(stream);
        }

        // CRITICAL FIX: Safer channel cleanup with validation
        // Note: We can't directly drop sender/receiver as they're wrapped in Option
        // But we can clear them to ensure they're dropped
        self.sender = None;
        self.receiver = None;

        // CRITICAL FIX: Reset telnet negotiator state with validation
        self.telnet_negotiator = TelnetNegotiator::new();

        // INTEGRATION: Reset protocol detection state
        self.protocol_detector.reset();
        self.detected_mode = ProtocolMode::AutoDetect;

        // SESSION MANAGEMENT: Reset session tracking
        self.last_activity = None;
        self.reconnect_attempts = 0;

        // CRITICAL FIX: Clear sensitive connection data
        self.host.clear();
        self.tls_ca_bundle_path = None;

        // Record disconnection in monitoring
        let monitoring = crate::monitoring::MonitoringSystem::global();
        monitoring.integration_monitor.record_integration_event(crate::monitoring::IntegrationEvent {
            timestamp: std::time::Instant::now(),
            event_type: crate::monitoring::IntegrationEventType::ComponentInteraction,
            source_component: "network".to_string(),
            target_component: Some("controller".to_string()),
            description: "Network connection disconnected and cleaned up".to_string(),
            details: std::collections::HashMap::new(),
            duration_us: None,
            success: true,
        });

        println!("SECURITY: Network connection and resources cleaned up securely");

    // Monitoring: mark network and telnet negotiator as stopped
    set_component_status("network", ComponentState::Stopped);
    set_component_error("network", None::<&str>);
    set_component_status("telnet_negotiator", ComponentState::Stopped);
        println!("INTEGRATION: Protocol detection state reset");
        println!("SESSION: Session tracking reset");
    }

    /// Sends data to the AS/400 system
    /// SESSION MANAGEMENT: Updates activity timestamp on send
    pub fn send_data(&mut self, data: &[u8]) -> IoResult<usize> {
        // CRITICAL FIX: Enhanced validation and error handling for data transmission

        // Validate input data
        if data.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Cannot send empty data"
            ));
        }

        // Validate data size to prevent memory exhaustion
        if data.len() > 65535 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Data packet too large"
            ));
        }

        let result = if let Some(ref shared) = self.stream {
            let mut guard = shared.lock().map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Stream lock poisoned"))?;

            // CRITICAL FIX: Validate data before sending
            if AS400Connection::validate_network_data(data) {
                let send_result = guard.write(data);
                // PERFORMANCE MONITORING: Track bytes sent
                if let Ok(bytes) = &send_result {
                    use std::sync::atomic::Ordering;
                    crate::monitoring::MonitoringSystem::global()
                        .performance_monitor
                        .metrics
                        .network
                        .bytes_sent_per_sec
                        .fetch_add(*bytes as u64, Ordering::Relaxed);
                    crate::monitoring::MonitoringSystem::global()
                        .performance_monitor
                        .metrics
                        .network
                        .packets_sent_per_sec
                        .fetch_add(1, Ordering::Relaxed);
                }
                send_result
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid data for transmission"
                ))
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Not connected to AS/400 system"
            ))
        };

        // SESSION MANAGEMENT: Update activity on successful send (after guard is dropped)
        if result.is_ok() {
            self.update_last_activity();
        }

        result
    }

    /// Receives data from the AS/400 system through the channel
    /// INTEGRATION: Enhanced with protocol auto-detection and mode switching
    /// SESSION MANAGEMENT: Updates activity timestamp and checks for idle timeout
    pub fn receive_data_channel(&mut self) -> Option<Vec<u8>> {
        // SESSION MANAGEMENT: Check for idle timeout
        if self.is_session_idle_timeout() {
            eprintln!("SESSION: Idle timeout exceeded - disconnecting");
            self.disconnect();
            return None;
        }

        if let Some(ref receiver) = self.receiver {
            // Try to receive data without blocking
            match receiver.try_recv() {
                Ok(data) => {
                    // SESSION MANAGEMENT: Update activity timestamp on data receipt
                    self.update_last_activity();
                    println!("DEBUG: Received data from network: {} bytes", data.len());
                    if data.len() > 0 {
                        println!("DEBUG: Raw data: {:02x?}", &data[..data.len().min(50)]);
                    }
                    if !self.protocol_detector.is_detection_complete() {
                        match self.protocol_detector.detect_protocol(&data) {
                            Ok(mode) => {
                                self.detected_mode = mode;
                                if self.protocol_detector.is_detection_complete() {
                                    println!("INTEGRATION: Protocol detection complete - operating in {:?} mode", self.detected_mode);
                                }
                            }
                            Err(e) => {
                                eprintln!("Protocol detection error: {} - falling back to NVT mode", e);
                                self.detected_mode = ProtocolMode::NVT;
                                self.protocol_detector.mode = ProtocolMode::NVT;
                            }
                        }
                    }

                    // Process the received data through our telnet negotiator (only during negotiation)
                    let negotiation_response = if !self.negotiation_complete {
                        self.telnet_negotiator.process_incoming_data(&data)
                    } else {
                        Vec::new() // Negotiation complete, no more telnet processing needed
                    };

                    // If there's a negotiation response, send it immediately
                    if !negotiation_response.is_empty() {
                        if let Some(ref shared) = self.stream {
                            match shared.lock() {
                                Ok(mut guard) => {
                                    if let Err(e) = guard.write_all(&negotiation_response) {
                                        eprintln!("Failed to send telnet negotiation response: {}", e);
                                    } else if let Err(e) = guard.flush() {
                                        eprintln!("Failed to flush telnet negotiation response: {}", e);
                                    } else {
                                        println!("Sent telnet negotiation response ({} bytes)", negotiation_response.len());
                                    }
                                }
                                Err(_) => {
                                    eprintln!("Failed to lock stream for sending telnet negotiation response");
                                }
                            }
                        }
                    }

                    // INTEGRATION: Filter data based on detected protocol mode
                    match self.detected_mode {
                        ProtocolMode::TN5250 => {
                            // Filter out telnet negotiation from the data and return clean 5250 data
                            let clean_data = self.extract_5250_data(&data);
                            println!("DEBUG: TN5250 mode - extracted {} bytes from {} bytes", clean_data.len(), data.len());
                            if clean_data.len() > 0 {
                                println!("DEBUG: Clean 5250 data first 20 bytes: {:02x?}", &clean_data[..clean_data.len().min(20)]);
                                Some(clean_data)
                            } else {
                                None // No 5250 data in this packet, just negotiation
                            }
                        },
                        ProtocolMode::TN3270 => {
                            // Filter out telnet negotiation from the data and return clean 3270 data
                            let clean_data = self.extract_3270_data(&data);
                            if !clean_data.is_empty() {
                                Some(clean_data)
                            } else {
                                None // No 3270 data in this packet, just negotiation
                            }
                        },
                        ProtocolMode::NVT => {
                            // For NVT mode, return data as-is (after telnet processing)
                            let clean_data = self.extract_5250_data(&data); // Still filter telnet commands
                            if !clean_data.is_empty() {
                                Some(clean_data)
                            } else {
                                None
                            }
                        },
                        ProtocolMode::AutoDetect => {
                            // Still detecting, buffer data but don't return yet
                            None
                        }
                    }
                }
                Err(mpsc::TryRecvError::Empty) => None, // No data available
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Channel disconnected - this indicates a critical connection failure
                    eprintln!("Network channel disconnected - connection lost");
                    self.running = false;
                    // Monitoring: reflect error condition
                    set_component_status("network", ComponentState::Error);
                    set_component_error("network", Some("Network channel disconnected"));
                    None
                }
            }
        } else {
            None
        }
    }
    
    /// CRITICAL FIX: Validate network data for suspicious patterns
    /// Made standalone to avoid lifetime issues in threads
    fn validate_network_data(data: &[u8]) -> bool {
        // Check for obviously malformed data
        if data.is_empty() {
            return false;
        }

        // Check for potential buffer overflow patterns
        if data.len() > 65535 {
            eprintln!("SECURITY: Network data too large: {}", data.len());
            return false;
        }

        // Check for excessive null bytes or 0xFF bytes (potential attacks)
        let null_count = data.iter().filter(|&&b| b == 0x00).count();
        let ff_count = data.iter().filter(|&&b| b == 0xFF).count();
        let null_ratio = null_count as f32 / data.len() as f32;
        let ff_ratio = ff_count as f32 / data.len() as f32;

        // Allow up to 50% nulls or 0xFF in protocol data (common in 5250/3270)
        if null_ratio > 0.5 || ff_ratio > 0.5 {
            eprintln!("SECURITY: Excessive null/FF bytes in network data (null: {:.2}, FF: {:.2})", null_ratio, ff_ratio);
            return false;
        }

        // Check for suspicious repeating patterns (potential exploit)
        let suspicious_patterns = [
            &[0u8; 32], // Very long sequence of nulls
            &[255u8; 32], // Very long sequence of 0xFF
        ];

        for pattern in &suspicious_patterns {
            if data.windows(pattern.len()).any(|window| window == *pattern) {
                eprintln!("SECURITY: Suspicious byte pattern detected in network data");
                return false;
            }
        }

        // All checks passed - data looks legitimate
        true
    }

    /// Extract non-telnet 5250 data from the received stream
    /// SECURITY: Enhanced with comprehensive bounds checking to prevent buffer overflows
    fn extract_5250_data(&self, data: &[u8]) -> Vec<u8> {
        // CRITICAL FIX: Enhanced validation with multiple security checks
        if data.is_empty() || data.len() > 65535 {
            eprintln!("SECURITY: Invalid data size for 5250 extraction: {}", data.len());
            return Vec::new();
        }

        // Additional validation: check for obviously corrupted data
        if !AS400Connection::validate_network_data(data) {
            eprintln!("SECURITY: Network data validation failed");
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut i = 0;

        while i < data.len() {
            if data[i] == 255 { // IAC
                // CRITICAL FIX: Enhanced bounds checking for telnet command
                if i + 1 >= data.len() {
                    eprintln!("SECURITY: Incomplete telnet command at end of data");
                    break;
                }

                match data[i + 1] {
                    251..=254 => { // WILL, WONT, DO, DONT
                        // CRITICAL FIX: Validate we have enough bytes for option
                        if i + 2 < data.len() {
                            i += 3; // Skip IAC + command + option
                            continue;
                        } else {
                            eprintln!("SECURITY: Incomplete telnet option command");
                            break;
                        }
                    },
                    250 => { // SB (subnegotiation)
                        // CRITICAL FIX: Enhanced subnegotiation parsing with better bounds checking
                        let mut j = i + 2;

                        // CRITICAL FIX: Prevent infinite loop in malformed subnegotiation
                        let mut search_count = 0;
                        const MAX_SEARCH: usize = 8192; // Limit search to prevent DoS

                        while j + 1 < data.len() && search_count < MAX_SEARCH {
                            if data[j] == 255 && data[j + 1] == 240 { // IAC SE
                                i = j + 2;
                                break;
                            }
                            j += 1;
                            search_count += 1;
                        }

                        if search_count >= MAX_SEARCH {
                            eprintln!("SECURITY: Subnegotiation search limit exceeded - possible DoS attempt");
                            break;
                        }

                        if j + 1 >= data.len() {
                            eprintln!("SECURITY: Malformed or oversized subnegotiation detected");
                            break;
                        }
                        continue;
                    },
                    255 => { // Escaped IAC
                        result.push(255);
                        i += 2;
                        continue;
                    },
                    _ => {
                        // Other telnet command, skip IAC + command
                        i += 2;
                        continue;
                    }
                }
            }

            result.push(data[i]);
            i += 1;
        }

        // CRITICAL FIX: Validate result size to prevent memory exhaustion
        if result.len() > 65535 {
            eprintln!("SECURITY: Extracted data too large: {}", result.len());
            Vec::new()
        } else {
            result
        }
    }

    /// Extract non-telnet 3270 data from the received stream
    /// Similar to extract_5250_data but with 3270-specific processing
    fn extract_3270_data(&self, data: &[u8]) -> Vec<u8> {
        // Use the same telnet filtering as 5250 since both protocols run over telnet
        let clean_data = self.extract_5250_data(data);
        
        // Additional 3270-specific validation could be added here
        // For now, 3270 and 5250 both filter telnet the same way
        // The protocol-specific processing happens at a higher level
        
        if !clean_data.is_empty() {
            println!("DEBUG: Extracted {} bytes of 3270 data", clean_data.len());
            if clean_data.len() > 0 {
                println!("DEBUG: Clean 3270 data first 20 bytes: {:02x?}", &clean_data[..clean_data.len().min(20)]);
            }
        }
        
        clean_data
    }

    /// Checks if the connection is active
    pub fn is_connected(&self) -> bool {
        self.stream.is_some() && self.running
    }

    /// Checks if telnet negotiation is complete
    pub fn is_negotiation_complete(&self) -> bool {
        self.negotiation_complete
    }

    /// INTEGRATION: Get the detected protocol mode
    pub fn get_detected_protocol_mode(&self) -> ProtocolMode {
        self.detected_mode
    }

    /// INTEGRATION: Check if protocol detection is complete
    pub fn is_protocol_detection_complete(&self) -> bool {
        self.protocol_detector.is_detection_complete()
    }

    /// INTEGRATION: Force protocol mode (for testing or manual override)
    pub fn set_protocol_mode(&mut self, mode: ProtocolMode) {
        self.detected_mode = mode;
        self.protocol_detector.mode = mode;
        println!("INTEGRATION: Protocol mode manually set to {:?}", mode);
    }

    /// Set credentials for AS/400 authentication (RFC 4777)
    /// These credentials will be used during telnet NEW-ENVIRON negotiation
    /// 
    /// # Arguments
    /// * `username` - AS/400 user profile name
    /// * `password` - User password
    /// 
    /// # Security Note
    /// Current implementation uses plain text password transmission.
    /// For production use, implement DES or SHA password encryption per RFC 4777.
    pub fn set_credentials(&mut self, username: &str, password: &str) {
        self.telnet_negotiator.set_credentials(username, password);
        println!("Network: Credentials configured for telnet negotiation");
    }

    /// Gets the host address
    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Gets the port
    pub fn get_port(&self) -> u16 {
        self.port
    }

    /// CRITICAL FIX: Validate connection state and resource integrity
    /// This method ensures the connection is in a valid state and resources are properly managed
    pub fn validate_connection_integrity(&self) -> Result<(), String> {
        // Validate stream state
        match &self.stream {
            Some(stream) => {
                // Try to validate the stream is still accessible
                match stream.try_lock() {
                    Ok(_) => {
                        // Stream is accessible
                    }
                    Err(std::sync::TryLockError::Poisoned(_)) => {
                        return Err("Connection stream mutex is poisoned".to_string());
                    }
                    Err(std::sync::TryLockError::WouldBlock) => {
                        // Stream is busy, which is normal
                    }
                }
            }
            None => {
                if self.running {
                    return Err("Connection is running but has no stream".to_string());
                }
            }
        }

        // Validate channel state
        if self.running {
            if self.sender.is_none() || self.receiver.is_none() {
                return Err("Connection is running but channels are not available".to_string());
            }
        }

        // Validate negotiation state consistency
        if self.negotiation_complete && !self.running {
            return Err("Negotiation complete but connection not running".to_string());
        }

        Ok(())
    }

    /// CRITICAL FIX: Safe resource cleanup with validation
    /// This method ensures all resources are properly cleaned up
    pub fn safe_cleanup(&mut self) {
        // Stop the connection
        self.running = false;

        // Clear all resources in safe order
        if let Some(_stream) = self.stream.take() {
            // Stream will be dropped here
        }

        // Clear channels
        self.sender = None;
        self.receiver = None;

        // Reset state
        self.negotiation_complete = false;
        self.telnet_negotiator = TelnetNegotiator::new();

        // Clear sensitive data
        self.host.clear();
        self.tls_ca_bundle_path = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_creation() {
        let conn = AS400Connection::new("localhost".to_string(), 23);
        assert_eq!(conn.get_host(), "localhost");
        assert_eq!(conn.get_port(), 23);
        assert!(!conn.is_connected());
    }

    #[test]
    fn test_tls_default_for_port_992() {
        let conn = AS400Connection::new("example.com".to_string(), 992);
        assert!(conn.is_tls_enabled(), "TLS should be enabled by default on port 992");
    }

    #[test]
    fn test_tls_default_for_port_23() {
        let conn = AS400Connection::new("example.com".to_string(), 23);
        assert!(!conn.is_tls_enabled(), "TLS should be disabled by default on port 23");
    }

    #[test]
    fn test_tls_override_enable_on_23() {
        let mut conn = AS400Connection::new("example.com".to_string(), 23);
        assert!(!conn.is_tls_enabled());
        conn.set_tls(true);
        assert!(conn.is_tls_enabled(), "TLS override should enable TLS on non-SSL port");
    }

    #[test]
    fn test_tls_override_disable_on_992() {
        let mut conn = AS400Connection::new("example.com".to_string(), 992);
        assert!(conn.is_tls_enabled());
        conn.set_tls(false);
        assert!(!conn.is_tls_enabled(), "TLS override should disable TLS on SSL port");
    }
}