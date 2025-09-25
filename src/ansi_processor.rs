//! ANSI/VT100 escape sequence processor for terminal emulation
//! 
//! This module handles ANSI escape sequences commonly used in VT100/NVT mode
//! terminal sessions, converting them to terminal screen updates.

use crate::terminal::{TerminalScreen, CharAttribute, TerminalChar};

#[derive(Debug, Clone)]
pub struct AnsiProcessor {
    /// Current cursor position (row, col) - 1-based
    cursor_row: usize,
    cursor_col: usize,
    
    /// Current text attributes
    current_attributes: CharAttribute,
    
    /// Buffer for collecting escape sequences
    escape_buffer: String,
    
    /// Whether we're currently in an escape sequence
    in_escape: bool,
}

impl AnsiProcessor {
    pub fn new() -> Self {
        Self {
            cursor_row: 1,
            cursor_col: 1,
            current_attributes: CharAttribute::Normal,
            escape_buffer: String::new(),
            in_escape: false,
        }
    }
    
    /// Process incoming data and update terminal screen
    pub fn process_data(&mut self, data: &[u8], screen: &mut TerminalScreen) {
        for &byte in data {
            match byte {
                0x1B => { // ESC - start of escape sequence
                    self.in_escape = true;
                    self.escape_buffer.clear();
                    self.escape_buffer.push(byte as char);
                }
                _ if self.in_escape => {
                    self.escape_buffer.push(byte as char);
                    
                    // Check if we have a complete sequence
                    if self.is_complete_sequence() {
                        self.process_escape_sequence(screen);
                        self.in_escape = false;
                        self.escape_buffer.clear();
                    }
                }
                0x0A => { // LF - Line Feed
                    self.cursor_row = (self.cursor_row + 1).min(24);
                }
                0x0D => { // CR - Carriage Return
                    self.cursor_col = 1;
                }
                0x08 => { // BS - Backspace
                    if self.cursor_col > 1 {
                        self.cursor_col -= 1;
                    }
                }
                0x09 => { // TAB
                    self.cursor_col = ((self.cursor_col + 7) / 8) * 8 + 1;
                    self.cursor_col = self.cursor_col.min(80);
                }
                0x07 => { // BEL - Bell (ignore for now)
                }
                _ if byte >= 32 && byte <= 126 => { // Printable ASCII
                    self.write_char_at_cursor(byte as char, screen);
                }
                _ => {
                    // Other control characters - ignore for now
                }
            }
        }
    }
    
    /// Check if the current escape sequence is complete
    fn is_complete_sequence(&self) -> bool {
        if self.escape_buffer.len() < 2 {
            return false;
        }
        
        let chars: Vec<char> = self.escape_buffer.chars().collect();
        
        // ESC [ sequences end with A-Z or a-z
        if chars.len() >= 3 && chars[1] == '[' {
            let last_char = *chars.last().unwrap();
            return last_char.is_ascii_alphabetic();
        }
        
        // Other ESC sequences are typically 2 characters
        if chars[1] != '[' {
            return chars.len() >= 2;
        }
        
        false
    }
    
    /// Process a complete escape sequence
    fn process_escape_sequence(&mut self, screen: &mut TerminalScreen) {
        let seq = self.escape_buffer.clone(); // Clone to avoid borrowing issues
        
        if seq.starts_with("\x1B[") {
            self.process_csi_sequence(&seq[2..], screen);
        } else if seq.len() >= 2 {
            // Simple escape sequences
            match &seq[1..2] {
                "M" => { // Reverse Index
                    if self.cursor_row > 1 {
                        self.cursor_row -= 1;
                    }
                }
                "D" => { // Index
                    self.cursor_row = (self.cursor_row + 1).min(24);
                }
                "E" => { // Next Line
                    self.cursor_row = (self.cursor_row + 1).min(24);
                    self.cursor_col = 1;
                }
                _ => {
                    // Unknown escape sequence - ignore
                }
            }
        }
    }
    
