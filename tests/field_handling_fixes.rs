//! Tests for field handling fixes
//!
//! This test suite verifies the fixes for:
//! 1. MDT (Modified Data Tag) tracking
//! 2. Program Tab navigation
//! 3. Field length calculation with validation
//! 4. Field validation attributes enforcement

use tn5250r::lib3270::{
    codes::*,
    display::Display3270,
    field::{FieldAttribute, FieldManager, ExtendedAttributes},
    protocol::ProtocolProcessor3270,
};

#[test]
fn test_mdt_set_on_field_modification() {
    let mut display = Display3270::new();
    
    // Create an unprotected field at address 0
    let field_attr = FieldAttribute::new(0, 0x00); // Unprotected, not modified
    display.set_field_attribute(0, field_attr);
    
    // Write to the field (simulate user input)
    display.set_cursor(1); // Position after field attribute
    display.write_char(0xC1); // EBCDIC 'A'
    
    // Check that MDT bit is set
    let field = display.field_manager().find_field_at(1);
    assert!(field.is_some());
    assert!(field.unwrap().is_modified(), "MDT should be set after writing to unprotected field");
}

#[test]
fn test_mdt_not_set_on_protected_field() {
    let mut display = Display3270::new();
    
    // Create a protected field at address 0
    let field_attr = FieldAttribute::new(0, ATTR_PROTECTED);
    display.set_field_attribute(0, field_attr);
    
    // Try to write to the field
    display.set_cursor(1);
    display.write_char(0xC1);
    
    // Check that MDT bit is NOT set (field is protected)
    let field = display.field_manager().find_field_at(1);
    assert!(field.is_some());
    assert!(!field.unwrap().is_modified(), "MDT should not be set for protected field");
}

#[test]
fn test_get_modified_fields_returns_correct_fields() {
    let mut display = Display3270::new();
    let processor = ProtocolProcessor3270::new();
    
    // Create two unprotected fields
    let mut field1 = FieldAttribute::new(0, 0x00);
    field1.length = 10;
    display.set_field_attribute(0, field1);
    
    let mut field2 = FieldAttribute::new(20, 0x00);
    field2.length = 10;
    display.set_field_attribute(20, field2);
    
    // Modify only the first field
    display.set_cursor(1);
    display.write_char(0xC1); // EBCDIC 'A'
    display.write_char(0xC2); // EBCDIC 'B'
    
    // Get modified fields
    let modified = processor.get_modified_fields(&display);
    
    // Should return only field1
    assert_eq!(modified.len(), 1, "Should return exactly one modified field");
    assert_eq!(modified[0].0, 0, "Modified field should be at address 0");
    assert!(modified[0].1.contains('A'), "Modified field should contain 'A'");
}

#[test]
fn test_reset_mdt_clears_all_modified_flags() {
    let mut display = Display3270::new();
    
    // Create and modify a field
    let field_attr = FieldAttribute::new(0, 0x00);
    display.set_field_attribute(0, field_attr);
    display.set_cursor(1);
    display.write_char(0xC1);
    
    // Verify MDT is set
    assert!(display.field_manager().find_field_at(1).unwrap().is_modified());
    
    // Reset MDT
    display.field_manager_mut().reset_mdt();
    
    // Verify MDT is cleared
    assert!(!display.field_manager().find_field_at(1).unwrap().is_modified());
}

#[test]
fn test_program_tab_navigates_to_next_unprotected_field() {
    let mut display = Display3270::new();
    
    // Create fields: protected at 0, unprotected at 100, protected at 200
    display.set_field_attribute(0, FieldAttribute::new(0, ATTR_PROTECTED));
    display.set_field_attribute(100, FieldAttribute::new(100, 0x00)); // Unprotected
    display.set_field_attribute(200, FieldAttribute::new(200, ATTR_PROTECTED));
    
    // Start at position 50 (in first protected field)
    display.set_cursor(50);
    
    // Tab to next unprotected field
    let success = display.tab_to_next_field();
    
    assert!(success, "Tab should find next unprotected field");
    assert_eq!(display.cursor_address(), 101, "Cursor should be at position after field attribute at 100");
}

