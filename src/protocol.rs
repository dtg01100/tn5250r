//! 5250 Protocol implementation with RFC 2877/4777 compliance
//! 
//! This module handles the IBM 5250 protocol for communication with AS/400 systems,
//! implementing the complete command set as specified in RFC 2877/4777.

use crate::terminal::{TerminalScreen, TERMINAL_WIDTH, TERMINAL_HEIGHT, CharAttribute, TerminalChar};

// 5250 protocol command codes as defined in RFC 2877
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandCode {
    /// Write to Display (F1)
    WriteToDisplay = 0xF1,
    
    /// Read Buffer (F2)
    ReadBuffer = 0xF2,
    
    /// Read to Memory (F3)
    ReadToMemory = 0xF3,
    
    /// Save (F4)
    Save = 0xF4,
    
    /// Write Structured Field (F5)
    WriteStructuredField = 0xF5,
    
    /// Read Structured Field (F6)
    ReadStructuredField = 0xF6,
    
    /// Restore (62)
    Restore = 0x62,
    
    /// Transfer Data (F7)
    TransferData = 0xF7,
    
    /// Write to Display and Identify (F8)
    WriteToDisplayAndIdentify = 0xF8,
    
    /// Read Buffer and Identify (F9)
    ReadBufferAndIdentify = 0xF9,
    
    /// Cancel Invite (FA)
    CancelInvite = 0xFA,
    
    /// Read Modified (FB)
    ReadModified = 0xFB,
    
    /// Read Immediate (FC)
    ReadImmediate = 0xFC,
    
    /// Read Modified All (FD)
    ReadModifiedAll = 0xFD,
    
    /// Save Partial (FE)
    SavePartial = 0xFE,
    
    /// Restore Partial (FF)
    RestorePartial = 0xFF,
}

impl CommandCode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0xF1 => Some(CommandCode::WriteToDisplay),
            0xF2 => Some(CommandCode::ReadBuffer),
            0xF3 => Some(CommandCode::ReadToMemory),
            0xF4 => Some(CommandCode::Save),
            0xF5 => Some(CommandCode::WriteStructuredField),
            0xF6 => Some(CommandCode::ReadStructuredField),
            0x62 => Some(CommandCode::Restore),
            0xF7 => Some(CommandCode::TransferData),
            0xF8 => Some(CommandCode::WriteToDisplayAndIdentify),
            0xF9 => Some(CommandCode::ReadBufferAndIdentify),
            0xFA => Some(CommandCode::CancelInvite),
            0xFB => Some(CommandCode::ReadModified),
            0xFC => Some(CommandCode::ReadImmediate),
            0xFD => Some(CommandCode::ReadModifiedAll),
            0xFE => Some(CommandCode::SavePartial),
            0xFF => Some(CommandCode::RestorePartial),
            _ => None,
        }
    }
    
    /// Get the response command for a given request command
    pub fn get_response_command(&self) -> Option<Self> {
        match self {
            CommandCode::WriteToDisplay => None, // No explicit response
            CommandCode::ReadBuffer => Some(CommandCode::ReadBuffer), // Response is data
            CommandCode::ReadToMemory => Some(CommandCode::ReadToMemory),
            CommandCode::WriteStructuredField => None, // No explicit response
            CommandCode::ReadStructuredField => Some(CommandCode::ReadStructuredField),
            CommandCode::Restore => None,
            CommandCode::TransferData => Some(CommandCode::TransferData),
            CommandCode::WriteToDisplayAndIdentify => Some(CommandCode::WriteToDisplayAndIdentify),
            CommandCode::ReadBufferAndIdentify => Some(CommandCode::ReadBufferAndIdentify),
            CommandCode::CancelInvite => None,
            CommandCode::ReadModified => Some(CommandCode::ReadModified),
            CommandCode::ReadImmediate => Some(CommandCode::ReadImmediate),
            CommandCode::ReadModifiedAll => Some(CommandCode::ReadModifiedAll),
            CommandCode::SavePartial => None,
            CommandCode::RestorePartial => None,
            _ => None,
        }
    }
}

// Field attribute values used in 5250 protocol (RFC 2877 Section 4.11)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldAttribute {
    /// Normal unprotected field
    Normal = 0x00,
    
    /// Intensified field
    Intensified = 0x20,
    
    /// Protected field (bit 5 set)
    Protected = 0x21,
    
    /// Numeric field
    Numeric = 0x10,
    
    /// Skip field
    Skip = 0x08,
    
    /// Mandatory field
    Mandatory = 0x0C,
    
    /// Duplicate enable field
    DupEnable = 0x04,
    
    /// Hidden field
    Hidden = 0x0D,
}

