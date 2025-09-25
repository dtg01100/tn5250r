//! 5250 Protocol State Machine
//!
//! This module handles the state management for the IBM 5250 protocol,
//! including connection negotiation, command processing, and field management.

use crate::terminal::{TerminalScreen, TERMINAL_WIDTH, TERMINAL_HEIGHT, CharAttribute};

// EBCDIC CP037 to ASCII translation table for IBM 5250 terminals
// This is the standard IBM EBCDIC Code Page 037 (US English) mapping
// Used by AS/400 and IBM i systems for 5250 terminal communication
const EBCDIC_CP037_TO_ASCII: [char; 256] = [
    '\x00', '\x01', '\x02', '\x03', '\x37', '\x2D', '\x2E', '\x2F', // 0x00-0x07
    '\x16', '\x05', '\x25', '\x0B', '\x0C', '\r',   '\x0E', '\x0F', // 0x08-0x0F  
    '\x10', '\x11', '\x12', '\x13', '\x3C', '\x3D', '\x32', '\x26', // 0x10-0x17
    '\x18', '\x19', '\x3F', '\x27', '\x1C', '\x1D', '\x1E', '\x1F', // 0x18-0x1F
    '\x40', '\x5A', '\x7F', '\x7B', '\x5B', '\n',   '\x17', '\x1B', // 0x20-0x27
    '\x60', '\x61', '\x62', '\x63', '\x64', '\x65', '\x66', '\x67', // 0x28-0x2F
    '\x68', '\x69', '\x70', '\x71', '\x72', '\x73', '\x74', '\x75', // 0x30-0x37
    '\x76', '\x77', '\x78', '\x79', '\x7A', '\x7B', '\x7C', '\x7D', // 0x38-0x3F
    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    // 0x40-0x47 
    ' ',    ' ',    '[',    '.',    '<',    '(',    '+',    '|',    // 0x48-0x4F
    '&',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    // 0x50-0x57
    ' ',    ' ',    '!',    '$',    '*',    ')',    ';',    ' ',    // 0x58-0x5F
    '-',    '/',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    // 0x60-0x67
    ' ',    ' ',    '|',    ',',    '%',    '_',    '>',    '?',    // 0x68-0x6F
    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    // 0x70-0x77
    ' ',    '`',    ':',    '#',    '@',    '\'',   '=',    '"',    // 0x78-0x7F
    ' ',    'a',    'b',    'c',    'd',    'e',    'f',    'g',    // 0x80-0x87
    'h',    'i',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    // 0x88-0x8F
    ' ',    'j',    'k',    'l',    'm',    'n',    'o',    'p',    // 0x90-0x97
    'q',    'r',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    // 0x98-0x9F
    ' ',    '~',    's',    't',    'u',    'v',    'w',    'x',    // 0xA0-0xA7
    'y',    'z',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    // 0xA8-0xAF
    '^',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    // 0xB0-0xB7
    ' ',    ' ',    '[',    ']',    ' ',    ' ',    ' ',    ' ',    // 0xB8-0xBF
    '{',    'A',    'B',    'C',    'D',    'E',    'F',    'G',    // 0xC0-0xC7
    'H',    'I',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    // 0xC8-0xCF
    '}',    'J',    'K',    'L',    'M',    'N',    'O',    'P',    // 0xD0-0xD7
    'Q',    'R',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    // 0xD8-0xDF
    '\\',   ' ',    'S',    'T',    'U',    'V',    'W',    'X',    // 0xE0-0xE7
    'Y',    'Z',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    // 0xE8-0xEF
    '0',    '1',    '2',    '3',    '4',    '5',    '6',    '7',    // 0xF0-0xF7
    '8',    '9',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    // 0xF8-0xFF
];

// Standard EBCDIC to ASCII translation table for IBM 5250 terminals
// This table converts EBCDIC characters to their ASCII equivalents
const _STANDARD_EBCDIC_TO_ASCII: [char; 256] = [
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x00-0x07
    '\0', '\t', '\0', '\0', '\0', '\n', '\0', '\0', // 0x08-0x0F (tab, newline)
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x10-0x17
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x18-0x1F
    ' ', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x20-0x27 (space)
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x28-0x2F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x30-0x37
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x38-0x3F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x40-0x47
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x48-0x4F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x50-0x57
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x58-0x5F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x60-0x67
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x68-0x6F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x70-0x77
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x78-0x7F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x80-0x87
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x88-0x8F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x90-0x97
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x98-0x9F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xA0-0xA7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xA8-0xAF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xB0-0xB7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xB8-0xBF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xC0-0xC7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xC8-0xCF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xD0-0xD7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xD8-0xDF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xE0-0xE7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xE8-0xEF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xF0-0xF7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xF8-0xFF
];

