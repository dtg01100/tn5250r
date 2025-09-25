//! Terminal emulation module for the 5250 protocol
//! 
//! This module provides the core terminal emulation functionality for 
//! displaying and handling AS/400 screens using the 5250 protocol.

use std::fmt;

// Terminal dimensions - standard IBM 5250 terminal sizes
pub const TERMINAL_WIDTH: usize = 80;
pub const TERMINAL_HEIGHT: usize = 24;

// Character attributes for 5250 terminal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharAttribute {
    Normal,
    Intensified,      // Highlighted/bright text
    NonDisplay,       // Hidden characters (password fields)
    Protected,        // Non-editable fields
    Numeric,          // Numeric-only input
    FieldExit,        // Field-exit attribute
    DupEnable,        // Duplicate enable
    Hidden,           // Hidden field
}

// Represents a single character on the terminal screen
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalChar {
    pub character: char,
    pub attribute: CharAttribute,
}

impl Default for TerminalChar {
    fn default() -> Self {
        Self {
            character: ' ',
            attribute: CharAttribute::Normal,
        }
    }
}

// Represents the terminal screen buffer
#[derive(Debug)]
pub struct TerminalScreen {
    pub buffer: [[TerminalChar; TERMINAL_WIDTH]; TERMINAL_HEIGHT],
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub dirty: bool, // Flag to indicate if screen needs to be redrawn
}

impl TerminalScreen {
    pub fn new() -> Self {
        Self {
            buffer: [[TerminalChar::default(); TERMINAL_WIDTH]; TERMINAL_HEIGHT],
            cursor_x: 0,
            cursor_y: 0,
            dirty: true,
        }
    }

