/// TN5250 Session Management - Ported from lib5250/session.c
///
/// This module handles the core 5250 protocol session logic, including
/// command processing, display writing, field reading, and screen management.
///
/// Based on the original lib5250 session.c implementation from the tn5250 project.
/// Copyright (C) 1997-2008 Michael Madore
/// Rust port: 2024

use super::codes::*;
use super::display::Display;

/// Maximum number of input fields supported
const MAX_INPUT_FIELDS: usize = 65535;

/// Default device identification string
const DEFAULT_DEVICE_ID: &str = "IBM-3179-2";

/// Session state for 5250 protocol processing
#[derive(Debug)]
pub struct Session {
    /// Whether session is currently invited (unlocked for input)
    pub invited: bool,
    
    /// Current read operation command code (if any)
    pub read_opcode: u8,
    
    /// Buffer for incoming data stream
    data_buffer: Vec<u8>,
    
    /// Current position in data buffer
    buffer_pos: usize,
    
    /// Display buffer for terminal operations
    display: Display,
    
    /// Device identification string
    device_id: String,
    
    /// Whether enhanced 5250 features are enabled
    enhanced: bool,
}

impl Session {
    /// Create a new 5250 session
    pub fn new() -> Self {
        Self {
            invited: false,
            read_opcode: 0,
            data_buffer: Vec::new(),
            buffer_pos: 0,
            display: Display::new(),
            device_id: DEFAULT_DEVICE_ID.to_string(),
            enhanced: false,
        }
    }
    
    /// Enable or disable enhanced 5250 features
    pub fn set_enhanced(&mut self, enhanced: bool) {
        self.enhanced = enhanced;
    }
    
    /// Process incoming 5250 data stream
    pub fn process_stream(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        self.data_buffer.extend_from_slice(data);
        self.buffer_pos = 0;
        
        let mut responses = Vec::new();
        
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
        
        // Clear processed data
        self.data_buffer.clear();
        self.buffer_pos = 0;
        
        Ok(responses)
    }
    