#[test]
fn test_program_tab_wraps_around() {
    let mut display = Display3270::new();
    
    // Create unprotected field at beginning
    display.set_field_attribute(0, FieldAttribute::new(0, 0x00));
    
    // Create protected field later
    display.set_field_attribute(100, FieldAttribute::new(100, ATTR_PROTECTED));
    
    // Start near end of buffer
    display.set_cursor(1900);
    
    // Tab should wrap around to first unprotected field
    let success = display.tab_to_next_field();
    
    assert!(success, "Tab should wrap around to beginning");
    assert_eq!(display.cursor_address(), 1, "Cursor should be at first unprotected field");
}

#[test]
fn test_program_tab_no_unprotected_fields() {
    let mut display = Display3270::new();
    
    // Create only protected fields
    display.set_field_attribute(0, FieldAttribute::new(0, ATTR_PROTECTED));
    display.set_field_attribute(100, FieldAttribute::new(100, ATTR_PROTECTED));
    
    let original_pos = display.cursor_address();
    
    // Tab should fail
    let success = display.tab_to_next_field();
    
    assert!(!success, "Tab should fail when no unprotected fields exist");
    assert_eq!(display.cursor_address(), original_pos, "Cursor should not move");
}

#[test]
fn test_field_length_calculation_valid() {
    let mut manager = FieldManager::new();
    
    // Add three fields with valid addresses
    manager.add_field(FieldAttribute::new(0, 0));
    manager.add_field(FieldAttribute::new(100, 0));
    manager.add_field(FieldAttribute::new(200, 0));
    
    let result = manager.calculate_field_lengths(1920); // 24x80 buffer
    
    assert!(result.is_ok(), "Valid field boundaries should succeed");
    
    let fields = manager.fields();
    assert_eq!(fields[0].length, 100, "First field length should be 100");
    assert_eq!(fields[1].length, 100, "Second field length should be 100");
    assert_eq!(fields[2].length, 1720, "Last field length should be 1720");
}

#[test]
fn test_field_length_calculation_invalid_start_address() {
    let mut manager = FieldManager::new();
    
    // Add field with address beyond buffer size
    manager.add_field(FieldAttribute::new(2000, 0));
    
    let result = manager.calculate_field_lengths(1920);
    
    assert!(result.is_err(), "Field beyond buffer should fail");
    assert!(result.unwrap_err().contains("exceeds buffer size"));
}

#[test]
fn test_field_length_calculation_invalid_boundaries() {
    let mut manager = FieldManager::new();
    
    // This would create an invalid scenario if we had overlapping fields
    // The sorting by address prevents this, but we test the validation
    manager.add_field(FieldAttribute::new(0, 0));
    
    // Try with buffer size smaller than field address
    let result = manager.calculate_field_lengths(0);
    
    assert!(result.is_err(), "Zero buffer size should fail");
}

#[test]
fn test_field_validation_mandatory_fill() {
    let mut attr = FieldAttribute::new(0, 0);
    attr.extended_attrs.validation = Some(VALIDATION_MANDATORY_FILL);
    attr.length = 5;
    
    // Empty content should fail
    assert!(attr.validate_content(&[]).is_err(), "Empty content should fail mandatory fill");
    
    // Partial content should fail
    assert!(attr.validate_content(&[0xC1, 0xC2]).is_err(), "Partial content should fail mandatory fill");
    
    // Content with spaces should fail
    assert!(attr.validate_content(&[0xC1, 0x40, 0xC3, 0xC4, 0xC5]).is_err(), "Spaces should fail mandatory fill");
    
    // Full content should pass
    assert!(attr.validate_content(&[0xC1, 0xC2, 0xC3, 0xC4, 0xC5]).is_ok(), "Full content should pass mandatory fill");
}