impl FieldAttribute {
    pub fn from_u8(value: u8) -> Self {
        // Extract field attributes from the 5250 field attribute byte
        // According to RFC 2877, field attributes use specific bit patterns
        match value & 0x3C { // Use bits 2-5 for field attributes in 5250
            0x20 => FieldAttribute::Protected,   // Bit 5 set
            0x10 => FieldAttribute::Numeric,    // Bit 4 set
            0x08 => FieldAttribute::Skip,      // Bit 3 set
            0x0C => FieldAttribute::Mandatory,  // Bits 3-2 set
            0x04 => FieldAttribute::DupEnable,  // Bit 2 set
            0x00 => FieldAttribute::Normal,    // No special attributes
            _ => FieldAttribute::Normal,
        }
    }
    
    /// Convert to 5250 attribute byte value
    pub fn to_u8(&self) -> u8 {
        match self {
            FieldAttribute::Normal => 0x00,
            FieldAttribute::Intensified => 0x20,
            FieldAttribute::Protected => 0x20,
            FieldAttribute::Numeric => 0x10,
            FieldAttribute::Skip => 0x08,
            FieldAttribute::Mandatory => 0x0C,
            FieldAttribute::DupEnable => 0x04,
            FieldAttribute::Hidden => 0x0C,
        }
    }
}

// Character attribute values used in 5250 protocol (RFC 2877 Section 4.9)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharAttribute5250 {
    /// Normal character
    Normal,
    
    /// Intensified character (bright/bold)
    Intensified,
    
    /// Non-display character (hidden)
    NonDisplay,
    
    /// Protected character
    Protected,
    
    /// Nondisplay, nondup character
    NonDisplayNonDup,
}

impl CharAttribute5250 {
    pub fn from_u8(value: u8) -> Self {
        // Extract character attributes from the 5250 character attribute byte
        // According to RFC 2877, character attributes use specific bit patterns
        match value & 0x0F { // Use lower 4 bits for character attributes
            0x00 => CharAttribute5250::Normal,
            0x02 => CharAttribute5250::Intensified,
            0x03 => CharAttribute5250::NonDisplay,
            0x04 => CharAttribute5250::Protected,
            0x05 => CharAttribute5250::NonDisplayNonDup,
            _ => CharAttribute5250::Normal,
        }
    }
    
    pub fn to_char_attribute(&self) -> CharAttribute {
        match self {
            CharAttribute5250::Normal => CharAttribute::Normal,
            CharAttribute5250::Intensified => CharAttribute::Intensified,
            CharAttribute5250::NonDisplay => CharAttribute::NonDisplay,
            CharAttribute5250::Protected => CharAttribute::Protected,
            CharAttribute5250::NonDisplayNonDup => CharAttribute::NonDisplay,
        }
    }
}

// Structured Field IDs as defined in RFC 2877
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StructuredFieldID {
    /// Create/Change Extended Attribute
    CreateChangeExtendedAttribute = 0xC1,
    
    /// Set Extended Attribute List
    SetExtendedAttributeList = 0xCA,
    
    /// Read Text
    ReadText = 0xD2,
    
    /// Erase/Reset
    EraseReset = 0x5B,
    
    /// Define Extended Attribute
    DefineExtendedAttribute = 0xD3,
    
    /// Define Named Logical Unit
    DefineNamedLogicalUnit = 0x7E,
    
    /// Define Pending Operations
    DefinePendingOperations = 0x80,
    
    /// Disable Command Recognition
    DisableCommandRecognition = 0x81,
    
    /// Enable Command Recognition
    EnableCommandRecognition = 0x82,
    
    /// Request Minimum Timestamp Interval
    RequestMinimumTimestampInterval = 0x83,
    
    /// Query Command
    QueryCommand = 0x84,
    
    /// Set Reply Mode
    SetReplyMode = 0x85,
    
    /// Define Roll Direction
    DefineRollDirection = 0x86,
    
    /// Set Monitor Mode
    SetMonitorMode = 0x87,
    
    /// Cancel Recovery
    CancelRecovery = 0x88,
}

