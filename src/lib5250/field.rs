/// Field attribute and management logic for 5250

use crate::terminal::CharAttribute as FieldAttribute;

/// Detected field struct
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub label: Option<String>,
    pub row: usize,
    pub col: usize,
    pub length: usize,
    pub attribute: FieldAttribute,
}

/// Detect fields from a terminal screen and parse attributes
pub fn detect_fields_from_screen(screen: &crate::terminal::TerminalScreen) -> Vec<Field> {
    let mut fields = Vec::new();
    let screen_str = screen.to_string();
    for (row_idx, line) in screen_str.lines().enumerate() {
        // Example: detect fields with underscores
        if let Some(col) = line.find('_') {
            let length = line.chars().skip(col).take_while(|&c| c == '_').count();
            let label = if let Some(label_end) = line[..col].rfind(':') {
                Some(line[..label_end].trim().to_string())
            } else {
                None
            };
            fields.push(Field {
                label,
                row: row_idx + 1,
                col: col + 1,
                length,
                attribute: FieldAttribute::Protected, // Stub: always Protected
            });
        }
    }
    fields
}

/// Parse field attributes from 5250 protocol data
pub fn parse_field_attribute(attribute_byte: u8) -> FieldAttribute {
    FieldAttribute::from_u8(attribute_byte)
}

/// Detect fields from raw 5250 protocol data
pub fn detect_fields_from_protocol_data(data: &[u8]) -> Vec<Field> {
    let mut fields = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        // Look for field start markers in 5250 data
        // This is a simplified implementation - real 5250 parsing is more complex
        if data[pos] == 0x1D { // Start Field Extended (SFE) order
            pos += 1;
            if pos + 1 < data.len() {
                let field_length = data[pos] as usize;
                let attribute_byte = data[pos + 1];

                // Skip the attribute data for now
                pos += 2 + field_length;

                // Create field with parsed attribute
                let attribute = parse_field_attribute(attribute_byte);
                fields.push(Field {
                    label: None, // Would need more complex parsing for labels
                    row: 0, // Would need cursor position tracking
                    col: 0, // Would need cursor position tracking
                    length: field_length,
                    attribute,
                });
            } else {
                break;
            }
        } else {
            pos += 1;
        }
    }

    fields
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::TerminalScreen;

    #[test]
    fn test_detect_fields_with_label() {
        let mut screen = TerminalScreen::new();
        screen.write_string("Name: ______\n");
        let fields = detect_fields_from_screen(&screen);
        assert!(!fields.is_empty());
        assert_eq!(fields[0].attribute, FieldAttribute::Protected); // Stub logic
    }

    #[test]
    fn test_detect_fields_no_label() {
        let mut screen = TerminalScreen::new();
        screen.write_string("No fields here\n");
        let fields = detect_fields_from_screen(&screen);
        assert!(fields.is_empty());
    }

    #[test]
    fn test_parse_field_attribute_protected() {
        let attr = parse_field_attribute(0x20);
        assert_eq!(attr, FieldAttribute::Protected);
    }

    #[test]
    fn test_parse_field_attribute_numeric() {
        let attr = parse_field_attribute(0x10);
        assert_eq!(attr, FieldAttribute::Numeric);
    }

    #[test]
    fn test_parse_field_attribute_normal() {
        let attr = parse_field_attribute(0x00);
        assert_eq!(attr, FieldAttribute::Normal);
    }

    #[test]
    fn test_parse_field_attribute_mandatory() {
        let attr = parse_field_attribute(0x0C);
        assert_eq!(attr, FieldAttribute::Mandatory);
    }

    #[test]
    fn test_detect_fields_from_protocol_data() {
        // Mock 5250 protocol data with a field
        let data = vec![
            0x1D, // Start Field Extended
            0x05, // Field length
            0x20, // Protected attribute
            0x00, 0x00, 0x00, 0x00, 0x00, // Field data
        ];

        let fields = detect_fields_from_protocol_data(&data);
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].length, 5);
        assert_eq!(fields[0].attribute, FieldAttribute::Protected);
    }
}
