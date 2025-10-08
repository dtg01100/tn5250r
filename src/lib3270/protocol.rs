//! 3270 Protocol Implementation
//!
//! This module implements the core 3270 data stream parsing and command processing
//! following RFC 1205 and RFC 2355 specifications.

#![allow(dead_code)] // Complete TN3270 protocol implementation

use super::codes::*;
use super::display::{Display3270, addressing};
use super::field::{ExtendedAttributes, FieldAttribute};
// EBCDIC conversion functions available but not currently used in this module
use crate::protocol_common::traits::TerminalProtocol;

/// 3270 Protocol Processor
///
/// Handles parsing and processing of 3270 data streams, including
/// commands, orders, and structured fields.
#[derive(Debug)]
pub struct ProtocolProcessor3270 {
    /// Current state of the processor
    state: ProcessorState,

    /// Display buffer for the terminal
    display: Display3270,

    /// Use 14-bit addressing (for larger screens)
    use_14bit_addressing: bool,
}

/// Processor state
#[derive(Debug, Clone, PartialEq)]
enum ProcessorState {
    /// Ready to process commands
    Ready,

    /// Processing a command
    Processing,

    /// Pending Read Buffer response
    PendingReadBuffer,

    /// Pending Read Modified response
    PendingReadModified,

    /// Pending Read Modified All response
    PendingReadModifiedAll,
}

impl ProtocolProcessor3270 {
    /// Create a new protocol processor
    pub fn new() -> Self {
        Self {
            state: ProcessorState::Ready,
            display: Display3270::new(),
            use_14bit_addressing: false,
        }
    }
    
    /// Enable or disable 14-bit addressing
    pub fn set_14bit_addressing(&mut self, enabled: bool) {
        self.use_14bit_addressing = enabled;
    }
    
    /// Process a 3270 data stream
    ///
    /// Parses and executes commands from the host, updating the display buffer.
    pub fn process_data(&mut self, data: &[u8], display: &mut Display3270) -> Result<(), String> {
        if data.is_empty() {
            return Ok(());
        }
        
        let mut parser = DataStreamParser::new(data, self.use_14bit_addressing);
        let pending_state = parser.parse(display)?;
        if let Some(state) = pending_state {
            self.state = state;
        }
        Ok(())
    }

    /// Process a 3270 data stream using internal display
    ///
    /// This is used by the trait implementation.
    fn process_data_internal(&mut self, data: &[u8]) -> Result<(), String> {
        if data.is_empty() {
            return Ok(());
        }

        let mut parser = DataStreamParser::new(data, self.use_14bit_addressing);
        let pending_state = parser.parse(&mut self.display)?;
        if let Some(state) = pending_state {
            self.state = state;
        }
        Ok(())
    }

    /// Create a Read Buffer response
    ///
    /// Returns the entire display buffer contents with AID and cursor address.
    pub fn create_read_buffer_response(&self, display: &Display3270, aid: AidKey) -> Vec<u8> {
        let mut response = Vec::new();
        
        // Add AID byte
        response.push(aid.to_u8());
        
        // Add cursor address (2 bytes)
        let cursor_addr = display.cursor_address();
        let (b1, b2) = if self.use_14bit_addressing {
            addressing::encode_14bit_address(cursor_addr)
        } else {
            addressing::encode_12bit_address(cursor_addr)
        };
        response.push(b1);
        response.push(b2);
        
        // Add buffer data
        response.extend_from_slice(&display.get_buffer_data());
        
        response
    }
    
    /// Create a Read Modified response
    ///
    /// Returns only modified fields with AID and cursor address.
    pub fn create_read_modified_response(&self, display: &Display3270, aid: AidKey) -> Vec<u8> {
        let mut response = Vec::new();
        
        // Add AID byte
        response.push(aid.to_u8());
        
        // Add cursor address (2 bytes)
        let cursor_addr = display.cursor_address();
        let (b1, b2) = if self.use_14bit_addressing {
            addressing::encode_14bit_address(cursor_addr)
        } else {
            addressing::encode_12bit_address(cursor_addr)
        };
        response.push(b1);
        response.push(b2);
        
        // Get modified fields and encode them
        let modified_fields = self.get_modified_fields(display);
        let field_data = self.encode_field_data(&modified_fields);
        response.extend_from_slice(&field_data);
        
        response
    }
    
