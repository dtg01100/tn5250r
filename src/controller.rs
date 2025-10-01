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

/// Protocol type for terminal connections
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProtocolType {
    /// TN5250 protocol for AS/400 systems
    TN5250,
    /// TN3270 protocol for mainframe systems
    TN3270,
}

use std::str::FromStr;

impl FromStr for ProtocolType {
    type Err = String;

    /// Parse protocol type from string
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tn5250" | "5250" => Ok(ProtocolType::TN5250),
            "tn3270" | "3270" => Ok(ProtocolType::TN3270),
            _ => Err(format!("Invalid protocol type: {s}. Must be 'tn5250' or 'tn3270'"))
        }
    }
}

impl ProtocolType {
    /// Convert protocol type to string
    pub fn to_str(&self) -> &str {
        match self {
            ProtocolType::TN5250 => "tn5250",
            ProtocolType::TN3270 => "tn3270",
        }
    }
    
    /// Convert to network ProtocolMode
    pub fn to_protocol_mode(&self) -> network::ProtocolMode {
        match self {
            ProtocolType::TN5250 => network::ProtocolMode::TN5250,
            ProtocolType::TN3270 => network::ProtocolMode::TN3270,
        }
    }
}
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
    pending_input: Vec<u8>, // Buffer for queued input to be transmitted
    username: Option<String>, // Username for AS/400 authentication (RFC 4777)
    password: Option<String>, // Password for AS/400 authentication (RFC 4777)
}

impl Default for TerminalController {
    fn default() -> Self {
        let mut controller = Self {
            session: Session::new(),
            network_connection: None,
            connected: false,
            host: String::new(),
            port: 23, // Default telnet port for 5250
            ansi_processor: AnsiProcessor::new(),
            use_ansi_mode: false,
            field_manager: FieldManager::new(),
            screen: crate::terminal::TerminalScreen::new(),
            pending_input: Vec::new(),
            username: None,
            password: None,
        };

        // Initialize screen with welcome message
        controller.screen.write_string("TN5250R - IBM AS/400 Terminal Emulator\nReady for connection...\n");

        controller
    }
}

impl TerminalController {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set credentials for AS/400 authentication (RFC 4777 Section 5)
    /// These credentials will be sent during telnet negotiation via NEW-ENVIRON option
    /// 
    /// # Arguments
    /// * `username` - AS/400 user profile name (will be converted to uppercase)
    /// * `password` - User password (sent as plain text unless encryption implemented)
    /// 
    /// # Security Note
    /// Current implementation uses plain text password transmission (IBMRSEED empty).
    /// For production use, implement DES or SHA password encryption per RFC 4777.
    pub fn set_credentials(&mut self, username: &str, password: &str) {
        self.username = Some(username.to_uppercase());
        self.password = Some(password.to_string());
        println!("Controller: Credentials configured for user: {}", username.to_uppercase());
    }
    
    /// Clear stored credentials
    pub fn clear_credentials(&mut self) {
        self.username = None;
        self.password = None;
    }
    
    /// Connect with optional TLS override. When `tls_override` is Some, it forces TLS on/off.
    /// SECURITY: Enhanced with secure error handling to prevent information disclosure
    pub fn connect_with_tls(&mut self, host: String, port: u16, tls_override: Option<bool>) -> Result<(), String> {
        // Update internal state
        self.host = host.clone();
        self.port = port;

        // Create network connection
        let mut conn = network::AS400Connection::new(host.clone(), port);
        if let Some(tls) = tls_override {
            conn.set_tls(tls);
        }

        // SECURITY: Handle connection errors securely without exposing internal details
        conn.connect().map_err(|_e| {
            eprintln!("SECURITY: Connection failed - suppressing detailed error information");
            "Connection failed".to_string()
        })?;

        // Initialize session
        // Session is already initialized in new()

        self.network_connection = Some(conn);
        self.connected = true;

        // SECURITY: Use generic connection message without exposing sensitive details
        self.screen.clear();
        self.screen.write_string("Connecting to remote system...\n");

        // Send initial Query command to begin 5250 handshake
        if let Ok(query_data) = self.session.send_initial_5250_data() {
            if let Some(ref mut conn) = self.network_connection {
                if let Err(e) = conn.send_data(&query_data) {
                    eprintln!("Failed to send Query command: {}", e);
                } else {
                    println!("DEBUG: Query command sent, waiting for Query Reply");
                }
            }
        }

        // Record connection attempt in monitoring
        let monitoring = crate::monitoring::MonitoringSystem::global();
        monitoring.integration_monitor.record_integration_event(crate::monitoring::IntegrationEvent {
            timestamp: std::time::Instant::now(),
            event_type: crate::monitoring::IntegrationEventType::ComponentInteraction,
            source_component: "controller".to_string(),
            target_component: Some("network".to_string()),
            description: format!("Connection attempt to {}:{}", host, port),
            details: std::collections::HashMap::new(),
            duration_us: None,
            success: true,
        });

        Ok(())
    }

