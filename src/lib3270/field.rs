//! Field attribute and management logic for 3270
//!
//! This module handles 3270 field attributes, including both basic field
//! attributes (from SF order) and extended field attributes (from SFE order).

#![allow(dead_code)] // Complete TN3270 field implementation

use super::codes::*;

/// 3270 Field Attribute Structure
///
/// Represents a field on the 3270 screen with its attributes.
/// Unlike 5250, 3270 fields use buffer addressing and can have
/// extended attributes for color, highlighting, and validation.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldAttribute {
    /// Buffer address where the field starts
    pub address: u16,
    
    /// Base attribute byte (from SF order)
    pub base_attr: u8,
    
    /// Extended attributes (from SFE order)
    pub extended_attrs: ExtendedAttributes,
    
    /// Field length (calculated from next field or end of buffer)
    pub length: usize,
}

impl FieldAttribute {
    /// Create a new field attribute with base attribute only
    pub fn new(address: u16, base_attr: u8) -> Self {
        Self {
            address,
            base_attr,
            extended_attrs: ExtendedAttributes::default(),
            length: 0,
        }
    }
    
    /// Create a new field attribute with extended attributes
    pub fn new_extended(address: u16, base_attr: u8, extended_attrs: ExtendedAttributes) -> Self {
        Self {
            address,
            base_attr,
            extended_attrs,
            length: 0,
        }
    }
    
    /// Check if field is protected
    pub fn is_protected(&self) -> bool {
        (self.base_attr & ATTR_PROTECTED) != 0
    }
    
    /// Check if field is numeric
    pub fn is_numeric(&self) -> bool {
        (self.base_attr & ATTR_NUMERIC) != 0
    }
    
    /// Check if field is hidden (non-display)
    pub fn is_hidden(&self) -> bool {
        (self.base_attr & ATTR_DISPLAY) == DISPLAY_HIDDEN
    }
    
    /// Check if field is intensified
    pub fn is_intensified(&self) -> bool {
        (self.base_attr & ATTR_DISPLAY) == DISPLAY_INTENSIFIED
    }
    
    /// Check if Modified Data Tag (MDT) is set
    pub fn is_modified(&self) -> bool {
        (self.base_attr & ATTR_MDT) != 0
    }
    
    /// Set the Modified Data Tag (MDT)
    pub fn set_modified(&mut self, modified: bool) {
        if modified {
            self.base_attr |= ATTR_MDT;
        } else {
            self.base_attr &= !ATTR_MDT;
        }
    }
    
    /// Get display attribute (normal, intensified, or hidden)
    pub fn display_attr(&self) -> u8 {
        self.base_attr & ATTR_DISPLAY
    }
    
    /// Check if field has mandatory fill validation
    pub fn is_mandatory_fill(&self) -> bool {
        if let Some(validation) = self.extended_attrs.validation {
            (validation & VALIDATION_MANDATORY_FILL) != 0
        } else {
            false
        }
    }
    
    /// Check if field has mandatory entry validation
    pub fn is_mandatory_entry(&self) -> bool {
        if let Some(validation) = self.extended_attrs.validation {
            (validation & VALIDATION_MANDATORY_ENTRY) != 0
        } else {
            false
        }
    }
    
    /// Check if field has trigger validation
    pub fn is_trigger(&self) -> bool {
        if let Some(validation) = self.extended_attrs.validation {
            (validation & VALIDATION_TRIGGER) != 0
        } else {
            false
        }
    }
    
    /// Validate field content against field attributes
    /// Returns Ok(()) if valid, Err with message if validation fails
    pub fn validate_content(&self, content: &[u8]) -> Result<(), String> {
        // Check mandatory fill - all positions must be filled
        if self.is_mandatory_fill() {
            if content.len() < self.length {
                return Err("Mandatory fill: field must be completely filled".to_string());
            }
            // Check for null or space characters
            for ch in content {
                if *ch == 0x00 || *ch == 0x40 {  // Null or EBCDIC space
                    return Err("Mandatory fill: field must be completely filled".to_string());
                }
            }
        }
        
        // Check mandatory entry - at least one character must be entered
        if self.is_mandatory_entry() {
            let has_content = content.iter().any(|&ch| ch != 0x00 && ch != 0x40);
            if !has_content {
                return Err("Mandatory entry: field must have at least one character".to_string());
            }
        }
        
        // Check numeric field - only digits allowed
        if self.is_numeric() {
            for ch in content {
                // EBCDIC digits are 0xF0-0xF9
                if *ch != 0x00 && *ch != 0x40 && !(*ch >= 0xF0 && *ch <= 0xF9) {
                    return Err("Numeric field: only digits allowed".to_string());
                }
            }
        }
        
        Ok(())
    }
}