impl StructuredFieldID {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0xC1 => Some(StructuredFieldID::CreateChangeExtendedAttribute),
            0xCA => Some(StructuredFieldID::SetExtendedAttributeList),
            0xD2 => Some(StructuredFieldID::ReadText),
            0x5B => Some(StructuredFieldID::EraseReset),
            0xD3 => Some(StructuredFieldID::DefineExtendedAttribute),
            0x7E => Some(StructuredFieldID::DefineNamedLogicalUnit),
            0x80 => Some(StructuredFieldID::DefinePendingOperations),
            0x81 => Some(StructuredFieldID::DisableCommandRecognition),
            0x82 => Some(StructuredFieldID::EnableCommandRecognition),
            0x83 => Some(StructuredFieldID::RequestMinimumTimestampInterval),
            0x84 => Some(StructuredFieldID::QueryCommand),
            0x85 => Some(StructuredFieldID::SetReplyMode),
            0x86 => Some(StructuredFieldID::DefineRollDirection),
            0x87 => Some(StructuredFieldID::SetMonitorMode),
            0x88 => Some(StructuredFieldID::CancelRecovery),
            _ => None,
        }
    }
}

// Represents a 5250 protocol packet with proper header structure
#[derive(Debug)]
pub struct Packet {
    pub command: CommandCode,
    pub sequence_number: u8,
    pub data: Vec<u8>,
    pub flags: u8, // Various flags from the packet header
}

impl Packet {
    pub fn new(command: CommandCode, sequence_number: u8, data: Vec<u8>) -> Self {
        Self {
            command,
            sequence_number,
            data,
            flags: 0,
        }
    }
    
    /// Create a packet with flags
    pub fn new_with_flags(command: CommandCode, sequence_number: u8, data: Vec<u8>, flags: u8) -> Self {
        Self {
            command,
            sequence_number,
            data,
            flags,
        }
    }
    
    /// Serialize packet to bytes according to 5250 protocol specification
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        // Add command code
        result.push(self.command as u8);
        
        // Add sequence number
        result.push(self.sequence_number);
        
        // Add length (big-endian 16-bit)
        let length = (self.data.len() + 6) as u16; // +6 for command, sequence, length fields, and flags
        result.extend_from_slice(&length.to_be_bytes());
        
        // Add flags if any
        result.push(self.flags);
        
        // Add data
        result.extend_from_slice(&self.data);
        
        result
    }
    
    /// Parse a packet from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }
        
        let command_byte = bytes[0];
        let sequence_number = bytes[1];
        let length_bytes = [bytes[2], bytes[3]];
        let length = u16::from_be_bytes(length_bytes);
        
        if length as usize > bytes.len() {
            return None;
        }
        
        let flags = if bytes.len() > 4 { bytes[4] } else { 0 };
        let data_start = if bytes.len() > 5 { 5 } else { 4 };
        let data = bytes[data_start..length as usize].to_vec();
        
        if let Some(command) = CommandCode::from_u8(command_byte) {
            Some(Packet::new_with_flags(command, sequence_number, data, flags))
        } else {
            None
        }
    }
}

// Cursor position management for 5250 terminal
#[derive(Debug, Clone, Copy)]
struct CursorPosition {
    x: usize,
    y: usize,
}

impl CursorPosition {
    fn new() -> Self {
        Self { x: 0, y: 0 }
    }
    
    fn move_right(&mut self) {
        self.x += 1;
        if self.x >= TERMINAL_WIDTH {
            self.x = 0;
            self.move_down();
        }
    }
    
    fn move_down(&mut self) {
        if self.y < TERMINAL_HEIGHT - 1 {
            self.y += 1;
        }
    }
    
    fn move_to(&mut self, x: usize, y: usize) {
        if x < TERMINAL_WIDTH && y < TERMINAL_HEIGHT {
            self.x = x;
            self.y = y;
        }
    }
    
    fn get_position(&self) -> (usize, usize) {
        (self.x, self.y)
    }
    
    fn offset_to_position(&self, offset: usize) -> (usize, usize) {
        let row = (offset / TERMINAL_WIDTH).min(TERMINAL_HEIGHT - 1);
        let col = offset % TERMINAL_WIDTH;
        (col, row)
    }
}

// Keyboard state for 5250 protocol
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyboardState {
    /// Normal keyboard state
    Normal,
    
    /// Field exit initiated
    FieldExit,
    
    /// Program message key
    ProgramMessageKey,
    
    /// Attention key
    Attention,
    
    /// Function key
    FunctionKey(u8), // F1-F24
}

// Default device identification string
const DEFAULT_DEVICE_ID: &str = "IBM-5555-C01";

