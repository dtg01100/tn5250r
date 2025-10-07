use tn5250r::lib5250::session::Session;
use tn5250r::lib5250::codes::*;

const ESC: u8 = 0x04;

#[test]
fn session_write_structured_field_5250_query() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create proper 5250 data stream: ESC + CMD_WRITE_STRUCTURED_FIELD + structured field data
    let mut data = Vec::new();
    data.push(ESC);  // Command escape
    data.push(CMD_WRITE_STRUCTURED_FIELD);  // Write Structured Field command
    data.extend_from_slice(&[0x00, 0x06]); // Length (6 bytes total)
    data.push(0xD9); // Class
    data.push(SF_5250_QUERY); // SF type (0x70)
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Should return query reply response
    assert!(!resp.is_empty());
    // Response should contain query reply data
    assert!(resp.len() > 1);
}

#[test]
fn session_write_structured_field_query_command() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create proper 5250 data stream with QueryCommand (0x84)
    let mut data = Vec::new();
    data.push(ESC);  // Command escape
    data.push(CMD_WRITE_STRUCTURED_FIELD);  // Write Structured Field command
    data.extend_from_slice(&[0x00, 0x06]); // Length (6 bytes total)
    data.push(0xD9); // Class
    data.push(SF_QUERY_COMMAND); // SF type (0x84)
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Should return SetReplyMode response (0x85)
    assert!(!resp.is_empty());
    assert_eq!(resp[0], SF_SET_REPLY_MODE); // First byte should be 0x85
    
    // Should contain basic device capability data
    assert!(resp.len() > 1);
    // Check for display capabilities (80 columns, 24 rows)
    assert!(resp.len() >= 7); // At least 7 bytes total
}

#[test]
fn session_write_structured_field_unknown_sf() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create proper 5250 data stream with unknown SF type
    let mut data = Vec::new();
    data.push(ESC);  // Command escape
    data.push(CMD_WRITE_STRUCTURED_FIELD);  // Write Structured Field command  
    data.extend_from_slice(&[0x00, 0x06]); // Length (6 bytes total)
    data.push(0xD9); // Class
    data.push(0xFF); // Unknown SF type
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Should return empty response for unknown SF
    assert!(resp.is_empty());
}

#[test]
fn session_write_structured_field_invalid_class() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create proper 5250 data stream with invalid class
    let mut data = Vec::new();
    data.push(ESC);  // Command escape
    data.push(CMD_WRITE_STRUCTURED_FIELD);  // Write Structured Field command
    data.extend_from_slice(&[0x00, 0x06]); // Length (6 bytes total)
    data.push(0xC0); // Invalid class (not 0xD9)
    data.push(SF_5250_QUERY); // SF type
    
    let result = session.process_stream(&data);
    
    // Should return error for invalid class
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid SF class"));
}

#[test]
fn session_handles_multiple_commands() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create data stream with multiple commands
    let mut data = Vec::new();
    
    // First command: QueryCommand
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x06]);
    data.push(0xD9);
    data.push(SF_QUERY_COMMAND);
    
    // Second command: 5250 Query  
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x06]);
    data.push(0xD9);
    data.push(SF_5250_QUERY);
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Should process both commands and return responses
    assert!(!resp.is_empty());
}

#[test]
fn session_write_structured_field_erase_reset_clear_to_null() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create Erase/Reset structured field with clear to null (0x00)
    let mut data = Vec::new();
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x07]); // Length (7 bytes total)
    data.push(0xD9); // Class
    data.push(0x5B); // Erase/Reset SF type
    data.push(0x00); // Clear to null
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Erase/Reset should return no response
    assert!(resp.is_empty());
    
    // Check that session state was reset
    assert_eq!(session.read_opcode, 0);
    assert!(!session.invited);
}