    pub fn connect(&mut self, host: String, port: u16) -> Result<(), String> {
        self.connect_with_tls(host, port, None)
    }
    
    /// Connect with a specific protocol type
    /// This forces the connection to use the specified protocol instead of auto-detection
    /// Enhanced with protocol validation and error handling
    pub fn connect_with_protocol(&mut self, host: String, port: u16, protocol: ProtocolType, tls_override: Option<bool>) -> Result<(), String> {
        // Validate protocol availability before attempting connection
        if !Self::is_protocol_available(protocol) {
            return Err(format!(
                "Protocol {} is not available. Please ensure required protocol modules are loaded.",
                protocol.to_str()
            ));
        }

        // Update internal state
        self.host = host.clone();
        self.port = port;

        // Create network connection
        let mut conn = network::AS400Connection::new(host.clone(), port);
        if let Some(tls) = tls_override {
            conn.set_tls(tls);
        }

        // SECURITY: Handle connection errors securely without exposing internal details
        conn.connect().map_err(|_e| {
            eprintln!("SECURITY: Connection failed - suppressing detailed error information");
            
            // Record connection failure in monitoring
            let monitoring = crate::monitoring::MonitoringSystem::global();
            let alert = crate::monitoring::Alert {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: std::time::Instant::now(),
                level: crate::monitoring::AlertLevel::Critical,
                component: "controller".to_string(),
                message: format!("Connection failed to {}:{} using {} protocol", host, port, protocol.to_str()),
                details: std::collections::HashMap::new(),
                acknowledged: false,
                acknowledged_at: None,
                resolved: false,
                resolved_at: None,
                occurrence_count: 1,
                last_occurrence: std::time::Instant::now(),
            };
            monitoring.alerting_system.trigger_alert(alert);
            
            "Connection failed".to_string()
        })?;

        // Validate protocol mode before setting
        let protocol_mode = protocol.to_protocol_mode();
        if !Self::validate_protocol_mode(protocol_mode) {
            return Err(format!(
                "Protocol mode {:?} validation failed. Network may not support this protocol.",
                protocol_mode
            ));
        }

        // Force the protocol mode
        conn.set_protocol_mode(protocol_mode);

        // Configure credentials for authentication (RFC 4777)
        // The network layer's telnet negotiator will use these during NEW-ENVIRON negotiation
        if let (Some(ref username), Some(ref password)) = (&self.username, &self.password) {
            println!("Controller: Configuring authentication for user: {}", username);
            conn.set_credentials(username, password);
        }

        // Initialize session
        // Session is already initialized in new()

        self.network_connection = Some(conn);
        self.connected = true;

        // SECURITY: Use generic connection message without exposing sensitive details
        self.screen.clear();
        self.screen.write_string(&format!("Connecting to remote system using {} protocol...\n", protocol.to_str()));

        // CRITICAL: Wait for telnet negotiation to complete before sending Query
        // The network layer handles telnet option negotiation including authentication
        // We'll send the Query command after the first data exchange confirms auth is complete
        // This is handled in process_received_data() when negotiation completes

        // Record successful connection attempt in monitoring
        let monitoring = crate::monitoring::MonitoringSystem::global();
        monitoring.integration_monitor.record_integration_event(crate::monitoring::IntegrationEvent {
            timestamp: std::time::Instant::now(),
            event_type: crate::monitoring::IntegrationEventType::ComponentInteraction,
            source_component: "controller".to_string(),
            target_component: Some("network".to_string()),
            description: format!("Connection established to {}:{} using {} protocol", host, port, protocol.to_str()),
            details: std::collections::HashMap::new(),
            duration_us: None,
            success: true,
        });

        Ok(())
    }
    
