//! Rust port of lib5250 display.c - Core display operations for 5250 protocol
//!
//! This module provides the display buffer management and screen update functions
//! that are called by the session module during 5250 protocol processing.

use crate::ebcdic;
use crate::terminal::TerminalScreen;

/// Display buffer that manages the 5250 terminal screen state
/// This is a bridge between lib5250 session logic and our TerminalScreen
#[derive(Debug)]
pub struct Display {
    /// The underlying terminal screen buffer
    screen: TerminalScreen,
    
    /// Current cursor position (row, col) - 0-based
    cursor_row: usize,
    cursor_col: usize,
    
    /// Screen dimensions
    width: usize,
    height: usize,
    
    /// Display indicators (system state flags)
    indicators: u32,
    
    /// Keyboard state
    keyboard_locked: bool,
    
    /// Pending insert cursor position
    pending_insert: bool,
    insert_cursor_row: usize,
    insert_cursor_col: usize,
}

// Display indicator flags (from original lib5250)
pub const TN5250_DISPLAY_IND_INHIBIT: u32 = 0x0001;
pub const TN5250_DISPLAY_IND_MESSAGE_WAITING: u32 = 0x0002;
pub const TN5250_DISPLAY_IND_X_SYSTEM: u32 = 0x0004;
pub const TN5250_DISPLAY_IND_X_CLOCK: u32 = 0x0008;
pub const TN5250_DISPLAY_IND_INSERT: u32 = 0x0010;
pub const TN5250_DISPLAY_IND_FER: u32 = 0x0020;
pub const TN5250_DISPLAY_IND_MACRO: u32 = 0x0040;

impl Display {
    /// Create a new display with standard 24x80 dimensions
    pub fn new() -> Self {
        Self {
            screen: TerminalScreen::new_with_size(80, 24),
            cursor_row: 0,
            cursor_col: 0,
            width: 80,
            height: 24,
            indicators: 0,
            keyboard_locked: true,
            pending_insert: false,
            insert_cursor_row: 0,
            insert_cursor_col: 0,
        }
    }

    /// Get a read-only reference to the underlying terminal screen
    pub fn screen_ref(&self) -> &TerminalScreen {
        &self.screen
    }

    /// Get the current screen content as a string
    /// Prefer `Display` impl for formatting; keep helper for compatibility
    pub fn screen_to_string(&self) -> String {
        self.screen.to_string()
    }
    
    /// Get reference to the underlying terminal screen
    pub fn screen(&mut self) -> &mut TerminalScreen {
        &mut self.screen
    }
    