    /// Encode field data for transmission in 3270 format
    /// Returns encoded field data with buffer addresses and field contents
    pub fn encode_field_data(&self, field_data: &[(u16, String)]) -> Vec<u8> {
        let mut encoded = Vec::new();
        
        for (address, content) in field_data {
            // Add Set Buffer Address (SBA) order
            encoded.push(ORDER_SBA);
            
            // Add buffer address (12-bit or 14-bit encoding)
            let (b1, b2) = if self.use_14bit_addressing {
                addressing::encode_14bit_address(*address)
            } else {
                addressing::encode_12bit_address(*address)
            };
            encoded.push(b1);
            encoded.push(b2);
            
            // Add field content (convert to EBCDIC)
            for ch in content.chars() {
                let ebcdic_byte = crate::protocol_common::ebcdic::ascii_to_ebcdic(ch);
                encoded.push(ebcdic_byte);
            }
        }
        
        encoded
    }
    
    /// Send input fields with Read Modified response format
    /// This is called when Enter or a function key is pressed
    pub fn send_input_fields(&self, display: &Display3270, aid: AidKey, modified_fields: &[(u16, String)]) -> Vec<u8> {
        let mut response = Vec::new();
        
        // Add AID byte
        response.push(aid.to_u8());
        
        // Add cursor address (2 bytes)
        let cursor_addr = display.cursor_address();
        let (b1, b2) = if self.use_14bit_addressing {
            addressing::encode_14bit_address(cursor_addr)
        } else {
            addressing::encode_12bit_address(cursor_addr)
        };
        response.push(b1);
        response.push(b2);
        
        // Encode and add modified field data
        let field_data = self.encode_field_data(modified_fields);
        response.extend_from_slice(&field_data);
        
        response
    }
    
    /// Send field data with AID key and pending input
    /// This combines pending input with AID key for transmission
    pub fn send_field_input(&self, display: &Display3270, aid: AidKey, pending_input: &[u8]) -> Vec<u8> {
        let mut response = Vec::new();
        
        // Add AID byte
        response.push(aid.to_u8());
        
        // Add cursor address (2 bytes)
        let cursor_addr = display.cursor_address();
        let (b1, b2) = if self.use_14bit_addressing {
            addressing::encode_14bit_address(cursor_addr)
        } else {
            addressing::encode_12bit_address(cursor_addr)
        };
        response.push(b1);
        response.push(b2);
        
        // Add pending input data
        response.extend_from_slice(pending_input);
        
        response
    }
    
    /// Get modified fields from display for transmission
    /// Returns list of (address, content) tuples for modified fields
    pub fn get_modified_fields(&self, display: &Display3270) -> Vec<(u16, String)> {
        let mut modified_fields = Vec::new();
        
        // Get all fields with MDT bit set
        let fields = display.field_manager().modified_fields();
        
        for field in fields {
            let start_addr = field.address + 1; // Skip field attribute byte
            let end_addr = start_addr + field.length as u16;
            
            // Extract field content
            let mut content = String::new();
            for addr in start_addr..end_addr.min(display.buffer_size() as u16) {
                if let Some(ch) = display.read_char_at(addr) {
                    // Convert EBCDIC to ASCII for transmission
                    let ascii_ch = crate::protocol_common::ebcdic::ebcdic_to_ascii(ch);
                    if ascii_ch != '\0' {  // Skip null characters
                        content.push(ascii_ch);
                    }
                }
            }
            
            // Only include fields with actual content
            if !content.trim().is_empty() {
                modified_fields.push((field.address, content));
            }
        }
        
        modified_fields
    }
}

