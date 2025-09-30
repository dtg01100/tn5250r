//! 3270 Protocol Implementation
//!
//! This module implements the core 3270 data stream parsing and command processing
//! following RFC 1205 and RFC 2355 specifications.

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
    
    /// Error state
    Error(String),
}

impl ProtocolProcessor3270 {
    /// Create a new protocol processor
    pub fn new() -> Self {
        Self {
            state: ProcessorState::Ready,
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
        parser.parse(display)
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
        
        // Add modified field data
        // TODO: Implement proper Read Modified logic
        // For now, return empty modified data
        
        response
    }
}

impl Default for ProtocolProcessor3270 {
    fn default() -> Self {
        Self::new()
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
    fn parse(&mut self, display: &mut Display3270) -> Result<(), String> {
        while self.pos < self.data.len() {
            let cmd_byte = self.data[self.pos];
            self.pos += 1;
            
            if let Some(command) = CommandCode::from_u8(cmd_byte) {
                self.process_command(command, display)?;
            } else {
                return Err(format!("Unknown command code: 0x{:02X}", cmd_byte));
            }
        }
        
        Ok(())
    }
    
    /// Process a command
    fn process_command(&mut self, command: CommandCode, display: &mut Display3270) -> Result<(), String> {
        match command {
            CommandCode::Write => self.process_write(display, false, false),
            CommandCode::EraseWrite => self.process_write(display, true, false),
            CommandCode::EraseWriteAlternate => self.process_write(display, true, true),
            CommandCode::ReadBuffer => Ok(()), // Response handled separately
            CommandCode::ReadModified => Ok(()), // Response handled separately
            CommandCode::ReadModifiedAll => Ok(()), // Response handled separately
            CommandCode::EraseAllUnprotected => self.process_erase_all_unprotected(display),
            CommandCode::WriteStructuredField => self.process_write_structured_field(display),
        }
    }
    
    /// Process Write, Erase/Write, or Erase/Write Alternate command
    fn process_write(&mut self, display: &mut Display3270, erase: bool, alternate: bool) -> Result<(), String> {
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
    fn process_set_attribute(&mut self, display: &mut Display3270) -> Result<(), String> {
        if self.pos + 1 >= self.data.len() {
            return Err("Insufficient data for SA order".to_string());
        }
        
        let attr_type = self.data[self.pos];
        let attr_value = self.data[self.pos + 1];
        self.pos += 2;
        
        // TODO: Apply attribute to current position
        // This requires tracking character attributes separately from field attributes
        
        Ok(())
    }
    
    /// Process Modify Field (MF) order
    fn process_modify_field(&mut self, display: &mut Display3270) -> Result<(), String> {
        if self.pos >= self.data.len() {
            return Err("Missing MF count byte".to_string());
        }
        
        let count = self.data[self.pos] as usize;
        self.pos += 1;
        
        if self.pos + (count * 2) > self.data.len() {
            return Err("Insufficient data for MF attributes".to_string());
        }
        
        // TODO: Modify field attributes at current cursor position
        self.pos += count * 2;
        
        Ok(())
    }
    
    /// Process Insert Cursor (IC) order
    fn process_insert_cursor(&mut self, display: &mut Display3270) -> Result<(), String> {
        // IC order marks where the cursor should be positioned after the write
        // The actual cursor position is set at the current location
        // For now, we just note this position
        Ok(())
    }
    
    /// Process Program Tab (PT) order
    fn process_program_tab(&mut self, display: &mut Display3270) -> Result<(), String> {
        // TODO: Implement tab to next unprotected field
        // For now, just advance cursor
        let current = display.cursor_address();
        display.set_cursor(current + 1);
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
        // TODO: Implement structured field processing
        // Structured fields are used for advanced features like:
        // - Query replies
        // - Partition management
        // - Color definitions
        // - Extended data streams
        
        // For Phase 2, we'll just skip structured field data
        // This will be implemented in Phase 4
        
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
        // For the trait implementation, we need a display buffer
        // This is a simplified version - real usage would pass display separately
        let mut display = Display3270::new();
        self.process_data(data, &mut display)?;
        Ok(())
    }
    
    fn generate_response(&mut self) -> Option<Vec<u8>> {
        // TODO: Implement response generation for Phase 3
        None
    }
    
    fn reset(&mut self) {
        self.state = ProcessorState::Ready;
    }
    
    fn protocol_name(&self) -> &str {
        "TN3270"
    }
    
    fn is_connected(&self) -> bool {
        self.state == ProcessorState::Ready || self.state == ProcessorState::Processing
    }
    
    fn handle_negotiation(&mut self, _option: u8, _data: &[u8]) -> Option<Vec<u8>> {
        // TODO: Implement telnet negotiation for Phase 3
        None
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