/// Extended Field Attributes
///
/// 3270 supports extended attributes via the SFE (Start Field Extended) order.
/// These provide additional formatting capabilities beyond the base attribute.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExtendedAttributes {
    /// Highlighting attribute (normal, blink, reverse, underscore)
    pub highlighting: Option<u8>,
    
    /// Foreground color
    pub foreground_color: Option<u8>,
    
    /// Background color
    pub background_color: Option<u8>,
    
    /// Character set
    pub charset: Option<u8>,
    
    /// Field validation (mandatory fill, mandatory entry, trigger)
    pub validation: Option<u8>,
    
    /// Field outlining
    pub outlining: Option<u8>,
    
    /// Transparency
    pub transparency: Option<u8>,
}

impl ExtendedAttributes {
    /// Create new extended attributes with all fields set to None
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set highlighting attribute
    pub fn with_highlighting(mut self, highlighting: u8) -> Self {
        self.highlighting = Some(highlighting);
        self
    }
    
    /// Set foreground color
    pub fn with_foreground(mut self, color: u8) -> Self {
        self.foreground_color = Some(color);
        self
    }
    
    /// Set background color
    pub fn with_background(mut self, color: u8) -> Self {
        self.background_color = Some(color);
        self
    }
    
    /// Set character set
    pub fn with_charset(mut self, charset: u8) -> Self {
        self.charset = Some(charset);
        self
    }
    
    /// Set validation attribute
    pub fn with_validation(mut self, validation: u8) -> Self {
        self.validation = Some(validation);
        self
    }
    
    /// Parse extended attributes from SFE order data
    ///
    /// The SFE order format is:
    /// - Order code (0x29)
    /// - Number of attribute pairs
    /// - Attribute type/value pairs
    pub fn parse_from_sfe(data: &[u8]) -> Result<(Self, usize), String> {
        if data.is_empty() {
            return Err("Empty SFE data".to_string());
        }
        
        let num_pairs = data[0] as usize;
        let mut attrs = ExtendedAttributes::new();
        let mut pos = 1;
        
        for _ in 0..num_pairs {
            if pos + 1 >= data.len() {
                return Err("Incomplete attribute pair in SFE".to_string());
            }
            
            let attr_type = data[pos];
            let attr_value = data[pos + 1];
            pos += 2;
            
            match attr_type {
                XA_HIGHLIGHTING => attrs.highlighting = Some(attr_value),
                XA_FOREGROUND => attrs.foreground_color = Some(attr_value),
                XA_BACKGROUND => attrs.background_color = Some(attr_value),
                XA_CHARSET => attrs.charset = Some(attr_value),
                XA_VALIDATION => attrs.validation = Some(attr_value),
                XA_OUTLINING => attrs.outlining = Some(attr_value),
                XA_TRANSPARENCY => attrs.transparency = Some(attr_value),
                _ => {
                    // Unknown attribute type, skip it
                    eprintln!("Unknown extended attribute type: 0x{attr_type:02X}");
                }
            }
        }
        
        Ok((attrs, pos))
    }
}

/// Parse a base field attribute byte
///
/// Extracts the individual attribute bits from the base attribute byte
/// used in the SF (Start Field) order.
pub fn parse_base_attribute(attr_byte: u8) -> FieldAttributeInfo {
    FieldAttributeInfo {
        protected: (attr_byte & ATTR_PROTECTED) != 0,
        numeric: (attr_byte & ATTR_NUMERIC) != 0,
        display: attr_byte & ATTR_DISPLAY,
        modified: (attr_byte & ATTR_MDT) != 0,
        reserved: (attr_byte & ATTR_RESERVED) != 0,
    }
}