// Complete EBCDIC to ASCII translation table for IBM 5250 terminals
// This includes the most common EBCDIC character mappings
const _COMPLETE_EBCDIC_TO_ASCII: [char; 256] = [
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x00-0x07
    '\0', '\t', '\0', '\0', '\0', '\n', '\0', '\0', // 0x08-0x0F (tab, newline)
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x10-0x17
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x18-0x1F
    ' ', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x20-0x27 (space)
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x28-0x2F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x30-0x37
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x38-0x3F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x40-0x47
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x48-0x4F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x50-0x57
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x58-0x5F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x60-0x67
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x68-0x6F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x70-0x77
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x78-0x7F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x80-0x87
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x88-0x8F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x90-0x97
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x98-0x9F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xA0-0xA7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xA8-0xAF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xB0-0xB7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xB8-0xBF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xC0-0xC7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xC8-0xCF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xD0-0xD7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xD8-0xDF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xE0-0xE7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xE8-0xEF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xF0-0xF7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xF8-0xFF
];

// Standard EBCDIC to ASCII translation table for IBM 5250 terminals
// This is the most commonly used mapping for AS/400 systems
const _EBCDIC_TO_ASCII_TABLE: [char; 256] = [
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x00-0x07
    '\0', '\t', '\0', '\0', '\0', '\n', '\0', '\0', // 0x08-0x0F (tab, newline)
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x10-0x17
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x18-0x1F
    ' ', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x20-0x27 (space)
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x28-0x2F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x30-0x37
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x38-0x3F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x40-0x47
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x48-0x4F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x50-0x57
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x58-0x5F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x60-0x67
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x68-0x6F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x70-0x77
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x78-0x7F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x80-0x87
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x88-0x8F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x90-0x97
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x98-0x9F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xA0-0xA7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xA8-0xAF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xB0-0xB7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xB8-0xBF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xC0-0xC7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xC8-0xCF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xD0-0xD7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xD8-0xDF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xE0-0xE7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xE8-0xEF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xF0-0xF7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xF8-0xFF
];

// EBCDIC to ASCII translation table for IBM 5250 terminals
// This table maps EBCDIC character codes to their ASCII equivalents
// Based on standard IBM EBCDIC code page 037 (US English)
const _IBM_EBCDIC_TO_ASCII: [char; 256] = [
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x00-0x07
    '\0', '\t', '\0', '\0', '\0', '\n', '\0', '\0', // 0x08-0x0F (tab, newline)
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x10-0x17
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x18-0x1F
    ' ', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x20-0x27 (space)
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x28-0x2F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x30-0x37
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x38-0x3F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x40-0x47
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x48-0x4F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x50-0x57
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x58-0x5F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x60-0x67
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x68-0x6F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x70-0x77
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x78-0x7F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x80-0x87
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x88-0x8F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x90-0x97
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0x98-0x9F
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xA0-0xA7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xA8-0xAF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xB0-0xB7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xB8-0xBF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xC0-0xC7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xC8-0xCF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xD0-0xD7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xD8-0xDF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xE0-0xE7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xE8-0xEF
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xF0-0xF7
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', // 0xF8-0xFF
];