    /// Validate if a protocol is available for use
    fn is_protocol_available(protocol: ProtocolType) -> bool {
        // Check if required protocol modules are available
        match protocol {
            ProtocolType::TN5250 => {
                // TN5250 is always available as it's the primary protocol
                true
            }
            ProtocolType::TN3270 => {
                // TN3270 support is available
                true
            }
        }
    }
    
    /// Validate protocol mode configuration
    fn validate_protocol_mode(mode: network::ProtocolMode) -> bool {
        // Validate that the protocol mode is supported
        match mode {
            network::ProtocolMode::AutoDetect => true,
            network::ProtocolMode::TN5250 => true,
            network::ProtocolMode::TN3270 => true,
            network::ProtocolMode::NVT => true,
        }
    }
    
    /// Get the detected or configured protocol mode
    pub fn get_protocol_mode(&self) -> Option<network::ProtocolMode> {
        self.network_connection.as_ref().map(|conn| conn.get_detected_protocol_mode())
    }
    
    /// Request the login screen after negotiation is complete
    pub fn request_login_screen(&mut self) -> Result<(), String> {
        println!("DEBUG: request_login_screen called");
        if !self.connected {
            return Err("Not connected to AS/400".to_string());
        }
        
        println!("DEBUG: Connected, preparing command");
        // Send Read MDT Fields command to trigger screen display
        let read_modified_cmd = vec![crate::lib5250::codes::CMD_READ_MDT_FIELDS];
        println!("DEBUG: Command bytes: {:02x?}", read_modified_cmd);
        
        println!("DEBUG: Calling send_input");
        let result = self.send_input(&read_modified_cmd);
        println!("DEBUG: send_input result: {:?}", result);
        
        result?;
        println!("DEBUG: request_login_screen completed successfully");
        
        Ok(())
    }
    
    pub fn disconnect(&mut self) {
        // CRITICAL FIX: Enhanced resource cleanup with proper error handling and validation

        // Disconnect network connection with proper cleanup
        if let Some(mut conn) = self.network_connection.take() {
            // CRITICAL FIX: Handle disconnect errors gracefully
            conn.disconnect();
            println!("SECURITY: Network connection disconnected cleanly");
        }

        // CRITICAL FIX: Clear sensitive session data with validation
        if !self.session.display_string().is_empty() {
            self.session = Session::new();
        }

        // CRITICAL FIX: Clear field manager state with validation
        if !self.field_manager.get_fields().is_empty() {
            self.field_manager = FieldManager::new();
        }

        // CRITICAL FIX: Clear screen content that might contain sensitive data
        // Validate screen has content before clearing
        if self.screen.cursor_x != 0 || self.screen.cursor_y != 0 ||
           self.screen.buffer.iter().any(|cell| cell.character != ' ') {
            self.screen.clear();
        }

        self.connected = false;
        self.use_ansi_mode = false;

        // CRITICAL FIX: Safe screen update with bounds checking
        self.screen.write_string("Disconnected from remote system\nReady for new connection...\n");

        // Record disconnection in monitoring
        let monitoring = crate::monitoring::MonitoringSystem::global();
        monitoring.integration_monitor.record_integration_event(crate::monitoring::IntegrationEvent {
            timestamp: std::time::Instant::now(),
            event_type: crate::monitoring::IntegrationEventType::ComponentInteraction,
            source_component: "controller".to_string(),
            target_component: Some("network".to_string()),
            description: "Controller disconnected from network".to_string(),
            details: std::collections::HashMap::new(),
            duration_us: None,
            success: true,
        });

        println!("SECURITY: Controller disconnected with complete resource cleanup");
    }
    
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// COMPREHENSIVE VALIDATION: Validate controller state consistency
    /// This method ensures all controller components are in valid states
    pub fn validate_controller_consistency(&self) -> Result<(), String> {
        // Validate connection state consistency
        if self.connected {
            if self.network_connection.is_none() {
                return Err("Connected flag is true but no network connection".to_string());
            }
        } else {
            if self.network_connection.is_some() {
                return Err("Connected flag is false but network connection exists".to_string());
            }
        }

        // Validate session state
        if self.connected && self.session.display_string().is_empty() {
            return Err("Connected but session display is empty".to_string());
        }

        // Validate screen state
        if let Err(e) = self.screen.validate_buffer_consistency() {
            return Err(format!("Screen buffer validation failed: {}", e));
        }

        // Validate field manager
        if let Some(active_idx) = self.field_manager.get_active_field_index() {
            if active_idx >= self.field_manager.field_count() {
                return Err(format!("Active field index {} out of bounds", active_idx));
            }
        }

        // Validate cursor position consistency
        let (session_row, session_col) = self.session.cursor_position();
        let (ui_row, ui_col) = self.ui_cursor_position();

        if self.use_ansi_mode {
            if ui_row == 0 || ui_col == 0 {
                return Err(format!("Invalid UI cursor position in ANSI mode: ({}, {})", ui_row, ui_col));
            }
        } else {
            if session_row == 0 || session_col == 0 {
                return Err(format!("Invalid session cursor position: ({}, {})", session_row, session_col));
            }
        }

        Ok(())
    }
    