/// Parsed field attribute information
#[derive(Debug, Clone, PartialEq)]
pub struct FieldAttributeInfo {
    pub protected: bool,
    pub numeric: bool,
    pub display: u8,  // DISPLAY_NORMAL, DISPLAY_INTENSIFIED, or DISPLAY_HIDDEN
    pub modified: bool,
    pub reserved: bool,
}

impl FieldAttributeInfo {
    /// Check if field is hidden
    pub fn is_hidden(&self) -> bool {
        self.display == DISPLAY_HIDDEN
    }
    
    /// Check if field is intensified
    pub fn is_intensified(&self) -> bool {
        self.display == DISPLAY_INTENSIFIED
    }
    
    /// Check if field is normal display
    pub fn is_normal(&self) -> bool {
        self.display == DISPLAY_NORMAL
    }
}

/// Field Manager for tracking fields on the screen
///
/// Manages the collection of fields and provides methods for
/// finding and manipulating fields.
#[derive(Debug)]
pub struct FieldManager {
    fields: Vec<FieldAttribute>,
}

impl FieldManager {
    /// Create a new field manager
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
        }
    }
    
    /// Add a field to the manager
    pub fn add_field(&mut self, field: FieldAttribute) {
        self.fields.push(field);
        // Sort fields by address to maintain order
        self.fields.sort_by_key(|f| f.address);
    }
    
    /// Clear all fields
    pub fn clear(&mut self) {
        self.fields.clear();
    }
    
    /// Get all fields
    pub fn fields(&self) -> &[FieldAttribute] {
        &self.fields
    }
    
    /// Find field at or before a given buffer address
    pub fn find_field_at(&self, address: u16) -> Option<&FieldAttribute> {
        self.fields.iter()
            .rev()
            .find(|f| f.address <= address)
    }
    
    /// Find mutable field at or before a given buffer address
    pub fn find_field_at_mut(&mut self, address: u16) -> Option<&mut FieldAttribute> {
        self.fields.iter_mut()
            .rev()
            .find(|f| f.address <= address)
    }
    
    /// Get the next field after a given address
    pub fn next_field(&self, address: u16) -> Option<&FieldAttribute> {
        self.fields.iter()
            .find(|f| f.address > address)
    }
    
    /// Calculate field lengths based on next field positions
    /// Returns an error if any field has invalid boundaries
    pub fn calculate_field_lengths(&mut self, buffer_size: usize) -> Result<(), String> {
        let field_count = self.fields.len();
        
        for i in 0..field_count {
            let start_addr = self.fields[i].address as usize;
            
            // Validate start address is within buffer
            if start_addr >= buffer_size {
                return Err(format!(
                    "Field {} start address {} exceeds buffer size {}",
                    i, start_addr, buffer_size
                ));
            }
            
            let end_addr = if i + 1 < field_count {
                self.fields[i + 1].address as usize
            } else {
                buffer_size
            };
            
            // Validate end address
            if end_addr > buffer_size {
                return Err(format!(
                    "Field {} end address {} exceeds buffer size {}",
                    i, end_addr, buffer_size
                ));
            }
            
            // Calculate length with validation
            if end_addr < start_addr {
                return Err(format!(
                    "Field {} has invalid boundaries: start {} > end {}",
                    i, start_addr, end_addr
                ));
            }
            
            self.fields[i].length = end_addr - start_addr;
        }
        
        Ok(())
    }
    
    /// Get all modified fields (MDT bit set)
    pub fn modified_fields(&self) -> Vec<&FieldAttribute> {
        self.fields.iter()
            .filter(|f| f.is_modified())
            .collect()
    }
    
    /// Reset all MDT bits
    pub fn reset_mdt(&mut self) {
        for field in &mut self.fields {
            field.set_modified(false);
        }
    }
    
    /// Validate a field's content at a given address
    /// Returns Ok(()) if valid, Err with message if validation fails
    pub fn validate_field_at(&self, address: u16, content: &[u8]) -> Result<(), String> {
        if let Some(field) = self.find_field_at(address) {
            field.validate_content(content)
        } else {
            Ok(())  // No field at this address, no validation needed
        }
    }
}

