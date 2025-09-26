//! Controller module for handling terminal emulation and network communication
//! 
//! This module orchestrates the terminal emulator, protocol processor, and network connection.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::ansi_processor::AnsiProcessor;
use crate::field_manager::FieldManager;
use crate::network;
use crate::lib5250::session::Session;
use crate::keyboard;
use crate::terminal::{TerminalChar, CharAttribute};

/// Core terminal controller responsible for managing the connection and protocol
pub struct TerminalController {
    host: String,
    port: u16,
    connected: bool,
    session: Session,
    network_connection: Option<network::AS400Connection>,
    ansi_processor: AnsiProcessor,
    use_ansi_mode: bool,
    field_manager: FieldManager,
    screen: crate::terminal::TerminalScreen, // Screen management moved to controller level
}

impl TerminalController {
    pub fn new() -> Self {
        Self {
            session: Session::new(),
            network_connection: None,
            connected: false,
            host: String::new(),
            port: 23, // Default telnet port for 5250
            ansi_processor: AnsiProcessor::new(),
            use_ansi_mode: false,
            field_manager: FieldManager::new(),
            screen: crate::terminal::TerminalScreen::new(),
        }
    }
    
    /// Connect with optional TLS override. When `tls_override` is Some, it forces TLS on/off.
    pub fn connect_with_tls(&mut self, host: String, port: u16, tls_override: Option<bool>) -> Result<(), String> {
        // Update internal state
        self.host = host.clone();
        self.port = port;
        
        // Create network connection
        let mut conn = network::AS400Connection::new(host, port);
        if let Some(tls) = tls_override {
            conn.set_tls(tls);
        }
        conn.connect().map_err(|e| e.to_string())?;
        
        // Initialize session
        // Session is already initialized in new()
        
        self.network_connection = Some(conn);
        self.connected = true;
        
        // Update terminal screen with connection message
        self.screen.clear();
        self.screen.write_string(&format!("Connecting to {}:{}...\n", self.host, self.port));
        
        Ok(())
    }

    pub fn connect(&mut self, host: String, port: u16) -> Result<(), String> {
        self.connect_with_tls(host, port, None)
    }
    