impl Default for ProtocolProcessor3270 {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolProcessor3270 {
    /// Handle TN3270E subnegotiation commands
    fn handle_tn3270e_negotiation(&mut self, data: &[u8]) -> Option<Vec<u8>> {
        if data.is_empty() {
            return None;
        }

        match data[0] {
            // DEVICE-TYPE request
            2 => {
                // Server is requesting device type, respond with IS DEVICE-TYPE Model2Color
                let mut response = vec![255, 250, 40, 4, 2, 0x82]; // IAC SB TN3270E IS DEVICE-TYPE Model2Color
                response.extend_from_slice(&[255, 240]); // IAC SE
                Some(response)
            }
            // BIND command
            6 => {
                // Server is binding the session, respond with BIND response
                let mut response = vec![255, 250, 40, 4, 6]; // IAC SB TN3270E IS BIND
                // Add logical unit name (empty for now)
                response.push(0); // Null terminator
                response.extend_from_slice(&[255, 240]); // IAC SE
                Some(response)
            }
            // CONNECT command
            1 => {
                // Server wants to connect, acknowledge
                let response = vec![255, 250, 40, 4, 1, 255, 240]; // IAC SB TN3270E IS CONNECT IAC SE
                Some(response)
            }
            // Other commands - acknowledge with IS response
            _ => {
                let mut response = vec![255, 250, 40, 4, data[0]]; // IAC SB TN3270E IS <command>
                response.extend_from_slice(&[255, 240]); // IAC SE
                Some(response)
            }
        }
    }
}

/// Data stream parser for 3270 protocol
struct DataStreamParser<'a> {
    data: &'a [u8],
    pos: usize,
    use_14bit_addressing: bool,
}

impl<'a> DataStreamParser<'a> {
    /// Create a new parser
    fn new(data: &'a [u8], use_14bit_addressing: bool) -> Self {
        Self {
            data,
            pos: 0,
            use_14bit_addressing,
        }
    }
    
    /// Parse the data stream
    fn parse(&mut self, display: &mut Display3270) -> Result<Option<ProcessorState>, String> {
        let mut pending_state = None;
        while self.pos < self.data.len() {
            let cmd_byte = self.data[self.pos];
            self.pos += 1;

            if let Some(command) = CommandCode::from_u8(cmd_byte) {
                if let Some(state) = self.process_command(command, display)? {
                    pending_state = Some(state);
                }
            } else {
                return Err(format!("Unknown command code: 0x{cmd_byte:02X}"));
            }
        }

        Ok(pending_state)
    }
    
    /// Process a command
    fn process_command(&mut self, command: CommandCode, display: &mut Display3270) -> Result<Option<ProcessorState>, String> {
        match command {
            CommandCode::Write => {
                self.process_write(display, false, false)?;
                Ok(None)
            }
            CommandCode::EraseWrite => {
                self.process_write(display, true, false)?;
                Ok(None)
            }
            CommandCode::EraseWriteAlternate => {
                self.process_write(display, true, true)?;
                Ok(None)
            }
            CommandCode::ReadBuffer => Ok(Some(ProcessorState::PendingReadBuffer)),
            CommandCode::ReadModified => Ok(Some(ProcessorState::PendingReadModified)),
            CommandCode::ReadModifiedAll => Ok(Some(ProcessorState::PendingReadModifiedAll)),
            CommandCode::EraseAllUnprotected => {
                self.process_erase_all_unprotected(display)?;
                Ok(None)
            }
            CommandCode::WriteStructuredField => {
                self.process_write_structured_field(display)?;
                Ok(None)
            }
        }
    }
    
    /// Process Write, Erase/Write, or Erase/Write Alternate command
    fn process_write(&mut self, display: &mut Display3270, erase: bool, _alternate: bool) -> Result<(), String> {
        // KEYBOARD LOCK STATE MACHINE: Lock keyboard at start of Write command
        // The keyboard will remain locked until WCC_RESTORE bit unlocks it
        display.lock_keyboard();
        
        // Read WCC (Write Control Character)
        if self.pos >= self.data.len() {
            return Err("Missing WCC byte".to_string());
        }
        
        let wcc = self.data[self.pos];
        self.pos += 1;
        
        // Process WCC bits
        if erase {
            display.clear();
        }
        
        if (wcc & WCC_RESET) != 0 {
            // Reset operation
            display.field_manager_mut().reset_mdt();
        }
        
        if (wcc & WCC_ALARM) != 0 {
            display.set_alarm(true);
        }
        
        // KEYBOARD LOCK STATE MACHINE: Unlock keyboard if WCC_RESTORE bit is set
        // This is the proper 3270 behavior - keyboard locks on Write, unlocks on WCC restore
        if (wcc & WCC_RESTORE) != 0 {
            display.unlock_keyboard();
        }
        
        if (wcc & WCC_RESET_MDT) != 0 {
            display.field_manager_mut().reset_mdt();
        }
        
        // Process orders and data
        while self.pos < self.data.len() {
            let byte = self.data[self.pos];
            
            // Check if this is an order
            if let Some(order) = OrderCode::from_u8(byte) {
                self.pos += 1;
                self.process_order(order, display)?;
            } else {
                // Regular data character
                display.write_char(byte);
                self.pos += 1;
            }
        }
        
        Ok(())
    }
    
