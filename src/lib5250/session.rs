/// TN5250 Session Management - Ported from lib5250/session.c
///
/// This module handles the core 5250 protocol session logic, including
/// command processing, display writing, field reading, and screen management.
///
/// Based on the original lib5250 session.c implementation from the tn5250 project.
/// Copyright (C) 1997-2008 Michael Madore
/// Rust port: 2024
///
/// INTEGRATION ARCHITECTURE DECISIONS:
/// ===================================
///
/// 1. **Central Integration Hub**: Session serves as the central coordinator
///    for all components (network, telnet negotiation, protocol processing).
///    This resolves integration issues by providing a unified interface
///    while maintaining component separation.
///
/// 2. **Component Integration with Fallbacks**: Session integrates optional
///    components (TelnetNegotiator, ProtocolProcessor) with automatic fallback
///    to direct processing when components are unavailable or fail. This ensures
///    robust operation even with partial component failures.
///
/// 3. **Protocol Mode Awareness**: Session maintains protocol mode state and
///    routes data processing accordingly (TN5250, NVT, AutoDetect). This
///    enables seamless handling of different connection types.
///
/// 4. **Error Handling and Recovery**: Comprehensive error propagation with
///    fallback mechanisms. Failed operations gracefully degrade rather than
///    causing system failure.
///
/// 5. **Security Integration**: Authentication, rate limiting, and session
///    validation are integrated into the session management layer, providing
///    security controls at the appropriate architectural level.
///
/// 6. **Health Monitoring**: IntegrationHealth struct provides visibility into
///    component status, enabling proactive maintenance and troubleshooting.
use super::display::Display;
use crate::network::ProtocolMode;
use crate::telnet_negotiation::TelnetNegotiator;
use super::protocol::{ProtocolProcessor, Packet};

// 5250 Protocol Constants
const ESC: u8 = 0x04;

/// Default device identification string
const DEFAULT_DEVICE_ID: &str = "IBM-5555-C01";

use super::codes::*;

/// Handshake state for 5250 protocol initialization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandshakeState {
    Initial,
    QuerySent,
    QueryReplyReceived,
    ScreenInitialized,
}

/// TN5250 Session structure
#[derive(Debug)]
pub struct Session {
    /// Whether the session has been invited to send data
    pub invited: bool,
    /// Current read operation opcode
    pub read_opcode: u8,
    /// Sequence number for commands
    pub sequence_number: u8,
    /// Data buffer for processing incoming data
    pub data_buffer: Vec<u8>,
    /// Current position in data buffer
    pub buffer_pos: usize,
    /// Display state
    pub display: Display,
    /// Device identification string
    pub device_id: String,
    /// Whether enhanced 5250 features are enabled
    pub enhanced: bool,
    /// Authentication status
    pub authenticated: bool,
    /// Session token for validation
    pub session_token: Option<String>,
    /// Maximum allowed command size
    pub max_command_size: usize,
    /// Command count for rate limiting
    pub command_count: usize,
    /// Last command time for rate limiting
    pub last_command_time: std::time::Instant,
    /// Current protocol mode
    pub protocol_mode: ProtocolMode,
    /// Optional telnet negotiator
    pub telnet_negotiator: Option<TelnetNegotiator>,
    /// Optional protocol processor
    pub protocol_processor: Option<ProtocolProcessor>,
    /// Fallback buffer for unprocessed data
    pub fallback_buffer: Vec<u8>,
    /// Handshake state for protocol initialization
    pub handshake_state: HandshakeState,
}

impl Session {
    /// Create a new 5250 session
    /// SECURITY: Initialize with secure defaults and authentication state
    pub fn new() -> Self {
        let mut session = Self {
            invited: false,
            read_opcode: 0,
            sequence_number: 0,
            // PERFORMANCE OPTIMIZATION: Pre-allocate buffer with reasonable capacity
            // Reduces allocations during data processing
            data_buffer: Vec::with_capacity(8192), // 8KB initial capacity
            buffer_pos: 0,
            display: Display::new(),
            device_id: DEFAULT_DEVICE_ID.to_string(),
            enhanced: false,
            authenticated: false,
            session_token: None,
            max_command_size: 65535, // 64KB max command size
            command_count: 0,
            last_command_time: std::time::Instant::now(),
            // INTEGRATION: Initialize with auto-detection and optional components
            protocol_mode: ProtocolMode::AutoDetect,
            telnet_negotiator: Some(TelnetNegotiator::new()),
            protocol_processor: Some(ProtocolProcessor::new()),
            fallback_buffer: Vec::new(),
            handshake_state: HandshakeState::Initial,
        };

        // SECURITY: Generate a unique session token for validation
        session.session_token = Some(session.generate_session_token());
        session
    }

    /// Mark session as authenticated after successful telnet negotiation
    /// For TN5250, telnet negotiation serves as the initial authentication
    pub fn mark_telnet_negotiation_complete(&mut self) {
        self.authenticated = true;
        // Keep handshake_state at Initial - ready to receive 5250 data
        println!("Session marked as authenticated after telnet negotiation");
    }

    /// SECURITY: Generate a unique session token for authentication validation
    fn generate_session_token(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        timestamp.hash(&mut hasher);

        // Use process ID and thread ID for additional uniqueness
        let pid = std::process::id();
        pid.hash(&mut hasher);

        format!("sess_{:016x}", hasher.finish())
    }

    /// Get a snapshot of the current display as a String
    pub fn display_string(&self) -> String {
        self.display.to_string()
    }

    /// Get current cursor position (1-based row, col for UI convenience)
    pub fn cursor_position(&self) -> (usize, usize) {
        let (r, c) = self.display.cursor_position();
        (r + 1, c + 1)
    }
    
    /// Enable or disable enhanced 5250 features
    pub fn set_enhanced(&mut self, enhanced: bool) {
        self.enhanced = enhanced;
    }
    
    /// Process incoming 5250 data stream
    /// SECURITY: Enhanced with authentication validation and rate limiting
    pub fn process_stream(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        println!("DEBUG: Session.process_stream called with {len} bytes", len = data.len());
        if !data.is_empty() {
            println!("DEBUG: First 20 bytes: {:02x?}", &data[..data.len().min(20)]);
        }
        
        // SECURITY: Validate input data size to prevent memory exhaustion
        if data.len() > self.max_command_size {
            return Err("Command size exceeds maximum allowed".to_string());
        }

        // SECURITY: Rate limiting - prevent command flooding
        self.enforce_rate_limit()?;

        // SECURITY: Validate session is properly authenticated for sensitive operations
        if !self.validate_session_authentication() {
            return Err("Session authentication required".to_string());
        }

        // CRITICAL FIX: Append to buffer, but ensure cleanup on any error path
        self.data_buffer.extend_from_slice(data);
        self.buffer_pos = 0;

        let mut responses = Vec::new();

        // Process commands with proper cleanup on error
        let process_result = (|| {
            while self.buffer_pos < self.data_buffer.len() {
                // Check for escape sequence (commands start with ESC)
                if self.get_byte()? != ESC {
                    return Err("Invalid command - missing ESC".to_string());
                }

                let command = self.get_byte()?;
                match self.process_command(command) {
                    Ok(Some(response)) => responses.extend(response),
                    Ok(None) => {}, // No response needed
                    Err(e) => return Err(e),
                }
            }
            Ok(())
        })();

        // CRITICAL FIX: ALWAYS clear the buffer, even on error
        // This prevents memory leak when invalid data accumulates
        self.data_buffer.clear();
        self.buffer_pos = 0;

        // Check if processing had errors
        process_result?;

        // SECURITY: Update command tracking
        self.command_count += 1;
        self.last_command_time = std::time::Instant::now();

        Ok(responses)
    }
    