    // Clear the entire screen
    pub fn clear(&mut self) {
        for row in self.buffer.iter_mut() {
            for cell in row.iter_mut() {
                *cell = TerminalChar::default();
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.dirty = true;
    }

    // Write a character at the current cursor position
    pub fn write_char(&mut self, ch: char) {
        self.write_char_with_attr(ch, CharAttribute::Normal);
    }
    
    // Write a character at the current cursor position with specific attribute
    pub fn write_char_with_attr(&mut self, ch: char, attr: CharAttribute) {
        if self.cursor_y < TERMINAL_HEIGHT && self.cursor_x < TERMINAL_WIDTH {
            self.buffer[self.cursor_y][self.cursor_x] = TerminalChar {
                character: ch,
                attribute: attr,
            };
            self.dirty = true;
            
            // Move cursor to next position (unless it's a protected field)
            if !matches!(attr, CharAttribute::Protected | CharAttribute::Hidden) {
                self.cursor_x += 1;
                if self.cursor_x >= TERMINAL_WIDTH {
                    self.cursor_x = 0;
                    if self.cursor_y < TERMINAL_HEIGHT - 1 {
                        self.cursor_y += 1;
                    }
                }
            }
        }
    }

    // Write a string starting at the current cursor position
    pub fn write_string(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }

    // Move cursor to specific position
    pub fn move_cursor(&mut self, x: usize, y: usize) {
        if x < TERMINAL_WIDTH && y < TERMINAL_HEIGHT {
            self.cursor_x = x;
            self.cursor_y = y;
        }
    }

    // Write a character at a specific position
    pub fn write_char_at(&mut self, x: usize, y: usize, ch: char) {
        if x < TERMINAL_WIDTH && y < TERMINAL_HEIGHT {
            self.buffer[y][x] = TerminalChar {
                character: ch,
                attribute: CharAttribute::Normal,
            };
            self.dirty = true;
        }
    }

    // Set TerminalChar at specific position
    pub fn set_char_at(&mut self, row: usize, col: usize, terminal_char: TerminalChar) {
        if col < TERMINAL_WIDTH && row < TERMINAL_HEIGHT {
            self.buffer[row][col] = terminal_char;
            self.dirty = true;
        }
    }

    // Get character at specific position
    pub fn get_char_at(&self, x: usize, y: usize) -> Option<char> {
        if x < TERMINAL_WIDTH && y < TERMINAL_HEIGHT {
            Some(self.buffer[y][x].character)
        } else {
            None
        }
    }
}

impl fmt::Display for TerminalScreen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.buffer {
            for cell in row {
                write!(f, "{}", cell.character)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

// Terminal emulator state
pub struct TerminalEmulator {
    pub screen: TerminalScreen,
    pub connected: bool,
    pub host: String,
    pub data_buffer: Vec<u8>,
}

impl TerminalEmulator {
    pub fn new() -> Self {
        Self {
            screen: TerminalScreen::new(),
            connected: false,
            host: String::new(),
            data_buffer: Vec::new(),
        }
    }

    // Connect to a host
    pub fn connect(&mut self, host: String) -> Result<(), String> {
        // In a real implementation, this would establish the actual TCP connection
        // For now, we'll just update the state
        self.host = host;
        self.connected = true;
        self.screen.clear();
        self.screen.write_string("Connected to AS/400 system\nReady...");
        self.data_buffer.clear();
        Ok(())
    }

    // Disconnect from host
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.host.clear();
        self.screen.clear();
        self.screen.write_string("Disconnected from AS/400 system");
        self.data_buffer.clear();
    }

    // Process incoming data
    pub fn process_data(&mut self, data: &[u8]) -> Result<(), String> {
        // Store raw data for debugging
        self.data_buffer.extend_from_slice(data);
        
        // The actual protocol parsing and EBCDIC conversion should be done
        // by the protocol state machine, not here. This is a temporary
        // implementation that just displays raw data for debugging.
        // 
        // Note: Real 5250 protocol data contains command codes, structured fields,
        // and other binary data mixed with EBCDIC text. Simply converting all
        // bytes as EBCDIC will show gibberish for protocol commands.
        
        // For now, just indicate that data was received
        // The proper implementation should use the protocol_state module
        let data_info = format!("[Received {} bytes of 5250 data]\n", data.len());
        self.screen.write_string(&data_info);
        
        Ok(())
    }

    // Process keyboard input
    pub fn process_input(&mut self, input: &str) -> Result<Vec<u8>, String> {
        // In a real implementation, this would convert keyboard input to 5250 protocol commands
        // For now, we'll just echo the input to the screen and return it
        self.screen.write_string(input);
        Ok(input.as_bytes().to_vec())
    }

    // Check if screen needs to be redrawn
    pub fn is_dirty(&self) -> bool {
        self.screen.dirty
    }

    // Reset dirty flag after redrawing
    pub fn mark_clean(&mut self) {
        self.screen.dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_screen_creation() {
        let screen = TerminalScreen::new();
        assert_eq!(screen.buffer[0][0].character, ' ');
        assert_eq!(screen.cursor_x, 0);
        assert_eq!(screen.cursor_y, 0);
    }

    #[test]
    fn test_write_char() {
        let mut screen = TerminalScreen::new();
        screen.write_char('A');
        assert_eq!(screen.buffer[0][0].character, 'A');
        assert_eq!(screen.cursor_x, 1);
    }

    #[test]
    fn test_write_string() {
        let mut screen = TerminalScreen::new();
        screen.write_string("Hello");
        assert_eq!(screen.buffer[0][0].character, 'H');
        assert_eq!(screen.buffer[0][1].character, 'e');
        assert_eq!(screen.buffer[0][2].character, 'l');
        assert_eq!(screen.buffer[0][3].character, 'l');
        assert_eq!(screen.buffer[0][4].character, 'o');
    }

    #[test]
    fn test_terminal_emulator_creation() {
        let term = TerminalEmulator::new();
        assert!(!term.connected);
        assert!(term.host.is_empty());
    }

    #[test]
    fn test_connect_disconnect() {
        let mut term = TerminalEmulator::new();
        
        // Connect
        term.connect("test.host.com".to_string()).unwrap();
        assert!(term.connected);
        assert_eq!(term.host, "test.host.com");
        
        // Disconnect
        term.disconnect();
        assert!(!term.connected);
        assert!(term.host.is_empty());
    }
}