    /// Process a single 5250 command
    fn process_command(&mut self, command: u8) -> Result<Option<Vec<u8>>, String> {
        match command {
            CMD_CLEAR_UNIT => {
                self.clear_unit();
                Ok(None)
            }
            
            CMD_CLEAR_UNIT_ALTERNATE => {
                self.clear_unit_alternate()?;
                Ok(None)
            }
            
            CMD_CLEAR_FORMAT_TABLE => {
                self.clear_format_table();
                Ok(None)
            }
            
            CMD_WRITE_TO_DISPLAY => {
                self.write_to_display()?;
                Ok(None)
            }
            
            CMD_WRITE_ERROR_CODE | CMD_WRITE_ERROR_CODE_WINDOW => {
                self.write_error_code(command)?;
                Ok(None)
            }
            
            CMD_READ_INPUT_FIELDS | CMD_READ_MDT_FIELDS | CMD_READ_MDT_FIELDS_ALT => {
                self.read_command(command)?;
                Ok(None) // Response will be sent when AID key is pressed
            }
            
            CMD_READ_SCREEN_IMMEDIATE => {
                let response = self.read_screen_immediate()?;
                Ok(Some(response))
            }
            
            CMD_READ_IMMEDIATE => {
                let response = self.read_immediate()?;
                Ok(Some(response))
            }
            
            CMD_SAVE_SCREEN => {
                let response = self.save_screen()?;
                Ok(Some(response))
            }
            
            CMD_SAVE_PARTIAL_SCREEN => {
                let response = self.save_partial_screen()?;
                Ok(Some(response))
            }
            
            CMD_RESTORE_SCREEN => {
                // Ignored - following data should be valid WriteToDisplay
                Ok(None)
            }
            
            CMD_RESTORE_PARTIAL_SCREEN => {
                // Ignored - following data should be valid WriteToDisplay
                Ok(None)
            }
            
            CMD_ROLL => {
                self.roll()?;
                Ok(None)
            }
            
            CMD_WRITE_STRUCTURED_FIELD => {
                let response = self.write_structured_field()?;
                Ok(Some(response))
            }
            
            _ => {
                Err(format!("Unknown command: 0x{:02X}", command))
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
            return Err(format!("Invalid Clear Unit Alternate parameter: 0x{:02X}", param));
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
                        return Err(format!("Unknown order: 0x{:02X}", order));
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
        let reset_non_bypass_mdt = (cc1 & 0x40) != 0;
        let reset_all_mdt = (cc1 & 0x60) == 0x60;
        let null_non_bypass_mdt = (cc1 & 0x80) != 0;
        let null_non_bypass = (cc1 & 0xA0) == 0xA0;
        
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
            let ffw = (ffw1 as u16) << 8 | ffw2 as u16;
            
            // Process Field Control Words (FCW) if present
            let mut next_byte = self.get_byte()?;
            while (next_byte & 0xE0) != 0x20 {
                let fcw1 = next_byte;
                let fcw2 = self.get_byte()?;
                // TODO: Process FCW (continuous fields, word wrap, etc.)
                next_byte = self.get_byte()?;
            }
            
            // Attribute byte
            let attribute = next_byte;
            self.display.add_char(attribute);
            
            // Field length
            let len1 = self.get_byte()?;
            let len2 = self.get_byte()?;
            let length = (len1 as u16) << 8 | len2 as u16;
            
            // TODO: Create and add field with proper attributes
            
        } else {
            // Output-only field - just attribute
            let attribute = first_byte;
            self.display.add_char(attribute);
            
            let len1 = self.get_byte()?;
            let len2 = self.get_byte()?;
            let length = (len1 as u16) << 8 | len2 as u16;
            
            // TODO: Handle output field
        }
        
        Ok(())
    }
    
    /// Repeat to Address order
    fn repeat_to_address(&mut self) -> Result<(), String> {
        let end_row = self.get_byte()?;
        let end_col = self.get_byte()?;
        let repeat_char = self.get_byte()?;
        
        // TODO: Implement repeat logic
        // For now, just add the character once
        self.display.add_char(repeat_char);
        
        Ok(())
    }
    
    /// Erase to Address order
    fn erase_to_address(&mut self) -> Result<(), String> {
        let end_row = self.get_byte()?;
        let end_col = self.get_byte()?;
        let attr_count = self.get_byte()?;
        
        // Read attribute types to erase
        let mut attributes = Vec::new();
        for _ in 1..attr_count {
            attributes.push(self.get_byte()?);
        }
        
        // TODO: Implement selective erase logic
        // For now, erase everything if 0xFF is specified
        if attributes.contains(&0xFF) {
            let cur_row = self.display.cursor_row();
            let cur_col = self.display.cursor_col();
            self.display.erase_region(cur_row, cur_col, end_row as usize - 1, end_col as usize - 1, 0, self.display.width());
        }
        
        Ok(())
    }
    
    /// Start of Header order
    fn start_of_header(&mut self) -> Result<(), String> {
        let length = self.get_byte()?;
        if length > 7 {
            return Err(format!("Invalid SOH length: {}", length));
        }
        
        let mut header_data = Vec::new();
        for _ in 0..length {
            header_data.push(self.get_byte()?);
        }
        
        // TODO: Set header data in display
        self.display.clear_format_table();
        self.display.lock_keyboard();
        
        Ok(())
    }
    
    /// Write Error Code command
    fn write_error_code(&mut self, command: u8) -> Result<(), String> {
        if command == CMD_WRITE_ERROR_CODE_WINDOW {
            let _start_win = self.get_byte()?;
            let _end_win = self.get_byte()?;
        }
        
        // TODO: Process error message and display on error line
        // For now, just consume the data until ESC
        while self.buffer_pos < self.data_buffer.len() {
            let byte = self.get_byte()?;
            if byte == ESC {
                self.buffer_pos -= 1; // Put ESC back
                break;
            }
            // TODO: Process error message text
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
        
        let mut response = Vec::new();
        
        // Cursor position and AID code 0 for immediate read
        response.push(self.display.cursor_row() as u8 + 1);
        response.push(self.display.cursor_col() as u8 + 1);
        response.push(0); // AID code 0
        
        self.read_opcode = old_opcode;
        Ok(response)
    }
    
    /// Save Screen command
    fn save_screen(&mut self) -> Result<Vec<u8>, String> {
        // TODO: Create Write To Display data from current screen
        let mut response = Vec::new();
        
        // Add read command if we were in a read operation
        if self.read_opcode != 0 {
            response.push(ESC);
            response.push(self.read_opcode);
            response.push(0x00); // CC1
            response.push(0x00); // CC2
        }
        
        Ok(response)
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
            return Err(format!("Invalid SF class: 0x{:02X}", class));
        }
        
        match sf_type {
            SF_5250_QUERY | SF_5250_QUERY_STATION_STATE => {
                // Send Query Reply
                self.create_query_reply()
            }
            _ => {
                // TODO: Handle other structured field types
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
        let mut response = Vec::new();
        
        // Cursor position and AID code
        response.push(self.display.cursor_row() as u8 + 1);
        response.push(self.display.cursor_col() as u8 + 1);
        response.push(aid_code);
        
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
        for _ in 0..16 {
            response.push(0x00);
        }
        
        // Device type
        response.push(0x01); // Display emulation
        
        // Device model (IBM-3179-2 = 3179 model 2)
        response.extend(b"3179"); // Device type in EBCDIC
        response.extend(b"02");   // Model in EBCDIC
        
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
        // TODO: Use proper EBCDIC character map
        ch >= 0x20 && ch <= 0xFE
    }
    
    /// Get current display
    pub fn display(&self) -> &Display {
        &self.display
    }
    
    /// Get mutable display
    pub fn display_mut(&mut self) -> &mut Display {
        &mut self.display
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}