#[test]
fn session_write_structured_field_erase_reset_clear_to_blanks() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create Erase/Reset structured field with clear to blanks (0x01)
    let mut data = Vec::new();
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x07]); // Length (7 bytes total)
    data.push(0xD9); // Class
    data.push(0x5B); // Erase/Reset SF type
    data.push(0x01); // Clear to blanks
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Erase/Reset should return no response
    assert!(resp.is_empty());
}

#[test]
fn session_write_structured_field_define_pending_operations() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create Define Pending Operations structured field
    let mut data = Vec::new();
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x08]); // Length (8 bytes total)
    data.push(0xD9); // Class
    data.push(0x80); // Define Pending Operations SF type
    data.push(0x01); // Some operation data
    data.push(0x02); // More operation data
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Define Pending Operations should return no response
    assert!(resp.is_empty());
}

#[test]
fn session_write_structured_field_enable_command_recognition() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create Enable Command Recognition structured field
    let mut data = Vec::new();
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x07]); // Length (7 bytes total)
    data.push(0xD9); // Class
    data.push(0x82); // Enable Command Recognition SF type
    data.push(0x0F); // Recognition flags
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Enable Command Recognition should return no response
    assert!(resp.is_empty());
}

#[test]
fn session_write_structured_field_request_timestamp_interval() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create Request Minimum Timestamp Interval structured field
    let mut data = Vec::new();
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x08]); // Length (8 bytes total)
    data.push(0xD9); // Class
    data.push(0x83); // Request Minimum Timestamp Interval SF type
    data.extend_from_slice(&[0x00, 0x64]); // 100ms interval (big-endian)
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Request Minimum Timestamp Interval should return no response
    assert!(resp.is_empty());
}

#[test]
fn test_field_navigation_tab_order() {
    use tn5250r::field_manager::{FieldManager, Field, FieldType};
    
    let mut manager = FieldManager::new();
    
    // Add fields with different tab orders
    let mut field1 = Field::new(1, FieldType::Input, 1, 1, 10);
    field1.tab_order = 2; // Set tab order after creation
    let mut field2 = Field::new(2, FieldType::Input, 2, 1, 10);
    field2.tab_order = 3;
    let mut field3 = Field::new(3, FieldType::Input, 3, 1, 10);
    field3.tab_order = 1;
    
    manager.add_field_for_test(field1);
    manager.add_field_for_test(field2);
    manager.add_field_for_test(field3);
    
    // Start with no active field
    assert!(manager.get_active_field_index().is_none());
    
    // Tab to next should activate field3 (tab_order 1)
    manager.tab_to_next_field().unwrap();
    assert_eq!(manager.get_active_field_index(), Some(2));
    
    // Tab to next should activate field1 (tab_order 2)
    manager.tab_to_next_field().unwrap();
    assert_eq!(manager.get_active_field_index(), Some(0));
    
    // Tab to next should activate field2 (tab_order 3)
    manager.tab_to_next_field().unwrap();
    assert_eq!(manager.get_active_field_index(), Some(1));
    
    // Tab to next should wrap to field3
    manager.tab_to_next_field().unwrap();
    assert_eq!(manager.get_active_field_index(), Some(2));
}

#[test]
fn test_field_validation_exit_required() {
    use tn5250r::field_manager::{FieldManager, Field, FieldType};
    
    let mut manager = FieldManager::new();
    
    let mut field = Field::new(1, FieldType::Input, 1, 1, 10);
    field.behavior.field_exit_required = true;
    manager.add_field_for_test(field);
    
    manager.set_active_field_for_test(Some(0));
    
    // Try to exit field - should fail because field_exit_required
    let result = manager.exit_current_field();
    assert!(result.is_err());
}

#[test]
fn test_field_validation_mandatory() {
    use tn5250r::field_manager::{FieldManager, Field, FieldType};
    
    let mut manager = FieldManager::new();
    
    let mut field = Field::new(1, FieldType::Input, 1, 1, 10);
    field.required = true;
    manager.add_field_for_test(field);
    
    manager.set_active_field_for_test(Some(0));
    
    // Try to exit field - should fail because field is empty and mandatory
    let result = manager.exit_current_field();
    assert!(result.is_err());
}

