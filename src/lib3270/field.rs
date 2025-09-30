/// Field attribute and management logic for 3270
///
/// This module handles 3270 field attributes, including both basic field
/// attributes (from SF order) and extended field attributes (from SFE order).

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
}

/// Extended Field Attributes
///
/// 3270 supports extended attributes via the SFE (Start Field Extended) order.
/// These provide additional formatting capabilities beyond the base attribute.
#[derive(Debug, Clone, PartialEq)]
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

impl Default for ExtendedAttributes {
    fn default() -> Self {
        Self {
            highlighting: None,
            foreground_color: None,
            background_color: None,
            charset: None,
            validation: None,
            outlining: None,
            transparency: None,
        }
    }
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
                    eprintln!("Unknown extended attribute type: 0x{:02X}", attr_type);
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
    pub fn calculate_field_lengths(&mut self, buffer_size: usize) {
        let field_count = self.fields.len();
        for i in 0..field_count {
            let start_addr = self.fields[i].address as usize;
            let end_addr = if i + 1 < field_count {
                self.fields[i + 1].address as usize
            } else {
                buffer_size
            };
            self.fields[i].length = end_addr.saturating_sub(start_addr);
        }
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
        
        manager.calculate_field_lengths(1920); // 24x80 buffer
        
        assert_eq!(manager.fields()[0].length, 100);
        assert_eq!(manager.fields()[1].length, 100);
        assert_eq!(manager.fields()[2].length, 1720);
    }
}