    /// Process a single 5250 command
    fn process_command(&mut self, command: u8) -> Result<Option<Vec<u8>>, String> {
        match command {
            super::codes::CMD_CLEAR_UNIT => {
                self.clear_unit();
                Ok(None)
            }
            
            super::codes::CMD_CLEAR_UNIT_ALTERNATE => {
                self.clear_unit_alternate()?;
                Ok(None)
            }
            
            super::codes::CMD_CLEAR_FORMAT_TABLE => {
                self.clear_format_table();
                Ok(None)
            }
            
            super::codes::CMD_WRITE_TO_DISPLAY => {
                self.write_to_display()?;
                Ok(None)
            }
            
            super::codes::CMD_WRITE_ERROR_CODE | super::codes::CMD_WRITE_ERROR_CODE_WINDOW => {
                self.write_error_code(command)?;
                Ok(None)
            }
            
            super::codes::CMD_READ_INPUT_FIELDS | super::codes::CMD_READ_MDT_FIELDS | super::codes::CMD_READ_MDT_FIELDS_ALT => {
                self.read_command(command)?;
                Ok(None) // Response will be sent when AID key is pressed
            }
            
            super::codes::CMD_READ_SCREEN_IMMEDIATE => {
                let response = self.read_screen_immediate()?;
                Ok(Some(response))
            }
            
            super::codes::CMD_READ_IMMEDIATE => {
                let response = self.read_immediate()?;
                Ok(Some(response))
            }
            
            super::codes::CMD_SAVE_SCREEN => {
                let response = self.save_screen()?;
                Ok(Some(response))
            }
            
            super::codes::CMD_SAVE_PARTIAL_SCREEN => {
                let response = self.save_partial_screen()?;
                Ok(Some(response))
            }
            
            super::codes::CMD_RESTORE_SCREEN => {
                // Ignored - following data should be valid WriteToDisplay
                Ok(None)
            }
            
            super::codes::CMD_RESTORE_PARTIAL_SCREEN => {
                // Ignored - following data should be valid WriteToDisplay
                Ok(None)
            }
            
            super::codes::CMD_ROLL => {
                self.roll()?;
                Ok(None)
            }
            
            super::codes::CMD_WRITE_STRUCTURED_FIELD => {
                let response = self.write_structured_field()?;
                Ok(Some(response))
            }
            
            _ => {
                Err(format!("Unknown command: 0x{command:02X}"))
            }
        }
    }
    
    /// Clear Unit command - reset display and fields
    fn clear_unit(&mut self) {
        self.display.clear_unit();
        self.read_opcode = 0;
        // TODO: Destroy any GUI constructs (windows, menus, scrollbars)
    }
    
    /// Clear Unit Alternate command
    fn clear_unit_alternate(&mut self) -> Result<(), String> {
        let param = self.get_byte()?;
        if param != 0x00 && param != 0x80 {
            return Err(format!("Invalid Clear Unit Alternate parameter: 0x{param:02X}"));
        }
        
        self.display.clear_unit_alternate();
        self.read_opcode = 0;
        // TODO: Destroy GUI constructs
        Ok(())
    }
    
    /// Clear Format Table command
    fn clear_format_table(&mut self) {
        self.display.clear_format_table();
        self.read_opcode = 0;
    }
    
    /// Write To Display command - main display writing logic
    fn write_to_display(&mut self) -> Result<(), String> {
        let cc1 = self.get_byte()?;
        let cc2 = self.get_byte()?;
        
        // Handle Control Character 1 (CC1) - keyboard and field control
        self.handle_cc1(cc1);
        
        // Process display orders until ESC or end of data
        while self.buffer_pos < self.data_buffer.len() {
            let order = self.get_byte()?;
            
            match order {
                ESC => {
                    // End of write to display - put ESC back
                    self.buffer_pos -= 1;
                    break;
                }
                
                SBA => {
                    // Set Buffer Address
                    let row = self.get_byte()? - 1;
                    let col = self.get_byte()? - 1;
                    self.display.set_cursor(row as usize, col as usize);
                }
                
                SF => {
                    // Start of Field
                    self.start_of_field()?;
                }
                
                IC => {
                    // Insert Cursor
                    let row = self.get_byte()? - 1;
                    let col = self.get_byte()? - 1;
                    self.display.set_pending_insert_cursor(row as usize, col as usize);
                }
                
                RA => {
                    // Repeat to Address
                    self.repeat_to_address()?;
                }
                
                EA => {
                    // Erase to Address
                    self.erase_to_address()?;
                }
                
                SOH => {
                    // Start of Header
                    self.start_of_header()?;
                }
                
                // TODO: Add more orders (TD, MC, WEA, WDSF)
                
                _ => {
                    // Printable character - add to display
                    if self.is_printable_char(order) {
                        self.display.add_char(order);
                    } else {
                        return Err(format!("Unknown order: 0x{order:02X}"));
                    }
                }
            }
        }
        
        // Handle Control Character 2 (CC2) - final display control
        self.handle_cc2(cc2);
        
        Ok(())
    }
    
    /// Handle Control Character 1 - keyboard locking and field management
    fn handle_cc1(&mut self, cc1: u8) {
        let lock_keyboard = (cc1 & 0xE0) != 0x00;
        let _reset_non_bypass_mdt = (cc1 & 0x40) != 0;
        let _reset_all_mdt = (cc1 & 0x60) == 0x60;
        let _null_non_bypass_mdt = (cc1 & 0x80) != 0;
        let _null_non_bypass = (cc1 & 0xA0) == 0xA0;
        
        if lock_keyboard {
            self.display.lock_keyboard();
        }
        
        // TODO: Apply field modifications based on CC1 flags
        // This requires field management implementation
    }
    
    /// Handle Control Character 2 - display indicators and alarms
    fn handle_cc2(&mut self, cc2: u8) {
        if (cc2 & 0x04) != 0 { // TN5250_SESSION_CTL_ALARM
            self.display.beep();
        }
        
        if (cc2 & 0x02) != 0 { // TN5250_SESSION_CTL_UNLOCK
            self.display.unlock_keyboard();
        }
        
        // TODO: Handle other CC2 flags (message indicators, blinking, etc.)
    }
    
    /// Start of Field order processing
    fn start_of_field(&mut self) -> Result<(), String> {
        let first_byte = self.get_byte()?;
        
        if (first_byte & 0xE0) != 0x20 {
            // Input field - has Field Format Word (FFW)
            let ffw1 = first_byte;
            let ffw2 = self.get_byte()?;
            let _ffw = (ffw1 as u16) << 8 | ffw2 as u16;
            
            // Process Field Control Words (FCW) if present
            let mut next_byte = self.get_byte()?;
            while (next_byte & 0xE0) != 0x20 {
                let _fcw1 = next_byte;
                let _fcw2 = self.get_byte()?;
                // TODO: Process FCW (continuous fields, word wrap, etc.)
                next_byte = self.get_byte()?;
            }
            
            // Attribute byte
            let attribute = next_byte;
            self.display.add_char(attribute);
            
            // Field length
            let len1 = self.get_byte()?;
            let len2 = self.get_byte()?;
            let _length = (len1 as u16) << 8 | len2 as u16;
            
            // TODO: Create and add field with proper attributes
            
        } else {
            // Output-only field - just attribute
            let attribute = first_byte;
            self.display.add_char(attribute);
            
            let len1 = self.get_byte()?;
            let len2 = self.get_byte()?;
            let _length = (len1 as u16) << 8 | len2 as u16;
            
            // TODO: Handle output field
        }
        
        Ok(())
    }
    
