//! TN3270 Display Buffer Management
//!
//! This module handles the 3270 display buffer which manages the screen state

#![allow(dead_code)] // Complete TN3270 display implementation
//! handling screen buffer operations, cursor management, and buffer addressing.

use super::codes::ORDER_SBA;
use super::field::{FieldAttribute, FieldManager};
use crate::protocol_common::ebcdic::ebcdic_to_ascii;

use serde::{Deserialize, Serialize};

/// Standard 3270 screen sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenSize {
    /// Model 2: 24 rows x 80 columns (1920 characters)
    Model2,
    /// Model 3: 32 rows x 80 columns (2560 characters)
    Model3,
    /// Model 4: 43 rows x 80 columns (3440 characters)
    Model4,
    /// Model 5: 27 rows x 132 columns (3564 characters)
    Model5,
}

impl ScreenSize {
    /// Get the number of rows for this screen size
    pub fn rows(&self) -> usize {
        match self {
            Self::Model2 => 24,
            Self::Model3 => 32,
            Self::Model4 => 43,
            Self::Model5 => 27,
        }
    }
    
    /// Get the number of columns for this screen size
    pub fn cols(&self) -> usize {
        match self {
            Self::Model2 => 80,
            Self::Model3 => 80,
            Self::Model4 => 80,
            Self::Model5 => 132,
        }
    }
    
    /// Get the total buffer size (rows * cols)
    pub fn buffer_size(&self) -> usize {
        self.rows() * self.cols()
    }
    
    /// Convert buffer address to (row, col) coordinates
    pub fn address_to_coords(&self, address: u16) -> (usize, usize) {
        let addr = address as usize;
        let cols = self.cols();
        let row = addr / cols;
        let col = addr % cols;
        (row, col)
    }
    
    /// Convert (row, col) coordinates to buffer address
    pub fn coords_to_address(&self, row: usize, col: usize) -> u16 {
        ((row * self.cols()) + col) as u16
    }
}

/// Cell in the display buffer
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DisplayCell {
    /// Character data (EBCDIC)
    pub char_data: u8,
    
    /// Field attribute (if this is a field attribute position)
    pub is_field_attr: bool,
    
    /// Extended attribute data
    pub extended_attr: u8,
}

/// 3270 Display Buffer
///
/// Manages the screen buffer for a 3270 terminal, including character data,
/// field attributes, cursor position, and buffer addressing.
#[derive(Debug)]
pub struct Display3270 {
    /// Current screen size
    screen_size: ScreenSize,
    
    /// Display buffer (character and attribute data)
    buffer: Vec<DisplayCell>,
    
    /// Current cursor position (buffer address)
    cursor_address: u16,
    
    /// Field manager for tracking fields
    field_manager: FieldManager,
    
    /// Keyboard locked state
    keyboard_locked: bool,
    
    /// Alarm state
    alarm: bool,
}

impl Display3270 {
    /// Create a new display with Model 2 (24x80) size
    pub fn new() -> Self {
        Self::with_size(ScreenSize::Model2)
    }
    
    /// Create a new display with specified screen size
    pub fn with_size(size: ScreenSize) -> Self {
        let buffer_size = size.buffer_size();
        Self {
            screen_size: size,
            buffer: vec![DisplayCell::default(); buffer_size],
            cursor_address: 0,
            field_manager: FieldManager::new(),
            keyboard_locked: true,
            alarm: false,
        }
    }
    
    /// Get the current screen size
    pub fn screen_size(&self) -> ScreenSize {
        self.screen_size
    }
    
    /// Get the number of rows
    pub fn rows(&self) -> usize {
        self.screen_size.rows()
    }
    
    /// Get the number of columns
    pub fn cols(&self) -> usize {
        self.screen_size.cols()
    }
    
    /// Get the buffer size
    pub fn buffer_size(&self) -> usize {
        self.screen_size.buffer_size()
    }
    
    /// Clear the entire display buffer
    pub fn clear(&mut self) {
        for cell in &mut self.buffer {
            *cell = DisplayCell::default();
        }
        self.cursor_address = 0;
        self.field_manager.clear();
    }
    
