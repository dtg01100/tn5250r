//! Controller module for handling terminal emulation and network communication
//! 
//! This module orchestrates the terminal emulator, protocol processor, and network connection.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::{network, protocol_state, keyboard};

/// Synchronous terminal controller
pub struct TerminalController {
    protocol_state_machine: protocol_state::ProtocolStateMachine,
    network_connection: Option<network::AS400Connection>,
    connected: bool,
    host: String,
    port: u16,
    input_buffer: Vec<u8>,
}

impl TerminalController {
    pub fn new() -> Self {
        Self {
            protocol_state_machine: protocol_state::ProtocolStateMachine::new(),
            network_connection: None,
            connected: false,
            host: String::new(),
            port: 23, // Default telnet port for 5250
            input_buffer: Vec::new(),
        }
    }
    
    pub fn connect(&mut self, host: String, port: u16) -> Result<(), String> {
        // Update internal state
        self.host = host.clone();
        self.port = port;
        
        // Create network connection
        let mut conn = network::AS400Connection::new(host, port);
        conn.connect().map_err(|e| e.to_string())?;
        
        // Initialize protocol state machine
        self.protocol_state_machine.connect();
        
        self.network_connection = Some(conn);
        self.connected = true;
        
        // Update terminal screen with connection message
        self.protocol_state_machine.screen.clear();
        self.protocol_state_machine.screen.write_string(&format!("Connecting to {}:{}...\n", self.host, self.port));
        
        Ok(())
    }
    
    pub fn disconnect(&mut self) {
        if let Some(mut conn) = self.network_connection.take() {
            conn.disconnect();
        }
        self.connected = false;
        self.protocol_state_machine.disconnect();
        
        // Update terminal screen with disconnection message
        self.protocol_state_machine.screen.clear();
        self.protocol_state_machine.screen.write_string("Disconnected from AS/400 system\nReady for new connection...\n");
    }
    
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    pub fn send_input(&mut self, input: &[u8]) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to AS/400".to_string());
        }
        
        // Add to local input buffer
        self.input_buffer.extend_from_slice(input);
        
        // Send to network
        if let Some(ref mut conn) = self.network_connection {
            conn.send_data(input).map_err(|e| e.to_string())?;
        }
        
        Ok(())
    }
    
    pub fn send_function_key(&mut self, func_key: keyboard::FunctionKey) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to AS/400".to_string());
        }
        
        // Convert function key to protocol bytes
        let key_bytes = func_key.to_bytes();
        self.send_input(&key_bytes)?;
        
        Ok(())
    }
    
    pub fn get_terminal_content(&self) -> String {
        self.protocol_state_machine.screen.to_string()
    }
    
    // Process any incoming data from the network connection
    pub fn process_incoming_data(&mut self) -> Result<(), String> {
        if !self.connected {
            return Ok(());
        }
        
        // Check for incoming data from network
        if let Some(ref mut conn) = self.network_connection {
            if let Some(received_data) = conn.receive_data_channel() {
                // Process the incoming data through the protocol state machine
                let _ = self.protocol_state_machine.process_data(&received_data);
                
                // Update the terminal screen with connection success message
                if self.protocol_state_machine.screen.to_string().contains("Connecting") {
                    self.protocol_state_machine.screen.clear();
                    self.protocol_state_machine.screen.write_string(&format!("Connected to {}:{}\nReady...\n", self.host, self.port));
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_host(&self) -> &str {
        &self.host
    }
    
    pub fn get_port(&self) -> u16 {
        self.port
    }
}

/// Asynchronous terminal controller that handles background networking
pub struct AsyncTerminalController {
    controller: Arc<Mutex<TerminalController>>,
    running: bool,
    handle: Option<thread::JoinHandle<()>>,
}

impl AsyncTerminalController {
    pub fn new() -> Self {
        Self {
            controller: Arc::new(Mutex::new(TerminalController::new())),
            running: false,
            handle: None,
        }
    }
    
    pub fn connect(&mut self, host: String, port: u16) -> Result<(), String> {
        if self.running {
            self.disconnect();
        }
        
        {
            let mut ctrl = self.controller.lock().unwrap();
            ctrl.connect(host, port)?;
        }
        
        self.running = true;
        
        // Start background networking thread
        self.start_network_thread();
        
        Ok(())
    }
    
    fn start_network_thread(&mut self) {
        let controller_ref = Arc::clone(&self.controller);
        
        // Stop any existing thread
        if let Some(_handle) = self.handle.take() {
            // In a real implementation, we'd have a proper shutdown mechanism
            // For now, we'll just let the thread finish naturally
        }
        
        // Create new thread
        let handle = thread::spawn(move || {
            loop {
                // Check if we should continue running
                // In a real implementation, we'd need a shared flag for this
                // For now, we'll just process data if connected
                let mut lock_acquired = false;
                let mut retry_count = 0;
                const MAX_RETRIES: u32 = 3;

                // Retry logic for acquiring the lock
                while retry_count < MAX_RETRIES {
                    match controller_ref.try_lock() {
                        Ok(mut ctrl) => {
                            if ctrl.is_connected() {
                                // Process incoming data
                                let _ = ctrl.process_incoming_data();
                            } else {
                                // If not connected, break out of the loop
                                return;
                            }
                            lock_acquired = true;
                            break;
                        }
                        Err(_) => {
                            retry_count += 1;
                            if retry_count < MAX_RETRIES {
                                // Exponential backoff: wait longer between retries
                                thread::sleep(Duration::from_millis(10 * (1 << retry_count)));
                            }
                        }
                    }
                }

                // If we couldn't acquire the lock after all retries, log and continue
                if !lock_acquired {
                    eprintln!("Warning: Failed to acquire controller lock after {} retries", MAX_RETRIES);
                }

                // Sleep to avoid busy waiting
                thread::sleep(Duration::from_millis(50));
            }
        });
        
        self.handle = Some(handle);
    }
    
    pub fn disconnect(&mut self) {
        {
            let mut ctrl = self.controller.lock().unwrap();
            ctrl.disconnect();
        }
        
        self.running = false;
        
        // Wait for the thread to finish, if it exists
        if let Some(_handle) = self.handle.take() {
            // In a real implementation we'd have proper signaling
            // For now, we'll just let it finish naturally
        }
    }
    
    pub fn is_connected(&self) -> bool {
        if let Ok(ctrl) = self.controller.lock() {
            ctrl.is_connected()
        } else {
            false
        }
    }
    
    pub fn send_input(&self, input: &[u8]) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.send_input(input)
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn send_function_key(&self, func_key: keyboard::FunctionKey) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.send_function_key(func_key)
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn get_terminal_content(&self) -> Result<String, String> {
        if let Ok(ctrl) = self.controller.lock() {
            Ok(ctrl.get_terminal_content())
        } else {
            Err("Controller lock failed".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_creation() {
        let controller = TerminalController::new();
        assert!(!controller.is_connected());
    }

    #[test]
    fn test_async_controller_creation() {
        let mut controller = AsyncTerminalController::new();
        assert!(!controller.is_connected());
    }
}