//! Terminal emulation module for the 5250 protocol
//! 
//! This module provides the core terminal emulation functionality for 
//! displaying and handling AS/400 screens using the 5250 protocol.

use std::fmt;
use crate::monitoring::{set_component_status, set_component_error, ComponentState};
use crate::performance_metrics::PerformanceMetrics;

// Terminal dimensions - standard IBM 5250 terminal sizes
pub const TERMINAL_WIDTH: usize = 80;
pub const TERMINAL_HEIGHT: usize = 24;

// Character attributes for 5250 terminal
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CharAttribute {
    #[default]
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
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TerminalChar {
    pub character: char,
    pub attribute: CharAttribute,
}


// Represents the terminal screen buffer
#[derive(Debug)]
pub struct TerminalScreen {
    // PERFORMANCE OPTIMIZATION: Use Vec for better cache locality
    // 1D vector provides better memory access patterns than 2D arrays
    pub buffer: Vec<TerminalChar>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub dirty: bool, // Flag to indicate if screen needs to be redrawn
}

impl TerminalScreen {
    pub fn new() -> Self {
        let screen = Self {
            buffer: vec![TerminalChar::default(); TERMINAL_WIDTH * TERMINAL_HEIGHT],
            cursor_x: 0,
            cursor_y: 0,
            dirty: true,
        };
        set_component_status("terminal", ComponentState::Running);
        set_component_error("terminal", None::<&str>);
        screen
    }

    /// PERFORMANCE OPTIMIZATION: Calculate 1D index from 2D coordinates
    /// Inlined for maximum performance
    #[inline(always)]
    pub fn buffer_index(x: usize, y: usize) -> usize {
        y * TERMINAL_WIDTH + x
    }

    /// PERFORMANCE OPTIMIZATION: Bulk buffer operations for better cache locality
    /// Clear entire buffer with optimized memory access pattern
    #[inline]
    pub fn clear_buffer_optimized(&mut self) {
        // PERFORMANCE: Use raw pointer operations for maximum speed
        // This avoids bounds checking and iterator overhead
        let default_char = TerminalChar {
            character: ' ',
            attribute: CharAttribute::Normal,
        };

        // SAFETY: We know the buffer size is TERMINAL_WIDTH * TERMINAL_HEIGHT
        // and we're filling it with valid data
        unsafe {
            let ptr = self.buffer.as_mut_ptr();
            for i in 0..TERMINAL_WIDTH * TERMINAL_HEIGHT {
                *ptr.add(i) = default_char;
            }
        }

        self.cursor_x = 0;
        self.cursor_y = 0;
        self.dirty = true;

        // PERFORMANCE MONITORING: Track buffer clear operations
        crate::performance_metrics::PerformanceMetrics::global()
            .terminal_metrics
            .buffer_clears
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// PERFORMANCE OPTIMIZATION: Bulk character writing with cache-friendly access
    /// Write multiple characters to a row with optimized memory access
    #[inline]
    pub fn write_chars_to_row(&mut self, row: usize, col_start: usize, chars: &[char], attr: CharAttribute) {
        if row >= TERMINAL_HEIGHT {
            return;
        }

        let start_idx = Self::buffer_index(col_start, row);
        let max_chars = TERMINAL_WIDTH.saturating_sub(col_start);
        let chars_to_write = chars.len().min(max_chars);

        // PERFORMANCE: Direct buffer access with bounds checking minimized
        for (i, &ch) in chars.iter().enumerate().take(chars_to_write) {
            let buffer_idx = start_idx + i;
            if buffer_idx < self.buffer.len() {
                self.buffer[buffer_idx] = TerminalChar {
                    character: ch,
                    attribute: attr,
                };
            }
        }

        self.dirty = true;
    }

    /// PERFORMANCE OPTIMIZATION: Bulk attribute setting for regions
    /// Set attributes for a rectangular region efficiently
    #[inline]
    pub fn set_region_attributes(&mut self, start_row: usize, start_col: usize,
                                width: usize, height: usize, attr: CharAttribute) {
        let end_row = (start_row + height).min(TERMINAL_HEIGHT);
        let end_col = (start_col + width).min(TERMINAL_WIDTH);

        // PERFORMANCE: Iterate row by row for better cache locality
        for row in start_row..end_row {
            let row_start_idx = Self::buffer_index(start_col, row);
            let row_end_idx = Self::buffer_index(end_col, row);

            // PERFORMANCE: Direct slice access for contiguous memory
            let row_slice = &mut self.buffer[row_start_idx..row_end_idx];
            for cell in row_slice.iter_mut() {
                cell.attribute = attr;
            }
        }

        self.dirty = true;
    }

    /// PERFORMANCE OPTIMIZATION: Fast buffer copy operations
    /// Copy a region from one buffer to another with optimized access
    #[inline]
    pub fn copy_region(&mut self, src: &TerminalScreen, src_row: usize, src_col: usize,
                      dst_row: usize, dst_col: usize, width: usize, height: usize) {
        let src_end_row = (src_row + height).min(TERMINAL_HEIGHT);
        let src_end_col = (src_col + width).min(TERMINAL_WIDTH);
        let dst_end_row = (dst_row + height).min(TERMINAL_HEIGHT);
        let dst_end_col = (dst_col + width).min(TERMINAL_WIDTH);

        let copy_height = (src_end_row - src_row).min(dst_end_row - dst_row);
        let copy_width = (src_end_col - src_col).min(dst_end_col - dst_col);

        // PERFORMANCE: Row-by-row copy for cache efficiency
        for row_offset in 0..copy_height {
            let src_row_idx = Self::buffer_index(src_col, src_row + row_offset);
            let dst_row_idx = Self::buffer_index(dst_col, dst_row + row_offset);

            // PERFORMANCE: Use copy_from_slice for bulk memory operations
            let src_slice = &src.buffer[src_row_idx..src_row_idx + copy_width];
            let dst_slice = &mut self.buffer[dst_row_idx..dst_row_idx + copy_width];
            dst_slice.copy_from_slice(src_slice);
        }

        self.dirty = true;
    }

    // Clear the entire screen
    pub fn clear(&mut self) {
        self.clear_buffer_optimized();
        set_component_status("terminal", ComponentState::Running);
        set_component_error("terminal", None::<&str>);
    }

    // Write a character at the current cursor position
    pub fn write_char(&mut self, ch: char) {
        self.write_char_with_attr(ch, CharAttribute::Normal);
    }
    
    // Write a character at the current cursor position with specific attribute
    pub fn write_char_with_attr(&mut self, ch: char, attr: CharAttribute) {
        // CRITICAL FIX: Enhanced boundary checking with proper edge case handling
        // Prevent buffer overflow and ensure cursor stays within valid bounds

        // Validate cursor position before writing
        if self.cursor_y >= TERMINAL_HEIGHT || self.cursor_x >= TERMINAL_WIDTH {
            eprintln!("SECURITY: Attempted to write outside terminal bounds at ({}, {})", self.cursor_y, self.cursor_x);
            return;
        }

        // PERFORMANCE OPTIMIZATION: Use 1D vector indexing for better cache locality
        let index = Self::buffer_index(self.cursor_x, self.cursor_y);
        self.buffer[index] = TerminalChar {
            character: ch,
            attribute: attr,
        };
        self.dirty = true;

        // PERFORMANCE MONITORING: Track character write operations
        crate::performance_metrics::PerformanceMetrics::global()
            .terminal_metrics
            .character_writes
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Move cursor to next position (unless it's a protected field)
        if !matches!(attr, CharAttribute::Protected | CharAttribute::Hidden) {
            // CRITICAL FIX: Safer cursor advancement with bounds checking
            if self.cursor_x + 1 >= TERMINAL_WIDTH {
                // Move to next line
                self.cursor_x = 0;
                if self.cursor_y + 1 < TERMINAL_HEIGHT {
                    self.cursor_y += 1;
                } else {
                    // CRITICAL FIX: Handle end of screen gracefully
                    // Option 1: Stay at end of last line
                    self.cursor_y = TERMINAL_HEIGHT - 1;
                    self.cursor_x = TERMINAL_WIDTH - 1;
                }
            } else {
                self.cursor_x += 1;
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
        // CRITICAL FIX: Enhanced boundary validation with edge case handling
        // Prevent cursor from going out of bounds and handle invalid coordinates

        // Validate coordinates are within reasonable bounds
        if x > TERMINAL_WIDTH || y > TERMINAL_HEIGHT {
            eprintln!("SECURITY: Invalid cursor position ({y}, {x}) - exceeds terminal dimensions");
            return;
        }

        // Additional validation: ensure coordinates are not negative (though usize prevents this)
        // But we should handle very large values that could cause overflow
        let safe_x = if x >= TERMINAL_WIDTH { TERMINAL_WIDTH - 1 } else { x };
        let safe_y = if y >= TERMINAL_HEIGHT { TERMINAL_HEIGHT - 1 } else { y };

        self.cursor_x = safe_x;
        self.cursor_y = safe_y;
    }

    // Write a character at a specific position
    pub fn write_char_at(&mut self, x: usize, y: usize, ch: char) {
        // CRITICAL FIX: Enhanced boundary validation with comprehensive checking
        // Prevent buffer overflow and validate all parameters

        // Validate coordinates are within bounds
        if x >= TERMINAL_WIDTH || y >= TERMINAL_HEIGHT {
            eprintln!("SECURITY: Attempted to write outside terminal bounds at ({y}, {x})");
            return;
        }

        // Additional validation: ensure coordinates are reasonable
        if x > TERMINAL_WIDTH || y > TERMINAL_HEIGHT {
            eprintln!("SECURITY: Invalid coordinates ({y}, {x}) - exceeds maximum values");
            return;
        }

        // Validate character is safe to write
        if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
            eprintln!("SECURITY: Attempted to write control character: {}", ch as u32);
            return;
        }

        // PERFORMANCE OPTIMIZATION: Use 1D vector indexing for better cache locality
        let index = Self::buffer_index(x, y);
        self.buffer[index] = TerminalChar {
            character: ch,
            attribute: CharAttribute::Normal,
        };
        self.dirty = true;
    }

    // Get character at specific position
    pub fn get_char_at(&self, x: usize, y: usize) -> Option<char> {
        // CRITICAL FIX: Enhanced boundary validation for safe access
        if x < TERMINAL_WIDTH && y < TERMINAL_HEIGHT {
            // PERFORMANCE OPTIMIZATION: Use 1D vector indexing for better cache locality
            let index = Self::buffer_index(x, y);
            Some(self.buffer[index].character)
        } else {
            None
        }
    }

    // Set character at specific position
    pub fn set_char_at(&mut self, x: usize, y: usize, ch: TerminalChar) {
        // CRITICAL FIX: Enhanced boundary validation for safe modification
        if x < TERMINAL_WIDTH && y < TERMINAL_HEIGHT {
            // PERFORMANCE OPTIMIZATION: Use 1D vector indexing for better cache locality
            let index = Self::buffer_index(x, y);
            self.buffer[index] = ch;
            self.dirty = true;
        }
    }

    /// CRITICAL FIX: Validate terminal screen buffer consistency
    /// This method ensures the buffer is in a valid state
    pub fn validate_buffer_consistency(&self) -> Result<(), String> {
        // PERFORMANCE OPTIMIZATION: Validate buffer dimensions for 1D vector
        if self.buffer.len() != TERMINAL_WIDTH * TERMINAL_HEIGHT {
            set_component_status("terminal", ComponentState::Error);
            set_component_error("terminal", Some("Invalid buffer size"));
            return Err(format!("Invalid buffer size: {} (expected {})",
                             self.buffer.len(), TERMINAL_WIDTH * TERMINAL_HEIGHT));
        }

        // PERFORMANCE OPTIMIZATION: Iterate through 1D vector for better cache locality
        for index in 0..self.buffer.len() {
            let terminal_char = &self.buffer[index];
            let row_idx = index / TERMINAL_WIDTH;
            let col_idx = index % TERMINAL_WIDTH;

            // Check for invalid Unicode or dangerous characters
            if (terminal_char.character as u32) > 0x10FFFF {
                set_component_status("terminal", ComponentState::Error);
                set_component_error("terminal", Some("Invalid Unicode character in buffer"));
                return Err(format!("Invalid Unicode character at ({}, {}): {}",
                                 row_idx, col_idx, terminal_char.character as u32));
            }

            // Check for dangerous control characters that shouldn't be in buffer
            if terminal_char.character.is_control() &&
               terminal_char.character != '\n' &&
               terminal_char.character != '\r' &&
               terminal_char.character != '\t' {
                set_component_status("terminal", ComponentState::Error);
                set_component_error("terminal", Some("Dangerous control character in buffer"));
                return Err(format!("Dangerous control character at ({}, {}): {}",
                                 row_idx, col_idx, terminal_char.character as u32));
            }
        }

        // Validate cursor position
        if self.cursor_x >= TERMINAL_WIDTH || self.cursor_y >= TERMINAL_HEIGHT {
            set_component_status("terminal", ComponentState::Error);
            set_component_error("terminal", Some("Invalid cursor position"));
            return Err(format!("Invalid cursor position: ({}, {})", self.cursor_y, self.cursor_x));
        }

        set_component_status("terminal", ComponentState::Running);
        set_component_error("terminal", None::<&str>);
        Ok(())
    }

    /// CRITICAL FIX: Safe buffer clearing with validation
    pub fn safe_clear(&mut self) {
        // PERFORMANCE OPTIMIZATION: Clear 1D vector directly for better cache locality
        for cell in self.buffer.iter_mut() {
            *cell = TerminalChar::default();
        }

        // Reset cursor to safe position
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.dirty = true;
        set_component_status("terminal", ComponentState::Running);
        set_component_error("terminal", None::<&str>);
    }

    /// CRITICAL FIX: Safe cursor positioning with validation
    pub fn set_cursor_safe(&mut self, x: usize, y: usize) {
        // Validate and clamp coordinates to safe values
        let safe_x = if x >= TERMINAL_WIDTH { TERMINAL_WIDTH - 1 } else { x };
        let safe_y = if y >= TERMINAL_HEIGHT { TERMINAL_HEIGHT - 1 } else { y };

        self.cursor_x = safe_x;
        self.cursor_y = safe_y;
    }

    /// Set cursor position (alias for set_cursor_safe for compatibility)
    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.set_cursor_safe(x, y);
    }

    /// Clear alternate screen (placeholder for compatibility)
    pub fn clear_alternate(&mut self) {
        self.safe_clear();
    }

    /// Clear format table (placeholder for compatibility)
    pub fn clear_format_table(&mut self) {
        // No-op for now - format table is not implemented in basic terminal
    }

    /// Add character to screen (placeholder for compatibility)
    pub fn add_char(&mut self, _ch: u8) {
        // No-op for now - character addition is handled differently
    }

    /// Erase region (placeholder for compatibility)
    pub fn erase_region(&mut self, _start_row: usize, _start_col: usize, _end_row: usize, _end_col: usize) {
        // No-op for now - region erase not implemented in basic terminal
    }

    /// Roll screen (placeholder for compatibility)
    pub fn roll(&mut self, _top: usize, _bottom: usize, _lines: i8) {
        // No-op for now - screen rolling not implemented in basic terminal
    }

    /// Lock keyboard (placeholder for compatibility)
    pub fn lock_keyboard(&mut self) {
        // No-op for now - keyboard locking not implemented in basic terminal
    }

    /// Unlock keyboard (placeholder for compatibility)
    pub fn unlock_keyboard(&mut self) {
        // No-op for now - keyboard unlocking not implemented in basic terminal
    }
}

impl Clone for TerminalScreen {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            cursor_x: self.cursor_x,
            cursor_y: self.cursor_y,
            dirty: self.dirty,
        }
    }
}

impl Default for TerminalScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TerminalScreen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // PERFORMANCE OPTIMIZATION: Iterate through 1D vector for better cache locality
        for row in 0..TERMINAL_HEIGHT {
            for col in 0..TERMINAL_WIDTH {
                let index = Self::buffer_index(col, row);
                write!(f, "{}", self.buffer[index].character)?;
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
        set_component_status("terminal", ComponentState::Running);
        set_component_error("terminal", None::<&str>);
        Ok(())
    }

    // Disconnect from host
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.host.clear();
        self.screen.clear();
        self.screen.write_string("Disconnected from AS/400 system");
        self.data_buffer.clear();
        set_component_status("terminal", ComponentState::Stopped);
        set_component_error("terminal", None::<&str>);
    }

    // Process incoming data
    pub fn process_data(&mut self, data: &[u8]) -> Result<(), String> {
        // Store raw data for debugging
        self.data_buffer.extend_from_slice(data);
        let data_info = format!("[Received {} bytes of 5250 data]\n", data.len());
        self.screen.write_string(&data_info);
        set_component_status("terminal", ComponentState::Running);
        set_component_error("terminal", None::<&str>);
        Ok(())
    }

    // Process keyboard input
    pub fn process_input(&mut self, input: &str) -> Result<Vec<u8>, String> {
        self.screen.write_string(input);
        set_component_status("terminal", ComponentState::Running);
        set_component_error("terminal", None::<&str>);
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

impl Default for TerminalEmulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_screen_creation() {
        let screen = TerminalScreen::new();
        // PERFORMANCE OPTIMIZATION: Use 1D vector indexing
        let index = TerminalScreen::buffer_index(0, 0);
        assert_eq!(screen.buffer[index].character, ' ');
        assert_eq!(screen.cursor_x, 0);
        assert_eq!(screen.cursor_y, 0);
    }

    #[test]
    fn test_write_char() {
        let mut screen = TerminalScreen::new();
        screen.write_char('A');
        // PERFORMANCE OPTIMIZATION: Use 1D vector indexing
        let index = TerminalScreen::buffer_index(0, 0);
        assert_eq!(screen.buffer[index].character, 'A');
        assert_eq!(screen.cursor_x, 1);
    }

    #[test]
    fn test_write_string() {
        let mut screen = TerminalScreen::new();
        screen.write_string("Hello");
        // PERFORMANCE OPTIMIZATION: Use 1D vector indexing
        assert_eq!(screen.buffer[TerminalScreen::buffer_index(0, 0)].character, 'H');
        assert_eq!(screen.buffer[TerminalScreen::buffer_index(1, 0)].character, 'e');
        assert_eq!(screen.buffer[TerminalScreen::buffer_index(2, 0)].character, 'l');
        assert_eq!(screen.buffer[TerminalScreen::buffer_index(3, 0)].character, 'l');
        assert_eq!(screen.buffer[TerminalScreen::buffer_index(4, 0)].character, 'o');
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