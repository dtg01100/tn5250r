//! Network module for handling AS/400 connections using the 5250 protocol
//! 
//! This module provides the TCP networking functionality for connecting
//! to AS/400 systems using the TN5250 protocol.

use std::net::TcpStream;
use std::io::{Read, Write, Result as IoResult};
use std::time::Duration;
use std::sync::mpsc;
use std::thread;

use crate::telnet_negotiation::TelnetNegotiator;

/// Represents a connection to an AS/400 system
pub struct AS400Connection {
    stream: Option<TcpStream>,
    host: String,
    port: u16,
    receiver: Option<mpsc::Receiver<Vec<u8>>>,
    sender: Option<mpsc::Sender<Vec<u8>>>,
    running: bool,
    telnet_negotiator: TelnetNegotiator,
    negotiation_complete: bool,
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
        }
    }

    /// Connects to the AS/400 system
    pub fn connect(&mut self) -> IoResult<()> {
        let address = format!("{}:{}", self.host, self.port);
        let mut stream = TcpStream::connect(&address)?;
        
        // Set read/write timeouts
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
        
        // Perform RFC-compliant telnet option negotiation
        self.perform_telnet_negotiation(&mut stream)?;
        
        self.stream = Some(stream);
        self.running = true;
        
        // Start a background thread to receive data
        self.start_receive_thread();
        
        Ok(())
    }
    
    /// Performs RFC-compliant telnet option negotiation
    fn perform_telnet_negotiation(&mut self, stream: &mut TcpStream) -> IoResult<()> {
        // Send initial negotiation requests
        let initial_negotiation = self.telnet_negotiator.generate_initial_negotiation();
        if !initial_negotiation.is_empty() {
            println!("Sending initial telnet negotiation ({} bytes)", initial_negotiation.len());
            stream.write_all(&initial_negotiation)?;
            stream.flush()?;
        }
        
        // Wait for negotiation responses with timeout
        stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
        
        let mut negotiation_attempts = 0;
        const MAX_NEGOTIATION_ATTEMPTS: usize = 50;
        
        while !self.telnet_negotiator.is_negotiation_complete() && negotiation_attempts < MAX_NEGOTIATION_ATTEMPTS {
            let mut buffer = [0u8; 1024];
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Connection closed during negotiation");
                    break;
                }
                Ok(n) => {
                    println!("Received negotiation data ({} bytes): {:?}", n, &buffer[..n.min(20)]);
                    let response = self.telnet_negotiator.process_incoming_data(&buffer[..n]);
                    
                    if !response.is_empty() {
                        println!("Sending negotiation response ({} bytes)", response.len());
                        stream.write_all(&response)?;
                        stream.flush()?;
                    }
                    
                    negotiation_attempts += 1;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                    println!("Negotiation timeout or would block, continuing...");
                    break;
                }
                Err(e) => {
                    println!("Negotiation error: {}", e);
                    return Err(e);
                }
            }
        }
        
        // Reset to non-blocking mode
        stream.set_read_timeout(None)?;
        
        self.negotiation_complete = true;
        println!("Telnet negotiation completed after {} attempts", negotiation_attempts);
        Ok(())
    }
    
    /// Starts a background thread to receive data
    fn start_receive_thread(&mut self) {
        if let Some(stream) = &self.stream {
            match stream.try_clone() {
                Ok(mut stream_clone) => {
                    let sender = self.sender.clone().unwrap();
                    thread::spawn(move || {
                        let mut buffer = [0; 1024];
                        loop {
                            match stream_clone.read(&mut buffer) {
                                Ok(0) => break, // Connection closed
                                Ok(n) => {
                                    // Send received data through the channel
                                    match sender.send(buffer[..n].to_vec()) {
                                        Ok(()) => {
                                            // Data sent successfully
                                        }
                                        Err(_) => {
                                            // Channel send failed - receiver disconnected or channel full
                                            eprintln!("Failed to send received data through channel - connection may be lost");
                                            break;
                                        }
                                    }
                                }
                                Err(_) => break, // Error occurred
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to clone stream for receive thread: {}", e);
                    // Don't start the thread if cloning fails
                }
            }
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
        if let Some(ref mut stream) = self.stream {
            stream.write(data)
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
                        if let Some(ref mut stream) = self.stream {
                            if let Err(e) = stream.write_all(&negotiation_response) {
                                eprintln!("Failed to send telnet negotiation response: {}", e);
                            } else if let Err(e) = stream.flush() {
                                eprintln!("Failed to flush telnet negotiation response: {}", e);
                            } else {
                                println!("Sent telnet negotiation response ({} bytes)", negotiation_response.len());
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
}