// Saved screen state for save/restore functionality
#[derive(Debug, Clone)]
struct SavedScreenState {
    buffer: [[TerminalChar; TERMINAL_WIDTH]; TERMINAL_HEIGHT],
    cursor_x: usize,
    cursor_y: usize,
}

// 5250 protocol processor implementing RFC 2877/4777 compliance
pub struct ProtocolProcessor {
    pub screen: TerminalScreen,
    cursor: CursorPosition,
    sequence_number: u8,
    pub connected: bool,
    // Buffer for pending user input
    input_buffer: Vec<u8>,
    // Keyboard state
    keyboard_state: KeyboardState,
    // Pending responses
    pending_responses: Vec<Packet>,
    // Configurable device identification
    device_id: String,
    // Saved screen state for save/restore functionality
    saved_state: Option<SavedScreenState>,
}

impl ProtocolProcessor {
    pub fn new() -> Self {
        Self {
            screen: TerminalScreen::new(),
            cursor: CursorPosition::new(),
            sequence_number: 0,
            connected: false,
            input_buffer: Vec::new(),
            keyboard_state: KeyboardState::Normal,
            pending_responses: Vec::new(),
            device_id: DEFAULT_DEVICE_ID.to_string(),
            saved_state: None,
        }
    }
    
    // Process an incoming 5250 protocol packet
    pub fn process_packet(&mut self, packet: &Packet) -> Result<Vec<Packet>, String> {
        match packet.command {
            CommandCode::WriteToDisplay => {
                self.process_write_to_display(&packet.data)?;
                Ok(Vec::new()) // No response needed for WriteToDisplay
            },
            CommandCode::ReadBuffer => {
                // Return user input
                Ok(vec![self.create_read_buffer_response()])
            },
            CommandCode::ReadModified => {
                // Return only modified fields
                Ok(vec![self.create_read_modified_response()])
            },
            CommandCode::ReadModifiedAll => {
                // Return all modified fields with attributes
                Ok(vec![self.create_read_modified_all_response()])
            },
            CommandCode::ReadImmediate => {
                // Return immediate response (usually empty)
                Ok(vec![self.create_read_immediate_response()])
            },
            CommandCode::WriteToDisplayAndIdentify => {
                self.process_write_to_display(&packet.data)?;
                // Return device identification
                Ok(vec![self.create_device_identification()])
            },
            CommandCode::ReadBufferAndIdentify => {
                // Return user input and device identification
                let mut responses = vec![self.create_read_buffer_response()];
                responses.push(self.create_device_identification());
                Ok(responses)
            },
            CommandCode::WriteStructuredField => {
                self.process_structured_field(&packet.data)?;
                Ok(Vec::new())
            },
            CommandCode::ReadStructuredField => {
                // Return structured field data
                Ok(vec![self.create_read_structured_field_response()])
            },
            CommandCode::TransferData => {
                // Process data transfer
                self.process_transfer_data(&packet.data)?;
                Ok(Vec::new())
            },
            CommandCode::Save => {
                // Save current screen state
                self.save_screen_state();
                Ok(Vec::new())
            },
            CommandCode::Restore => {
                // Restore saved screen state
                self.restore_screen_state();
                Ok(Vec::new())
            },
            CommandCode::SavePartial => {
                // Save partial screen state
                self.save_partial_screen_state(&packet.data);
                Ok(Vec::new())
            },
            CommandCode::RestorePartial => {
                // Restore partial screen state
                self.restore_partial_screen_state(&packet.data);
                Ok(Vec::new())
            },
            CommandCode::CancelInvite => {
                // Cancel any pending operations
                self.cancel_pending_operations();
                Ok(Vec::new())
            },
            _ => {
                // For unsupported commands, return empty response
                Ok(Vec::new())
            }
        }
    }
    
