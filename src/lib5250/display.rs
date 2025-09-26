//! Rust port of lib5250 display.c - Core display operations for 5250 protocol
//! 
//! This module provides the display buffer management and screen update functions
//! that are called by the session module during 5250 protocol processing.

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
            screen: TerminalScreen::new(),
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
    pub fn to_string(&self) -> String {
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
            self.screen.set_cursor(row, col);
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
            // For now, just go to 0,0 - TODO: Find first non-bypass field
            self.set_cursor(0, 0);
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
                self.screen.add_char(ascii_char as u8);
            
            // Advance cursor
            self.cursor_col += 1;
            if self.cursor_col >= self.width {
                self.cursor_col = 0;
                self.cursor_row += 1;
                if self.cursor_row >= self.height {
                    self.cursor_row = self.height - 1;
                }
            }
            self.screen.set_cursor(self.cursor_row, self.cursor_col);
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
            self.screen.roll(top, bottom, lines);
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

    /// Get screen data as bytes (placeholder implementation)
    pub fn get_screen_data(&self) -> Vec<u8> {
        // TODO: Implement actual screen data extraction
        // For now, return empty data
        Vec::new()
    }

    /// Sound the terminal bell/beep
    /// Equivalent to tn5250_display_beep()
    pub fn beep(&mut self) {
        // TODO: Implement actual beep - for now just a placeholder
        println!("\x07"); // ASCII bell character
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

    // ===== EBCDIC conversion =====

    /// Convert EBCDIC character to ASCII
    /// TODO: This is a simplified conversion - should use proper EBCDIC tables
    fn ebcdic_to_ascii(&self, ebcdic: u8) -> char {
        match ebcdic {
            0x40 => ' ',  // Space
            0x4a => '!',
            0x4f => '|',
            0x50..=0x59 => ('0' as u8 + (ebcdic - 0x50)) as char, // 0-9
            0x5a => '!',
            0x5b => '$',
            0x5c => '*',
            0x5d => ')',
            0x5e => ';',
            0x5f => '^',
            0x60 => '-',
            0x61 => '/',
            0x81..=0x89 => ('a' as u8 + (ebcdic - 0x81)) as char, // a-i
            0x91..=0x99 => ('j' as u8 + (ebcdic - 0x91)) as char, // j-r
            0xa2..=0xa9 => ('s' as u8 + (ebcdic - 0xa2)) as char, // s-z
            0xc1..=0xc9 => ('A' as u8 + (ebcdic - 0xc1)) as char, // A-I
            0xd1..=0xd9 => ('J' as u8 + (ebcdic - 0xd1)) as char, // J-R
            0xe2..=0xe9 => ('S' as u8 + (ebcdic - 0xe2)) as char, // S-Z
            _ => '?',  // Unknown character
        }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}