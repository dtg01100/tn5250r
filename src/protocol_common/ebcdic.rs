//! EBCDIC to ASCII conversion utilities
//!
//! This module provides shared EBCDIC (Extended Binary Coded Decimal Interchange Code)
//! to ASCII conversion functionality for both TN5250 and TN3270 protocols.
//!
//! The conversion tables implement the CP037 (EBCDIC US/Canada) code page, which is
//! the most commonly used EBCDIC variant in IBM mainframe and AS/400 systems.

/// Enhanced EBCDIC to ASCII translation table (CP037) with comprehensive mapping
///
/// This table maps all 256 EBCDIC code points to their ASCII equivalents.
/// Code page 037 is the standard EBCDIC encoding for US/Canada English.
const EBCDIC_CP037_TO_ASCII: [char; 256] = [
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

/// Convert an EBCDIC byte to an ASCII character
///
/// # Arguments
///
/// * `byte` - The EBCDIC byte to convert
///
/// # Returns
///
/// The corresponding ASCII character from the CP037 code page
///
/// # Examples
///
/// ```
/// use tn5250r::protocol_common::ebcdic::ebcdic_to_ascii;
///
/// // EBCDIC 0xC1 is ASCII 'A'
/// assert_eq!(ebcdic_to_ascii(0xC1), 'A');
///
/// // EBCDIC 0x81 is ASCII 'a'
/// assert_eq!(ebcdic_to_ascii(0x81), 'a');
///
/// // EBCDIC 0xF0 is ASCII '0'
/// assert_eq!(ebcdic_to_ascii(0xF0), '0');
/// ```
pub fn ebcdic_to_ascii(byte: u8) -> char {
    EBCDIC_CP037_TO_ASCII[byte as usize]
}

/// Convert an ASCII character to an EBCDIC byte
///
/// This function provides a reverse mapping from ASCII to EBCDIC CP037.
/// For characters not in the mapping, it returns 0x40 (EBCDIC space).
///
/// # Arguments
///
/// * `ch` - The ASCII character to convert
///
/// # Returns
///
/// The corresponding EBCDIC byte in CP037 encoding
///
/// # Examples
///
/// ```
/// use tn5250r::protocol_common::ebcdic::ascii_to_ebcdic;
///
/// // ASCII 'A' is EBCDIC 0xC1
/// assert_eq!(ascii_to_ebcdic('A'), 0xC1);
///
/// // ASCII 'a' is EBCDIC 0x81
/// assert_eq!(ascii_to_ebcdic('a'), 0x81);
///
/// // ASCII '0' is EBCDIC 0xF0
/// assert_eq!(ascii_to_ebcdic('0'), 0xF0);
/// ```
pub fn ascii_to_ebcdic(ch: char) -> u8 {
    match ch {
        ' ' => 0x40,
        'a'..='i' => 0x81 + (ch as u8 - b'a'),
        'j'..='r' => 0x91 + (ch as u8 - b'j'),
        's'..='z' => 0xA2 + (ch as u8 - b's'),
        'A'..='I' => 0xC1 + (ch as u8 - b'A'),
        'J'..='R' => 0xD1 + (ch as u8 - b'J'),
        'S'..='Z' => 0xE2 + (ch as u8 - b'S'),
        '0'..='9' => 0xF0 + (ch as u8 - b'0'),
        '.' => 0x4B,
        '<' => 0x4C,
        '(' => 0x4D,
        '+' => 0x4E,
        '|' => 0x4F,
        '&' => 0x50,
        '!' => 0x5A,
        '$' => 0x5B,
        '*' => 0x5C,
        ')' => 0x5D,
        ';' => 0x5E,
        '-' => 0x60,
        '/' => 0x61,
        ',' => 0x6B,
        '%' => 0x6C,
        '_' => 0x6D,
        '>' => 0x6E,
        '?' => 0x6F,
        ':' => 0x7A,
        '#' => 0x7B,
        '@' => 0x7C,
        '\'' => 0x7D,
        '=' => 0x7E,
        '"' => 0x7F,
        _ => 0x40, // Default to space for unmapped characters
    }
}

/// Convert an EBCDIC byte slice to an ASCII String
///
/// This is a convenience function that converts multiple EBCDIC bytes
/// to an ASCII string in one operation.
///
/// # Arguments
///
/// * `bytes` - A slice of EBCDIC bytes to convert
///
/// # Returns
///
/// A String containing the ASCII representation
///
/// # Examples
///
/// ```
/// use tn5250r::protocol_common::ebcdic::ebcdic_to_ascii_string;
///
/// let ebcdic_data = vec![0xC8, 0xC5, 0xD3, 0xD3, 0xD6]; // "HELLO" in EBCDIC
/// assert_eq!(ebcdic_to_ascii_string(&ebcdic_data), "HELLO");
/// ```
pub fn ebcdic_to_ascii_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| ebcdic_to_ascii(b)).collect()
}