    /// Process an order
    fn process_order(&mut self, order: OrderCode, display: &mut Display3270) -> Result<(), String> {
        match order {
            OrderCode::StartField => self.process_start_field(display),
            OrderCode::StartFieldExtended => self.process_start_field_extended(display),
            OrderCode::SetBufferAddress => self.process_set_buffer_address(display),
            OrderCode::SetAttribute => self.process_set_attribute(display),
            OrderCode::ModifyField => self.process_modify_field(display),
            OrderCode::InsertCursor => self.process_insert_cursor(display),
            OrderCode::ProgramTab => self.process_program_tab(display),
            OrderCode::RepeatToAddress => self.process_repeat_to_address(display),
            OrderCode::EraseUnprotectedToAddress => self.process_erase_unprotected_to_address(display),
            OrderCode::GraphicEscape => self.process_graphic_escape(display),
        }
    }
    
    /// Process Start Field (SF) order
    fn process_start_field(&mut self, display: &mut Display3270) -> Result<(), String> {
        if self.pos >= self.data.len() {
            return Err("Missing field attribute byte".to_string());
        }
        
        let attr_byte = self.data[self.pos];
        self.pos += 1;
        
        let current_addr = display.cursor_address();
        let field_attr = FieldAttribute::new(current_addr, attr_byte);
        display.set_field_attribute(current_addr, field_attr);
        
        // Move cursor past the field attribute
        display.set_cursor(current_addr + 1);
        
        Ok(())
    }
    
    /// Process Start Field Extended (SFE) order
    fn process_start_field_extended(&mut self, display: &mut Display3270) -> Result<(), String> {
        if self.pos >= self.data.len() {
            return Err("Missing SFE count byte".to_string());
        }
        
        let count = self.data[self.pos] as usize;
        self.pos += 1;
        
        if self.pos + (count * 2) > self.data.len() {
            return Err("Insufficient data for SFE attributes".to_string());
        }
        
        // Parse base attribute (first pair should be 3270 field attribute)
        let mut base_attr = 0u8;
        let mut extended_attrs = ExtendedAttributes::new();
        
        for _ in 0..count {
            let attr_type = self.data[self.pos];
            let attr_value = self.data[self.pos + 1];
            self.pos += 2;
            
            match attr_type {
                XA_3270 => base_attr = attr_value,
                XA_HIGHLIGHTING => extended_attrs.highlighting = Some(attr_value),
                XA_FOREGROUND => extended_attrs.foreground_color = Some(attr_value),
                XA_BACKGROUND => extended_attrs.background_color = Some(attr_value),
                XA_CHARSET => extended_attrs.charset = Some(attr_value),
                XA_VALIDATION => extended_attrs.validation = Some(attr_value),
                XA_OUTLINING => extended_attrs.outlining = Some(attr_value),
                XA_TRANSPARENCY => extended_attrs.transparency = Some(attr_value),
                _ => {
                    // Unknown attribute type, skip
                }
            }
        }
        
        let current_addr = display.cursor_address();
        let field_attr = FieldAttribute::new_extended(current_addr, base_attr, extended_attrs);
        display.set_field_attribute(current_addr, field_attr);
        
        // Move cursor past the field attribute
        display.set_cursor(current_addr + 1);
        
        Ok(())
    }
    
    /// Process Set Buffer Address (SBA) order
    fn process_set_buffer_address(&mut self, display: &mut Display3270) -> Result<(), String> {
        let address = self.read_buffer_address()?;
        display.set_cursor(address);
        Ok(())
    }
    
    /// Process Set Attribute (SA) order
    fn process_set_attribute(&mut self, _display: &mut Display3270) -> Result<(), String> {
        if self.pos + 1 >= self.data.len() {
            return Err("Insufficient data for SA order".to_string());
        }
        
        let _attr_type = self.data[self.pos];
        let _attr_value = self.data[self.pos + 1];
        self.pos += 2;

        Ok(())
    }
    