impl Default for FieldManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_attribute_protected() {
        let attr = FieldAttribute::new(0, ATTR_PROTECTED);
        assert!(attr.is_protected());
        assert!(!attr.is_numeric());
    }

    #[test]
    fn test_field_attribute_numeric() {
        let attr = FieldAttribute::new(0, ATTR_NUMERIC);
        assert!(!attr.is_protected());
        assert!(attr.is_numeric());
    }

    #[test]
    fn test_field_attribute_hidden() {
        let attr = FieldAttribute::new(0, DISPLAY_HIDDEN);
        assert!(attr.is_hidden());
        assert!(!attr.is_intensified());
    }

    #[test]
    fn test_field_attribute_mdt() {
        let mut attr = FieldAttribute::new(0, 0);
        assert!(!attr.is_modified());
        
        attr.set_modified(true);
        assert!(attr.is_modified());
        
        attr.set_modified(false);
        assert!(!attr.is_modified());
    }

    #[test]
    fn test_parse_base_attribute() {
        let info = parse_base_attribute(ATTR_PROTECTED | ATTR_NUMERIC);
        assert!(info.protected);
        assert!(info.numeric);
        assert!(!info.modified);
    }

    #[test]
    fn test_extended_attributes_builder() {
        let attrs = ExtendedAttributes::new()
            .with_highlighting(HIGHLIGHT_BLINK)
            .with_foreground(COLOR_RED);
        
        assert_eq!(attrs.highlighting, Some(HIGHLIGHT_BLINK));
        assert_eq!(attrs.foreground_color, Some(COLOR_RED));
        assert_eq!(attrs.background_color, None);
    }

    #[test]
    fn test_field_manager() {
        let mut manager = FieldManager::new();
        
        manager.add_field(FieldAttribute::new(100, ATTR_PROTECTED));
        manager.add_field(FieldAttribute::new(200, ATTR_NUMERIC));
        
        assert_eq!(manager.fields().len(), 2);
        
        let field = manager.find_field_at(150);
        assert!(field.is_some());
        assert_eq!(field.unwrap().address, 100);
    }

    #[test]
    fn test_field_manager_calculate_lengths() {
        let mut manager = FieldManager::new();
        
        manager.add_field(FieldAttribute::new(0, 0));
        manager.add_field(FieldAttribute::new(100, 0));
        manager.add_field(FieldAttribute::new(200, 0));
        
        let result = manager.calculate_field_lengths(1920); // 24x80 buffer
        assert!(result.is_ok());
        
        assert_eq!(manager.fields()[0].length, 100);
        assert_eq!(manager.fields()[1].length, 100);
        assert_eq!(manager.fields()[2].length, 1720);
    }
    
    #[test]
    fn test_field_length_validation_errors() {
        let mut manager = FieldManager::new();
        
        // Add field with address beyond buffer size
        manager.add_field(FieldAttribute::new(2000, 0));
        
        let result = manager.calculate_field_lengths(1920);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds buffer size"));
    }
    
    #[test]
    fn test_field_validation_mandatory_fill() {
        let mut attr = FieldAttribute::new(0, 0);
        attr.extended_attrs.validation = Some(VALIDATION_MANDATORY_FILL);
        attr.length = 5;
        
        // Empty content should fail
        assert!(attr.validate_content(&[]).is_err());
        
        // Partial content should fail
        assert!(attr.validate_content(&[0xC1, 0xC2]).is_err());
        
        // Full content should pass
        assert!(attr.validate_content(&[0xC1, 0xC2, 0xC3, 0xC4, 0xC5]).is_ok());
    }
    
    #[test]
    fn test_field_validation_mandatory_entry() {
        let mut attr = FieldAttribute::new(0, 0);
        attr.extended_attrs.validation = Some(VALIDATION_MANDATORY_ENTRY);
        
        // Empty content should fail
        assert!(attr.validate_content(&[]).is_err());
        
        // Spaces only should fail
        assert!(attr.validate_content(&[0x40, 0x40]).is_err());
        
        // Any character should pass
        assert!(attr.validate_content(&[0xC1]).is_ok());
    }
    
    #[test]
    fn test_field_validation_numeric() {
        let attr = FieldAttribute::new(0, ATTR_NUMERIC);
        
        // Digits should pass
        assert!(attr.validate_content(&[0xF1, 0xF2, 0xF3]).is_ok());
        
        // Letters should fail
        assert!(attr.validate_content(&[0xC1, 0xC2]).is_err());
        
        // Mixed should fail
        assert!(attr.validate_content(&[0xF1, 0xC1]).is_err());
    }
}