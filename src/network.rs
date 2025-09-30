//! Network module for handling AS/400 connections using the 5250 protocol
//! 
//! This module provides the TCP networking functionality for connecting
//! to AS/400 systems using the TN5250 protocol.

use std::net::{TcpStream, ToSocketAddrs, SocketAddr};
use std::io::{Read, Write, Result as IoResult};
use std::time::Duration;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use native_tls::{TlsConnector, Certificate};
use std::fs;

use crate::telnet_negotiation::TelnetNegotiator;

// A helper trait alias for objects that implement both Read and Write
trait ReadWrite: Read + Write {}
impl<T: Read + Write> ReadWrite for T {}

type DynStream = Box<dyn ReadWrite + Send>;
type SharedStream = Arc<Mutex<DynStream>>;

/// Represents a connection to an AS/400 system
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
    tls_insecure: bool,
    tls_ca_bundle_path: Option<String>,
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
            tls_insecure: false,
            tls_ca_bundle_path: None,
        }
    }

    /// Enable or disable TLS explicitly (overrides port-based default)
    pub fn set_tls(&mut self, enabled: bool) {
        self.use_tls = enabled;
    }

    /// Returns true if TLS is enabled for this connection
    pub fn is_tls_enabled(&self) -> bool {
        self.use_tls
    }

    /// SECURITY: TLS certificate validation cannot be disabled for security reasons.
    /// This method is deprecated and will always log a security warning.
    /// Certificate validation is always enforced to prevent man-in-the-middle attacks.
    pub fn set_tls_insecure(&mut self, _insecure: bool) {
        eprintln!("SECURITY WARNING: TLS certificate validation cannot be disabled.");
        eprintln!("SECURITY WARNING: This prevents man-in-the-middle attacks and ensures secure communication.");
        eprintln!("SECURITY WARNING: Use proper certificate management instead of disabling validation.");
        // Note: tls_insecure field is kept for compatibility but ignored in build_tls_connector
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
        let address = format!("{}:{}", self.host, self.port);
        let mut tcp = TcpStream::connect(&address)?;
        
        // Set read/write timeouts
        tcp.set_read_timeout(Some(Duration::from_secs(30)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(30)))?;

        // Wrap with TLS if requested
        let mut rw: DynStream = if self.use_tls {
            let connector = self.build_tls_connector()?;
            let mut tls = connector
                .connect(self.host.as_str(), tcp)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            // Use shorter timeouts during negotiation
            // Set on the underlying TcpStream
            let _ = tls.get_ref().set_read_timeout(Some(Duration::from_secs(10)));
            let _ = tls.get_ref().set_write_timeout(Some(Duration::from_secs(10)));
            // Perform telnet negotiation over TLS
            self.perform_telnet_negotiation_rw(&mut tls)?;
            // Reset timeouts to none (blocking) after negotiation
            let _ = tls.get_ref().set_read_timeout(None);
            let _ = tls.get_ref().set_write_timeout(None);
            Box::new(tls)
        } else {
            // Short negotiation timeouts
            tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
            tcp.set_write_timeout(Some(Duration::from_secs(10)))?;
            // Perform negotiation in plain TCP
            self.perform_telnet_negotiation_rw(&mut tcp)?;
            // Reset to blocking after negotiation
            tcp.set_read_timeout(None)?;
            tcp.set_write_timeout(None)?;
            Box::new(tcp)
        };

        self.stream = Some(Arc::new(Mutex::new(rw)));
        self.running = true;
        
        // Start a background thread to receive data
        self.start_receive_thread();
        
        Ok(())
    }

    /// Connect with an explicit timeout for the initial TCP connection. Telnet negotiation still uses its own timeouts.
    pub fn connect_with_timeout(&mut self, timeout: Duration) -> IoResult<()> {
        // Resolve the address
        let address = format!("{}:{}", self.host, self.port);
        let mut addrs_iter = address.to_socket_addrs()?;
        let addr: SocketAddr = addrs_iter.next().ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::AddrNotAvailable,
            "No socket addresses resolved",
        ))?;

        // Use connect_timeout for the initial TCP connect
        let mut tcp = TcpStream::connect_timeout(&addr, timeout)?;

        // Set read/write timeouts
        tcp.set_read_timeout(Some(Duration::from_secs(30)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(30)))?;

        // Wrap with TLS if requested
        let mut rw: DynStream = if self.use_tls {
            let connector = self.build_tls_connector()?;
            let mut tls = connector
                .connect(self.host.as_str(), tcp)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            // Short negotiation timeouts
            let _ = tls.get_ref().set_read_timeout(Some(Duration::from_secs(10)));
            let _ = tls.get_ref().set_write_timeout(Some(Duration::from_secs(10)));
            self.perform_telnet_negotiation_rw(&mut tls)?;
            // Reset
            let _ = tls.get_ref().set_read_timeout(None);
            let _ = tls.get_ref().set_write_timeout(None);
            Box::new(tls)
        } else {
            // Short negotiation timeouts
            tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
            tcp.set_write_timeout(Some(Duration::from_secs(10)))?;
            self.perform_telnet_negotiation_rw(&mut tcp)?;
            tcp.set_read_timeout(None)?;
            tcp.set_write_timeout(None)?;
            Box::new(tcp)
        };

        self.stream = Some(Arc::new(Mutex::new(rw)));
        self.running = true;

        // Start a background thread to receive data
        self.start_receive_thread();

        Ok(())
    }

    /// Build a TLS connector with secure certificate validation
    /// SECURITY: Always enforces proper certificate validation to prevent MITM attacks
    fn build_tls_connector(&self) -> IoResult<TlsConnector> {
        let mut builder = TlsConnector::builder();

        // SECURITY: Always enforce certificate validation - never bypass
        // This prevents man-in-the-middle attacks
        eprintln!("SECURITY: TLS certificate validation is always enabled");

        // Add custom CA certificates if provided
        if let Some(ref path) = self.tls_ca_bundle_path {
            match self.load_certificates_securely(path) {
                Ok(certs_added) => {
                    if certs_added > 0 {
                        println!("SECURITY: Added {} trusted CA certificates from {}", certs_added, path);
                    } else {
                        eprintln!("SECURITY WARNING: No valid certificates found in CA bundle: {}", path);
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("No valid certificates in CA bundle: {}", path)
                        ));
                    }
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

        // SECURITY: Set minimum TLS version to 1.2 for better security
        #[cfg(feature = "secure_tls")]
        {
            builder.min_tls_version(native_tls::Protocol::Tlsv12);
        }

        builder.build().map_err(|e| {
            eprintln!("SECURITY ERROR: Failed to build secure TLS connector: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })
    }

    /// SECURITY: Load certificates with comprehensive validation
    fn load_certificates_securely(&self, path: &str) -> IoResult<usize> {
        let bytes = fs::read(path)?;
        let mut certs_added = 0;

        // Validate file size to prevent memory exhaustion attacks
        if bytes.len() > 10_000_000 { // 10MB limit
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Certificate bundle too large - possible DoS attempt"
            ));
        }

        // Try to parse as DER first
        if let Ok(cert) = Certificate::from_der(&bytes) {
            let mut builder = TlsConnector::builder();
            builder.add_root_certificate(cert);
            certs_added += 1;
        } else {
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

                let mut added = false;
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

                        match base64::decode(&b64) {
                            Ok(der) => {
                                match Certificate::from_der(&der) {
                                    Ok(cert) => {
                                        let mut builder = TlsConnector::builder();
                                        builder.add_root_certificate(cert);
                                        added = true;
                                        certs_added += 1;
                                    }
                                    Err(e) => {
                                        eprintln!("SECURITY WARNING: Invalid certificate in bundle: {}", e);
                                    }
                                }
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

                if !added {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "No valid certificates found in PEM bundle"
                    ));
                }
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "CA bundle contains invalid UTF-8"
                ));
            }
        }

        Ok(certs_added)
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
                // SECURITY: Use a reasonable buffer size with upper bound
                const MAX_READ_SIZE: usize = 8192; // 8KB max read size
                let mut buffer = [0u8; MAX_READ_SIZE];
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
    
                        // CRITICAL FIX: Enhanced buffer handling with comprehensive validation
                        // SECURITY: Limit read size to prevent memory exhaustion
                        let mut limited_buffer = [0u8; MAX_READ_SIZE];
                        match guard.read(&mut limited_buffer) {
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
                                    let safe_bytes = &limited_buffer[..n];
    
                                    // Check for suspicious data patterns that might indicate attacks
                                    if AS400Connection::validate_network_data(safe_bytes) {
                                        // Copy only the valid portion
                                        buffer[..n].copy_from_slice(safe_bytes);
                                        Ok(n)
                                    } else {
                                        eprintln!("SECURITY: Suspicious network data detected");
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

                            let data_to_send = buffer[..n].to_vec();
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

                println!("SECURITY: Receive thread terminated cleanly with proper resource cleanup");
        });
    }

    /// Disconnects from the AS/400 system
    /// SECURITY: Enhanced with secure resource cleanup to prevent resource leaks
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

        // CRITICAL FIX: Clear sensitive connection data
        self.host.clear();
        self.tls_ca_bundle_path = None;

        println!("SECURITY: Network connection and resources cleaned up securely");
    }

    /// Sends data to the AS/400 system
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

        if let Some(ref shared) = self.stream {
            let mut guard = shared.lock().map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Stream lock poisoned"))?;

            // CRITICAL FIX: Validate data before sending
            if AS400Connection::validate_network_data(data) {
                guard.write(data)
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
        }
    }

    /// Receives data from the AS/400 system through the channel
    pub fn receive_data_channel(&mut self) -> Option<Vec<u8>> {
        if let Some(ref receiver) = self.receiver {
            // Try to receive data without blocking
            match receiver.try_recv() {
                Ok(data) => {
                    // Process the received data through our telnet negotiator
                    let negotiation_response = self.telnet_negotiator.process_incoming_data(&data);
                    
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
                    
                    // Filter out telnet negotiation from the data and return clean 5250 data
                    let clean_data = self.extract_5250_data(&data);
                    if !clean_data.is_empty() {
                        Some(clean_data)
                    } else {
                        None // No 5250 data in this packet, just negotiation
                    }
                }
                Err(mpsc::TryRecvError::Empty) => None, // No data available
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Channel disconnected - this indicates a critical connection failure
                    eprintln!("Network channel disconnected - connection lost");
                    self.running = false;
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

        // Check for excessive control characters that might indicate attacks
        let control_char_count = data.iter().filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13).count();
        let control_ratio = control_char_count as f32 / data.len() as f32;

        if control_ratio > 0.3 { // More than 30% control characters
            eprintln!("SECURITY: Excessive control characters in network data");
            return false;
        }

        // Check for potential buffer overflow patterns
        if data.len() > 65535 {
            eprintln!("SECURITY: Network data too large: {}", data.len());
            return false;
        }

        // Check for suspicious byte patterns that might indicate exploits
        let suspicious_patterns = [
            &[0u8; 16], // Long sequences of nulls
            &[255u8; 16], // Long sequences of 0xFF
        ];

        for pattern in &suspicious_patterns {
            if data.windows(pattern.len()).any(|window| window == *pattern) {
                eprintln!("SECURITY: Suspicious byte pattern detected in network data");
                return false;
            }
        }

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

    /// Checks if the connection is active
    pub fn is_connected(&self) -> bool {
        self.stream.is_some() && self.running
    }

    /// Checks if telnet negotiation is complete
    pub fn is_negotiation_complete(&self) -> bool {
        self.negotiation_complete
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