    /// Process Modify Field (MF) order
    fn process_modify_field(&mut self, _display: &mut Display3270) -> Result<(), String> {
        if self.pos >= self.data.len() {
            return Err("Missing MF count byte".to_string());
        }
        
        let count = self.data[self.pos] as usize;
        self.pos += 1;
        
        if self.pos + (count * 2) > self.data.len() {
            return Err("Insufficient data for MF attributes".to_string());
        }

        self.pos += count * 2;

        Ok(())
    }
    
    /// Process Insert Cursor (IC) order
    fn process_insert_cursor(&mut self, _display: &mut Display3270) -> Result<(), String> {
        // IC order marks where the cursor should be positioned after the write
        // The actual cursor position is set at the current location
        // For now, we just note this position
        Ok(())
    }
    
    /// Process Program Tab (PT) order
    fn process_program_tab(&mut self, display: &mut Display3270) -> Result<(), String> {
        // Tab to the next unprotected field
        if !display.tab_to_next_field() {
            // If no unprotected field found, stay at current position
            // This is the standard 3270 behavior
        }
        Ok(())
    }
    
    /// Process Repeat to Address (RA) order
    fn process_repeat_to_address(&mut self, display: &mut Display3270) -> Result<(), String> {
        let target_address = self.read_buffer_address()?;
        
        if self.pos >= self.data.len() {
            return Err("Missing character for RA order".to_string());
        }
        
        let ch = self.data[self.pos];
        self.pos += 1;
        
        display.repeat_to_address(ch, target_address);
        
        Ok(())
    }
    
    /// Process Erase Unprotected to Address (EUA) order
    fn process_erase_unprotected_to_address(&mut self, display: &mut Display3270) -> Result<(), String> {
        let target_address = self.read_buffer_address()?;
        display.erase_unprotected_to_address(target_address);
        Ok(())
    }
    
    /// Process Graphic Escape (GE) order
    fn process_graphic_escape(&mut self, display: &mut Display3270) -> Result<(), String> {
        if self.pos >= self.data.len() {
            return Err("Missing character for GE order".to_string());
        }
        
        let ch = self.data[self.pos];
        self.pos += 1;
        
        // Write the character as-is (graphic escape allows special characters)
        display.write_char(ch);
        
        Ok(())
    }
    
    /// Process Erase All Unprotected command
    fn process_erase_all_unprotected(&mut self, display: &mut Display3270) -> Result<(), String> {
        display.clear_unprotected();
        display.unlock_keyboard();
        Ok(())
    }
    
    /// Process Write Structured Field command
    fn process_write_structured_field(&mut self, display: &mut Display3270) -> Result<(), String> {
        // Parse structured fields from the data stream
        while self.pos < self.data.len() {
            // Read structured field length (2 bytes, big-endian)
            if self.pos + 2 > self.data.len() {
                return Err("Insufficient data for structured field length".to_string());
            }
            let length = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]) as usize;
            self.pos += 2;

            if length < 4 {
                return Err("Invalid structured field length".to_string());
            }

            // Read structured field type (2 bytes, big-endian)
            if self.pos + 2 > self.data.len() {
                return Err("Insufficient data for structured field type".to_string());
            }
            let sf_type = u16::from_be_bytes([self.data[self.pos], self.data[self.pos + 1]]);
            self.pos += 2;

            // Read structured field data
            let data_len = length - 4;
            if self.pos + data_len > self.data.len() {
                return Err("Insufficient data for structured field content".to_string());
            }
            let sf_data = &self.data[self.pos..self.pos + data_len];
            self.pos += data_len;