#[test]
fn test_field_validation_mandatory_entry() {
    let mut attr = FieldAttribute::new(0, 0);
    attr.extended_attrs.validation = Some(VALIDATION_MANDATORY_ENTRY);
    attr.length = 10;
    
    // Empty content should fail
    assert!(attr.validate_content(&[]).is_err(), "Empty content should fail mandatory entry");
    
    // Only spaces should fail
    assert!(attr.validate_content(&[0x40, 0x40, 0x40]).is_err(), "Spaces only should fail mandatory entry");
    
    // Any real content should pass
    assert!(attr.validate_content(&[0xC1]).is_ok(), "Any character should pass mandatory entry");
    assert!(attr.validate_content(&[0x40, 0xC1, 0x40]).is_ok(), "Character with spaces should pass");
}

#[test]
fn test_field_validation_numeric() {
    let attr = FieldAttribute::new(0, ATTR_NUMERIC);
    
    // Digits should pass (EBCDIC digits 0xF0-0xF9)
    assert!(attr.validate_content(&[0xF1, 0xF2, 0xF3]).is_ok(), "Digits should pass numeric validation");
    
    // Spaces should pass (allowed in numeric fields)
    assert!(attr.validate_content(&[0xF1, 0x40, 0xF3]).is_ok(), "Spaces should pass numeric validation");
    
    // Letters should fail
    assert!(attr.validate_content(&[0xC1, 0xC2]).is_err(), "Letters should fail numeric validation");
    
    // Mixed content should fail
    assert!(attr.validate_content(&[0xF1, 0xC1]).is_err(), "Mixed content should fail numeric validation");
}

#[test]
fn test_field_validation_trigger() {
    let mut attr = FieldAttribute::new(0, 0);
    attr.extended_attrs.validation = Some(VALIDATION_TRIGGER);
    
    // Trigger fields are marked for special processing but don't have content validation
    assert!(attr.is_trigger(), "Field should be marked as trigger");
    
    // Any content should pass
    assert!(attr.validate_content(&[0xC1, 0xC2, 0xC3]).is_ok());
}

#[test]
fn test_field_validation_combined_attributes() {
    // Test numeric + mandatory entry
    let mut attr = FieldAttribute::new(0, ATTR_NUMERIC);
    attr.extended_attrs.validation = Some(VALIDATION_MANDATORY_ENTRY);
    attr.length = 5;
    
    // Empty should fail mandatory entry
    assert!(attr.validate_content(&[]).is_err());
    
    // Letters should fail numeric
    assert!(attr.validate_content(&[0xC1]).is_err());
    
    // Digits should pass both
    assert!(attr.validate_content(&[0xF1, 0xF2]).is_ok());
}

#[test]
fn test_read_modified_response_includes_modified_fields() {
    let mut display = Display3270::new();
    let processor = ProtocolProcessor3270::new();
    
    // Create fields and modify one
    let mut field1 = FieldAttribute::new(0, 0x00);
    field1.length = 10;
    display.set_field_attribute(0, field1);
    
    display.set_cursor(1);
    display.write_char(0xC1); // EBCDIC 'A'
    display.write_char(0xC2); // EBCDIC 'B'
    
    // Create Read Modified response
    let response = processor.create_read_modified_response(&display, AidKey::Enter);
    
    // Response should include: AID (1 byte) + cursor address (2 bytes) + field data
    assert!(response.len() > 3, "Response should include modified field data");
    assert_eq!(response[0], AID_ENTER, "First byte should be AID");
}

#[test]
fn test_field_manager_validation() {
    let mut manager = FieldManager::new();
    
    // Add field with validation attributes
    let mut field = FieldAttribute::new(0, ATTR_NUMERIC);
    field.extended_attrs.validation = Some(VALIDATION_MANDATORY_ENTRY);
    field.length = 5;
    manager.add_field(field);
    
    // Valid numeric content
    let result = manager.validate_field_at(1, &[0xF1, 0xF2, 0xF3]);
    assert!(result.is_ok(), "Valid numeric content should pass");
    
    // Invalid non-numeric content
    let result = manager.validate_field_at(1, &[0xC1, 0xC2]);
    assert!(result.is_err(), "Non-numeric content should fail");
}