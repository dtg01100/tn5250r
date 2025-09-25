//! Controller module for handling terminal emulation and network communication
//! 
//! This module orchestrates the terminal emulator, protocol processor, and network connection.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::ansi_processor::AnsiProcessor;
use crate::field_manager::FieldManager;
use crate::network;
use crate::protocol_state;
use crate::keyboard;
use crate::terminal::{TerminalChar, CharAttribute};

/// Core terminal controller responsible for managing the connection and protocol
pub struct TerminalController {
    host: String,
    port: u16,
    connected: bool,
    protocol_state_machine: protocol_state::ProtocolStateMachine,
    network_connection: Option<network::AS400Connection>,
    ansi_processor: AnsiProcessor,
    use_ansi_mode: bool,
    field_manager: FieldManager,
}

impl TerminalController {
    pub fn new() -> Self {
        Self {
            protocol_state_machine: protocol_state::ProtocolStateMachine::new(),
            network_connection: None,
            connected: false,
            host: String::new(),
            port: 23, // Default telnet port for 5250
            ansi_processor: AnsiProcessor::new(),
            use_ansi_mode: false,
            field_manager: FieldManager::new(),
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
    
    /// Request the login screen after negotiation is complete
    pub fn request_login_screen(&mut self) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to AS/400".to_string());
        }
        
        // Send ReadModified command to trigger screen display
        let read_modified_cmd = vec![0xFB]; // ReadModified command from protocol.rs
        self.send_input(&read_modified_cmd)?;
        
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
                // Detect if this looks like ANSI escape sequences
                if !self.use_ansi_mode && self.contains_ansi_sequences(&received_data) {
                    self.use_ansi_mode = true;
                    println!("Detected ANSI sequences - switching to ANSI terminal mode");
                }
                
                if self.use_ansi_mode {
                    // Process as ANSI terminal data
                    self.ansi_processor.process_data(&received_data, &mut self.protocol_state_machine.screen);
                    
                    // Detect fields after processing ANSI data
                    self.field_manager.detect_fields(&self.protocol_state_machine.screen);
                } else {
                    // Process through the 5250 protocol state machine
                    let _ = self.protocol_state_machine.process_data(&received_data);
                    
                    // Detect fields after processing 5250 data
                    self.field_manager.detect_fields(&self.protocol_state_machine.screen);
                }
                
                // Update the terminal screen with connection success message if needed
                if !self.use_ansi_mode && self.protocol_state_machine.screen.to_string().contains("Connecting") {
                    self.protocol_state_machine.screen.clear();
                    self.protocol_state_machine.screen.write_string(&format!("Connected to {}:{}\nReady...\n", self.host, self.port));
                }
            }
        }
        
        Ok(())
    }
    
    /// Detect if data contains ANSI escape sequences
    fn contains_ansi_sequences(&self, data: &[u8]) -> bool {
        // Look for ESC [ sequences (CSI - Control Sequence Introducer)
        for i in 0..data.len().saturating_sub(1) {
            if data[i] == 0x1B && data[i + 1] == b'[' {
                return true;
            }
        }
        false
    }
    
    /// Check if negotiation is complete and request login screen if needed
    pub fn check_and_request_login_screen(&mut self) -> Result<(), String> {
        if let Some(ref conn) = self.network_connection {
            if conn.is_negotiation_complete() {
                // Small delay to ensure negotiation is fully complete
                std::thread::sleep(std::time::Duration::from_millis(100));
                self.request_login_screen()?;
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
    
    /// Navigate to next field
    pub fn next_field(&mut self) {
        self.field_manager.next_field();
    }
    
    /// Navigate to previous field
    pub fn previous_field(&mut self) {
        self.field_manager.previous_field();
    }
    
    /// Type character into active field
    pub fn type_char(&mut self, ch: char) -> Result<(), String> {
        // First get the field ID to avoid borrowing conflicts
        let field_id = if let Some(active_field) = self.field_manager.get_active_field() {
            active_field.id
        } else {
            return Err("No active field".to_string());
        };
        
        // Now get mutable reference and insert character
        if let Some(field) = self.field_manager.get_active_field_mut() {
            let offset = field.content.len();
            if field.insert_char(ch, offset) {
                // Update the screen display
                self.update_field_display(field_id);
                Ok(())
            } else {
                Err("Cannot insert character in this field".to_string())
            }
        } else {
            Err("No active field".to_string())
        }
    }
    
    /// Backspace in active field
    pub fn backspace(&mut self) -> Result<(), String> {
        // First get the field ID to avoid borrowing conflicts
        let field_id = if let Some(active_field) = self.field_manager.get_active_field() {
            active_field.id
        } else {
            return Err("No active field".to_string());
        };
        
        if let Some(field) = self.field_manager.get_active_field_mut() {
            let offset = field.content.len();
            if field.backspace(offset) {
                self.update_field_display(field_id);
                Ok(())
            } else {
                Err("Cannot backspace in this field".to_string())
            }
        } else {
            Err("No active field".to_string())
        }
    }
    
    /// Delete character in active field
    pub fn delete(&mut self) -> Result<(), String> {
        // First get the field ID to avoid borrowing conflicts
        let field_id = if let Some(active_field) = self.field_manager.get_active_field() {
            active_field.id
        } else {
            return Err("No active field".to_string());
        };
        
        if let Some(field) = self.field_manager.get_active_field_mut() {
            let offset = field.content.len();
            if field.delete_char(offset) {
                self.update_field_display(field_id);
                Ok(())
            } else {
                Err("Cannot delete in this field".to_string())
            }
        } else {
            Err("No active field".to_string())
        }
    }
    
    /// Clear active field
    pub fn clear_active_field(&mut self) {
        // First get the field ID to avoid borrowing conflicts
        let field_id = if let Some(active_field) = self.field_manager.get_active_field() {
            active_field.id
        } else {
            return;
        };
        
        if let Some(field) = self.field_manager.get_active_field_mut() {
            field.clear();
            self.update_field_display(field_id);
        }
    }
    
    /// Get field information for display
    pub fn get_fields_info(&self) -> Vec<(String, String, bool)> {
        self.field_manager.get_fields().iter().map(|field| {
            let label = field.label.clone().unwrap_or_else(|| format!("Field {}", field.id));
            let content = field.get_display_content();
            (label, content, field.active)
        }).collect()
    }
    
    /// Update field display on screen
    fn update_field_display(&mut self, field_id: usize) {
        // Find the field and update its display on screen
        if let Some(field) = self.field_manager.get_fields().iter().find(|f| f.id == field_id) {
            let display_content = field.get_display_content();
            
            // Clear the field area first
            for i in 0..field.length {
                if field.start_col + i <= 80 {
                    self.protocol_state_machine.screen.set_char_at(
                        field.start_row - 1, 
                        field.start_col + i - 1, 
                        TerminalChar {
                            character: ' ',
                            attribute: CharAttribute::Normal,
                        }
                    );
                }
            }
            
            // Write the field content
            for (i, ch) in display_content.chars().enumerate() {
                if i < field.length && field.start_col + i <= 80 {
                    self.protocol_state_machine.screen.set_char_at(
                        field.start_row - 1, 
                        field.start_col + i - 1, 
                        TerminalChar {
                            character: ch,
                            attribute: CharAttribute::Normal,
                        }
                    );
                }
            }
            
            // Show cursor in active field
            if field.active && display_content.len() < field.length {
                self.protocol_state_machine.screen.set_char_at(
                    field.start_row - 1,
                    field.start_col + display_content.len() - 1,
                    TerminalChar {
                        character: '_',
                        attribute: CharAttribute::Intensified,
                    }
                );
            }
        }
    }
    
    /// Get field values for form submission
    pub fn get_field_values(&self) -> std::collections::HashMap<String, String> {
        self.field_manager.get_field_values()
    }
    
    /// Validate all fields
    pub fn validate_fields(&self) -> Vec<(String, String)> {
        self.field_manager.validate_all().into_iter()
            .map(|(id, error)| {
                let field_name = self.field_manager.get_fields().iter()
                    .find(|f| f.id == id)
                    .and_then(|f| f.label.clone())
                    .unwrap_or_else(|| format!("Field {}", id));
                (field_name, error)
            })
            .collect()
    }
    
    /// Click/activate field at position
    pub fn activate_field_at_position(&mut self, row: usize, col: usize) -> bool {
        self.field_manager.set_active_field_at_position(row, col)
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
    
    pub fn request_login_screen(&self) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.request_login_screen()
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn send_enter(&self) -> Result<(), String> {
        // Send Enter key (usually mapped to a function key or newline)
        self.send_function_key(keyboard::FunctionKey::Enter)
    }
    
    pub fn backspace(&self) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.field_manager.backspace()
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn delete(&self) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.field_manager.delete()
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn next_field(&self) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.next_field();
            Ok(())
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn previous_field(&self) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.previous_field();
            Ok(())
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn type_char(&self, ch: char) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.type_char(ch)
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn get_fields_info(&self) -> Result<Vec<(String, String, bool)>, String> {
        if let Ok(ctrl) = self.controller.lock() {
            Ok(ctrl.get_fields_info())
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn activate_field_at_position(&self, row: usize, col: usize) -> Result<bool, String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            Ok(ctrl.activate_field_at_position(row, col))
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn get_cursor_position(&self) -> Result<(usize, usize), String> {
        if let Ok(ctrl) = self.controller.lock() {
            Ok(ctrl.field_manager.get_cursor_position())
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn set_cursor_position(&self, row: usize, col: usize) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.field_manager.set_cursor_position(row, col);
            Ok(())
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn click_at_position(&self, row: usize, col: usize) -> Result<bool, String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            Ok(ctrl.field_manager.click_at_position(row, col))
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