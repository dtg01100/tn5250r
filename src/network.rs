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

    /// Set TLS to accept invalid certs (insecure). Use with caution.
    pub fn set_tls_insecure(&mut self, insecure: bool) {
        self.tls_insecure = insecure;
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

    /// Build a TLS connector honoring insecure and custom CA bundle options
    fn build_tls_connector(&self) -> IoResult<TlsConnector> {
        let mut builder = TlsConnector::builder();
        if self.tls_insecure {
            // Note: In native-tls, danger_accept_invalid_certs and danger_accept_invalid_hostnames are available on some platforms
            builder.danger_accept_invalid_certs(true);
            builder.danger_accept_invalid_hostnames(true);
        }
        if let Some(ref path) = self.tls_ca_bundle_path {
            match fs::read(path) {
                Ok(bytes) => {
                    // Try to parse as DER first, fall back to PEM by extracting certs
                    // native-tls expects DER; for PEM we attempt to decode common markers
                    if let Ok(cert) = Certificate::from_der(&bytes) {
                        builder.add_root_certificate(cert);
                    } else {
                        // Simple PEM parse: extract sections between BEGIN/END CERTIFICATE
                        if let Ok(text) = String::from_utf8(bytes) {
                            let mut added = false;
                            let marker_begin = "-----BEGIN CERTIFICATE-----";
                            let marker_end = "-----END CERTIFICATE-----";
                            let mut start = 0;
                            while let Some(b) = text[start..].find(marker_begin) {
                                let bpos = start + b + marker_begin.len();
                                if let Some(e) = text[bpos..].find(marker_end) {
                                    let epos = bpos + e;
                                    let b64 = text[bpos..epos].replace('\n', "").replace('\r', "");
                                    if let Ok(der) = base64::decode(b64) {
                                        if let Ok(cert) = Certificate::from_der(&der) {
                                            builder.add_root_certificate(cert);
                                            added = true;
                                        }
                                    }
                                    start = epos + marker_end.len();
                                } else { break; }
                            }
                            if !added {
                                eprintln!("Warning: No certificates parsed from PEM bundle at {}", path);
                            }
                        } else {
                            eprintln!("Warning: CA bundle at {} not valid UTF-8 for PEM parsing", path);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: failed to read CA bundle {}: {}", path, e);
                }
            }
        }
        builder.build().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
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
    fn start_receive_thread(&mut self) {
        if let Some(shared) = &self.stream {
            let shared = Arc::clone(shared);
            let sender = self.sender.clone().unwrap();
            thread::spawn(move || {
                let mut buffer = [0u8; 1024];
                loop {
                    // Lock the stream only for the duration of the read
                    let read_result = {
                        let mut guard = match shared.lock() {
                            Ok(g) => g,
                            Err(poisoned) => poisoned.into_inner(),
                        };
                        guard.read(&mut buffer)
                    };

                    match read_result {
                        Ok(0) => break, // connection closed
                        Ok(n) => {
                            if sender.send(buffer[..n].to_vec()).is_err() {
                                eprintln!("Failed to send received data through channel - connection may be lost");
                                break;
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                            // Just try again
                            thread::sleep(Duration::from_millis(50));
                            continue;
                        }
                        Err(_) => break,
                    }
                }
            });
        }
    }

    /// Disconnects from the AS/400 system
    pub fn disconnect(&mut self) {
        self.running = false;
        self.negotiation_complete = false;
        self.stream = None;
    }

    /// Sends data to the AS/400 system
    pub fn send_data(&mut self, data: &[u8]) -> IoResult<usize> {
        if let Some(ref shared) = self.stream {
            let mut guard = shared.lock().map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Stream lock poisoned"))?;
            guard.write(data)
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
    
    /// Extract non-telnet 5250 data from the received stream
    fn extract_5250_data(&self, data: &[u8]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < data.len() {
            if data[i] == 255 { // IAC
                // Skip telnet commands
                if i + 1 < data.len() {
                    match data[i + 1] {
                        251..=254 => { // WILL, WONT, DO, DONT
                            if i + 2 < data.len() {
                                i += 3; // Skip IAC + command + option
                                continue;
                            }
                        },
                        250 => { // SB (subnegotiation)
                            // Find the SE (end of subnegotiation)
                            let mut j = i + 2;
                            while j + 1 < data.len() {
                                if data[j] == 255 && data[j + 1] == 240 { // IAC SE
                                    i = j + 2;
                                    break;
                                }
                                j += 1;
                            }
                            if j + 1 >= data.len() {
                                // Incomplete subnegotiation, skip rest
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
            }
            
            result.push(data[i]);
            i += 1;
        }
        
        result
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