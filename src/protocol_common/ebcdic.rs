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
/// Based on IBM Code Page 37 specification with full character coverage.
const EBCDIC_CP037_TO_ASCII: [char; 256] = [
    // 0x00-0x0F: Control characters
    '\x00', '\x01', '\x02', '\x03', '\u{009C}', '\t', '\u{0086}', '\x7F',
    '\u{0097}', '\u{008D}', '\u{008E}', '\x0B', '\x0C', '\r', '\x0E', '\x0F',
    // 0x10-0x1F: Control characters
    '\x10', '\x11', '\x12', '\x13', '\u{009D}', '\u{0085}', '\x08', '\u{0087}',
    '\x18', '\x19', '\u{0092}', '\u{008F}', '\x1C', '\x1D', '\x1E', '\x1F',
    // 0x20-0x2F: Control characters and special
    '\u{0080}', '\u{0081}', '\u{0082}', '\u{0083}', '\u{0084}', '\n', '\x17', '\x1B',
    '\u{0088}', '\u{0089}', '\u{008A}', '\u{008B}', '\u{008C}', '\x05', '\x06', '\x07',
    // 0x30-0x3F: Control characters
    '\u{0090}', '\u{0091}', '\x16', '\u{0093}', '\u{0094}', '\u{0095}', '\u{0096}', '\x04',
    '\u{0098}', '\u{0099}', '\u{009A}', '\u{009B}', '\x14', '\x15', '\u{009E}', '\x1A',
    // 0x40-0x4F: Space and special characters
    ' ', '\u{00A0}', '\u{00E2}', '\u{00E4}', '\u{00E0}', '\u{00E1}', '\u{00E3}', '\u{00E5}',
    '\u{00E7}', '\u{00F1}', '\u{00A2}', '.', '<', '(', '+', '|',
    // 0x50-0x5F: Ampersand and special characters
    '&', '\u{00E9}', '\u{00EA}', '\u{00EB}', '\u{00E8}', '\u{00ED}', '\u{00EE}', '\u{00EF}',
    '\u{00EC}', '\u{00DF}', '!', '$', '*', ')', ';', '\u{00AC}',
    // 0x60-0x6F: Dash and special characters
    '-', '/', '\u{00C2}', '\u{00C4}', '\u{00C0}', '\u{00C1}', '\u{00C3}', '\u{00C5}',
    '\u{00C7}', '\u{00D1}', '\u{00A6}', ',', '%', '_', '>', '?',
    // 0x70-0x7F: Special characters and quotes
    '\u{00F8}', '\u{00C9}', '\u{00CA}', '\u{00CB}', '\u{00C8}', '\u{00CD}', '\u{00CE}', '\u{00CF}',
    '\u{00CC}', '`', ':', '#', '@', '\'', '=', '"',
    // 0x80-0x8F: Special character and lowercase a-i
    '\u{00D8}', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'i', '\u{00AB}', '\u{00BB}', '\u{00F0}', '\u{00FD}', '\u{00FE}', '\u{00B1}',
    // 0x90-0x9F: Degree symbol and lowercase j-r
    '\u{00B0}', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
    'q', 'r', '\u{00AA}', '\u{00BA}', '\u{00E6}', '\u{00B8}', '\u{00C6}', '\u{00A4}',
    // 0xA0-0xAF: Micro sign and lowercase s-z
    '\u{00B5}', '~', 's', 't', 'u', 'v', 'w', 'x',
    'y', 'z', '\u{00A1}', '\u{00BF}', '\u{00D0}', '\u{00DD}', '\u{00DE}', '\u{00AE}',
    // 0xB0-0xBF: Caret and special characters
    '^', '\u{00A3}', '\u{00A5}', '\u{00B7}', '\u{00A9}', '\u{00A7}', '\u{00B6}', '\u{00BC}',
    '\u{00BD}', '\u{00BE}', '[', ']', '\u{00AF}', '\u{00A8}', '\u{00B4}', '\u{00D7}',
    // 0xC0-0xCF: Left brace and uppercase A-I
    '{', 'A', 'B', 'C', 'D', 'E', 'F', 'G',
    'H', 'I', '\u{00AD}', '\u{00F4}', '\u{00F6}', '\u{00F2}', '\u{00F3}', '\u{00F5}',
    // 0xD0-0xDF: Right brace and uppercase J-R
    '}', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
    'Q', 'R', '\u{00B9}', '\u{00FB}', '\u{00FC}', '\u{00F9}', '\u{00FA}', '\u{00FF}',
    // 0xE0-0xEF: Backslash and uppercase S-Z
    '\\', '\u{00F7}', 'S', 'T', 'U', 'V', 'W', 'X',
    'Y', 'Z', '\u{00B2}', '\u{00D4}', '\u{00D6}', '\u{00D2}', '\u{00D3}', '\u{00D5}',
    // 0xF0-0xFF: Digits 0-9 and special characters
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', '\u{00B3}', '\u{00DB}', '\u{00DC}', '\u{00D9}', '\u{00DA}', '\u{009F}',
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
        // Control characters (0x00-0x1F, 0x7F)
        '\x00' => 0x00,
        '\x01' => 0x01,
        '\x02' => 0x02,
        '\x03' => 0x03,
        '\x04' => 0x37,
        '\x05' => 0x2D,
        '\x06' => 0x2E,
        '\x07' => 0x2F,
        '\x08' => 0x16,
        '\t' => 0x05,
        '\n' => 0x25,
        '\x0B' => 0x0B,
        '\x0C' => 0x0C,
        '\r' => 0x0D,
        '\x0E' => 0x0E,
        '\x0F' => 0x0F,
        '\x10' => 0x10,
        '\x11' => 0x11,
        '\x12' => 0x12,
        '\x13' => 0x13,
        '\x14' => 0x3C,
        '\x15' => 0x3D,
        '\x16' => 0x32,
        '\x17' => 0x26,
        '\x18' => 0x18,
        '\x19' => 0x19,
        '\x1A' => 0x3F,
        '\x1B' => 0x27,
        '\x1C' => 0x1C,
        '\x1D' => 0x1D,
        '\x1E' => 0x1E,
        '\x1F' => 0x1F,
        '\x7F' => 0x07,
        
        // Space and printable ASCII
        ' ' => 0x40,
        '!' => 0x5A,
        '"' => 0x7F,
        '#' => 0x7B,
        '$' => 0x5B,
        '%' => 0x6C,
        '&' => 0x50,
        '\'' => 0x7D,
        '(' => 0x4D,
        ')' => 0x5D,
        '*' => 0x5C,
        '+' => 0x4E,
        ',' => 0x6B,
        '-' => 0x60,
        '.' => 0x4B,
        '/' => 0x61,
        
        // Digits
        '0'..='9' => 0xF0 + (ch as u8 - b'0'),
        
        // Special characters
        ':' => 0x7A,
        ';' => 0x5E,
        '<' => 0x4C,
        '=' => 0x7E,
        '>' => 0x6E,
        '?' => 0x6F,
        '@' => 0x7C,
        
        // Uppercase letters
        'A'..='I' => 0xC1 + (ch as u8 - b'A'),
        'J'..='R' => 0xD1 + (ch as u8 - b'J'),
        'S'..='Z' => 0xE2 + (ch as u8 - b'S'),
        
        // Brackets and special
        '[' => 0xBA,
        '\\' => 0xE0,
        ']' => 0xBB,
        '^' => 0xB0,
        '_' => 0x6D,
        '`' => 0x79,
        
        // Lowercase letters
        'a'..='i' => 0x81 + (ch as u8 - b'a'),
        'j'..='r' => 0x91 + (ch as u8 - b'j'),
        's'..='z' => 0xA2 + (ch as u8 - b's'),
        
        // Braces and special
        '{' => 0xC0,
        '|' => 0x4F,
        '}' => 0xD0,
        '~' => 0xA1,
        
        // Extended Latin-1 characters (Unicode)
        '\u{0080}' => 0x20,
        '\u{0081}' => 0x21,
        '\u{0082}' => 0x22,
        '\u{0083}' => 0x23,
        '\u{0084}' => 0x24,
        '\u{0085}' => 0x15,
        '\u{0086}' => 0x06,
        '\u{0087}' => 0x17,
        '\u{0088}' => 0x28,
        '\u{0089}' => 0x29,
        '\u{008A}' => 0x2A,
        '\u{008B}' => 0x2B,
        '\u{008C}' => 0x2C,
        '\u{008D}' => 0x09,
        '\u{008E}' => 0x0A,
        '\u{008F}' => 0x1B,
        '\u{0090}' => 0x30,
        '\u{0091}' => 0x31,
        '\u{0092}' => 0x1A,
        '\u{0093}' => 0x33,
        '\u{0094}' => 0x34,
        '\u{0095}' => 0x35,
        '\u{0096}' => 0x36,
        '\u{0097}' => 0x08,
        '\u{0098}' => 0x38,
        '\u{0099}' => 0x39,
        '\u{009A}' => 0x3A,
        '\u{009B}' => 0x3B,
        '\u{009C}' => 0x04,
        '\u{009D}' => 0x14,
        '\u{009E}' => 0x3E,
        '\u{009F}' => 0xFF,
        '\u{00A0}' => 0x41, // Non-breaking space
        '\u{00A1}' => 0xAA, // Inverted exclamation
        '\u{00A2}' => 0x4A, // Cent sign
        '\u{00A3}' => 0xB1, // Pound sign
        '\u{00A4}' => 0x9F, // Currency sign
        '\u{00A5}' => 0xB2, // Yen sign
        '\u{00A6}' => 0x6A, // Broken bar
        '\u{00A7}' => 0xB5, // Section sign
        '\u{00A8}' => 0xBD, // Diaeresis
        '\u{00A9}' => 0xB4, // Copyright sign
        '\u{00AA}' => 0x9A, // Feminine ordinal
        '\u{00AB}' => 0x8A, // Left angle quotation
        '\u{00AC}' => 0x5F, // Not sign
        '\u{00AD}' => 0xCA, // Soft hyphen
        '\u{00AE}' => 0xAF, // Registered sign
        '\u{00AF}' => 0xBC, // Macron
        '\u{00B0}' => 0x90, // Degree sign
        '\u{00B1}' => 0x8F, // Plus-minus sign
        '\u{00B2}' => 0xEA, // Superscript two
        '\u{00B3}' => 0xFA, // Superscript three
        '\u{00B4}' => 0xBE, // Acute accent
        '\u{00B5}' => 0xA0, // Micro sign
        '\u{00B6}' => 0xB6, // Pilcrow sign
        '\u{00B7}' => 0xB3, // Middle dot
        '\u{00B8}' => 0x9D, // Cedilla
        '\u{00B9}' => 0xDA, // Superscript one
        '\u{00BA}' => 0x9B, // Masculine ordinal
        '\u{00BB}' => 0x8B, // Right angle quotation
        '\u{00BC}' => 0xB7, // Vulgar fraction 1/4
        '\u{00BD}' => 0xB8, // Vulgar fraction 1/2
        '\u{00BE}' => 0xB9, // Vulgar fraction 3/4
        '\u{00BF}' => 0xAB, // Inverted question mark
        '\u{00C0}' => 0x64, // A grave
        '\u{00C1}' => 0x65, // A acute
        '\u{00C2}' => 0x62, // A circumflex
        '\u{00C3}' => 0x66, // A tilde
        '\u{00C4}' => 0x63, // A diaeresis
        '\u{00C5}' => 0x67, // A ring
        '\u{00C6}' => 0x9E, // AE ligature
        '\u{00C7}' => 0x68, // C cedilla
        '\u{00C8}' => 0x74, // E grave
        '\u{00C9}' => 0x71, // E acute
        '\u{00CA}' => 0x72, // E circumflex
        '\u{00CB}' => 0x73, // E diaeresis
        '\u{00CC}' => 0x78, // I grave
        '\u{00CD}' => 0x75, // I acute
        '\u{00CE}' => 0x76, // I circumflex
        '\u{00CF}' => 0x77, // I diaeresis
        '\u{00D0}' => 0xAC, // Eth
        '\u{00D1}' => 0x69, // N tilde
        '\u{00D2}' => 0xED, // O grave
        '\u{00D3}' => 0xEE, // O acute
        '\u{00D4}' => 0xEB, // O circumflex
        '\u{00D5}' => 0xEF, // O tilde
        '\u{00D6}' => 0xEC, // O diaeresis
        '\u{00D7}' => 0xBF, // Multiplication sign
        '\u{00D8}' => 0x80, // O slash
        '\u{00D9}' => 0xFD, // U grave
        '\u{00DA}' => 0xFE, // U acute
        '\u{00DB}' => 0xFB, // U circumflex
        '\u{00DC}' => 0xFC, // U diaeresis
        '\u{00DD}' => 0xAD, // Y acute
        '\u{00DE}' => 0xAE, // Thorn
        '\u{00DF}' => 0x59, // Sharp s
        '\u{00E0}' => 0x44, // a grave
        '\u{00E1}' => 0x45, // a acute
        '\u{00E2}' => 0x42, // a circumflex
        '\u{00E3}' => 0x46, // a tilde
        '\u{00E4}' => 0x43, // a diaeresis
        '\u{00E5}' => 0x47, // a ring
        '\u{00E6}' => 0x9C, // ae ligature
        '\u{00E7}' => 0x48, // c cedilla
        '\u{00E8}' => 0x54, // e grave
        '\u{00E9}' => 0x51, // e acute
        '\u{00EA}' => 0x52, // e circumflex
        '\u{00EB}' => 0x53, // e diaeresis
        '\u{00EC}' => 0x58, // i grave
        '\u{00ED}' => 0x55, // i acute
        '\u{00EE}' => 0x56, // i circumflex
        '\u{00EF}' => 0x57, // i diaeresis
        '\u{00F0}' => 0x8C, // eth
        '\u{00F1}' => 0x49, // n tilde
        '\u{00F2}' => 0xCD, // o grave
        '\u{00F3}' => 0xCE, // o acute
        '\u{00F4}' => 0xCB, // o circumflex
        '\u{00F5}' => 0xCF, // o tilde
        '\u{00F6}' => 0xCC, // o diaeresis
        '\u{00F7}' => 0xE1, // Division sign
        '\u{00F8}' => 0x70, // o slash
        '\u{00F9}' => 0xDD, // u grave
        '\u{00FA}' => 0xDE, // u acute
        '\u{00FB}' => 0xDB, // u circumflex
        '\u{00FC}' => 0xDC, // u diaeresis
        '\u{00FD}' => 0x8D, // y acute
        '\u{00FE}' => 0x8E, // thorn
        '\u{00FF}' => 0xDF, // y diaeresis
        
        // Default to space for unmapped characters
        _ => 0x40,
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