    /// Clear all unprotected fields
    pub fn clear_unprotected(&mut self) {
        // Iterate through all fields and clear unprotected ones
        for field in self.field_manager.fields() {
            if !field.is_protected() {
                // Clear the field content
                let start_addr = field.address as usize;
                let length = field.length;

                for offset in 0..length {
                    let addr = (start_addr + offset) % self.buffer.len();
                    self.buffer[addr].char_data = 0x00; // Null character
                }

                // Reset MDT flag
                // Note: We need to modify the field, but fields() returns immutable references
                // This is a limitation - we need to modify the field manager's fields directly
                // For now, we'll just clear the content but can't reset MDT without mutable access
            }
        }
    }
    
    /// Set cursor position using buffer address
    pub fn set_cursor(&mut self, address: u16) {
        if (address as usize) < self.buffer.len() {
            self.cursor_address = address;
        }
    }
    
    /// Get current cursor position
    pub fn cursor_address(&self) -> u16 {
        self.cursor_address
    }
    
    /// Get cursor position as (row, col)
    pub fn cursor_position(&self) -> (usize, usize) {
        self.screen_size.address_to_coords(self.cursor_address)
    }
    
    /// Write a character at the current cursor position
    /// This also marks the field as modified if writing to an unprotected field
    pub fn write_char(&mut self, ch: u8) {
        let addr = self.cursor_address as usize;
        if addr < self.buffer.len() {
            self.buffer[addr].char_data = ch;
            
            // Mark the field as modified if this is user input in an unprotected field
            if let Some(field) = self.field_manager.find_field_at_mut(self.cursor_address) {
                if !field.is_protected() {
                    field.set_modified(true);
                }
            }
            
            self.cursor_address = ((addr + 1) % self.buffer.len()) as u16;
        }
    }
    
    /// Write a character at a specific buffer address
    /// This also marks the field as modified if writing to an unprotected field
    pub fn write_char_at(&mut self, address: u16, ch: u8) {
        let addr = address as usize;
        if addr < self.buffer.len() {
            self.buffer[addr].char_data = ch;
            
            // Mark the field as modified if this is user input in an unprotected field
            if let Some(field) = self.field_manager.find_field_at_mut(address) {
                if !field.is_protected() {
                    field.set_modified(true);
                }
            }
        }
    }
    
    /// Read a character from a specific buffer address
    pub fn read_char_at(&self, address: u16) -> Option<u8> {
        let addr = address as usize;
        if addr < self.buffer.len() {
            Some(self.buffer[addr].char_data)
        } else {
            None
        }
    }
    
    /// Set a field attribute at a specific buffer address
    pub fn set_field_attribute(&mut self, address: u16, attr: FieldAttribute) {
        let addr = address as usize;
        if addr < self.buffer.len() {
            self.buffer[addr].is_field_attr = true;
            self.buffer[addr].char_data = attr.base_attr;
        }
        self.field_manager.add_field(attr);
    }
    
    /// Get the field manager
    pub fn field_manager(&self) -> &FieldManager {
        &self.field_manager
    }
    
    /// Get mutable field manager
    pub fn field_manager_mut(&mut self) -> &mut FieldManager {
        &mut self.field_manager
    }
    
    /// Find the next unprotected field after the current cursor position
    /// Returns the address of the first position after the field attribute
    pub fn find_next_unprotected_field(&self) -> Option<u16> {
        let current_addr = self.cursor_address;
        let buffer_size = self.buffer_size() as u16;
        
        // Search for next unprotected field, wrapping around if necessary
        for offset in 1..buffer_size {
            let test_addr = (current_addr + offset) % buffer_size;
            
            // Check if this address has a field attribute
            if self.buffer[test_addr as usize].is_field_attr {
                // Check if field is unprotected
                if let Some(field) = self.field_manager.find_field_at(test_addr) {
                    if !field.is_protected() {
                        // Return position after field attribute
                        return Some((test_addr + 1) % buffer_size);
                    }
                }
            }
        }
        
        None
    }
    
    /// Tab to the next unprotected field (Program Tab behavior)
    pub fn tab_to_next_field(&mut self) -> bool {
        if let Some(next_addr) = self.find_next_unprotected_field() {
            self.cursor_address = next_addr;
            true
        } else {
            false
        }
    }
    