    /// Request the login screen after negotiation is complete
    pub fn request_login_screen(&mut self) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to AS/400".to_string());
        }
        
    // Send Read MDT Fields command to trigger screen display
    let read_modified_cmd = vec![crate::lib5250::codes::CMD_READ_MDT_FIELDS];
        self.send_input(&read_modified_cmd)?;
        
        Ok(())
    }
    
    pub fn disconnect(&mut self) {
        if let Some(mut conn) = self.network_connection.take() {
            conn.disconnect();
        }
        self.connected = false;
        // Session cleanup (if needed in the future)

        // Update terminal screen with disconnection message
        self.screen.clear();
        self.screen.write_string("Disconnected from AS/400 system\nReady for new connection...\n");
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
        // Prefer session's display buffer in 5250 mode
        if !self.use_ansi_mode {
            return self.session.display_string();
        }
        // ANSI mode falls back to screen buffer
        self.screen.to_string()
    }

    /// Get the UI cursor position (1-based). In 5250 mode use Session display cursor; in ANSI use screen cursor.
    pub fn ui_cursor_position(&self) -> (usize, usize) {
        if !self.use_ansi_mode {
            return self.session.cursor_position();
        }
        // ANSI mode: cursor comes from TerminalScreen (convert to 1-based)
        (self.screen.cursor_y + 1, self.screen.cursor_x + 1)
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
                    self.ansi_processor.process_data(&received_data, &mut self.screen);

                    // Detect fields after processing ANSI data
                    self.field_manager.detect_fields(&self.screen);
                } else {
                    // Process through the 5250 session processor
                    let _ = self.session.process_stream(&received_data);

                    // Detect fields after processing 5250 data
                    // Use the session display's screen snapshot for field detection
                    let screen_ref = self.session.display().screen_ref();
                    self.field_manager.detect_fields(screen_ref);
                }

                // Update the terminal screen with connection success message if needed
                if !self.use_ansi_mode && self.screen.to_string().contains("Connecting") {
                    self.screen.clear();
                    self.screen.write_string(&format!("Connected to {}:{}\nReady...\n", self.host, self.port));
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
    pub fn next_field(&mut self) -> Result<(), String> {
        match self.field_manager.next_field() {
            Ok(()) => Ok(()),
            Err(error) => Err(error.get_user_message().to_string()),
        }
    }
    
    /// Navigate to previous field
    pub fn previous_field(&mut self) -> Result<(), String> {
        match self.field_manager.previous_field() {
            Ok(()) => Ok(()),
            Err(error) => Err(error.get_user_message().to_string()),
        }
    }
    
    /// Type character into active field
    pub fn type_char(&mut self, ch: char) -> Result<(), String> {
        // For now, update field manager (local echo) until session input path is completed
        let field_id = if let Some(active_field) = self.field_manager.get_active_field() {
            active_field.id
        } else {
            return Err("No active field".to_string());
        };
        if let Some(field) = self.field_manager.get_active_field_mut() {
            let offset = field.content.len();
            match field.insert_char(ch, offset) {
                Ok(_) => {
                    self.update_field_display(field_id);
                    Ok(())
                }
                Err(error) => Err(error.get_user_message().to_string()),
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
            
            // Write into the session display's underlying screen so UI render reflects it
            let screen_ref = self.session.display_mut().screen();

            // Clear the field area first
            for i in 0..field.length {
                if field.start_col + i <= 80 {
                    screen_ref.set_char_at(
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
                    screen_ref.set_char_at(
                        field.start_row - 1,
                        field.start_col + i - 1,
                        TerminalChar {
                            character: ch,
                            attribute: CharAttribute::Normal,
                        }
                    );
                }
            }
            
            // Position the session/display cursor at the insertion point for active field
            if field.active {
                let col = field.start_col + display_content.len();
                if col >= 1 {
                    // Update the cursor in the lib5250 Display so the UI can render it
                    self.session
                        .display_mut()
                        .set_cursor(field.start_row - 1, col - 1);
                }
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
        let activated = self.field_manager.set_active_field_at_position(row, col);
        if activated {
            // Reflect cursor move in session display for 5250 mode rendering
            self.session.display_mut().set_cursor(row - 1, col - 1);
        }
        activated
    }
}

/// Asynchronous terminal controller that handles background networking
pub struct AsyncTerminalController {
    controller: Arc<Mutex<TerminalController>>,
    running: bool,
    handle: Option<thread::JoinHandle<()>>,
    // Async connect state
    connect_in_progress: Arc<AtomicBool>,
    last_connect_error: Arc<Mutex<Option<String>>>,
    cancel_connect_flag: Arc<AtomicBool>,
}

impl AsyncTerminalController {
    pub fn new() -> Self {
        Self {
            controller: Arc::new(Mutex::new(TerminalController::new())),
            running: false,
            handle: None,
            connect_in_progress: Arc::new(AtomicBool::new(false)),
            last_connect_error: Arc::new(Mutex::new(None)),
            cancel_connect_flag: Arc::new(AtomicBool::new(false)),
        }
    }
    
    pub fn connect(&mut self, host: String, port: u16) -> Result<(), String> {
        if self.running {
            self.disconnect();
        }
        
        {
            let mut ctrl = self.controller.lock().unwrap();
            ctrl.connect_with_tls(host, port, None)?;
        }
        
        self.running = true;
        
        // Start background networking thread
        self.start_network_thread();
        
        Ok(())
    }

    /// Non-blocking connect: perform network connect and telnet negotiation on a background thread
    /// Returns immediately; use `is_connected()`/`is_connecting()`/`get_last_connect_error()` to track status
    pub fn connect_async(&mut self, host: String, port: u16) -> Result<(), String> {
        self.connect_async_with_tls(host, port, None)
    }

    /// TLS-aware non-blocking connect with optional TLS override
    pub fn connect_async_with_tls(&mut self, host: String, port: u16, tls_override: Option<bool>) -> Result<(), String> {
        self.connect_async_with_tls_options(host, port, tls_override, None, None)
    }

    /// TLS-aware non-blocking connect with extra TLS options (insecure, ca bundle path)
    pub fn connect_async_with_tls_options(&mut self, host: String, port: u16, tls_override: Option<bool>, tls_insecure: Option<bool>, ca_bundle_path: Option<String>) -> Result<(), String> {
        // If already processing, restart cleanly
        if self.running {
            self.disconnect();
        }
        self.connect_in_progress.store(true, Ordering::SeqCst);
        self.cancel_connect_flag.store(false, Ordering::SeqCst);
        if let Ok(mut err) = self.last_connect_error.lock() {
            *err = None;
        }

        let controller_ref = Arc::clone(&self.controller);
        let connect_flag = Arc::clone(&self.connect_in_progress);
        let err_ref = Arc::clone(&self.last_connect_error);
        let cancel_flag = Arc::clone(&self.cancel_connect_flag);

        // Spawn a single thread that performs connect then enters the processing loop
        let handle = thread::spawn(move || {
            // Do the blocking network connect without holding the controller lock
            let connect_result = (|| {
                // Early cancel check
                if cancel_flag.load(Ordering::SeqCst) {
                    return Err("Connection canceled by user".to_string());
                }

                let mut conn = network::AS400Connection::new(host.clone(), port);
                if let Some(tls) = tls_override {
                    conn.set_tls(tls);
                }
                if let Some(insec) = tls_insecure { conn.set_tls_insecure(insec); }
                if let Some(ref path) = ca_bundle_path { conn.set_tls_ca_bundle_path(path.clone()); }
                // Use a bounded timeout for TCP connect + then telnet negotiation handles its own timeouts
                let timeout = Duration::from_secs(10);
                conn.connect_with_timeout(timeout).map_err(|e| e.to_string())?;

                // Prepare connection message outside of the controller lock to avoid borrow conflicts
                let connected_msg = format!(
                    "Connected to {}:{}\nReady...\n",
                    host, port
                );

                // If cancel requested after successful connect, drop connection and return canceled
                if cancel_flag.load(Ordering::SeqCst) {
                    return Err("Connection canceled by user".to_string());
                }

                // Quick state update under lock
                match controller_ref.lock() {
                    Ok(mut ctrl) => {
                        // Update controller state with established connection
                        ctrl.host = host.clone();
                        ctrl.port = port;
                        ctrl.network_connection = Some(conn);
                        ctrl.connected = true;
                        // Optional: update screen message
                        ctrl.screen.clear();
                        ctrl.screen.write_string(&connected_msg);
                        Ok(())
                    }
                    Err(_) => Err("Controller lock failed".to_string()),
                }
            })();

            // Mark connection attempt finished (success or error)
            connect_flag.store(false, Ordering::SeqCst);

            match connect_result {
                Ok(()) => {
                    // Enter processing loop similar to start_network_thread
                    loop {
                        // Try to lock and process
                        let mut processed = false;
                        match controller_ref.try_lock() {
                            Ok(mut ctrl) => {
                                if ctrl.is_connected() {
                                    let _ = ctrl.process_incoming_data();
                                } else {
                                    break;
                                }
                                processed = true;
                            }
                            Err(_) => {
                                // Could not lock; fall through to sleep
                            }
                        }
                        // Allow cancel to stop the loop if user disconnects quickly
                        if cancel_flag.load(Ordering::SeqCst) {
                            break;
                        }
                        if !processed {
                            // Reduce busy wait when lock contention occurs
                            thread::sleep(Duration::from_millis(5));
                        } else {
                            thread::sleep(Duration::from_millis(50));
                        }
                    }
                }
                Err(e) => {
                    if let Ok(mut err) = err_ref.lock() {
                        *err = Some(e);
                    }
                }
            }
        });

        // Store handle so we can manage lifecycle if needed
        self.handle = Some(handle);
        Ok(())
    }

    /// TLS-aware blocking connect wrapper for symmetry with sync controller
    pub fn connect_with_tls(&mut self, host: String, port: u16, tls_override: Option<bool>) -> Result<(), String> {
        if self.running {
            self.disconnect();
        }
        {
            let mut ctrl = self.controller.lock().unwrap();
            ctrl.connect_with_tls(host, port, tls_override)?;
        }
        self.running = true;
        self.start_network_thread();
        Ok(())
    }

    /// Cancel an in-progress async connection attempt
    pub fn cancel_connect(&self) {
        self.cancel_connect_flag.store(true, Ordering::SeqCst);
        self.connect_in_progress.store(false, Ordering::SeqCst);
        if let Ok(mut err) = self.last_connect_error.lock() {
            *err = Some("Connection canceled by user".to_string());
        }
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
        self.connect_in_progress.store(false, Ordering::SeqCst);
        
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

    /// Returns true if a background connection attempt is in progress
    pub fn is_connecting(&self) -> bool {
        self.connect_in_progress.load(Ordering::SeqCst)
    }

    /// Get the last connect error (if any) and clear it
    pub fn take_last_connect_error(&self) -> Option<String> {
        if let Ok(mut err) = self.last_connect_error.lock() {
            err.take()
        } else {
            None
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
            ctrl.backspace()
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn delete(&self) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.delete()
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn next_field(&self) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.next_field()
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    pub fn previous_field(&self) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.previous_field()
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
            Ok(ctrl.ui_cursor_position())
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
            Ok(ctrl.activate_field_at_position(row, col))
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
    let controller = AsyncTerminalController::new();
        assert!(!controller.is_connected());
    }
}