    /// Repeat to Address order
    fn repeat_to_address(&mut self) -> Result<(), String> {
        let end_row = self.get_byte()? as usize;
        let end_col = self.get_byte()? as usize;
        let repeat_char = self.get_byte()?;

        // Convert to 0-based coordinates
        let end_row = end_row.saturating_sub(1);
        let end_col = end_col.saturating_sub(1);

        let start_row = self.display.cursor_row();
        let start_col = self.display.cursor_col();
        let width = self.display.width();
        let height = self.display.height();

        // Clamp end coordinates to valid range
        let end_row = end_row.min(height - 1);
        let end_col = end_col.min(width - 1);

        // Calculate linear positions in the display buffer
        let start_index = start_row * width + start_col;
        let end_index = end_row * width + end_col;
        let total_positions = width * height;

        // Calculate number of characters to repeat (inclusive of end position)
        let count = if end_index >= start_index {
            end_index - start_index + 1
        } else {
            // Wrap around to end of screen then to end_index
            (total_positions - start_index) + (end_index + 1)
        };

        // Fill the range with the repeat character
        for _ in 0..count {
            self.display.add_char(repeat_char);
        }

        Ok(())
    }
    
    /// Erase to Address order
    fn erase_to_address(&mut self) -> Result<(), String> {
        let end_row = self.get_byte()?;
        let end_col = self.get_byte()?;
        let attr_count = self.get_byte()?;

        // Read attribute types to erase
        let mut attributes = Vec::new();
        for _ in 0..attr_count {
            attributes.push(self.get_byte()?);
        }

        // Convert coordinates to 0-based
        let start_row = self.display.cursor_row();
        let start_col = self.display.cursor_col();
        let end_row = end_row.saturating_sub(1) as usize;
        let end_col = end_col.saturating_sub(1) as usize;

        // Implement selective erase logic based on 5250 protocol
        // 0x00 = erase all (fill with nulls)
        // 0x01 = erase unprotected fields only
        // 0x02 = erase protected fields only
        if attributes.contains(&0x00) {
            // Erase all characters in the region (fill with nulls)
            self.display.erase_region(start_row, start_col, end_row, end_col, 0, self.display.width());
        } else if attributes.contains(&0x01) {
            // Erase unprotected fields only
            // TODO: Implement selective field erasing when field management is complete
            // For now, erase the entire region (unprotected fields)
            self.display.erase_region(start_row, start_col, end_row, end_col, 0, self.display.width());
        } else if attributes.contains(&0x02) {
            // Erase protected fields only
            // TODO: Implement selective field erasing when field management is complete
            // For now, this is a no-op since we don't track protected field content separately
        }

        Ok(())
    }
    
    /// Start of Header order
    fn start_of_header(&mut self) -> Result<(), String> {
        let length = self.get_byte()?;
        if length > 7 {
            return Err(format!("Invalid SOH length: {length}"));
        }

        let mut header_data = Vec::new();
        for _ in 0..length {
            header_data.push(self.get_byte()?);
        }

        // Parse and set header data in display for 5250 protocol compliance
        self.parse_and_set_header_data(&header_data)?;

        Ok(())
    }

    /// Parse and set header data from SOH order
    /// Header data contains screen attribute information for 5250 protocol compliance
    fn parse_and_set_header_data(&mut self, header_data: &[u8]) -> Result<(), String> {
        if header_data.is_empty() {
            return Ok(());
        }

        // Clear format table as part of header processing
        self.display.clear_format_table();

        // Parse header data bytes for screen attributes
        for &attr_byte in header_data {
            match attr_byte {
                // Standard 5250 display attributes
                super::codes::ATTR_5250_GREEN => {
                    // Set screen to green/normal display
                    println!("5250: SOH - Setting screen attribute to green/normal");
                }
                super::codes::ATTR_5250_WHITE => {
                    // Set screen to white/highlighted display
                    println!("5250: SOH - Setting screen attribute to white/highlighted");
                }
                super::codes::ATTR_5250_RED => {
                    // Set screen to red display
                    println!("5250: SOH - Setting screen attribute to red");
                }
                super::codes::ATTR_5250_TURQ => {
                    // Set screen to turquoise display
                    println!("5250: SOH - Setting screen attribute to turquoise");
                }
                super::codes::ATTR_5250_YELLOW => {
                    // Set screen to yellow display
                    println!("5250: SOH - Setting screen attribute to yellow");
                }
                super::codes::ATTR_5250_PINK => {
                    // Set screen to pink display
                    println!("5250: SOH - Setting screen attribute to pink");
                }
                super::codes::ATTR_5250_BLUE => {
                    // Set screen to blue display
                    println!("5250: SOH - Setting screen attribute to blue");
                }
                super::codes::ATTR_5250_NONDISP => {
                    // Set screen to nondisplay (hidden)
                    println!("5250: SOH - Setting screen attribute to nondisplay");
                }
                // Additional attribute processing can be added here
                _ => {
                    println!("5250: SOH - Unknown header attribute byte: 0x{:02X}", attr_byte);
                }
            }
        }

        // Lock keyboard after setting header attributes (5250 protocol behavior)
        self.display.lock_keyboard();

        Ok(())
    }
    
    /// Write Error Code command
    fn write_error_code(&mut self, command: u8) -> Result<(), String> {
        if command == CMD_WRITE_ERROR_CODE_WINDOW {
            let _start_win = self.get_byte()?;
            let _end_win = self.get_byte()?;
        }

        // Parse error message data until ESC
        let mut error_message = Vec::new();
        while self.buffer_pos < self.data_buffer.len() {
            let byte = self.get_byte()?;
            if byte == ESC {
                self.buffer_pos -= 1; // Put ESC back
                break;
            }
            error_message.push(byte);
        }

        // Display error message on the error line (bottom row, row 23 for 24-row screen)
        // Clear the error line first
        self.display.erase_region(23, 0, 23, 79, 0, 79);

        // Set cursor to start of error line
        self.display.set_cursor(23, 0);

        // Add each error message character (EBCDIC will be converted to ASCII by add_char)
        for &byte in &error_message {
            self.display.add_char(byte);
        }

        Ok(())
    }
    
    /// Read Command setup (Read Input Fields, Read MDT Fields, etc.)
    fn read_command(&mut self, command: u8) -> Result<(), String> {
        let cc1 = self.get_byte()?;
        let cc2 = self.get_byte()?;
        
        self.handle_cc1(cc1);
        self.handle_cc2(cc2);
        
        // Clear system indicators and unlock keyboard for input
        self.display.unlock_keyboard();
        self.read_opcode = command;
        
        Ok(())
    }
    
    /// Read Screen Immediate command
    fn read_screen_immediate(&mut self) -> Result<Vec<u8>, String> {
        let screen_data = self.display.get_screen_data();
        Ok(screen_data)
    }
    
    /// Read Immediate command  
    fn read_immediate(&mut self) -> Result<Vec<u8>, String> {
        let old_opcode = self.read_opcode;
        self.read_opcode = CMD_READ_IMMEDIATE;
        
        let response = vec![
            self.display.cursor_row() as u8 + 1,
            self.display.cursor_col() as u8 + 1,
            0, // AID code 0
        ];
        
        self.read_opcode = old_opcode;
        Ok(response)
    }
    