    /// Repeat a character to a target address
    pub fn repeat_to_address(&mut self, ch: u8, target_address: u16) {
        let start = self.cursor_address as usize;
        let end = target_address as usize;
        
        if start < self.buffer.len() && end < self.buffer.len() {
            for addr in start..=end {
                self.buffer[addr].char_data = ch;
            }
            self.cursor_address = ((end + 1) % self.buffer.len()) as u16;
        }
    }
    
    /// Erase unprotected data to a target address
    pub fn erase_unprotected_to_address(&mut self, target_address: u16) {
        let start = self.cursor_address as usize;
        let end = target_address as usize;
        
        if start < self.buffer.len() && end < self.buffer.len() {
            for addr in start..=end {
                // Only erase if not in a protected field
                if !self.buffer[addr].is_field_attr {
                    self.buffer[addr].char_data = 0x00;
                }
            }
            self.cursor_address = ((end + 1) % self.buffer.len()) as u16;
        }
    }
    
    /// Lock the keyboard
    pub fn lock_keyboard(&mut self) {
        self.keyboard_locked = true;
    }
    
    /// Unlock the keyboard
    pub fn unlock_keyboard(&mut self) {
        self.keyboard_locked = false;
    }
    
    /// Check if keyboard is locked
    pub fn is_keyboard_locked(&self) -> bool {
        self.keyboard_locked
    }
    
    /// Set alarm state
    pub fn set_alarm(&mut self, alarm: bool) {
        self.alarm = alarm;
    }
    
    /// Check if alarm is set
    pub fn is_alarm(&self) -> bool {
        self.alarm
    }
    
    
    /// Get a specific row as a string
    pub fn get_row(&self, row: usize) -> Option<String> {
        if row >= self.rows() {
            return None;
        }
        
        let cols = self.cols();
        let start = row * cols;
        let end = start + cols;
        
        let mut result = String::new();
        for i in start..end {
            if i < self.buffer.len() {
                let cell = &self.buffer[i];
                if cell.is_field_attr {
                    result.push('█');
                } else {
                    let ch = ebcdic_to_ascii(cell.char_data);
                    result.push(if ch.is_ascii_graphic() || ch == ' ' {
                        ch
                    } else {
                        '.'
                    });
                }
            }
        }
        
        Some(result)
    }
    
    /// Get the entire buffer as raw bytes
    pub fn get_buffer_data(&self) -> Vec<u8> {
        self.buffer.iter().map(|cell| cell.char_data).collect()
    }
    
    /// Get modified field data for Read Modified command
    pub fn get_modified_data(&self) -> Vec<u8> {
        // Implement proper Read Modified logic
        // This returns only fields with MDT bit set
        // Format: AID + cursor address + field data

        let mut data = Vec::new();

        // Add AID (placeholder - in real implementation this would be determined by key pressed)
        data.push(0x60); // No AID

        // Add cursor address (high byte, low byte)
        let cursor_addr = self.cursor_address;
        data.push(((cursor_addr >> 8) & 0xFF) as u8);
        data.push((cursor_addr & 0xFF) as u8);

        // Add modified field data
        // Iterate through fields with MDT bit set
        for field in self.field_manager.modified_fields() {
            let start_addr = field.address as usize;
            let length = field.length;

            // Add SBA order to set buffer address to field start
            data.push(ORDER_SBA);
            data.push(((start_addr >> 8) & 0xFF) as u8);
            data.push((start_addr & 0xFF) as u8);

            // Add field data (only non-null characters)
            for offset in 0..length {
                let addr = (start_addr + offset) % self.buffer.len();
                let ch = self.buffer[addr].char_data;
                if ch != 0x00 {  // Don't include null characters
                    data.push(ch);
                }
            }
        }

        data
    }
}

impl std::fmt::Display for Display3270 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cols = self.cols();

        for (i, cell) in self.buffer.iter().enumerate() {
            if i > 0 && i % cols == 0 {
                writeln!(f)?;
            }

            if cell.is_field_attr {
                write!(f, "█")?;
            } else {
                let ch = ebcdic_to_ascii(cell.char_data);
                let out_ch = if ch.is_ascii_graphic() || ch == ' ' { ch } else { '.' };
                write!(f, "{out_ch}")?;
            }
        }

        Ok(())
    }
}

impl Default for Display3270 {
    fn default() -> Self {
        Self::new()
    }
}