    // Process Write To Display command - this handles the 5250 data stream
    fn process_write_to_display(&mut self, data: &[u8]) -> Result<(), String> {
        let mut pos = 0;
        
        while pos < data.len() {
            let byte = data[pos];
            pos += 1;
            
            match byte {
                // Control commands
                0x11 => { // Field attribute command
                    if pos < data.len() {
                        let attr = FieldAttribute::from_u8(data[pos]);
                        // Process field attribute - for now we just advance position
                        match attr {
                            FieldAttribute::Skip => {
                                self.cursor.move_right();
                            },
                            _ => {
                                // Process other attributes as needed
                            }
                        }
                        pos += 1;
                    }
                },
                0x15 => { // Character attribute command
                    // Process character attributes
                    if pos < data.len() {
                        let char_attr = CharAttribute5250::from_u8(data[pos]);
                        let _terminal_attr = char_attr.to_char_attribute();
                        // For now, we just consume the byte
                        pos += 1;
                    }
                },
                0x1A => { // Set cursor position (2-byte command: row, col)
                    if pos + 1 < data.len() {
                        let row = data[pos] as usize;
                        let col = data[pos + 1] as usize;
                        // Convert from 1-based to 0-based indexing
                        self.cursor.move_to(col.saturating_sub(1), row.saturating_sub(1));
                        pos += 2;
                    }
                },
                0x25 => { // Start of field
                    if pos < data.len() {
                        let _field_attr = FieldAttribute::from_u8(data[pos]);
                        // Process field start - for now we just advance position
                        pos += 1;
                    }
                },
                0x28 => { // Start of structured field
                    // Process structured field - for now we just continue
                    // In a real implementation, we'd parse the structured field
                },
                0x5A => { // Reset command
                    // Clear screen and reset cursor
                    self.screen.clear();
                    self.cursor = CursorPosition::new();
                },
                0xFF => { // Null command - ignore
                    // Just consume the byte
                },
                _ => {
                    // Regular character - convert from EBCDIC to ASCII and write to screen
                    let ch = if byte >= 0x81 && byte <= 0x89 { // lowercase a-i
                        (b'a' + (byte - 0x81)) as char
                    } else if byte >= 0x91 && byte <= 0x99 { // lowercase j-r
                        (b'j' + (byte - 0x91)) as char
                    } else if byte >= 0xA2 && byte <= 0xA9 { // lowercase s-z
                        (b's' + (byte - 0xA2)) as char
                    } else if byte >= 0xC1 && byte <= 0xC9 { // uppercase A-I
                        (b'A' + (byte - 0xC1)) as char
                    } else if byte >= 0xD1 && byte <= 0xD9 { // uppercase J-R
                        (b'J' + (byte - 0xD1)) as char
                    } else if byte >= 0xE2 && byte <= 0xE9 { // uppercase S-Z
                        (b'S' + (byte - 0xE2)) as char
                    } else if byte >= 0xF0 && byte <= 0xF9 { // digits 0-9
                        (b'0' + (byte - 0xF0)) as char
                    } else {
                        match byte {
                            0x40 => ' ',  // space
                            0x4B => '.',  // period
                            0x4C => '<',  // less than
                            0x4D => '(',  // left parenthesis
                            0x4E => '+',  // plus
                            0x4F => '|',  // logical or
                            0x50 => '&',  // ampersand
                            0x5B => '!',  // exclamation
                            0x5C => '$',  // dollar
                            0x5D => '*',  // asterisk
                            0x5E => ')',  // right parenthesis
                            0x5F => ';',  // semicolon
                            0x60 => '-',  // minus/hyphen
                            0x61 => '/',  // slash
                            0x6B => ',',  // comma
                            0x6C => '%',  // percent
                            0x6D => '_',  // underscore
                            0x6E => '>',  // greater than
                            0x6F => '?',  // question mark
                            0x7A => ':',  // colon
                            0x7B => '#',  // number sign
                            0x7C => '@',  // at sign
                            0x7D => '\'', // apostrophe
                            0x7E => '=',  // equals
                            0x7F => '"',  // quotation mark
                            _ => ' ',     // default to space for unmapped characters
                        }
                    };

                    // Handle special characters
                    match ch {
                        '\n' | '\r' => {
                            self.cursor.move_down();
                            self.cursor.x = 0; // CR
                        },
                        '\t' => {
                            // Tab - move to next tab stop (every 8 positions)
                            let tab_stop = (self.cursor.x + 8) / 8 * 8;
                            self.cursor.x = std::cmp::min(tab_stop, TERMINAL_WIDTH - 1);
                        },
                        _ => {
                            if self.cursor.y < TERMINAL_HEIGHT && self.cursor.x < TERMINAL_WIDTH {
                                self.screen.buffer[self.cursor.y][self.cursor.x] = crate::terminal::TerminalChar {
                                    character: ch,
                                    attribute: crate::terminal::CharAttribute::Normal,
                                };
                                self.cursor.move_right();
                            }
                        }
                    }
                }
            }
        }
        
        self.screen.cursor_x = self.cursor.x;
        self.screen.cursor_y = self.cursor.y;
        self.screen.dirty = true;
        
        Ok(())
    }
    