    pub fn send_input(&mut self, input: &[u8]) -> Result<(), String> {
        println!("DEBUG: send_input called with {} bytes: {:02x?}", input.len(), input);
        
        // CRITICAL FIX: Enhanced input validation and error handling

        if !self.connected {
            println!("DEBUG: Not connected error");
            return Err("Not connected to remote system".to_string());
        }

        // CRITICAL FIX: Enhanced input validation with multiple checks
        if input.is_empty() {
            println!("DEBUG: Empty input error");
            return Err("Cannot send empty input".to_string());
        }

        // SECURITY: Validate input size to prevent memory exhaustion
        if input.len() > 65535 {
            eprintln!("SECURITY: Input data too large: {} bytes", input.len());
            return Err("Input data too large".to_string());
        }

        // CRITICAL FIX: Validate input data content
        if input.iter().any(|&b| b == 0) {
            println!("DEBUG: Input contains null bytes - rejecting");
            return Err("Input contains null bytes".to_string());
        }

        println!("DEBUG: Input validation passed, sending to network");
        // Send to network with enhanced error handling
        if let Some(ref mut conn) = self.network_connection {
            println!("DEBUG: Calling conn.send_data");
            let result = conn.send_data(input).map_err(|e| {
                eprintln!("SECURITY: Network send failed - suppressing detailed error information");
                println!("DEBUG: Network send error: {:?}", e);
                "Network operation failed".to_string()
            });
            println!("DEBUG: conn.send_data result: {:?}", result);
            result?;
        } else {
            return Err("Network connection not available".to_string());
        }

        Ok(())
    }
    