// Convert EBCDIC byte to ASCII character using CP037 translation table
pub fn ebcdic_to_ascii(ebcdic_byte: u8) -> char {
    EBCDIC_CP037_TO_ASCII[ebcdic_byte as usize]
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProtocolState {
    InitialNegotiation,
    Connected,
    Receiving,
    Sending,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldType {
    Normal,
    Protected,
    Numeric,
    Skip,
    Mandatory,
    DupEnable,
    Hidden,
}

#[derive(Debug)]
pub struct Field {
    pub start_position: usize,
    pub length: usize,
    pub field_type: FieldType,
    pub attribute: u8,
}

#[derive(Debug)]
pub struct DeviceAttributes {
    pub model_number: u8,
    pub character_set: u8,
    pub extended_char_set: bool,
    pub color_support: bool,
    pub highlighting_support: bool,
    pub max_buffer_size: u16,
}

impl DeviceAttributes {
    pub fn new() -> Self {
        Self {
            model_number: 0x02, // Common model for IBM i Access
            character_set: 0x00, // EBCDIC
            extended_char_set: false,
            color_support: false,
            highlighting_support: true,
            max_buffer_size: 1920, // Standard for 80x24
        }
    }
}

#[derive(Debug)]
pub struct ProtocolStateMachine {
    pub state: ProtocolState,
    pub screen: TerminalScreen,
    pub fields: Vec<Field>,
    pub cursor_position: usize,
    pub device_attributes: DeviceAttributes,
    pub connected: bool,
}

impl ProtocolStateMachine {
    pub fn new() -> Self {
        Self {
            state: ProtocolState::InitialNegotiation,
            screen: TerminalScreen::new(),
            fields: Vec::new(),
            cursor_position: 0,
            device_attributes: DeviceAttributes::new(),
            connected: false,
        }
    }
    
    pub fn connect(&mut self) {
        self.state = ProtocolState::Connected;
        self.connected = true;
        self.screen.clear();
        self.screen.write_string("Connected to AS/400 system\nReady...\n");
    }
    
    pub fn disconnect(&mut self) {
        self.state = ProtocolState::InitialNegotiation;
        self.connected = false;
        self.screen.clear();
        self.screen.write_string("Disconnected from AS/400 system\n");
    }
    
    pub fn process_data(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }
        
        match self.state {
            ProtocolState::Connected | ProtocolState::Receiving => {
                self.state = ProtocolState::Receiving;
                self.parse_data_stream(data)?;
                Ok(Vec::new()) // Usually no response needed for display data
            },
            ProtocolState::InitialNegotiation => {
                // Handle initial negotiation
                self.handle_negotiation(data)?;
                Ok(self.create_device_identification_response())
            },
            _ => Err("Invalid protocol state".to_string()),
        }
    }
    
    fn parse_data_stream(&mut self, data: &[u8]) -> Result<(), String> {
        let mut pos = 0;
        let mut current_field_type = FieldType::Normal;
        let mut _current_attribute: Option<u8> = None;
        
        while pos < data.len() {
            let byte = data[pos];
            pos += 1;
            
            match byte {
                0x11 => { // Field attribute
                    if pos < data.len() {
                        let attr = data[pos];
                        _current_attribute = Some(attr);
                        current_field_type = self.determine_field_type(attr);
                        pos += 1;
                    }
                },
                0x15 => { // Character attribute
                    if pos < data.len() {
                        let _char_attr = data[pos];
                        // Process character attribute
                        pos += 1;
                    }
                },
                0x1A => { // Set cursor position (2-byte command: row, col)
                    if pos + 1 < data.len() {
                        let row = data[pos] as usize;
                        let col = data[pos + 1] as usize;
                        // Convert from 1-based to 0-based indexing
                        self.set_cursor_position(
                            col.saturating_sub(1), 
                            row.saturating_sub(1)
                        );
                        pos += 2;
                    }
                },
                0x25 => { // Start of field
                    if pos < data.len() {
                        let field_attr = data[pos];
                        _current_attribute = Some(field_attr);
                        current_field_type = self.determine_field_type(field_attr);
                        // Record field start position
                        self.add_field(self.cursor_position, 0, current_field_type, field_attr);
                        pos += 1;
                    }
                },
                0x28 => { // Start of structured field
                    // Handle structured field - for now, just skip
                    if pos + 1 < data.len() {
                        let length = ((data[pos] as u16) << 8) | data[pos + 1] as u16;
                        pos += (length as usize) + 2; // Skip length + data
                    }
                },
                0x5A => { // Reset command
                    self.screen.clear();
                    self.cursor_position = 0;
                },
                _ => {
                    // Regular character - convert from EBCDIC to ASCII and write to screen
                    let ch = ebcdic_to_ascii(byte);

                    match ch {
                        '\n' => {
                            // Move to next line
                            let current_row = self.cursor_position / TERMINAL_WIDTH;
                            if current_row < TERMINAL_HEIGHT - 1 {
                                self.cursor_position =
                                    (current_row + 1) * TERMINAL_WIDTH + (self.cursor_position % TERMINAL_WIDTH);
                            }
                        },
                        '\r' => {
                            // Move to beginning of line
                            let current_row = self.cursor_position / TERMINAL_WIDTH;
                            self.cursor_position = current_row * TERMINAL_WIDTH;
                        },
                        '\t' => {
                            // Tab - move to next tab stop
                            let tab_stop = (self.cursor_position % TERMINAL_WIDTH + 8) / 8 * 8;
                            let current_row = self.cursor_position / TERMINAL_WIDTH;
                            self.cursor_position = current_row * TERMINAL_WIDTH + tab_stop;
                        },
                        _ => {
                            if self.cursor_position < TERMINAL_WIDTH * TERMINAL_HEIGHT {
                                // Determine character attribute based on field type
                                let char_attr = match current_field_type {
                                    FieldType::Protected => CharAttribute::Protected,
                                    FieldType::Numeric => CharAttribute::Numeric,
                                    FieldType::Hidden => CharAttribute::Hidden,
                                    FieldType::DupEnable => CharAttribute::DupEnable,
                                    FieldType::Mandatory => CharAttribute::NonDisplay, // Simplified
                                    FieldType::Skip => CharAttribute::NonDisplay,    // Simplified
                                    FieldType::Normal => CharAttribute::Normal,
                                };

                                // Write character to screen
                                let row = self.cursor_position / TERMINAL_WIDTH;
                                let col = self.cursor_position % TERMINAL_WIDTH;

                                if row < TERMINAL_HEIGHT && col < TERMINAL_WIDTH {
                                    self.screen.buffer[row][col] = crate::terminal::TerminalChar {
                                        character: ch,
                                        attribute: char_attr,
                                    };
                                    self.cursor_position += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn handle_negotiation(&mut self, _data: &[u8]) -> Result<(), String> {
        // Initial negotiation - identify our terminal capabilities
        self.state = ProtocolState::Connected;
        Ok(())
    }
    
    fn create_device_identification_response(&self) -> Vec<u8> {
        // Return device identification information
        // This is a simplified version
        vec![
            0xF0, 0xF0, 0xF0, 0xF0, // Device type
            0xF1, 0xF2, 0xF3, 0xF4, // Additional info
        ]
    }
    
    fn determine_field_type(&self, attribute: u8) -> FieldType {
        // Determine field type from attribute byte
        // This is a simplified interpretation
        if attribute & 0x20 != 0 { // Protected field
            FieldType::Protected
        } else if attribute & 0x10 != 0 { // Numeric field
            FieldType::Numeric
        } else if attribute & 0x08 != 0 { // Skip field
            FieldType::Skip
        } else if attribute & 0x18 != 0 { // Mandatory field
            FieldType::Mandatory
        } else if attribute & 0x04 != 0 { // Duplicate enable
            FieldType::DupEnable
        } else if attribute & 0x0C != 0 { // Hidden field
            FieldType::Hidden
        } else {
            FieldType::Normal
        }
    }
    
    pub fn set_cursor_position(&mut self, col: usize, row: usize) {
        if row < TERMINAL_HEIGHT && col < TERMINAL_WIDTH {
            self.cursor_position = row * TERMINAL_WIDTH + col;
        }
    }
    
    pub fn get_cursor_position(&self) -> (usize, usize) {
        let row = self.cursor_position / TERMINAL_WIDTH;
        let col = self.cursor_position % TERMINAL_WIDTH;
        (col, row)
    }
    
    fn add_field(&mut self, start: usize, length: usize, field_type: FieldType, attribute: u8) {
        self.fields.push(Field {
            start_position: start,
            length,
            field_type,
            attribute,
        });
    }
    
    pub fn read_buffer(&self) -> Vec<u8> {
        // Read the current screen buffer as bytes
        // Optimized implementation: pre-allocate with estimated capacity and use flatten
        let estimated_capacity = TERMINAL_WIDTH * TERMINAL_HEIGHT / 4; // Rough estimate
        let mut buffer = Vec::with_capacity(estimated_capacity);

        // Use flat_map to efficiently collect non-empty characters
        buffer.extend(
            self.screen.buffer.iter()
                .flatten()
                .filter_map(|terminal_char| {
                    let ch = terminal_char.character as u8;
                    if ch != 0 && ch != b' ' {
                        Some(ch)
                    } else {
                        None
                    }
                })
        );

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_state_machine_creation() {
        let proto = ProtocolStateMachine::new();
        assert_eq!(proto.state, ProtocolState::InitialNegotiation);
        assert!(!proto.connected);
    }

    #[test]
    fn test_connection() {
        let mut proto = ProtocolStateMachine::new();
        proto.connect();
        assert_eq!(proto.state, ProtocolState::Connected);
        assert!(proto.connected);
    }

    #[test]
    fn test_field_type_determination() {
        let proto = ProtocolStateMachine::new();
        
        // Test protected field (bit 5 set)
        assert_eq!(proto.determine_field_type(0x20), FieldType::Protected);
        
        // Test numeric field (bit 4 set)
        assert_eq!(proto.determine_field_type(0x10), FieldType::Numeric);
        
        // Test normal field
        assert_eq!(proto.determine_field_type(0x00), FieldType::Normal);
    }

    #[test]
    fn test_cursor_position() {
        let mut proto = ProtocolStateMachine::new();
        proto.set_cursor_position(10, 5); // col=10, row=5
        let (col, row) = proto.get_cursor_position();
        assert_eq!((col, row), (10, 5));
    }
}