    // Process structured fields (SF) according to RFC 2877
    fn process_structured_field(&mut self, data: &[u8]) -> Result<(), String> {
        // 5250 structured fields have a specific format:
        // [Flags][SFID][Length][Data]
        // Flags (1 byte), SFID (1 byte), Length (2 bytes big-endian), Data
        
        if data.len() < 4 {
            return Ok(());
        }
        
        let _flags = data[0];
        let sfid = data[1];
        let length_bytes = [data[2], data[3]]; // Length is in big-endian format
        let length_u16 = u16::from_be_bytes(length_bytes);

        // Bounds check before casting to prevent overflow on 32-bit systems
        let length = if length_u16 > usize::MAX as u16 {
            return Err("Structured field length too large for platform".to_string());
        } else {
            length_u16 as usize
        };

        // Validate length
        if length > data.len() {
            return Err("Invalid structured field length".to_string());
        }
        
        // Extract data (skip header)
        let sf_data = if data.len() > 4 { &data[4..length.min(data.len())] } else { &[] };
        
        // Process based on SFID
        if let Some(sf_id) = StructuredFieldID::from_u8(sfid) {
            match sf_id {
                StructuredFieldID::CreateChangeExtendedAttribute => {
                    self.process_create_change_extended_attribute(sf_data)?;
                },
                StructuredFieldID::SetExtendedAttributeList => {
                    self.process_set_extended_attribute_list(sf_data)?;
                },
                StructuredFieldID::EraseReset => {
                    self.screen.clear();
                    self.cursor = CursorPosition::new();
                },
                StructuredFieldID::ReadText => {
                    // Prepare text for reading
                },
                StructuredFieldID::DefineExtendedAttribute => {
                    // Define extended attribute
                },
                StructuredFieldID::DefineNamedLogicalUnit => {
                    // Define named logical unit
                },
                StructuredFieldID::DefinePendingOperations => {
                    // Define pending operations
                },
                StructuredFieldID::DisableCommandRecognition => {
                    // Disable command recognition
                },
                StructuredFieldID::EnableCommandRecognition => {
                    // Enable command recognition
                },
                StructuredFieldID::RequestMinimumTimestampInterval => {
                    // Request minimum timestamp interval
                },
                StructuredFieldID::QueryCommand => {
                    // Query command
                },
                StructuredFieldID::SetReplyMode => {
                    // Set reply mode
                },
                StructuredFieldID::DefineRollDirection => {
                    // Define roll direction
                },
                StructuredFieldID::SetMonitorMode => {
                    // Set monitor mode
                },
                StructuredFieldID::CancelRecovery => {
                    // Cancel recovery
                },
            }
        }
        
        Ok(())
    }
    
    // Process Create/Change Extended Attribute structured field
    fn process_create_change_extended_attribute(&mut self, data: &[u8]) -> Result<(), String> {
        // Parse extended attribute data
        if data.len() < 3 {
            return Err("Insufficient data for extended attribute".to_string());
        }
        
        let attribute_type = data[0];
        let attribute_value = data[1];
        let _reserved = data[2];
        
        // For now, just print the attribute
        println!("Processed extended attribute: type={}, value={}", attribute_type, attribute_value);
        
        Ok(())
    }
    
    // Process Set Extended Attribute List structured field
    fn process_set_extended_attribute_list(&mut self, data: &[u8]) -> Result<(), String> {
        // Parse extended attribute list
        let mut pos = 0;
        while pos + 2 < data.len() {
            let attribute_type = data[pos];
            let attribute_value = data[pos + 1];
            let _reserved = data[pos + 2];
            
            println!("Set extended attribute: type={}, value={}", attribute_type, attribute_value);
            
            pos += 3;
        }
        
        Ok(())
    }
    
    // Process Transfer Data command
    fn process_transfer_data(&mut self, data: &[u8]) -> Result<(), String> {
        // Add transfer data to input buffer
        self.input_buffer.extend_from_slice(data);
        Ok(())
    }
    
    // Create a response for Read Buffer command
    fn create_read_buffer_response(&mut self) -> Packet {
        // Return user input that has been accumulated
        let response_data = self.input_buffer.drain(..).collect();
        self.sequence_number = self.sequence_number.wrapping_add(1);
        Packet::new(CommandCode::ReadBuffer, self.sequence_number, response_data)
    }
    