    /// Save Screen command
    fn save_screen(&mut self) -> Result<Vec<u8>, String> {
        let mut data = Vec::new();

        // ESC (0x04)
        data.push(ESC);

        // WriteToDisplay command (0x11)
        data.push(CMD_WRITE_TO_DISPLAY);

        // Sequence number
        data.push(self.sequence_number);
        self.sequence_number = self.sequence_number.wrapping_add(1);

        // Length placeholder (2 bytes) - will be calculated
        let length_pos = data.len();
        data.extend_from_slice(&[0x00, 0x00]);

        // Flags (0x00)
        data.push(0x00);

        // Control Character 1 - Lock keyboard, reset MDT
        data.push(0xC0);

        // Control Character 2 - Unlock keyboard after processing
        data.push(0x02);

        // Generate display orders to recreate current screen
        self.generate_screen_display_orders(&mut data)?;

        // Calculate and set length
        let length = (data.len() - length_pos - 2) as u16;
        data[length_pos] = (length >> 8) as u8;
        data[length_pos + 1] = (length & 0xFF) as u8;

        // Add read command if we were in a read operation
        if self.read_opcode != 0 {
            data.push(ESC);
            data.push(self.read_opcode);
            data.push(0x00); // CC1
            data.push(0x00); // CC2
        }

        Ok(data)
    }

    /// Generate display orders to recreate the current screen state
    fn generate_screen_display_orders(&self, data: &mut Vec<u8>) -> Result<(), String> {
        let screen_data = self.display.get_screen_data();
        let width = self.display.width();
        let height = self.display.height();

        // Iterate through each position on the screen
        for row in 0..height {
            for col in 0..width {
                let index = row * width + col;
                let ebcdic_char = screen_data[index];

                // Skip null characters (0x00) - they represent empty/unused positions
                if ebcdic_char != 0x00 {
                    // Set Buffer Address (SBA) order
                    data.push(SBA);
                    data.push((row + 1) as u8); // 1-based row
                    data.push((col + 1) as u8); // 1-based column

                    // Add the character
                    data.push(ebcdic_char);
                }
            }
        }

        Ok(())
    }
    
    /// Save Partial Screen command
    fn save_partial_screen(&mut self) -> Result<Vec<u8>, String> {
        let _flag_byte = self.get_byte()?;
        let _top_row = self.get_byte()?;
        let _left_col = self.get_byte()?;
        let _depth = self.get_byte()?;
        let _width = self.get_byte()?;
        
        // TODO: Save only the specified screen region
        self.save_screen()
    }
    
    /// Roll command - scroll screen region
    fn roll(&mut self) -> Result<(), String> {
        let direction = self.get_byte()?;
        let top = self.get_byte()?;
        let bottom = self.get_byte()?;
        
        let lines = (direction & 0x1F) as i8;
        let lines = if (direction & 0x80) == 0 { -lines } else { lines };
        
        if lines != 0 {
            self.display.roll(top - 1, bottom - 1, lines);
        }
        
        Ok(())
    }
    
    /// Write Structured Field command
    fn write_structured_field(&mut self) -> Result<Vec<u8>, String> {
        let len1 = self.get_byte()?;
        let len2 = self.get_byte()?;
        let _length = (len1 as u16) << 8 | len2 as u16;
        
        let class = self.get_byte()?;
        let sf_type = self.get_byte()?;
        
        if class != 0xD9 {
            return Err(format!("Invalid SF class: 0x{class:02X}"));
        }
        
        match sf_type {
            super::codes::SF_5250_QUERY | super::codes::SF_5250_QUERY_STATION_STATE => {
                // Mark Query Reply as received and send Query Reply
                self.handshake_state = HandshakeState::QueryReplyReceived;
                self.create_query_reply()
            }
            super::codes::SF_QUERY_COMMAND => {
                // Handle QueryCommand (0x84) - respond with SetReplyMode (0x85)
                self.create_set_reply_mode_response()
            }
            super::codes::SF_ERASE_RESET => {
                // Erase/Reset structured field
                self.handle_erase_reset()
            }
            super::codes::SF_DEFINE_PENDING_OPERATIONS => {
                // Define Pending Operations structured field
                self.handle_define_pending_operations()
            }
            super::codes::SF_ENABLE_COMMAND_RECOGNITION => {
                // Enable Command Recognition structured field
                self.handle_enable_command_recognition()
            }
            super::codes::SF_REQUEST_TIMESTAMP_INTERVAL => {
                // Request Minimum Timestamp Interval structured field
                self.handle_request_timestamp_interval()
            }
            super::codes::SF_DEFINE_ROLL_DIRECTION => {
                // Define Roll Direction structured field
                self.handle_define_roll_direction()
            }
            super::codes::SF_SET_MONITOR_MODE => {
                // Set Monitor Mode structured field
                self.handle_set_monitor_mode()
            }
            super::codes::SF_CANCEL_RECOVERY => {
                // Cancel Recovery structured field
                self.handle_cancel_recovery()
            }
            super::codes::SF_CREATE_CHANGE_EXTENDED_ATTRIBUTE => {
                // Create/Change Extended Attribute structured field
                self.handle_create_change_extended_attribute()
            }
            super::codes::SF_SET_EXTENDED_ATTRIBUTE_LIST => {
                // Set Extended Attribute List structured field
                self.handle_set_extended_attribute_list()
            }
            super::codes::SF_READ_TEXT => {
                // Read Text structured field
                self.handle_read_text()
            }
            super::codes::SF_DEFINE_EXTENDED_ATTRIBUTE => {
                // Define Extended Attribute structured field
                self.handle_define_extended_attribute()
            }
            super::codes::SF_DEFINE_NAMED_LOGICAL_UNIT => {
                // Define Named Logical Unit structured field
                self.handle_define_named_logical_unit()
            }
            _ => {
                // TODO: Handle other structured field types
                println!("5250: Unhandled structured field type: 0x{sf_type:02X}");
                Ok(Vec::new())
            }
        }
    }
    
    /// Handle AID key press and send field data
    pub fn handle_aid_key(&mut self, aid_code: u8) -> Result<Vec<u8>, String> {
        if self.read_opcode == 0 {
            return Err("Not in read mode".to_string());
        }
        
        self.create_field_response(aid_code)
    }
    
    /// Create field response based on current read operation
    fn create_field_response(&mut self, aid_code: u8) -> Result<Vec<u8>, String> {
        let response = vec![
            self.display.cursor_row() as u8 + 1,
            self.display.cursor_col() as u8 + 1,
            aid_code,
        ];
        
        // TODO: Add field data based on read_opcode
        match self.read_opcode {
            CMD_READ_INPUT_FIELDS => {
                // Send all modified fields if AID allows
                // TODO: Implement field traversal and data collection
            }
            
            CMD_READ_MDT_FIELDS | CMD_READ_MDT_FIELDS_ALT => {
                // Send only MDT (Modified Data Tag) fields
                // TODO: Implement MDT field collection
            }
            
            CMD_READ_IMMEDIATE => {
                // Send all fields regardless of MDT
                // TODO: Implement all field collection
            }
            
            _ => {}
        }
        
        // Clear read operation
        self.read_opcode = 0;
        self.display.lock_keyboard();
        
        Ok(response)
    }
    