    /// Process Control Sequence Introducer (CSI) sequences
    fn process_csi_sequence(&mut self, seq: &str, screen: &mut TerminalScreen) {
        if seq.is_empty() {
            return;
        }
        
        let command = seq.chars().last().unwrap();
        let params = &seq[..seq.len()-1];
        
        match command {
            'H' | 'f' => { // Cursor Position
                let parts: Vec<&str> = params.split(';').collect();
                let row = if parts.is_empty() || parts[0].is_empty() {
                    1
                } else {
                    parts[0].parse().unwrap_or(1)
                };
                let col = if parts.len() < 2 || parts[1].is_empty() {
                    1
                } else {
                    parts[1].parse().unwrap_or(1)
                };
                
                self.cursor_row = row.max(1).min(24);
                self.cursor_col = col.max(1).min(80);
            }
            'A' => { // Cursor Up
                let count: usize = if params.is_empty() {
                    1
                } else {
                    params.parse().unwrap_or(1)
                };
                self.cursor_row = self.cursor_row.saturating_sub(count).max(1);
            }
            'B' => { // Cursor Down
                let count: usize = if params.is_empty() {
                    1
                } else {
                    params.parse().unwrap_or(1)
                };
                self.cursor_row = (self.cursor_row + count).min(24);
            }
            'C' => { // Cursor Right
                let count: usize = if params.is_empty() {
                    1
                } else {
                    params.parse().unwrap_or(1)
                };
                self.cursor_col = (self.cursor_col + count).min(80);
            }
            'D' => { // Cursor Left
                let count: usize = if params.is_empty() {
                    1
                } else {
                    params.parse().unwrap_or(1)
                };
                self.cursor_col = self.cursor_col.saturating_sub(count).max(1);
            }
            'J' => { // Erase Display
                let mode: u32 = if params.is_empty() {
                    0
                } else {
                    params.parse().unwrap_or(0)
                };
                
                match mode {
                    0 => { // Clear from cursor to end of screen
                        self.clear_from_cursor_to_end(screen);
                    }
                    1 => { // Clear from start of screen to cursor
                        self.clear_from_start_to_cursor(screen);
                    }
                    2 => { // Clear entire screen
                        screen.clear();
                        self.cursor_row = 1;
                        self.cursor_col = 1;
                    }
                    _ => {} // Other modes not implemented
                }
            }
            'K' => { // Erase Line
                let mode: u32 = if params.is_empty() {
                    0
                } else {
                    params.parse().unwrap_or(0)
                };
                
                match mode {
                    0 => { // Clear from cursor to end of line
                        self.clear_line_from_cursor(screen);
                    }
                    1 => { // Clear from start of line to cursor
                        self.clear_line_to_cursor(screen);
                    }
                    2 => { // Clear entire line
                        self.clear_entire_line(screen);
                    }
                    _ => {} // Other modes not implemented
                }
            }
            'm' => { // Select Graphic Rendition (SGR) - text attributes
                self.process_sgr_sequence(params);
            }
            'l' | 'h' => { // Reset/Set Mode
                // Various terminal modes - most can be ignored for basic display
                if params.starts_with("?3") {
                    // Column mode changes (80/132) - ignore for now
                } else if params.starts_with("?7") {
                    // Auto-wrap mode - ignore for now
                }
            }
            _ => {
                // Unknown CSI sequence - ignore
            }
        }
    }
    
    /// Process Select Graphic Rendition (text attributes)
    fn process_sgr_sequence(&mut self, params: &str) {
        if params.is_empty() {
            self.current_attributes = CharAttribute::Normal;
            return;
        }
        
        let codes: Vec<&str> = params.split(';').collect();
        for code in codes {
            match code.parse::<u32>().unwrap_or(0) {
                0 => self.current_attributes = CharAttribute::Normal,
                1 => self.current_attributes = CharAttribute::Intensified,
                4 => self.current_attributes = CharAttribute::Intensified, // Use Intensified for underline
                7 => self.current_attributes = CharAttribute::Intensified, // Use Intensified for reverse
                _ => {} // Other codes not implemented yet
            }
        }
    }
    
    /// Write character at current cursor position
    fn write_char_at_cursor(&mut self, ch: char, screen: &mut TerminalScreen) {
        if self.cursor_row >= 1 && self.cursor_row <= 24 && 
           self.cursor_col >= 1 && self.cursor_col <= 80 {
            
            let terminal_char = TerminalChar {
                character: ch,
                attribute: self.current_attributes,
            };
            
            screen.set_char_at(self.cursor_row - 1, self.cursor_col - 1, terminal_char);
            
            // Advance cursor
            self.cursor_col += 1;
            if self.cursor_col > 80 {
                self.cursor_col = 1;
                self.cursor_row = (self.cursor_row + 1).min(24);
            }
        }
    }
    
    /// Clear from cursor position to end of screen
    fn clear_from_cursor_to_end(&self, screen: &mut TerminalScreen) {
        // Clear from current position to end of current line
        for col in (self.cursor_col - 1)..80 {
            screen.set_char_at(self.cursor_row - 1, col, TerminalChar::default());
        }
        
        // Clear all lines below current
        for row in self.cursor_row..24 {
            for col in 0..80 {
                screen.set_char_at(row, col, TerminalChar::default());
            }
        }
    }
    
    /// Clear from start of screen to cursor position
    fn clear_from_start_to_cursor(&self, screen: &mut TerminalScreen) {
        // Clear all lines above current
        for row in 0..(self.cursor_row - 1) {
            for col in 0..80 {
                screen.set_char_at(row, col, TerminalChar::default());
            }
        }
        
        // Clear from start of current line to cursor
        for col in 0..self.cursor_col {
            screen.set_char_at(self.cursor_row - 1, col, TerminalChar::default());
        }
    }
    
    /// Clear from cursor to end of current line
    fn clear_line_from_cursor(&self, screen: &mut TerminalScreen) {
        for col in (self.cursor_col - 1)..80 {
            screen.set_char_at(self.cursor_row - 1, col, TerminalChar::default());
        }
    }
    
    /// Clear from start of line to cursor
    fn clear_line_to_cursor(&self, screen: &mut TerminalScreen) {
        for col in 0..self.cursor_col {
            screen.set_char_at(self.cursor_row - 1, col, TerminalChar::default());
        }
    }
    
    /// Clear entire current line
    fn clear_entire_line(&self, screen: &mut TerminalScreen) {
        for col in 0..80 {
            screen.set_char_at(self.cursor_row - 1, col, TerminalChar::default());
        }
    }
    
    /// Get current cursor position
    pub fn get_cursor_position(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }
}