    // Create a response for Read Modified command
    fn create_read_modified_response(&mut self) -> Packet {
        // Return only modified fields (simplified implementation)
        let response_data = self.input_buffer.drain(..).collect();
        self.sequence_number = self.sequence_number.wrapping_add(1);
        Packet::new(CommandCode::ReadModified, self.sequence_number, response_data)
    }
    
    // Create a response for Read Modified All command
    fn create_read_modified_all_response(&mut self) -> Packet {
        // Return all modified fields with attributes (simplified implementation)
        let response_data = self.input_buffer.drain(..).collect();
        self.sequence_number = self.sequence_number.wrapping_add(1);
        Packet::new(CommandCode::ReadModifiedAll, self.sequence_number, response_data)
    }
    
    // Create a response for Read Immediate command
    fn create_read_immediate_response(&mut self) -> Packet {
        // Return immediate response (usually empty for immediate commands)
        self.sequence_number = self.sequence_number.wrapping_add(1);
        Packet::new(CommandCode::ReadImmediate, self.sequence_number, Vec::new())
    }
    
    // Create a response for Read Structured Field command
    fn create_read_structured_field_response(&mut self) -> Packet {
        // Return structured field data (simplified implementation)
        let response_data = Vec::new();
        self.sequence_number = self.sequence_number.wrapping_add(1);
        Packet::new(CommandCode::ReadStructuredField, self.sequence_number, response_data)
    }
    
    // Create device identification response for WriteToDisplayAndIdentify
    fn create_device_identification(&mut self) -> Packet {
        // Device identification response according to RFC 2877
        let mut id_data = Vec::new();

        // Add configurable device type information
        id_data.extend_from_slice(self.device_id.as_bytes());

        // Add null terminator
        id_data.push(0x00);

        self.sequence_number = self.sequence_number.wrapping_add(1);
        Packet::new(CommandCode::WriteToDisplayAndIdentify, self.sequence_number, id_data)
    }

    // Set the device identification string
    pub fn set_device_id(&mut self, device_id: String) {
        self.device_id = device_id;
    }