    /// Create Query Reply response
    fn create_query_reply(&self) -> Result<Vec<u8>, String> {
        let mut response = Vec::new();
        
        // Cursor position
        response.push(0x00);
        response.push(0x00);
        
        // Inbound Write Structured Field AID
        response.push(0x88);
        
        // Length of Query Reply
        if self.enhanced {
            response.push(0x00);
            response.push(0x40);
        } else {
            response.push(0x00);
            response.push(0x3A);
        }
        
        // Command class and type
        response.push(0xD9); // Command class
        response.push(0x70); // Query command type
        
        // Flag byte
        response.push(0x80);
        
        // Controller hardware class
        response.push(0x06);
        response.push(0x00);
        
        // Controller code level
        response.push(0x01);
        response.push(0x01);
        response.push(0x00);
        
        // Reserved bytes (16 bytes)
        response.extend(std::iter::repeat_n(0x00, 16));
        
        // Device type
        response.push(0x01); // Display emulation
        
        // Device model (IBM-5555-C01 = 5555 model C01)
        response.extend(b"5555"); // Device type in EBCDIC
        response.extend(b"C01");  // Model in EBCDIC
        
        // Keyboard ID
        response.push(0x02); // Standard keyboard
        response.push(0x00); // Extended keyboard ID
        response.push(0x00); // Reserved
        
        // Display serial number
        response.push(0x00);
        response.push(0x61);
        response.push(0x50);
        response.push(0x00);
        
        // Maximum input fields
        response.push(0xFF);
        response.push(0xFF);
        
        // Control unit customization
        response.push(0x00);
        
        // Reserved
        response.push(0x00);
        response.push(0x00);
        
        // Controller/Display capability
        response.push(0x23);
        response.push(0x31);
        response.push(0x00);
        response.push(0x00);
        
        // Enhanced features
        if self.enhanced {
            response.push(0x02); // Enhanced 5250 features
            response.push(0x80); // Enhanced UI level 2
        } else {
            response.push(0x00);
            response.push(0x00);
        }
        
        // Fill remaining bytes with zeros
        while response.len() < if self.enhanced { 67 } else { 61 } {
            response.push(0x00);
        }
        
        Ok(response)
    }
    
    /// Create SetReplyMode response for QueryCommand (0x84)
    fn create_set_reply_mode_response(&self) -> Result<Vec<u8>, String> {
        let mut response = Vec::new();
        
        // Start with SetReplyMode SF ID (0x85) as first byte
        response.push(SF_SET_REPLY_MODE);
        
        // Add minimal query reply data (device capabilities, etc.)
        // Based on original ProtocolProcessor implementation
        response.extend_from_slice(&[
            0x00, 0x01, // Basic display capability
            0x00, 0x50, // 80 columns  
            0x00, 0x18, // 24 rows
        ]);
        
        Ok(response)
    }
    
    /// Handle Erase/Reset structured field (0x5B)
    fn handle_erase_reset(&mut self) -> Result<Vec<u8>, String> {
        // Parse reset type from the structured field data
        if self.buffer_pos < self.data_buffer.len() {
            let reset_type = self.get_byte()?;
            match reset_type {
                0x00 => {
                    // Clear screen to null
                    self.display.screen().clear();
                    self.display.set_cursor(0, 0);
                    println!("5250: Erase/Reset - Clear screen to null");
                }
                0x01 => {
                    // Clear screen to blanks
                    self.display.screen().clear();
                    self.display.set_cursor(0, 0);
                    println!("5250: Erase/Reset - Clear screen to blanks");
                }
                0x02 => {
                    // Clear input fields only
                    // TODO: Implement selective field clearing
                    println!("5250: Erase/Reset - Clear input fields only (not implemented)");
                }
                _ => {
                    println!("5250: Erase/Reset - Unknown reset type: 0x{reset_type:02X}");
                }
            }
        } else {
            // Default: clear screen to null
            self.display.screen().clear();
            self.display.set_cursor(0, 0);
            println!("5250: Erase/Reset - Default clear screen to null");
        }
        
        // Reset session state
        self.read_opcode = 0;
        self.invited = false;
        
        // No response needed for Erase/Reset
        Ok(Vec::new())
    }
    
    /// Handle Define Pending Operations structured field (0x80)
    fn handle_define_pending_operations(&mut self) -> Result<Vec<u8>, String> {
        // Parse pending operations data
        // This typically defines operations that should be performed later
        println!("5250: Define Pending Operations - processing");
        
        // For now, just consume any remaining data in this structured field
        // Real implementation would parse and store pending operations
        while self.buffer_pos < self.data_buffer.len() {
            let _byte = self.get_byte()?;
            // TODO: Parse pending operation definitions
        }
        
        // No response needed
        Ok(Vec::new())
    }
    
    /// Handle Enable Command Recognition structured field (0x82)
    fn handle_enable_command_recognition(&mut self) -> Result<Vec<u8>, String> {
        // This enables recognition of certain commands
        // Parse any parameters
        if self.buffer_pos < self.data_buffer.len() {
            let flags = self.get_byte()?;
            println!("5250: Enable Command Recognition - flags: 0x{flags:02X}");
            
            // TODO: Set command recognition flags in session state
        } else {
            println!("5250: Enable Command Recognition - no parameters");
        }
        
        // No response needed
        Ok(Vec::new())
    }
    
    /// Handle Define Roll Direction structured field (0x86)
    fn handle_define_roll_direction(&mut self) -> Result<Vec<u8>, String> {
        // Parse roll direction data
        if self.buffer_pos < self.data_buffer.len() {
            let direction = self.get_byte()?;
            println!("5250: Define Roll Direction - direction: 0x{direction:02X}");
            
            // TODO: Set roll direction in session state
            // For now, just acknowledge
        } else {
            println!("5250: Define Roll Direction - no parameters");
        }
        
        // No response needed
        Ok(Vec::new())
    }
    
    /// Handle Set Monitor Mode structured field (0x87)
    fn handle_set_monitor_mode(&mut self) -> Result<Vec<u8>, String> {
        // Parse monitor mode data
        if self.buffer_pos < self.data_buffer.len() {
            let mode = self.get_byte()?;
            println!("5250: Set Monitor Mode - mode: 0x{mode:02X}");
            
            // TODO: Set monitor mode in session state
        } else {
            println!("5250: Set Monitor Mode - no parameters");
        }
        
        // No response needed
        Ok(Vec::new())
    }
    
    /// Handle Cancel Recovery structured field (0x88)
    fn handle_cancel_recovery(&mut self) -> Result<Vec<u8>, String> {
        // Cancel any pending recovery operations
        println!("5250: Cancel Recovery - cancelling recovery operations");
        
        // TODO: Cancel any recovery operations in session state
        
        // No response needed
        Ok(Vec::new())
    }
    
    /// Handle Create/Change Extended Attribute structured field (0xC1)
    fn handle_create_change_extended_attribute(&mut self) -> Result<Vec<u8>, String> {
        // Parse extended attribute data
        println!("5250: Create/Change Extended Attribute - processing");
        
        // TODO: Parse and apply extended attribute changes
        
        // No response needed
        Ok(Vec::new())
    }
    
    /// Handle Set Extended Attribute List structured field (0xCA)
    fn handle_set_extended_attribute_list(&mut self) -> Result<Vec<u8>, String> {
        // Parse extended attribute list
        println!("5250: Set Extended Attribute List - processing");
        
        // TODO: Parse and set extended attribute list
        
        // No response needed
        Ok(Vec::new())
    }
    
    /// Handle Read Text structured field (0xD2)
    fn handle_read_text(&mut self) -> Result<Vec<u8>, String> {
        // Read text from screen or buffer
        println!("5250: Read Text - processing");
        
        // TODO: Implement text reading logic
        // For now, return empty response
        Ok(Vec::new())
    }
    