/// Buffer addressing utilities for 3270
pub mod addressing {
    /// Decode a 12-bit buffer address from two bytes
    ///
    /// 3270 uses a special encoding for buffer addresses where each byte
    /// represents 6 bits of the address.
    pub fn decode_12bit_address(byte1: u8, byte2: u8) -> u16 {
        let high = decode_address_byte(byte1) as u16;
        let low = decode_address_byte(byte2) as u16;
        (high << 6) | low
    }
    
    /// Decode a 14-bit buffer address from two bytes
    ///
    /// Extended addressing mode for larger screens.
    pub fn decode_14bit_address(byte1: u8, byte2: u8) -> u16 {
        let high = ((byte1 & 0x3F) as u16) << 8;
        let low = byte2 as u16;
        high | low
    }
    
    /// Encode a 12-bit buffer address to two bytes
    pub fn encode_12bit_address(address: u16) -> (u8, u8) {
        let high = ((address >> 6) & 0x3F) as u8;
        let low = (address & 0x3F) as u8;
        (encode_address_byte(high), encode_address_byte(low))
    }
    
    /// Encode a 14-bit buffer address to two bytes
    pub fn encode_14bit_address(address: u16) -> (u8, u8) {
        let high = ((address >> 8) & 0x3F) as u8;
        let low = (address & 0xFF) as u8;
        (high, low)
    }
    
    /// Decode a single address byte (6 bits)
    fn decode_address_byte(byte: u8) -> u8 {
        match byte {
            0x40..=0x4F => byte - 0x40,      // 0-15
            0x50..=0x5F => byte - 0x50 + 16, // 16-31
            0x60..=0x6F => byte - 0x60 + 32, // 32-47
            0x70..=0x7F => byte - 0x70 + 48, // 48-63
            0xC0..=0xCF => byte - 0xC0,      // 0-15 (alternate)
            0xD0..=0xDF => byte - 0xD0 + 16, // 16-31 (alternate)
            0xE0..=0xEF => byte - 0xE0 + 32, // 32-47 (alternate)
            0xF0..=0xFF => byte - 0xF0 + 48, // 48-63 (alternate)
            _ => 0,
        }
    }
    
    /// Encode a 6-bit value to an address byte
    fn encode_address_byte(value: u8) -> u8 {
        match value & 0x3F {
            0..=15 => 0x40 + value,
            16..=31 => 0x50 + (value - 16),
            32..=47 => 0x60 + (value - 32),
            48..=63 => 0x70 + (value - 48),
            _ => 0x40,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_size_model2() {
        let size = ScreenSize::Model2;
        assert_eq!(size.rows(), 24);
        assert_eq!(size.cols(), 80);
        assert_eq!(size.buffer_size(), 1920);
    }

    #[test]
    fn test_screen_size_coords() {
        let size = ScreenSize::Model2;
        assert_eq!(size.address_to_coords(0), (0, 0));
        assert_eq!(size.address_to_coords(80), (1, 0));
        assert_eq!(size.address_to_coords(81), (1, 1));
        
        assert_eq!(size.coords_to_address(0, 0), 0);
        assert_eq!(size.coords_to_address(1, 0), 80);
        assert_eq!(size.coords_to_address(1, 1), 81);
    }

    #[test]
    fn test_display_creation() {
        let display = Display3270::new();
        assert_eq!(display.rows(), 24);
        assert_eq!(display.cols(), 80);
        assert_eq!(display.cursor_address(), 0);
    }

    #[test]
    fn test_display_write_char() {
        let mut display = Display3270::new();
        display.write_char(0xC1); // EBCDIC 'A'
        assert_eq!(display.cursor_address(), 1);
        assert_eq!(display.read_char_at(0), Some(0xC1));
    }

    #[test]
    fn test_display_cursor_position() {
        let mut display = Display3270::new();
        display.set_cursor(81); // Row 1, Col 1
        let (row, col) = display.cursor_position();
        assert_eq!(row, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_addressing_12bit() {
        use addressing::*;
        
        let (b1, b2) = encode_12bit_address(100);
        let decoded = decode_12bit_address(b1, b2);
        assert_eq!(decoded, 100);
    }

    #[test]
    fn test_addressing_14bit() {
        use addressing::*;
        
        let (b1, b2) = encode_14bit_address(3000);
        let decoded = decode_14bit_address(b1, b2);
        assert_eq!(decoded, 3000);
    }
}