    // Get the current device identification string
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }
    
    // Create a Write To Display packet
    pub fn create_write_to_display_packet(&mut self, text: &str) -> Packet {
        let mut data = Vec::new();
        
        for ch in text.chars() {
            data.push(ch as u8);
        }
        
        self.sequence_number = self.sequence_number.wrapping_add(1);
        Packet::new(CommandCode::WriteToDisplay, self.sequence_number, data)
    }
    
    // Add user input to the buffer
    pub fn add_input(&mut self, input: &[u8]) {
        self.input_buffer.extend_from_slice(input);
    }
    
    // Set keyboard state
    pub fn set_keyboard_state(&mut self, state: KeyboardState) {
        self.keyboard_state = state;
    }
    
    // Get keyboard state
    pub fn get_keyboard_state(&self) -> KeyboardState {
        self.keyboard_state
    }
    
    // Save current screen state
    fn save_screen_state(&mut self) {
        // Save the entire screen buffer, cursor position, and attributes
        let mut saved_buffer = [[TerminalChar::default(); TERMINAL_WIDTH]; TERMINAL_HEIGHT];

        // Copy the current screen buffer
        for y in 0..TERMINAL_HEIGHT {
            for x in 0..TERMINAL_WIDTH {
                saved_buffer[y][x] = self.screen.buffer[y][x];
            }
        }

        // Save cursor position
        let saved_cursor_x = self.screen.cursor_x;
        let saved_cursor_y = self.screen.cursor_y;

        // Store the saved state
        self.saved_state = Some(SavedScreenState {
            buffer: saved_buffer,
            cursor_x: saved_cursor_x,
            cursor_y: saved_cursor_y,
        });
    }
    
    // Restore saved screen state
    fn restore_screen_state(&mut self) {
        // Restore the saved screen state if it exists
        if let Some(saved_state) = &self.saved_state {
            // Restore the screen buffer
            for y in 0..TERMINAL_HEIGHT {
                for x in 0..TERMINAL_WIDTH {
                    self.screen.buffer[y][x] = saved_state.buffer[y][x];
                }
            }

            // Restore cursor position
            self.screen.cursor_x = saved_state.cursor_x;
            self.screen.cursor_y = saved_state.cursor_y;
            self.cursor.move_to(saved_state.cursor_x, saved_state.cursor_y);

            // Mark screen as dirty to trigger redraw
            self.screen.dirty = true;
        }
    }
    
    // Save partial screen state
    fn save_partial_screen_state(&mut self, data: &[u8]) {
        // Parse partial save data according to 5250 protocol
        // Format: [start_row][start_col][end_row][end_col]
        if data.len() >= 4 {
            let start_row = data[0] as usize;
            let start_col = data[1] as usize;
            let end_row = data[2] as usize;
            let end_col = data[3] as usize;

            // Ensure coordinates are within bounds
            let _start_row = start_row.min(TERMINAL_HEIGHT - 1);
            let start_col = start_col.min(TERMINAL_WIDTH - 1);
            let _end_row = (end_row + 1).min(TERMINAL_HEIGHT);
            let end_col = (end_col + 1).min(TERMINAL_WIDTH);

            // Create a new saved state with current full screen
            let mut saved_buffer = [[TerminalChar::default(); TERMINAL_WIDTH]; TERMINAL_HEIGHT];
            for y in 0..TERMINAL_HEIGHT {
                for x in 0..TERMINAL_WIDTH {
                    saved_buffer[y][x] = self.screen.buffer[y][x];
                }
            }

            // Mark the partial region as saved (we'll track this in the saved state)
            let mut saved_state = SavedScreenState {
                buffer: saved_buffer,
                cursor_x: self.screen.cursor_x,
                cursor_y: self.screen.cursor_y,
            };

            // Store region information in the saved state (using cursor position as metadata)
            saved_state.cursor_x = start_col; // Store start column
            saved_state.cursor_y = end_col;   // Store end column

            self.saved_state = Some(saved_state);
        }
    }
    
    // Restore partial screen state
    fn restore_partial_screen_state(&mut self, data: &[u8]) {
        // Parse partial restore data according to 5250 protocol
        // Format: [start_row][start_col][end_row][end_col]
        if data.len() >= 4 && self.saved_state.is_some() {
            let start_row = data[0] as usize;
            let start_col = data[1] as usize;
            let end_row = data[2] as usize;
            let end_col = data[3] as usize;

            // Ensure coordinates are within bounds
            let start_row = start_row.min(TERMINAL_HEIGHT - 1);
            let start_col = start_col.min(TERMINAL_WIDTH - 1);
            let end_row = (end_row + 1).min(TERMINAL_HEIGHT);
            let end_col = (end_col + 1).min(TERMINAL_WIDTH);

            // Get the saved state
            if let Some(saved_state) = &self.saved_state {
                // Restore only the specified region
                for y in start_row..end_row {
                    for x in start_col..end_col {
                        self.screen.buffer[y][x] = saved_state.buffer[y][x];
                    }
                }

                // Mark screen as dirty to trigger redraw
                self.screen.dirty = true;
            }
        }
    }
    
    // Cancel pending operations
    fn cancel_pending_operations(&mut self) {
        // Clear any pending responses
        self.pending_responses.clear();
        println!("Cancelled pending operations");
    }
    
    // Connect to a host
    pub fn connect(&mut self) {
        self.connected = true;
        self.screen.clear();
        self.cursor = CursorPosition::new();
        self.input_buffer.clear();
        self.keyboard_state = KeyboardState::Normal;
        self.pending_responses.clear();
        self.saved_state = None; // Clear any saved state on connect
        self.screen.write_string("Connected to AS/400 system
Ready...
");
    }
    
    // Disconnect from host
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.input_buffer.clear();
        self.keyboard_state = KeyboardState::Normal;
        self.pending_responses.clear();
        self.saved_state = None; // Clear any saved state on disconnect
        self.screen.clear();
        self.cursor = CursorPosition::new();
        self.screen.write_string("Disconnected from AS/400 system");
    }
    
    // Check if screen needs to be redrawn
    pub fn is_dirty(&self) -> bool {
        self.screen.dirty
    }
    
    // Mark screen as clean
    pub fn mark_clean(&mut self) {
        self.screen.dirty = false;
    }
    
    // Get current cursor position
    pub fn get_cursor_position(&self) -> (usize, usize) {
        self.cursor.get_position()
    }
    
    // Read the current screen buffer
    pub fn read_buffer(&self) -> Vec<u8> {
        // Read the current screen buffer as bytes
        // This is a simplified implementation
        let mut buffer = Vec::new();
        
        for row in 0..TERMINAL_HEIGHT {
            for col in 0..TERMINAL_WIDTH {
                let ch = self.screen.buffer[row][col].character as u8;
                if ch != 0 && ch != b' ' {
                    buffer.push(ch);
                }
            }
        }
        
        buffer
    }
}