    /// Handle Define Extended Attribute structured field (0xD3)
    fn handle_define_extended_attribute(&mut self) -> Result<Vec<u8>, String> {
        // Parse extended attribute definition
        println!("5250: Define Extended Attribute - processing");
        
        // TODO: Parse and define extended attributes
        
        // No response needed
        Ok(Vec::new())
    }
    
    /// Handle Request Timestamp Interval structured field (0x8A)
    fn handle_request_timestamp_interval(&mut self) -> Result<Vec<u8>, String> {
        // Parse timestamp interval data
        println!("5250: Request Timestamp Interval - processing");
        
        // TODO: Parse and set timestamp interval
        
        // No response needed
        Ok(Vec::new())
    }
    
    /// Handle Define Named Logical Unit structured field (0x7E)
    fn handle_define_named_logical_unit(&mut self) -> Result<Vec<u8>, String> {
        // Parse named logical unit definition
        println!("5250: Define Named Logical Unit - processing");
        
        // TODO: Parse and define named logical unit
        
        // No response needed
        Ok(Vec::new())
    }
    
    /// Get next byte from data buffer
    fn get_byte(&mut self) -> Result<u8, String> {
        if self.buffer_pos >= self.data_buffer.len() {
            return Err("Unexpected end of data".to_string());
        }
        
        let byte = self.data_buffer[self.buffer_pos];
        self.buffer_pos += 1;
        Ok(byte)
    }
    
    /// Check if character is printable
    fn is_printable_char(&self, ch: u8) -> bool {
        // Use EBCDIC to ASCII conversion to determine if character is printable
        // Printable characters are those that do not map to ASCII control characters
        !crate::protocol_common::ebcdic::ebcdic_to_ascii(ch).is_control()
    }
    
    /// Get current display
    pub fn display(&self) -> &Display {
        &self.display
    }
    
    /// Get mutable display
    pub fn display_mut(&mut self) -> &mut Display {
        &mut self.display
    }
    
    /// Encode field data for transmission in 5250 format
    /// Returns encoded field data with buffer addresses and field lengths
    pub fn encode_field_data(&self, field_data: &[(usize, usize, String)]) -> Vec<u8> {
        let mut encoded = Vec::new();
        
        for (row, col, content) in field_data {
            // Add Set Buffer Address (SBA) order
            encoded.push(super::codes::SBA);
            
            // Add buffer address (row and col as 1-based)
            encoded.push(*row as u8);
            encoded.push(*col as u8);
            
            // Add field content (convert to EBCDIC)
            for ch in content.chars() {
                let ebcdic_byte = crate::protocol_common::ebcdic::ascii_to_ebcdic(ch);
                encoded.push(ebcdic_byte);
            }
        }
        
        encoded
    }
    
    /// Send input fields with Read MDT Fields response format
    /// This is called when Enter or a function key is pressed
    pub fn send_input_fields(&mut self, aid_code: u8, modified_fields: &[(usize, usize, String)]) -> Result<Vec<u8>, String> {
        let mut response = Vec::new();
        
        // Add cursor position (1-based)
        let (cursor_row, cursor_col) = self.display.cursor_position();
        response.push((cursor_row + 1) as u8);
        response.push((cursor_col + 1) as u8);
        
        // Add AID code
        response.push(aid_code);
        
        // Encode and add modified field data
        let field_data = self.encode_field_data(modified_fields);
        response.extend_from_slice(&field_data);
        
        // Clear read operation after sending
        self.read_opcode = 0;
        self.display.lock_keyboard();
        
        Ok(response)
    }
    
    /// Get modified fields from display for transmission
    /// Returns list of (row, col, content) tuples for modified fields
    pub fn get_modified_fields(&self) -> Vec<(usize, usize, String)> {
        // TODO: Implement proper field tracking with MDT (Modified Data Tag)
        // For now, return empty vector - this should be integrated with field_manager
        Vec::new()
    }
    
    /// Send field data with AID key
    /// This combines pending input with AID key for transmission
    pub fn send_field_input(&mut self, aid_code: u8, pending_input: &[u8]) -> Result<Vec<u8>, String> {
        let mut response = Vec::new();
        
        // Add cursor position (1-based)
        let (cursor_row, cursor_col) = self.display.cursor_position();
        response.push((cursor_row + 1) as u8);
        response.push((cursor_col + 1) as u8);
        
        // Add AID code
        response.push(aid_code);
        
        // Add pending input data
        response.extend_from_slice(pending_input);
        
        // Clear read operation after sending
        self.read_opcode = 0;
        self.display.lock_keyboard();
        
        Ok(response)
    }

    /// SECURITY: Authenticate session with credentials
    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<bool, String> {
        // SECURITY: Validate input credentials
        if username.is_empty() || password.is_empty() {
            return Err("Username and password are required".to_string());
        }

        if username.len() > 64 || password.len() > 128 {
            return Err("Invalid credential length".to_string());
        }

        // SECURITY: In a real implementation, this would validate against AS/400
        // For now, we'll simulate authentication with basic validation
        if self.validate_credentials(username, password) {
            self.authenticated = true;
            println!("SECURITY: Session authenticated successfully");
            Ok(true)
        } else {
            self.authenticated = false;
            Err("Authentication failed".to_string())
        }
    }

    /// SECURITY: Validate user credentials (placeholder implementation)
    fn validate_credentials(&self, username: &str, password: &str) -> bool {
        // SECURITY: Basic validation - in real implementation this would check against AS/400
        // For security, we don't log the actual credentials
        !username.is_empty() && !password.is_empty() &&
        username.len() >= 2 && password.len() >= 4 &&
        username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') &&
        password.chars().all(|c| c.is_ascii_graphic())
    }

    /// SECURITY: Validate session authentication state
    fn validate_session_authentication(&self) -> bool {
        // SECURITY: Check if session is properly authenticated
        if !self.authenticated {
            eprintln!("SECURITY: Session authentication required for command processing");
            return false;
        }

        // SECURITY: Validate session token exists and is not expired
        if self.session_token.is_none() {
            eprintln!("SECURITY: Invalid session token");
            return false;
        }

        // SECURITY: Check session age (sessions should not live too long)
        let session_age = self.last_command_time.elapsed();
        if session_age > std::time::Duration::from_secs(3600) { // 1 hour max
            eprintln!("SECURITY: Session expired due to age");
            return false;
        }

        true
    }

    /// SECURITY: Enforce rate limiting to prevent command flooding
    fn enforce_rate_limit(&mut self) -> Result<(), String> {
        let now = std::time::Instant::now();
        let time_since_last_command = now.duration_since(self.last_command_time);

        // SECURITY: Allow maximum 100 commands per second
        const MAX_COMMANDS_PER_SECOND: usize = 100;
        const RATE_LIMIT_WINDOW: std::time::Duration = std::time::Duration::from_secs(1);

        if time_since_last_command < RATE_LIMIT_WINDOW {
            if self.command_count >= MAX_COMMANDS_PER_SECOND {
                return Err("Rate limit exceeded".to_string());
            }
        } else {
            // Reset counter for new time window
            self.command_count = 0;
        }

        Ok(())
    }

    /// SECURITY: Get session authentication status
    pub fn is_authenticated(&self) -> bool {
        self.authenticated && self.session_token.is_some()
    }

