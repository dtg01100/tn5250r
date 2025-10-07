//! EBCDIC to ASCII character translation utilities for TN5250R
//!
//! This module provides EBCDIC character translation functionality needed for
//! IBM 5250 terminal protocol implementation. It consolidates all EBCDIC
//! translation logic into a single source of truth.

/// EBCDIC CP037 to ASCII translation table for IBM 5250 terminals
/// 
/// This table follows the EBCDIC Code Page 037 (US/Canada) standard
/// which is commonly used in IBM AS/400 systems.
pub const EBCDIC_CP037_TO_ASCII: [char; 256] = [
    '\x00', '\x01', '\x02', '\x03', '\x37', '\x2D', '\x2E', '\x2F',
    '\x16', '\x05', '\x25', '\x0B', '\x0C', '\r',   '\x0E', '\x0F',
    '\x10', '\x11', '\x12', '\x13', '\x3C', '\x3D', '\x32', '\x26',
    '\x18', '\x19', '\x3F', '\x27', '\x1C', '\x1D', '\x1E', '\x1F',
    '\x40', '\x5A', '\x7F', '\x7B', '\x5B', '\n',   '\x17', '\x1B',
    '\x60', '\x61', '\x62', '\x63', '\x64', '\x65', '\x66', '\x67',
    '\x68', '\x69', '\x70', '\x71', '\x72', '\x73', '\x74', '\x75',
    '\x76', '\x77', '\x78', '\x79', '\x7A', '\x7B', '\x7C', '\x7D',
    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',
    ' ',    ' ',    '[',    '.',    '<',    '(',    '+',    '|',
    '&',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',
    ' ',    ' ',    '!',    '$',    '*',    ')',    ';',    ' ',
    '-',    '/',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',
    ' ',    ' ',    '|',    ',',    '%',    '_',    '>',    '?',
    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',
    ' ',    '`',    ':',    '#',    '@',    '\'',   '=',    '"',
    ' ',    'a',    'b',    'c',    'd',    'e',    'f',    'g',
    'h',    'i',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',
    ' ',    'j',    'k',    'l',    'm',    'n',    'o',    'p',
    'q',    'r',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',
    ' ',    '~',    's',    't',    'u',    'v',    'w',    'x',
    'y',    'z',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',
    '^',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',
    ' ',    ' ',    '[',    ']',    ' ',    ' ',    ' ',    ' ',
    '{',    'A',    'B',    'C',    'D',    'E',    'F',    'G',
    'H',    'I',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',
    '}',    'J',    'K',    'L',    'M',    'N',    'O',    'P',
    'Q',    'R',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',
    '\\',   ' ',    'S',    'T',    'U',    'V',    'W',    'X',
    'Y',    'Z',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',
    '0',    '1',    '2',    '3',    '4',    '5',    '6',    '7',
    '8',    '9',    ' ',    ' ',    ' ',    ' ',    ' ',    ' ',
];

/// Convert an EBCDIC byte to its ASCII character equivalent
/// 
/// # Arguments
/// * `ebcdic_byte` - The EBCDIC byte value to convert
/// 
/// # Returns
/// The corresponding ASCII character
/// 
/// # Performance
/// This function uses a lookup table for O(1) conversion time.
#[inline(always)]
pub fn ebcdic_to_ascii(ebcdic_byte: u8) -> char {
    EBCDIC_CP037_TO_ASCII[ebcdic_byte as usize]
}

/// Convert an EBCDIC byte slice to an ASCII string
/// 
/// # Arguments
/// * `ebcdic_bytes` - The EBCDIC bytes to convert
/// 
/// # Returns
/// A String containing the ASCII equivalent characters
pub fn ebcdic_slice_to_ascii(ebcdic_bytes: &[u8]) -> String {
    ebcdic_bytes.iter()
        .map(|&byte| ebcdic_to_ascii(byte))
        .collect()
}

/// Convert an EBCDIC byte buffer to ASCII, filtering out null characters
/// 
/// # Arguments
/// * `ebcdic_bytes` - The EBCDIC bytes to convert
/// 
/// # Returns
/// A String containing the ASCII equivalent characters with nulls removed
pub fn ebcdic_to_ascii_string(ebcdic_bytes: &[u8]) -> String {
    ebcdic_bytes.iter()
        .map(|&byte| ebcdic_to_ascii(byte))
        .filter(|&ch| ch != '\0')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebcdic_to_ascii_basic() {
        // Test space character (0x40)
        assert_eq!(ebcdic_to_ascii(0x40), ' ');
        
        // Test letter A (0xC1)
        assert_eq!(ebcdic_to_ascii(0xC1), 'A');
        
        // Test letter a (0x81)
        assert_eq!(ebcdic_to_ascii(0x81), 'a');
        
        // Test digit 0 (0xF0)
        assert_eq!(ebcdic_to_ascii(0xF0), '0');
        
        // Test digit 9 (0xF9)
        assert_eq!(ebcdic_to_ascii(0xF9), '9');
    }

    #[test]
    fn test_ebcdic_slice_to_ascii() {
        // Test "HELLO" in EBCDIC: [0xC8, 0xC5, 0xD3, 0xD3, 0xD6]
        let ebcdic_hello = &[0xC8, 0xC5, 0xD3, 0xD3, 0xD6];
        let ascii_result = ebcdic_slice_to_ascii(ebcdic_hello);
        assert_eq!(ascii_result, "HELLO");
    }

    #[test]
    fn test_ebcdic_to_ascii_string_filters_nulls() {
        // Test with nulls mixed in
        let ebcdic_data = &[0x00, 0xC8, 0x00, 0xC9, 0x00]; // null, H, null, I, null
        let result = ebcdic_to_ascii_string(ebcdic_data);
        assert_eq!(result, "HI");
    }
}