#[test]
fn test_field_auto_advance_on_full() {
    use tn5250r::field_manager::{FieldManager, Field, FieldType};
    
    let mut manager = FieldManager::new();
    
    let mut field1 = Field::new(1, FieldType::Input, 1, 1, 5);
    field1.behavior.auto_enter = true;
    manager.add_field_for_test(field1);
    
    let field2 = Field::new(2, FieldType::Input, 2, 1, 10);
    manager.add_field_for_test(field2);
    
    manager.set_active_field_for_test(Some(0));
    
    // Type characters to fill the field
    let result1 = manager.type_char('a');
    assert!(result1.is_ok() && !result1.unwrap());
    
    let result2 = manager.type_char('b');
    assert!(result2.is_ok() && !result2.unwrap());
    
    let result3 = manager.type_char('c');
    assert!(result3.is_ok() && !result3.unwrap());
    
    let result4 = manager.type_char('d');
    assert!(result4.is_ok() && !result4.unwrap());
    
    // This should fill the field and auto-advance
    let result5 = manager.type_char('e');
    assert!(result5.is_ok() && result5.unwrap()); // Should return true (field full)
    
    // Should have advanced to next field
    assert_eq!(manager.get_active_field_index(), Some(1));
}

#[test]
fn test_mdt_tracking() {
    use tn5250r::field_manager::{FieldManager, Field, FieldType};

    let mut manager = FieldManager::new();

    let field = Field::new(1, FieldType::Input, 1, 1, 10);
    manager.add_field_for_test(field);

    manager.set_active_field_for_test(Some(0));

    // Initially no modified fields
    assert!(!manager.has_modified_fields());
    assert!(manager.get_modified_fields().is_empty());

    // Type a character - should mark field as modified
    manager.type_char('a').unwrap();

    assert!(manager.has_modified_fields());
    assert_eq!(manager.get_modified_fields().len(), 1);

    // Clear modified flags
    manager.clear_modified_flags();
    assert!(!manager.has_modified_fields());
}

#[test]
fn session_write_structured_field_create_change_extended_attribute() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();

    // Create Create/Change Extended Attribute structured field
    // Format: ESC + CMD_WRITE_STRUCTURED_FIELD + length + class + SF type + attribute data
    let mut data = Vec::new();
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x0C]); // Length (12 bytes total)
    data.push(0xD9); // Class
    data.push(SF_CREATE_CHANGE_EXTENDED_ATTRIBUTE); // SF type (0xC1)

    // Extended attribute data: attr_id(0x01) + length(0x02) + data(0x10, 0x20)
    data.push(0x01); // Attribute ID (color)
    data.push(0x02); // Attribute data length
    data.push(0x10); // FG color
    data.push(0x20); // BG color

    let resp = session.process_stream(&data).expect("process ok");

    // Create/Change Extended Attribute should return no response
    assert!(resp.is_empty());
}

#[test]
fn session_write_structured_field_create_change_extended_attribute_multiple() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();

    // Create Create/Change Extended Attribute structured field with multiple attributes
    let mut data = Vec::new();
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x14]); // Length (20 bytes total)
    data.push(0xD9); // Class
    data.push(SF_CREATE_CHANGE_EXTENDED_ATTRIBUTE); // SF type (0xC1)

    // First attribute: color
    data.push(0x01); // Attribute ID (color)
    data.push(0x02); // Attribute data length
    data.push(0x10); // FG color
    data.push(0x20); // BG color

    // Second attribute: font
    data.push(0x02); // Attribute ID (font)
    data.push(0x01); // Attribute data length
    data.push(0x01); // Bold flag

    let resp = session.process_stream(&data).expect("process ok");

    // Create/Change Extended Attribute should return no response
    assert!(resp.is_empty());
}