/// Convert an ASCII string to an EBCDIC byte vector
///
/// This is a convenience function that converts an ASCII string
/// to EBCDIC bytes in one operation.
///
/// # Arguments
///
/// * `s` - The ASCII string to convert
///
/// # Returns
///
/// A Vec<u8> containing the EBCDIC representation
///
/// # Examples
///
/// ```
/// use tn5250r::protocol_common::ebcdic::ascii_to_ebcdic_vec;
///
/// let ascii_str = "HELLO";
/// let ebcdic_data = ascii_to_ebcdic_vec(ascii_str);
/// assert_eq!(ebcdic_data, vec![0xC8, 0xC5, 0xD3, 0xD3, 0xD6]);
/// ```
pub fn ascii_to_ebcdic_vec(s: &str) -> Vec<u8> {
    s.chars().map(ascii_to_ebcdic).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ebcdic_to_ascii_letters() {
        // Test uppercase letters
        assert_eq!(ebcdic_to_ascii(0xC1), 'A');
        assert_eq!(ebcdic_to_ascii(0xC8), 'H');
        assert_eq!(ebcdic_to_ascii(0xE9), 'Z');

        // Test lowercase letters
        assert_eq!(ebcdic_to_ascii(0x81), 'a');
        assert_eq!(ebcdic_to_ascii(0x88), 'h');
        assert_eq!(ebcdic_to_ascii(0xA9), 'z');
    }

    #[test]
    fn test_ebcdic_to_ascii_digits() {
        assert_eq!(ebcdic_to_ascii(0xF0), '0');
        assert_eq!(ebcdic_to_ascii(0xF5), '5');
        assert_eq!(ebcdic_to_ascii(0xF9), '9');
    }

    #[test]
    fn test_ascii_to_ebcdic_letters() {
        // Test uppercase letters
        assert_eq!(ascii_to_ebcdic('A'), 0xC1);
        assert_eq!(ascii_to_ebcdic('H'), 0xC8);
        assert_eq!(ascii_to_ebcdic('Z'), 0xE9);

        // Test lowercase letters
        assert_eq!(ascii_to_ebcdic('a'), 0x81);
        assert_eq!(ascii_to_ebcdic('h'), 0x88);
        assert_eq!(ascii_to_ebcdic('z'), 0xA9);
    }

    #[test]
    fn test_ascii_to_ebcdic_digits() {
        assert_eq!(ascii_to_ebcdic('0'), 0xF0);
        assert_eq!(ascii_to_ebcdic('5'), 0xF5);
        assert_eq!(ascii_to_ebcdic('9'), 0xF9);
    }

    #[test]
    fn test_round_trip_conversion() {
        // Test that converting to EBCDIC and back gives the same character
        let test_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        for ch in test_chars.chars() {
            let ebcdic = ascii_to_ebcdic(ch);
            let ascii = ebcdic_to_ascii(ebcdic);
            assert_eq!(ch, ascii, "Round trip failed for '{}'", ch);
        }
    }

    #[test]
    fn test_string_conversion() {
        let ascii_str = "HELLO WORLD";
        let ebcdic_vec = ascii_to_ebcdic_vec(ascii_str);
        let result = ebcdic_to_ascii_string(&ebcdic_vec);
        assert_eq!(result, ascii_str);
    }
}