            // Process the structured field based on type
            self.process_structured_field(sf_type, sf_data, display)?;
        }

        Ok(())
    }

    /// Process a structured field
    fn process_structured_field(&mut self, sf_type: u16, sf_data: &[u8], _display: &mut Display3270) -> Result<(), String> {
        match sf_type {
            SF_QUERY_REPLY => self.process_query_reply(sf_data),
            SF_OUTBOUND_3270DS => {
                // Outbound 3270DS - data stream content
                // For now, just skip as it's handled elsewhere
                Ok(())
            }
            _ => {
                // Unknown structured field type, skip
                Ok(())
            }
        }
    }

    /// Process Query Reply structured field
    fn process_query_reply(&mut self, sf_data: &[u8]) -> Result<(), String> {
        // Query Reply contains terminal capabilities
        // Parse the reply to understand what features are supported
        let mut pos = 0;
        while pos < sf_data.len() {
            if pos + 1 > sf_data.len() {
                break;
            }
            let query_type = sf_data[pos];
            pos += 1;

            if pos + 1 > sf_data.len() {
                break;
            }
            let length = sf_data[pos] as usize;
            pos += 1;

            if pos + length > sf_data.len() {
                break;
            }
            let query_data = &sf_data[pos..pos + length];
            pos += length;

            // Process based on query type
            match query_type {
                0x81 => {
                    // Usable Area - host tells terminal the expected screen size
                    if query_data.len() >= 2 {
                        let _rows = query_data[0] as u16;
                        let _cols = query_data[1] as u16;
                        // Note: Our display size is fixed, but we acknowledge the host's expectation
                    }
                }
                0x82 => {
                    // Character Sets - terminal supports character sets
                }
                0x83 => {
                    // Highlighting - terminal supports highlighting
                }
                0x84 => {
                    // Color - terminal supports color
                }
                0x85 => {
                    // Field Outlining - terminal supports field outlining
                }
                0x86 => {
                    // Partition - terminal supports partitions
                }
                0x87 => {
                    // Field Validation - terminal supports field validation
                }
                _ => {
                    // Unknown query type, skip
                }
            }
        }

        Ok(())
    }

    /// Read a buffer address (12-bit or 14-bit)
    fn read_buffer_address(&mut self) -> Result<u16, String> {
        if self.pos + 1 >= self.data.len() {
            return Err("Insufficient data for buffer address".to_string());
        }
        
        let byte1 = self.data[self.pos];
        let byte2 = self.data[self.pos + 1];
        self.pos += 2;
        
        let address = if self.use_14bit_addressing {
            addressing::decode_14bit_address(byte1, byte2)
        } else {
            addressing::decode_12bit_address(byte1, byte2)
        };
        
        Ok(address)
    }
}

// Implement TerminalProtocol trait for 3270
impl TerminalProtocol for ProtocolProcessor3270 {
    fn process_data(&mut self, data: &[u8]) -> Result<(), String> {
        self.process_data_internal(data)
    }
    
    fn generate_response(&mut self) -> Option<Vec<u8>> {
        match self.state {
            ProcessorState::PendingReadBuffer => {
                let response = self.create_read_buffer_response(&self.display, AidKey::NoAid);
                self.state = ProcessorState::Ready;
                Some(response)
            }
            ProcessorState::PendingReadModified => {
                let response = self.create_read_modified_response(&self.display, AidKey::NoAid);
                self.state = ProcessorState::Ready;
                Some(response)
            }
            ProcessorState::PendingReadModifiedAll => {
                // For ReadModifiedAll, return all modified fields (same as ReadModified for now)
                let response = self.create_read_modified_response(&self.display, AidKey::NoAid);
                self.state = ProcessorState::Ready;
                Some(response)
            }
            _ => None,
        }
    }
    
    fn reset(&mut self) {
        self.state = ProcessorState::Ready;
    }
    
    fn protocol_name(&self) -> &str {
        "TN3270"
    }
    
    fn is_connected(&self) -> bool {
        matches!(self.state, ProcessorState::Ready | ProcessorState::Processing | ProcessorState::PendingReadBuffer | ProcessorState::PendingReadModified | ProcessorState::PendingReadModifiedAll)
    }
    