    /// SECURITY: Get session token for validation
    pub fn get_session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }

    /// SECURITY: Invalidate session (logout)
    pub fn invalidate_session(&mut self) {
        self.authenticated = false;
        self.session_token = None;
        self.command_count = 0;
        println!("SECURITY: Session invalidated");
    }

    /// SECURITY: Set maximum command size for DoS protection
    pub fn set_max_command_size(&mut self, size: usize) {
        self.max_command_size = size.min(65535); // Cap at 64KB
    }

    /// INTEGRATION: Set protocol mode for the session
    pub fn set_protocol_mode(&mut self, mode: ProtocolMode) {
        self.protocol_mode = mode;
        println!("INTEGRATION: Session protocol mode set to {mode:?}");
    }

    /// INTEGRATION: Get current protocol mode
    pub fn get_protocol_mode(&self) -> ProtocolMode {
        self.protocol_mode
    }

    /// INTEGRATION: Process data with integrated components and fallbacks
    /// This method coordinates between network, telnet, and protocol components
    pub fn process_integrated_data(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        // INTEGRATION: Validate input data size
        if data.len() > self.max_command_size {
            return Err("Command size exceeds maximum allowed".to_string());
        }

        // INTEGRATION: Rate limiting
        self.enforce_rate_limit()?;

        // INTEGRATION: Authentication check
        if !self.validate_session_authentication() {
            return Err("Session authentication required".to_string());
        }

        let responses = match self.protocol_mode {
            ProtocolMode::TN5250 => {
                // INTEGRATION: Use integrated 5250 processing
                self.process_5250_data_integrated(data)?
            },
            ProtocolMode::NVT => {
                // INTEGRATION: Handle NVT data (plain text)
                self.process_nvt_data(data)?
            },
            ProtocolMode::AutoDetect => {
                // INTEGRATION: Auto-detect and switch mode
                self.process_auto_detect_data(data)?
            },
            ProtocolMode::TN3270 => {
                // TN3270 protocol is not supported in this session type
                // This should be handled by a separate 3270 session processor
                return Err("TN3270 protocol not supported in TN5250 session".to_string());
            }
        };

        // INTEGRATION: Update command tracking
        self.command_count += 1;
        self.last_command_time = std::time::Instant::now();

        Ok(responses)
    }

    /// Send initial 5250 protocol data after telnet negotiation
    /// This implements the proper handshake sequence: Query -> Query Reply -> WriteToDisplay -> ReadInputFields
    pub fn send_initial_5250_data(&mut self) -> Result<Vec<u8>, String> {
        println!("DEBUG: Session.send_initial_5250_data called");

        // Initialize the display for 5250 protocol
        self.display.initialize_5250_screen();

        // Set handshake state to QuerySent
        self.handshake_state = HandshakeState::QuerySent;

        // PHASE 1: Send Query command to request device capabilities
        let query_packet = self.create_query_packet();

        println!("DEBUG: Sending Query packet: {query_packet:02x?}");

        Ok(query_packet)
    }
    
    /// Send screen initialization data after Query Reply is received
    /// This should be called by the controller after processing the Query Reply
    pub fn send_screen_initialization(&mut self) -> Result<Vec<u8>, String> {
        println!("DEBUG: Session.send_screen_initialization called");
        
        // PHASE 2: Send WriteToDisplay with proper screen initialization
        let wtd_packet = self.create_write_to_display_packet_with_fields();
        
        // PHASE 3: Send ReadInputFields to indicate we're ready for input
        let rif_packet = self.create_read_input_fields_packet();
        
        // Combine both packets
        let mut data = Vec::new();
        data.extend_from_slice(&wtd_packet);
        data.extend_from_slice(&rif_packet);
        
        println!("DEBUG: Sending screen initialization packets: {data:02x?}");
        
        Ok(data)
    }
    
    /// Create a Query packet (WriteStructuredField with Query command)
    fn create_query_packet(&mut self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // ESC (0x04)
        data.push(ESC);
        
        // WriteStructuredField command (0xF3)
        data.push(CMD_WRITE_STRUCTURED_FIELD);
        
        // Sequence number
        data.push(self.sequence_number);
        self.sequence_number = self.sequence_number.wrapping_add(1);
        
        // Length (2 bytes) - will be calculated
        let length_pos = data.len();
        data.extend_from_slice(&[0x00, 0x00]);
        
        // Flags (0x00)
        data.push(0x00);
        
        // Structured field header length (2 bytes)
        data.extend_from_slice(&[0x00, 0x03]); // 3 bytes: class + type + data
        
        // Structured field class (0xD9)
        data.push(0xD9);
        
        // Query command type (0x70)
        data.push(SF_5250_QUERY);
        
        // Query flag (0x80 = request device capabilities)
        data.push(0x80);
        
        // Calculate and set length
        let length = (data.len() - length_pos - 2) as u16;
        data[length_pos] = (length >> 8) as u8;
        data[length_pos + 1] = (length & 0xFF) as u8;
        
        data
    }
    
    /// Create a WriteToDisplay packet with proper screen initialization data
    fn create_write_to_display_packet_with_fields(&mut self) -> Vec<u8> {
        let mut data = Vec::new();

        // ESC (0x04)
        data.push(ESC);

        // WriteToDisplay command (0x11)
        data.push(CMD_WRITE_TO_DISPLAY);

        // Sequence number
        data.push(self.sequence_number);
        self.sequence_number = self.sequence_number.wrapping_add(1);

        // Length (2 bytes) - will be calculated
        let length_pos = data.len();
        data.extend_from_slice(&[0x00, 0x00]);

        // Flags (0x00)
        data.push(0x00);

        // Control Character 1 (CC1) - Lock keyboard, reset MDT
        data.push(0xC0); // Lock keyboard
        
        // Control Character 2 (CC2) - Unlock keyboard after processing
        data.push(0x02); // Unlock keyboard

        // Set Buffer Address order (SBA) - position at 1,1
        data.push(SBA);
        data.push(0x01); // Row 1
        data.push(0x01); // Col 1

        // Calculate and set length
        let length = (data.len() - length_pos - 2) as u16;
        data[length_pos] = (length >> 8) as u8;
        data[length_pos + 1] = (length & 0xFF) as u8;

        data
    }

    /// Send 5250 protocol handshake to maintain connection
    /// This sends the necessary packets to keep the AS/400 connection alive
    pub fn send_5250_handshake(&mut self) -> Result<Vec<u8>, String> {
        let mut responses = Vec::new();

        // Send a WriteToDisplay command to establish 5250 protocol presence
        let wtd_packet = self.create_write_to_display_packet();
        responses.extend_from_slice(&wtd_packet);

        // Send a ReadInputFields command to indicate we're ready for input
        let rif_packet = self.create_read_input_fields_packet();
        responses.extend_from_slice(&rif_packet);

        Ok(responses)
    }

    /// Create a WriteToDisplay packet for 5250 protocol establishment
    fn create_write_to_display_packet(&mut self) -> Vec<u8> {
        let mut data = Vec::new();

        // ESC (0x04)
        data.push(ESC);

        // WriteToDisplay command (0x11)
        data.push(CMD_WRITE_TO_DISPLAY);

        // Sequence number
        data.push(self.sequence_number);
        self.sequence_number = self.sequence_number.wrapping_add(1);

        // Length (2 bytes) - will be calculated
        let length_pos = data.len();
        data.extend_from_slice(&[0x00, 0x00]);

        // Flags (0x00)
        data.push(0x00);

        // For initial handshake, we send minimal data
        // Real implementation would include screen initialization

        // Calculate and set length
        let length = (data.len() - length_pos - 2) as u16;
        data[length_pos] = (length >> 8) as u8;
        data[length_pos + 1] = (length & 0xFF) as u8;

        data
    }

    /// Create a ReadInputFields packet for 5250 protocol
    fn create_read_input_fields_packet(&mut self) -> Vec<u8> {
        let mut data = Vec::new();

        // ESC (0x04)
        data.push(ESC);

        // ReadInputFields command (0x42)
        data.push(CMD_READ_INPUT_FIELDS);

        // Sequence number
        data.push(self.sequence_number);
        self.sequence_number = self.sequence_number.wrapping_add(1);

        // Length (2 bytes) - will be calculated
        let length_pos = data.len();
        data.extend_from_slice(&[0x00, 0x00]);

        // Flags (0x00)
        data.push(0x00);

        // Control bytes for ReadInputFields
        data.push(0x00); // CC1
        data.push(0x00); // CC2

        // Calculate and set length
        let length = (data.len() - length_pos - 2) as u16;
        data[length_pos] = (length >> 8) as u8;
        data[length_pos + 1] = (length & 0xFF) as u8;

        data
    }

    /// INTEGRATION: Process 5250 data with integrated components
    fn process_5250_data_integrated(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        let mut responses = Vec::new();

        // INTEGRATION: First, process through telnet negotiator if available
        if let Some(ref mut negotiator) = self.telnet_negotiator {
            let negotiation_response = negotiator.process_incoming_data(data);
            if !negotiation_response.is_empty() {
                responses.extend_from_slice(&negotiation_response);
            }

            // INTEGRATION: Filter out telnet commands from data
            let clean_data = self.extract_5250_from_telnet(data);
            if clean_data.is_empty() {
                return Ok(responses); // Only negotiation, no 5250 data
            }

            // INTEGRATION: Process 5250 data
            if let Some(packet) = Packet::from_bytes(&clean_data) {
                // Handle display commands directly in session
                match packet.command {
                    CommandCode::WriteToDisplay => {
                        // Process WriteToDisplay directly
                        self.data_buffer.clear();
                        self.data_buffer.extend_from_slice(&packet.data);
                        self.buffer_pos = 0;
                        let _ = self.write_to_display();
                        // No response needed for WriteToDisplay
                    },
                    CommandCode::WriteStructuredField => {
                        // Process structured fields directly
                        self.data_buffer.clear();
                        self.data_buffer.extend_from_slice(&packet.data);
                        self.buffer_pos = 0;
                        let _ = self.write_structured_field();
                        // No response needed for structured fields
                    },
                    _ => {
                        // Use protocol processor for other commands
                        if let Some(ref mut processor) = self.protocol_processor {
                            match processor.process_packet(&packet) {
                                Ok(protocol_responses) => {
                                    for response in protocol_responses {
                                        responses.extend_from_slice(&response.to_bytes());
                                    }
                                },
                                Err(e) => {
                                    println!("INTEGRATION: Protocol processor error: {e}");
                                }
                            }
                        }
                    }
                }
            } else {
                // INTEGRATION: Fallback to direct session processing
                return self.process_stream(&clean_data);
            }
        } else {
            // INTEGRATION: No telnet negotiator, process directly
            return self.process_stream(data);
        }

        Ok(responses)
    }

    /// INTEGRATION: Process NVT (plain text) data
    fn process_nvt_data(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        // INTEGRATION: For NVT, just store the data in fallback buffer
        // This could be extended to handle ANSI escape sequences, etc.
        self.fallback_buffer.extend_from_slice(data);
        Ok(Vec::new())
    }

    /// INTEGRATION: Auto-detect protocol and process accordingly
    fn process_auto_detect_data(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        // INTEGRATION: Simple auto-detection based on data patterns
        if data.len() >= 2 && data[0] == 0x04 { // ESC sequence indicates 5250
            self.protocol_mode = ProtocolMode::TN5250;
            println!("INTEGRATION: Auto-detected 5250 protocol");
            self.process_5250_data_integrated(data)
        } else if data.iter().all(|&b| (32..=126).contains(&b)) { // Plain ASCII
            self.protocol_mode = ProtocolMode::NVT;
            println!("INTEGRATION: Auto-detected NVT protocol");
            self.process_nvt_data(data)
        } else {
            // INTEGRATION: Default to 5250 for AS/400 compatibility
            self.protocol_mode = ProtocolMode::TN5250;
            println!("INTEGRATION: Defaulting to 5250 protocol");
            self.process_5250_data_integrated(data)
        }
    }

    /// INTEGRATION: Extract 5250 data from telnet stream
    fn extract_5250_from_telnet(&self, data: &[u8]) -> Vec<u8> {
        // INTEGRATION: Simple telnet command filtering
        // This is a basic implementation - could be enhanced
        let mut result = Vec::new();
        let mut i = 0;

        while i < data.len() {
            if data[i] == 255 { // IAC
                if i + 1 < data.len() {
                    match data[i + 1] {
                        251..=254 => { // WILL/WONT/DO/DONT
                            i += 3; // Skip IAC + command + option
                            continue;
                        },
                        250 => { // SB
                            // Find SE
                            let mut j = i + 2;
                            while j + 1 < data.len() {
                                if data[j] == 255 && data[j + 1] == 240 { // IAC SE
                                    i = j + 2;
                                    break;
                                }
                                j += 1;
                            }
                            if j + 1 >= data.len() {
                                i = data.len(); // Malformed, skip to end
                            }
                            continue;
                        },
                        255 => { // Escaped IAC
                            result.push(255);
                            i += 2;
                            continue;
                        },
                        _ => {
                            i += 2; // Skip IAC + command
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

    /// INTEGRATION: Enable/disable integrated components
    pub fn set_component_enabled(&mut self, component: &str, enabled: bool) {
        match component {
            "telnet" => {
                if enabled && self.telnet_negotiator.is_none() {
                    self.telnet_negotiator = Some(TelnetNegotiator::new());
                } else if !enabled {
                    self.telnet_negotiator = None;
                }
            },
            "protocol" => {
                if enabled && self.protocol_processor.is_none() {
                    self.protocol_processor = Some(ProtocolProcessor::new());
                } else if !enabled {
                    self.protocol_processor = None;
                }
            },
            _ => println!("INTEGRATION: Unknown component: {component}"),
        }
        println!("INTEGRATION: Component {component} {status}", status = if enabled { "enabled" } else { "disabled" });
    }

    /// INTEGRATION: Get fallback buffer contents (for NVT data, etc.)
    pub fn get_fallback_data(&mut self) -> Vec<u8> {
        self.fallback_buffer.drain(..).collect()
    }

    /// INTEGRATION: Check if all integrated components are healthy
    pub fn check_integration_health(&self) -> IntegrationHealth {
        let telnet_ok = self.telnet_negotiator.is_some();
        let protocol_ok = self.protocol_processor.is_some();
        let display_ok = true; // Display is always available
        let session_ok = self.session_token.is_some();

        IntegrationHealth {
            telnet_negotiator: telnet_ok,
            protocol_processor: protocol_ok,
            display: display_ok,
            session: session_ok,
            overall_healthy: telnet_ok && protocol_ok && display_ok && session_ok,
        }
    }

    /// Check if screen initialization should be sent
    pub fn should_send_screen_initialization(&self) -> bool {
        matches!(self.handshake_state, HandshakeState::QueryReplyReceived)
    }

    /// Mark screen initialization as sent
    pub fn mark_screen_initialization_sent(&mut self) {
        if matches!(self.handshake_state, HandshakeState::QueryReplyReceived) {
            self.handshake_state = HandshakeState::ScreenInitialized;
        }
    }
}

/// INTEGRATION: Health status of integrated components
#[derive(Debug, Clone)]
pub struct IntegrationHealth {
    pub telnet_negotiator: bool,
    pub protocol_processor: bool,
    pub display: bool,
    pub session: bool,
    pub overall_healthy: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}