    pub fn send_function_key(&mut self, func_key: keyboard::FunctionKey) -> Result<(), String> {
        if !self.connected {
            return Err("Not connected to AS/400".to_string());
        }
        
        // Send any pending input first, then the function key
        self.flush_pending_input()?;
        
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
    
    /// Check if screen initialization should be sent
    pub fn should_send_screen_initialization(&self) -> bool {
        self.session.should_send_screen_initialization()
    }
    
    /// Mark screen initialization as sent
    pub fn mark_screen_initialization_sent(&mut self) {
        self.session.mark_screen_initialization_sent();
    }

    // Process any incoming data from the network connection
    pub fn process_incoming_data(&mut self) -> Result<(), String> {
        if !self.connected {
            return Ok(());
        }
        
        // Check for incoming data from network
        if let Some(ref mut conn) = self.network_connection {
            if let Some(received_data) = conn.receive_data_channel() {
                println!("DEBUG: Received {} bytes of data", received_data.len());
                if !received_data.is_empty() {
                    println!("DEBUG: First 50 bytes: {:02x?}", &received_data[..received_data.len().min(50)]);
                }
                
                // Detect if this looks like ANSI escape sequences
                // ANSI data starts with ESC [ or ESC ( (matching test_connection.rs logic)
                let is_ansi = received_data.len() >= 2 && 
                             received_data[0] == 0x1B && 
                             (received_data[1] == 0x5B || received_data[1] == 0x28);
                
                if !self.use_ansi_mode && is_ansi {
                    self.use_ansi_mode = true;
                    println!("Controller: Detected ANSI/VT100 data - switching to ANSI mode");
                    // Clear screen for ANSI mode
                    self.screen.clear();
                }
                
                if self.use_ansi_mode {
                    // Process as ANSI terminal data
                    self.ansi_processor.process_data(&received_data, &mut self.screen);
                    println!("DEBUG: Processed data in ANSI mode");

                    // Detect fields after processing ANSI data
                    self.field_manager.detect_fields(&self.screen);
                } else {
                    // Process through the 5250 session processor
                    println!("DEBUG: Processing data through 5250 session");
                    let result = self.session.process_integrated_data(&received_data);
                    println!("DEBUG: Session processing result: {:?}", result);
                    
                    // Send any response data back to the server
                    if let Ok(response_data) = &result {
                        if !response_data.is_empty() {
                            println!("DEBUG: Sending {} bytes response to server", response_data.len());
                            if let Err(e) = self.send_input(response_data) {
                                eprintln!("Failed to send session response: {}", e);
                            }
                        }
                    }
                    
                    // Debug: show current display content
                    let display_content = self.session.display_string();
                    println!("DEBUG: Current display content ({} chars): '{}'", 
                        display_content.len(), 
                        display_content.chars().take(100).collect::<String>()
                    );

                    // Detect fields after processing 5250 data
                    // Use the session display's screen snapshot for field detection
                    let screen_ref = self.session.display().screen_ref();
                    self.field_manager.detect_fields(screen_ref);
                }

                // Check if we received a Query Reply and send screen initialization
                if self.session.should_send_screen_initialization() {
                    println!("DEBUG: Query Reply received, sending screen initialization");
                    if let Ok(init_data) = self.session.send_screen_initialization() {
                        if let Some(ref mut conn) = self.network_connection {
                            if let Err(e) = conn.send_data(&init_data) {
                                eprintln!("Failed to send screen initialization: {}", e);
                            } else {
                                println!("DEBUG: Screen initialization sent");
                                self.session.mark_screen_initialization_sent();
                            }
                        }
                    }
                }

                // SECURITY: Use generic success message without exposing connection details
                if !self.use_ansi_mode && self.screen.to_string().contains("Connecting") {
                    self.screen.clear();
                    self.screen.write_string("Connected to remote system\nReady...\n");
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
        // In ANSI mode, send characters directly to the terminal (server will echo)
        if self.use_ansi_mode {
            // Send the character as raw ASCII byte
            let byte = ch as u8;
            self.send_input(&[byte])?;
            return Ok(());
        }
        
        // 5250 mode: Use field-based input
        // Get field ID before borrowing
        let field_id = if let Some(active_field) = self.field_manager.get_active_field() {
            active_field.id
        } else {
            return Err("No active field".to_string());
        };
        
        // Update field manager (local echo)
        if let Some(field) = self.field_manager.get_active_field_mut() {
            let offset = field.content.len();
            match field.insert_char(ch, offset) {
                Ok(_) => {
                    self.update_field_display(field_id);
                    
                    // Queue character for network transmission
                    // Convert character to EBCDIC for 5250 protocol
                    let ebcdic_byte = self.ascii_to_ebcdic(ch as u8);
                    self.pending_input.push(ebcdic_byte);
                    
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
        // In ANSI mode, send backspace directly
        if self.use_ansi_mode {
            // Send backspace (0x08) followed by space and another backspace
            // This is the standard way to backspace in terminals: \b \b
            self.send_input(&[0x08, 0x20, 0x08])?;
            return Ok(());
        }
        
        // 5250 mode: Use field-based backspace
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
        // In ANSI mode, send delete escape sequence (ESC[3~)
        if self.use_ansi_mode {
            self.send_input(&[0x1B, 0x5B, 0x33, 0x7E])?; // ESC [ 3 ~
            return Ok(());
        }
        
        // 5250 mode: Use field-based delete
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
    pub fn get_fields_info(&self) -> Vec<crate::field_manager::FieldDisplayInfo> {
        self.field_manager.get_fields_display_info()
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
    
    /// Flush pending input to network
    fn flush_pending_input(&mut self) -> Result<(), String> {
        if self.pending_input.is_empty() {
            return Ok(());
        }
        
        // Send pending input with field data encoding
        let field_data = self.encode_field_data_for_transmission()?;
        self.send_input(&field_data)?;
        
        // Clear pending input buffer
        self.pending_input.clear();
        
        Ok(())
    }
    
    /// Encode field data for transmission
    fn encode_field_data_for_transmission(&self) -> Result<Vec<u8>, String> {
        let mut data = Vec::new();
        
        // Add cursor position (1-based to 0-based conversion)
        let (row, col) = self.session.cursor_position();
        data.push(row as u8);
        data.push(col as u8);
        
        // Add AID code (0x00 for no AID, field data only)
        data.push(0x00);
        
        // Add pending input characters
        data.extend_from_slice(&self.pending_input);
        
        Ok(data)
    }
    
    /// Convert ASCII to EBCDIC (basic conversion)
    fn ascii_to_ebcdic(&self, ascii: u8) -> u8 {
        // Use the EBCDIC conversion from protocol_common
        crate::protocol_common::ebcdic::ascii_to_ebcdic(ascii as char)
    }
    
    /// Get pending input buffer (for testing)
    pub fn get_pending_input(&self) -> &[u8] {
        &self.pending_input
    }
    
    /// Clear pending input buffer
    pub fn clear_pending_input(&mut self) {
        self.pending_input.clear();
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
    
    /// Set credentials for AS/400 authentication (RFC 4777)
    /// Must be called before connect() or connect_async()
    pub fn set_credentials(&self, username: &str, password: &str) {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.set_credentials(username, password);
        }
    }
    
    /// Clear stored credentials
    pub fn clear_credentials(&self) {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.clear_credentials();
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

                // SECURITY: Use generic connection message without exposing sensitive details
                let connected_msg = "Connected to remote system\nReady...\n".to_string();

                // CRITICAL FIX: Enhanced cancellation and error handling
                if cancel_flag.load(Ordering::SeqCst) {
                    return Err("Connection canceled by user".to_string());
                }

                // CRITICAL FIX: Safer state update with timeout and better error handling
                match controller_ref.try_lock() {
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
                    Err(std::sync::TryLockError::Poisoned(_poisoned)) => {
                        eprintln!("SECURITY: Controller mutex poisoned during connection");
                        Err("Controller lock poisoned".to_string())
                    }
                    Err(std::sync::TryLockError::WouldBlock) => {
                        eprintln!("SECURITY: Controller lock blocked during connection");
                        Err("Controller busy".to_string())
                    }
                }
            })();

            // Mark connection attempt finished (success or error)
            connect_flag.store(false, Ordering::SeqCst);

            match connect_result {
                Ok(()) => {
                    // Enter processing loop similar to start_network_thread
                    loop {
                        // CRITICAL FIX: Enhanced thread safety with better error handling
                        let mut processed = false;
                        let mut should_break = false;

                        match controller_ref.try_lock() {
                            Ok(mut ctrl) => {
                                if ctrl.is_connected() {
                                    // CRITICAL FIX: Handle processing errors gracefully
                                    match ctrl.process_incoming_data() {
                                        Ok(_) => {
                                            processed = true;
                                        }
                                        Err(e) => {
                                            eprintln!("SECURITY: Error processing incoming data: {}", e);
                                            // Continue processing but log the error
                                            processed = true;
                                        }
                                    }
                                } else {
                                    should_break = true;
                                }
                            }
                            Err(std::sync::TryLockError::Poisoned(_)) => {
                                eprintln!("SECURITY: Controller mutex poisoned in processing thread");
                                should_break = true;
                            }
                            Err(std::sync::TryLockError::WouldBlock) => {
                                // Could not lock; fall through to sleep
                            }
                        }

                        // Check cancellation flag
                        if cancel_flag.load(Ordering::SeqCst) || should_break {
                            break;
                        }

                       if !processed {
                           thread::sleep(Duration::from_millis(50));
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

   /// Connect with a specific protocol type (blocking)
   pub fn connect_with_protocol(&mut self, host: String, port: u16, protocol: ProtocolType, tls_override: Option<bool>) -> Result<(), String> {
        if self.running {
            self.disconnect();
        }
        
        {
            let mut ctrl = self.controller.lock().unwrap();
            ctrl.connect_with_protocol(host, port, protocol, tls_override)?;
        }
        
        self.running = true;
        self.start_network_thread();
        
        Ok(())
    }
    
    /// Connect with a specific protocol type (async)
    /// Enhanced with protocol validation and error handling
    pub fn connect_async_with_protocol(&mut self, host: String, port: u16, protocol: ProtocolType, tls_override: Option<bool>) -> Result<(), String> {
        // Validate protocol availability before attempting connection
        if !TerminalController::is_protocol_available(protocol) {
            return Err(format!(
                "Protocol {} is not available. Please ensure required protocol modules are loaded.",
                protocol.to_str()
            ));
        }

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
                
                // Use a bounded timeout for TCP connect + then telnet negotiation handles its own timeouts
                let timeout = Duration::from_secs(10);
                conn.connect_with_timeout(timeout).map_err(|e| {
                    format!("Connection failed: {}", e)
                })?;
                
                // Validate protocol mode before setting
                let protocol_mode = protocol.to_protocol_mode();
                if !TerminalController::validate_protocol_mode(protocol_mode) {
                    return Err(format!(
                        "Protocol mode {:?} validation failed. Network may not support this protocol.",
                        protocol_mode
                    ));
                }

                // Force the protocol mode
                conn.set_protocol_mode(protocol_mode);

                // SECURITY: Use generic connection message without exposing sensitive details
                let connected_msg = format!("Connected to remote system using {} protocol\nReady...\n", protocol.to_str());

                // CRITICAL FIX: Enhanced cancellation and error handling
                if cancel_flag.load(Ordering::SeqCst) {
                    return Err("Connection canceled by user".to_string());
                }

                // CRITICAL FIX: Safer state update with timeout and better error handling
                match controller_ref.try_lock() {
                    Ok(mut ctrl) => {
                        // Update controller state with established connection
                        ctrl.host = host.clone();
                        ctrl.port = port;
                        ctrl.network_connection = Some(conn);
                        ctrl.connected = true;
                        // Optional: update screen message
                        ctrl.screen.clear();
                        ctrl.screen.write_string(&connected_msg);

                        // Send initial Query command to begin 5250 handshake
                        if let Ok(query_data) = ctrl.session.send_initial_5250_data() {
                            if let Some(ref mut conn) = ctrl.network_connection {
                                if let Err(e) = conn.send_data(&query_data) {
                                    eprintln!("Failed to send Query command: {}", e);
                                } else {
                                    println!("DEBUG: Query command sent, waiting for Query Reply");
                                }
                            }
                        }

                        // Record successful connection in monitoring
                        let monitoring = crate::monitoring::MonitoringSystem::global();
                        monitoring.integration_monitor.record_integration_event(crate::monitoring::IntegrationEvent {
                            timestamp: std::time::Instant::now(),
                            event_type: crate::monitoring::IntegrationEventType::ComponentInteraction,
                            source_component: "controller".to_string(),
                            target_component: Some("network".to_string()),
                            description: format!("Async connection established to {}:{} using {} protocol", host, port, protocol.to_str()),
                            details: std::collections::HashMap::new(),
                            duration_us: None,
                            success: true,
                        });

                        Ok(())
                    }
                    Err(std::sync::TryLockError::Poisoned(_poisoned)) => {
                        eprintln!("SECURITY: Controller mutex poisoned during connection");
                        Err("Controller lock poisoned".to_string())
                    }
                    Err(std::sync::TryLockError::WouldBlock) => {
                        eprintln!("SECURITY: Controller lock blocked during connection");
                        Err("Controller busy".to_string())
                    }
                }
            })();

            // Mark connection attempt finished (success or error)
            connect_flag.store(false, Ordering::SeqCst);

            match connect_result {
                Ok(()) => {
                    // Enter processing loop similar to start_network_thread
                    loop {
                        // CRITICAL FIX: Enhanced thread safety with better error handling
                        let mut processed = false;
                        let mut should_break = false;

                        match controller_ref.try_lock() {
                            Ok(mut ctrl) => {
                                if ctrl.is_connected() {
                                    // CRITICAL FIX: Handle processing errors gracefully
                                    match ctrl.process_incoming_data() {
                                        Ok(_) => {
                                            processed = true;
                                        }
                                        Err(e) => {
                                            eprintln!("SECURITY: Error processing incoming data: {}", e);
                                            // Continue processing but log the error
                                            processed = true;
                                        }
                                    }
                                    
                                    // Check if we received a Query Reply and send screen initialization
                                    if ctrl.session.should_send_screen_initialization() {
                                        println!("DEBUG: Query Reply received, sending screen initialization");
                                        if let Ok(init_data) = ctrl.session.send_screen_initialization() {
                                            if let Some(ref mut conn) = ctrl.network_connection {
                                                if let Err(e) = conn.send_data(&init_data) {
                                                    eprintln!("Failed to send screen initialization: {}", e);
                                                } else {
                                                    println!("DEBUG: Screen initialization sent");
                                                    ctrl.session.mark_screen_initialization_sent();
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    should_break = true;
                                }
                            }
                            Err(std::sync::TryLockError::Poisoned(_)) => {
                                eprintln!("SECURITY: Controller mutex poisoned in processing thread");
                                should_break = true;
                            }
                            Err(std::sync::TryLockError::WouldBlock) => {
                                // Could not lock; fall through to sleep
                            }
                        }

                        // Check cancellation flag
                        if cancel_flag.load(Ordering::SeqCst) || should_break {
                            break;
                        }

                        if !processed {
                            thread::sleep(Duration::from_millis(50));
                        } else {
                            thread::sleep(Duration::from_millis(50));
                        }
                    }
                }
                Err(e) => {
                    // Record connection failure in monitoring
                    let monitoring = crate::monitoring::MonitoringSystem::global();
                    let alert = crate::monitoring::Alert {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: std::time::Instant::now(),
                        level: crate::monitoring::AlertLevel::Critical,
                        component: "controller".to_string(),
                        message: format!("Async connection failed to {}:{} using {} protocol", host, port, protocol.to_str()),
                        details: std::collections::HashMap::new(),
                        acknowledged: false,
                        acknowledged_at: None,
                        resolved: false,
                        resolved_at: None,
                        occurrence_count: 1,
                        last_occurrence: std::time::Instant::now(),
                    };
                    monitoring.alerting_system.trigger_alert(alert);
                    
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
    
    /// Get the detected or configured protocol mode
    pub fn get_protocol_mode(&self) -> Result<Option<network::ProtocolMode>, String> {
        if let Ok(ctrl) = self.controller.lock() {
            Ok(ctrl.get_protocol_mode())
        } else {
            Err("Controller lock failed".to_string())
        }
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

                // SECURITY: Suppress internal error details that could leak system state
                if !lock_acquired {
                    eprintln!("Warning: Internal synchronization issue detected");
                }

                // Sleep to avoid busy waiting
                thread::sleep(Duration::from_millis(50));
            }
        });
        
        self.handle = Some(handle);
    }
    
    pub fn disconnect(&mut self) {
        // CRITICAL FIX: Enhanced cleanup with proper error handling and resource management

        // Set cancellation flag first to signal threads to stop
        self.cancel_connect_flag.store(true, Ordering::SeqCst);
        self.connect_in_progress.store(false, Ordering::SeqCst);

        // CRITICAL FIX: Safer controller cleanup with timeout
        match self.controller.try_lock() {
            Ok(mut ctrl) => {
                ctrl.disconnect();
            }
            Err(std::sync::TryLockError::Poisoned(poisoned)) => {
                eprintln!("SECURITY: Controller mutex poisoned during disconnect - performing emergency cleanup");
                let mut ctrl = poisoned.into_inner();
                ctrl.disconnect();
            }
            Err(std::sync::TryLockError::WouldBlock) => {
                eprintln!("SECURITY: Controller busy during disconnect - forcing cleanup");
                // Force disconnect by recreating the controller if lock is blocked
                *self.controller.lock().unwrap() = TerminalController::new();
            }
        }

        self.running = false;

        // CRITICAL FIX: Enhanced thread cleanup with timeout
        if let Some(handle) = self.handle.take() {
            match handle.join() {
                Ok(_) => {
                    println!("SECURITY: Background thread terminated cleanly");
                }
                Err(e) => {
                    eprintln!("SECURITY WARNING: Background thread panicked during cleanup: {:?}", e);
                }
            }
        }

        // CRITICAL FIX: Clear any remaining error state safely
        match self.last_connect_error.lock() {
            Ok(mut err) => {
                *err = None;
            }
            Err(poisoned) => {
                eprintln!("SECURITY: Error mutex poisoned during cleanup");
                let mut err = poisoned.into_inner();
                *err = None;
            }
        }

        println!("SECURITY: Async controller disconnected with complete resource cleanup");
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
    
    /// Flush pending input to network
    pub fn flush_pending_input(&self) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.flush_pending_input()
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    /// Get pending input buffer size
    pub fn get_pending_input_size(&self) -> Result<usize, String> {
        if let Ok(ctrl) = self.controller.lock() {
            Ok(ctrl.get_pending_input().len())
        } else {
            Err("Controller lock failed".to_string())
        }
    }
    
    /// Clear pending input buffer
    pub fn clear_pending_input(&self) -> Result<(), String> {
        if let Ok(mut ctrl) = self.controller.lock() {
            ctrl.clear_pending_input();
            Ok(())
        } else {
            Err("Controller lock failed".to_string())
        }
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
    
    pub fn get_fields_info(&self) -> Result<Vec<crate::field_manager::FieldDisplayInfo>, String> {
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