    fn handle_negotiation(&mut self, option: u8, data: &[u8]) -> Option<Vec<u8>> {
        match option {
            // Binary Transmission (0)
            0 => {
                if data.is_empty() {
                    // Server sent DO BINARY, respond with WILL BINARY
                    Some(vec![255, 251, 0]) // IAC WILL BINARY
                } else {
                    None
                }
            }
            // Suppress Go Ahead (3)
            3 => {
                if data.is_empty() {
                    // Server sent DO SGA, respond with WILL SGA
                    Some(vec![255, 251, 3]) // IAC WILL SGA
                } else {
                    None
                }
            }
            // End of Record (25)
            25 => {
                if data.is_empty() {
                    // Server sent DO EOR, respond with WILL EOR
                    Some(vec![255, 251, 25]) // IAC WILL EOR
                } else {
                    None
                }
            }
            // Terminal Type (24)
            24 => {
                if data.is_empty() {
                    // Server sent DO TERMINAL-TYPE, respond with WILL TERMINAL-TYPE
                    Some(vec![255, 251, 24]) // IAC WILL TERMINAL-TYPE
                } else if !data.is_empty() && data[0] == 1 {
                    // Server sent subnegotiation SEND, respond with IS IBM-3179-2
                    let mut response = vec![255, 250, 24, 0]; // IAC SB TERMINAL-TYPE IS
                    response.extend_from_slice(b"IBM-3179-2");
                    response.extend_from_slice(&[255, 240]); // IAC SE
                    Some(response)
                } else {
                    None
                }
            }
            // TN3270E (40)
            40 => {
                if data.is_empty() {
                    // Server sent WILL TN3270E, respond with DO TN3270E
                    Some(vec![255, 253, 40]) // IAC DO TN3270E
                } else if !data.is_empty() {
                    // Handle TN3270E subnegotiation
                    self.handle_tn3270e_negotiation(data)
                } else {
                    None
                }
            }
            // Unknown option - reject it
            _ => {
                if data.is_empty() {
                    // Server sent DO/WILL for unknown option, respond with DONT/WONT
                    Some(vec![255, 254, option]) // IAC DONT option
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = ProtocolProcessor3270::new();
        assert_eq!(processor.state, ProcessorState::Ready);
    }

    #[test]
    fn test_write_command_with_wcc() {
        let mut processor = ProtocolProcessor3270::new();
        let mut display = Display3270::new();
        
        // Write command with WCC and some data
        let data = vec![
            CMD_WRITE,
            WCC_RESTORE, // WCC byte
            0xC1,        // EBCDIC 'A'
            0xC2,        // EBCDIC 'B'
        ];
        
        let result = processor.process_data(&data, &mut display);
        assert!(result.is_ok());
        assert!(!display.is_keyboard_locked());
    }

    #[test]
    fn test_erase_write_command() {
        let mut processor = ProtocolProcessor3270::new();
        let mut display = Display3270::new();
        
        // Write some data first
        display.write_char(0xC1);
        
        // Erase/Write command
        let data = vec![
            CMD_ERASE_WRITE,
            0x00, // WCC byte
        ];
        
        let result = processor.process_data(&data, &mut display);
        assert!(result.is_ok());
        
        // Buffer should be cleared
        assert_eq!(display.cursor_address(), 0);
    }

    #[test]
    fn test_set_buffer_address_order() {
        let mut processor = ProtocolProcessor3270::new();
        let mut display = Display3270::new();
        
        // Write command with SBA order
        let (b1, b2) = addressing::encode_12bit_address(100);
        let data = vec![
            CMD_WRITE,
            0x00,      // WCC byte
            ORDER_SBA, // Set Buffer Address
            b1, b2,    // Address bytes
        ];
        
        let result = processor.process_data(&data, &mut display);
        assert!(result.is_ok());
        assert_eq!(display.cursor_address(), 100);
    }

    #[test]
    fn test_start_field_order() {
        let mut processor = ProtocolProcessor3270::new();
        let mut display = Display3270::new();
        
        // Write command with SF order
        let data = vec![
            CMD_WRITE,
            0x00,     // WCC byte
            ORDER_SF, // Start Field
            ATTR_PROTECTED | ATTR_NUMERIC, // Field attribute
        ];
        
        let result = processor.process_data(&data, &mut display);
        assert!(result.is_ok());
        
        // Field should be added
        assert_eq!(display.field_manager().fields().len(), 1);
    }

    #[test]
    fn test_read_buffer_response() {
        let processor = ProtocolProcessor3270::new();
        let display = Display3270::new();
        
        let response = processor.create_read_buffer_response(&display, AidKey::Enter);
        
        // Response should have AID + cursor address + buffer data
        assert!(response.len() >= 3);
        assert_eq!(response[0], AID_ENTER);
    }
}