    /// Get current cursor position (row, col) - 0-based
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }
    
    /// Get display width
    pub fn width(&self) -> usize {
        self.width
    }
    
    /// Get display height  
    pub fn height(&self) -> usize {
        self.height
    }

    // ===== Core display functions from original lib5250 =====

    /// Clear the display and set to standard 24x80 size
    /// Equivalent to tn5250_display_clear_unit()
    pub fn clear_unit(&mut self) {
        self.width = 80;
        self.height = 24;
        self.screen.resize(self.width, self.height, false);
        self.screen.clear();
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.indicator_set(TN5250_DISPLAY_IND_X_SYSTEM);
        self.keyboard_locked = true;
        self.indicator_clear(TN5250_DISPLAY_IND_INSERT | TN5250_DISPLAY_IND_INHIBIT | TN5250_DISPLAY_IND_FER);
        self.pending_insert = false;
    }

    /// Clear the display and set to alternate 27x132 size
    /// Equivalent to tn5250_display_clear_unit_alternate()
    pub fn clear_unit_alternate(&mut self) {
        self.width = 132;
        self.height = 27;
        self.screen.resize(self.width, self.height, false);
        self.screen.clear_alternate();
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.indicator_set(TN5250_DISPLAY_IND_X_SYSTEM);
        self.keyboard_locked = true;
        self.indicator_clear(TN5250_DISPLAY_IND_INSERT);
    }

    /// Clear the format table (field definitions)
    /// Equivalent to tn5250_display_clear_format_table()
    pub fn clear_format_table(&mut self) {
        self.screen.clear_format_table();
    }

    /// Set cursor position (0-based coordinates)
    /// Equivalent to tn5250_display_set_cursor()
    pub fn set_cursor(&mut self, row: usize, col: usize) {
        if row < self.height && col < self.width {
            self.cursor_row = row;
            self.cursor_col = col;
            // TerminalScreen uses (x,y) = (col,row)
            self.screen.set_cursor(col, row);
        }
    }

    /// Move cursor to home position (first non-bypass field or 0,0)
    /// Equivalent to tn5250_display_set_cursor_home()
    pub fn set_cursor_home(&mut self) {
        if self.pending_insert {
            self.cursor_row = self.insert_cursor_row;
            self.cursor_col = self.insert_cursor_col;
            self.screen.set_cursor(self.cursor_row, self.cursor_col);
        } else {
            // Scan display buffer to find first non-bypass (non-protected) field
            let mut found = false;
            for row in 0..self.height {
                for col in 0..self.width {
                    let index = self.screen.index(col, row);
                    if let Some(cell) = self.screen.buffer.get(index) {
                        // Bypass fields are protected fields - skip them
                        if !matches!(cell.attribute, crate::terminal::CharAttribute::Protected) {
                            self.set_cursor(row, col);
                            found = true;
                            break;
                        }
                    }
                }
                if found {
                    break;
                }
            }
            // If no non-bypass field found, default to (0,0)
            if !found {
                self.set_cursor(0, 0);
            }
        }
    }

    /// Add a character at the current cursor position
    /// Equivalent to tn5250_display_addch()
    pub fn addch(&mut self, ch: u8) {
        // Convert EBCDIC to ASCII if needed
        let ascii_char = self.ebcdic_to_ascii(ch);
        
        if self.cursor_row < self.height && self.cursor_col < self.width {
                            // Move cursor to position first, then add character
                            self.screen.move_cursor(self.cursor_col, self.cursor_row);
                self.screen.write_char(ascii_char);
            
            // Advance cursor
            self.cursor_col += 1;
            if self.cursor_col >= self.width {
                self.cursor_col = 0;
                self.cursor_row += 1;
                if self.cursor_row >= self.height {
                    self.cursor_row = self.height - 1;
                }
            }
            self.screen.set_cursor(self.cursor_col, self.cursor_row);
        }
    }

    /// Erase region from start to end coordinates
    /// Equivalent to tn5250_display_erase_region()
    pub fn erase_region(&mut self, start_row: usize, start_col: usize, 
                       end_row: usize, end_col: usize, _left_edge: usize, _right_edge: usize) {
        self.screen.erase_region(start_row, start_col, end_row, end_col);
    }

    /// Roll/scroll display region
    /// Equivalent to tn5250_display_roll()
        pub fn roll(&mut self, top: u8, bottom: u8, lines: i8) {
            self.screen.roll(top as usize, bottom as usize, lines);
    }

    /// Set pending insert cursor position
    /// Equivalent to tn5250_display_set_pending_insert()
    pub fn set_pending_insert(&mut self, row: usize, col: usize) {
        self.pending_insert = true;
        self.insert_cursor_row = row;
        self.insert_cursor_col = col;
    }

    /// Set pending insert cursor position (alias for set_pending_insert)
    pub fn set_pending_insert_cursor(&mut self, row: usize, col: usize) {
        self.set_pending_insert(row, col);
    }

    /// Add a character at the current cursor position (alias for addch)
    pub fn add_char(&mut self, ch: u8) {
        self.addch(ch);
    }

    /// Get current cursor row
    pub fn cursor_row(&self) -> usize {
        self.cursor_row
    }

    /// Get current cursor column
    pub fn cursor_col(&self) -> usize {
        self.cursor_col
    }

    /// Get screen data as bytes for 5250 protocol transmission
    pub fn get_screen_data(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Convert screen buffer to 5250 format
        for row in 0..self.height {
            for col in 0..self.width {
                let index = self.screen.index(col, row);
                let ch = self.screen.buffer[index].character;

                // Convert ASCII to EBCDIC for 5250 protocol
                let ebcdic_byte = self.ascii_to_ebcdic(ch as u8);
                data.push(ebcdic_byte);
            }
        }

        data
    }

    /// Convert ASCII character to EBCDIC for 5250 protocol
    fn ascii_to_ebcdic(&self, ascii: u8) -> u8 {
        match ascii {
            32 => 0x40, // Space
            b'!' => 0x5A,
            b'"' => 0x7F,
            b'#' => 0x7B,
            b'$' => 0x5B,
            b'%' => 0x6C,
            b'&' => 0x50,
            b'\'' => 0x7D,
            b'(' => 0x4D,
            b')' => 0x5D,
            b'*' => 0x5C,
            b'+' => 0x4E,
            b',' => 0x6B,
            b'-' => 0x60,
            b'.' => 0x4B,
            b'/' => 0x61,
            b'0'..=b'9' => 0xF0 + (ascii - b'0'), // 0-9
            b':' => 0x7A,
            b';' => 0x5E,
            b'<' => 0x4C,
            b'=' => 0x7E,
            b'>' => 0x6E,
            b'?' => 0x6F,
            b'@' => 0x7C,
            b'A'..=b'I' => 0xC1 + (ascii - b'A'), // A-I
            b'J'..=b'R' => 0xD1 + (ascii - b'J'), // J-R
            b'S'..=b'Z' => 0xE2 + (ascii - b'S'), // S-Z
            b'[' => 0xAD,
            b'\\' => 0xE0,
            b']' => 0xBD,
            b'^' => 0x5F,
            b'_' => 0x6D,
            b'`' => 0x79,
            b'a'..=b'i' => 0x81 + (ascii - b'a'), // a-i
            b'j'..=b'r' => 0x91 + (ascii - b'j'), // j-r
            b's'..=b'z' => 0xA2 + (ascii - b's'), // s-z
            b'{' => 0xC0,
            b'|' => 0x4F,
            b'}' => 0xD0,
            b'~' => 0xA1,
            _ => 0x40, // Default to space for unknown characters
        }
    }

    /// Initialize 24x80 screen buffer for 5250 protocol
    pub fn initialize_5250_screen(&mut self) {
        self.width = 80;
        self.height = 24;
        self.screen.resize(self.width, self.height, false);
        self.screen.clear();
        self.set_cursor(0, 0);
        self.unlock_keyboard();
    }

    /// Add 5250 protocol data to screen buffer
    pub fn add_5250_data(&mut self, data: &[u8]) -> Result<(), String> {
        for &byte in data {
            let ascii_char = self.ebcdic_to_ascii(byte);
            if self.cursor_row < self.height && self.cursor_col < self.width {
                let index = self.screen.index(self.cursor_col, self.cursor_row);
                self.screen.buffer[index] = crate::terminal::TerminalChar {
                    character: ascii_char,
                    attribute: crate::terminal::CharAttribute::Normal,
                };
                self.cursor_col += 1;
                if self.cursor_col >= self.width {
                    self.cursor_col = 0;
                    self.cursor_row += 1;
                    if self.cursor_row >= self.height {
                        self.cursor_row = self.height - 1;
                    }
                }
            }
        }
        self.screen.set_cursor(self.cursor_row, self.cursor_col);
        Ok(())
    }

    /// Sound the terminal bell/beep
    /// Equivalent to tn5250_display_beep()
    pub fn beep(&mut self) {
        // In a real implementation, this would trigger an actual audible beep
        // For now, we'll print the ASCII bell character which some terminals may interpret
        println!("\x07"); // ASCII bell character
        
        // Additionally, we could trigger a visual indication in a GUI implementation
        // For example, briefly flash the screen or show a notification
        println!("5250: BEEP - Audible alert triggered");
    }
    
    /// Set cursor blinking state
    /// Equivalent to tn5250_display_set_blinking_cursor()
    pub fn set_blinking_cursor(&mut self, blinking: bool) {
        // Set blinking cursor attribute at current cursor position
        if self.cursor_row < self.height && self.cursor_col < self.width {
            let index = self.screen.index(self.cursor_col, self.cursor_row);
            if blinking {
                self.screen.buffer[index].attribute = crate::terminal::CharAttribute::BlinkingCursor;
            } else {
                self.screen.buffer[index].attribute = crate::terminal::CharAttribute::Normal;
            }
            self.screen.dirty = true;
        }
    }

    /// Check if cursor is currently blinking
    /// Returns true if the character at the current cursor position has BlinkingCursor attribute
    pub fn is_cursor_blinking(&self) -> bool {
        if self.cursor_row < self.height && self.cursor_col < self.width {
            let index = self.screen.index(self.cursor_col, self.cursor_row);
            matches!(self.screen.buffer[index].attribute, crate::terminal::CharAttribute::BlinkingCursor)
        } else {
            false
        }
    }
    
    /// Set reverse image state
    /// Equivalent to tn5250_display_set_reverse_image()
    pub fn set_reverse_image(&mut self, reverse: bool) {
        // Set reverse image attribute at current cursor position
        if self.cursor_row < self.height && self.cursor_col < self.width {
            let index = self.screen.index(self.cursor_col, self.cursor_row);
            if reverse {
                self.screen.buffer[index].attribute = crate::terminal::CharAttribute::ReverseImage;
            } else {
                self.screen.buffer[index].attribute = crate::terminal::CharAttribute::Normal;
            }
            self.screen.dirty = true;
        }
    }

    /// Set underline state
    /// Equivalent to tn5250_display_set_underline()
    pub fn set_underline(&mut self, underline: bool) {
        // Set underline attribute at current cursor position
        if self.cursor_row < self.height && self.cursor_col < self.width {
            let index = self.screen.index(self.cursor_col, self.cursor_row);
            if underline {
                self.screen.buffer[index].attribute = crate::terminal::CharAttribute::Underline;
            } else {
                self.screen.buffer[index].attribute = crate::terminal::CharAttribute::Normal;
            }
            self.screen.dirty = true;
        }
    }

    // ===== Indicator management =====

    /// Set display indicators
    /// Equivalent to tn5250_display_indicator_set()
    pub fn indicator_set(&mut self, indicators: u32) {
        self.indicators |= indicators;
    }

    /// Clear display indicators
    /// Equivalent to tn5250_display_indicator_clear()
    pub fn indicator_clear(&mut self, indicators: u32) {
        self.indicators &= !indicators;
    }

    /// Get current indicators
    pub fn indicators(&self) -> u32 {
        self.indicators
    }

    /// Check if keyboard is locked
    pub fn keyboard_locked(&self) -> bool {
        self.keyboard_locked
    }

    /// Lock keyboard
    pub fn lock_keyboard(&mut self) {
        self.keyboard_locked = true;
        self.screen.lock_keyboard();
    }

    /// Unlock keyboard
    pub fn unlock_keyboard(&mut self) {
        self.keyboard_locked = false;
        self.screen.unlock_keyboard();
    }
    
    /// Set color attributes for display
    /// This is a placeholder implementation for color support
    pub fn set_color_attributes(&mut self, _fg_color: u8, _bg_color: u8) {
        // Color attributes are not currently supported in the basic TerminalChar structure
        // In a full implementation, this would extend TerminalChar to include color information
        // For now, this is a no-op that maintains compatibility
    }

    /// Set font attributes for display
    /// This is a placeholder implementation for font support
    pub fn set_font_attributes(&mut self, _bold: bool, _italic: bool, _underline: bool) {
        // Font attributes are not currently supported in the basic TerminalChar structure
        // In a full implementation, this would extend TerminalChar to include font information
        // For now, this is a no-op that maintains compatibility
    }

    /// Set display intensity
    /// This is a placeholder implementation for intensity support
    pub fn set_intensity(&mut self, intensity: u8) {
        // Set intensity attribute at current cursor position
        if self.cursor_row < self.height && self.cursor_col < self.width {
            let index = self.screen.index(self.cursor_col, self.cursor_row);
            match intensity {
                0x00 => self.screen.buffer[index].attribute = crate::terminal::CharAttribute::Normal,
                0x01 => self.screen.buffer[index].attribute = crate::terminal::CharAttribute::HighIntensity,
                0x02 => self.screen.buffer[index].attribute = crate::terminal::CharAttribute::Intensified, // Map low intensity to intensified for now
                _ => self.screen.buffer[index].attribute = crate::terminal::CharAttribute::Normal,
            }
            self.screen.dirty = true;
        }
    }

    /// Set reverse video mode
    /// This is a placeholder implementation for reverse video support
    pub fn set_reverse_video(&mut self, reverse: bool) {
        // Set reverse video attribute at current cursor position
        if self.cursor_row < self.height && self.cursor_col < self.width {
            let index = self.screen.index(self.cursor_col, self.cursor_row);
            if reverse {
                self.screen.buffer[index].attribute = crate::terminal::CharAttribute::ReverseVideo;
            } else {
                self.screen.buffer[index].attribute = crate::terminal::CharAttribute::Normal;
            }
            self.screen.dirty = true;
        }
    }

    /// Set blink mode
    /// This is a placeholder implementation for blink support
    pub fn set_blink(&mut self, blink: bool) {
        // Set blink attribute at current cursor position
        if self.cursor_row < self.height && self.cursor_col < self.width {
            let index = self.screen.index(self.cursor_col, self.cursor_row);
            if blink {
                self.screen.buffer[index].attribute = crate::terminal::CharAttribute::Blink;
            } else {
                self.screen.buffer[index].attribute = crate::terminal::CharAttribute::Normal;
            }
            self.screen.dirty = true;
        }
    }

    /// Set default attribute for field creation
    /// This is a placeholder implementation for default attribute support
    pub fn set_default_attribute(&mut self, _attr: u8) {
        // Default attribute setting is not currently implemented
        // In a full implementation, this would store the default attribute for new fields
        // For now, this is a no-op that maintains compatibility
    }

    /// Reset MDT (Modified Data Tag) flags for non-bypass fields
    /// In 5250 protocol, this clears the modified flag on input fields
    pub fn reset_non_bypass_mdt(&mut self) {
        // In a full implementation, this would iterate through all fields
        // and reset the MDT flag for non-bypass (input) fields
        println!("5250: Resetting MDT on non-bypass fields");
    }

    /// Reset all MDT flags regardless of field type
    pub fn reset_all_mdt(&mut self) {
        // For now, this is a placeholder that would reset all MDT flags
        println!("5250: Resetting all MDT flags (placeholder)");
    }

    /// Null (clear) MDT flags for non-bypass fields
    /// This is similar to reset but may have different semantics in some implementations
    pub fn null_non_bypass_mdt(&mut self) {
        // In a full implementation, this would iterate through all fields
        // and null (clear) the MDT flag for non-bypass (input) fields
        println!("5250: Nulling MDT on non-bypass fields");
    }

    /// Null MDT flags for fields matching specific criteria
    pub fn null_non_bypass_fields(&mut self) {
        // For now, this is a placeholder
        println!("5250: Nulling MDT on non-bypass fields (placeholder)");
    }

    // ===== EBCDIC conversion =====

    /// Convert EBCDIC character to ASCII using proper EBCDIC CP037 table
    fn ebcdic_to_ascii(&self, ebcdic: u8) -> char {
        ebcdic::ebcdic_to_ascii(ebcdic)
    }
}

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.screen_to_string())
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}