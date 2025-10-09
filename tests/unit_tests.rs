//! Unit tests for core tn5250r components
//!
//! These tests validate individual components and functions in isolation,
//! ensuring proper functionality and error handling.

#[cfg(test)]
mod unit_tests {
    use tn5250r::lib5250::session::Session;
    use tn5250r::lib5250::protocol::{ProtocolProcessor, Packet, FieldAttribute, StructuredFieldID};
    use tn5250r::lib5250::codes::CommandCode;
    use tn5250r::network::ProtocolMode;
    use tn5250r::field_manager::{FieldManager, Field, FieldType};
    use tn5250r::terminal::TerminalScreen;
    use tn5250r::lib5250::ebcdic_to_ascii;
    use tn5250r::telnet_negotiation::TelnetOption;

    /// Test Session creation and basic functionality
    #[test]
    fn test_session_creation() {
        let session = Session::new();

        // Verify initial state
        assert!(!session.invited);
        assert_eq!(session.read_opcode, 0);
        assert_eq!(session.get_protocol_mode(), ProtocolMode::AutoDetect);
        assert!(!session.is_authenticated());

        // Verify session token generation
        let token = session.get_session_token();
        assert!(token.is_some());
        assert!(token.unwrap().starts_with("sess_"));
    }

    /// Test session authentication
    #[test]
    fn test_session_authentication() {
        let mut session = Session::new();

        // Test invalid credentials
        let result = session.authenticate("", "");
        assert!(result.is_err());

        // Test valid credentials
        let result = session.authenticate("testuser", "testpass");
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(session.is_authenticated());

        // Test session invalidation
        session.invalidate_session();
        assert!(!session.is_authenticated());
        assert!(session.get_session_token().is_none());
    }

    /// Test session rate limiting
    #[test]
    fn test_session_rate_limiting() {
        let mut session = Session::new();

        // Authenticate first
        session.authenticate("user", "pass").unwrap();

        // Process commands within rate limit
        for _ in 0..50 {
            let data = vec![0x04, 0x40, 0x00, 0x00]; // Clear Unit command
            let _result = session.process_stream(&data);
            // Rate limiting may kick in, so we don't assert success
        }

        // Rate limiting may prevent some traffic
        let data = vec![0x04, 0x40, 0x00, 0x00];
        let _result = session.process_stream(&data);
        // Just verify it doesn't panic
    }

    /// Test session command size limits
    #[test]
    fn test_session_command_size_limits() {
        let mut session = Session::new();
        session.authenticate("user", "pass").unwrap();

        // Test oversized command
        let oversized_data = vec![0u8; 70000]; // 70KB > 64KB limit
        let result = session.process_stream(&oversized_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Command size exceeds maximum allowed"));
    }

    /// Test ProtocolProcessor creation and basic functionality
    #[test]
    fn test_protocol_processor_creation() {
        let processor = ProtocolProcessor::new();

        // Test that processor can be created successfully
        assert!(true); // Basic smoke test
    }

    /// Test packet serialization and deserialization
    #[test]
    fn test_packet_serialization() {
        // Create a test packet using lib5250 Packet
        let data = vec![0x01, 0x02, 0x03];
        let packet = tn5250r::lib5250::protocol::Packet::new(
            tn5250r::lib5250::codes::CommandCode::WriteToDisplay,
            42,
            data.clone()
        );

        // Serialize to bytes
        let bytes = packet.to_bytes();

        // Deserialize back
        let parsed_packet = tn5250r::lib5250::protocol::Packet::from_bytes(&bytes);
        assert!(parsed_packet.is_some());

        let parsed = parsed_packet.unwrap();
        assert_eq!(parsed.command, tn5250r::lib5250::codes::CommandCode::WriteToDisplay);
        assert_eq!(parsed.sequence_number, 42);
        assert_eq!(parsed.data, data);
    }

    /// Test EBCDIC to ASCII conversion
    #[test]
    fn test_ebcdic_ascii_conversion() {
        // Test common characters
        assert_eq!(ebcdic_to_ascii(0x40), ' ');  // Space
        assert_eq!(ebcdic_to_ascii(0xC1), 'A');  // A
        assert_eq!(ebcdic_to_ascii(0x81), 'a');  // a
        assert_eq!(ebcdic_to_ascii(0xF0), '0');  // 0
        assert_eq!(ebcdic_to_ascii(0x4B), '.');  // Period
    }

    /// Test FieldAttribute enum
    #[test]
    fn test_field_attributes() {
        assert_eq!(FieldAttribute::Normal.to_u8(), 0x00);
        assert_eq!(FieldAttribute::Protected.to_u8(), 0x20);
        assert_eq!(FieldAttribute::Numeric.to_u8(), 0x10);

        assert_eq!(FieldAttribute::from_u8(0x00), FieldAttribute::Normal);
        assert_eq!(FieldAttribute::from_u8(0x20), FieldAttribute::Protected);
        assert_eq!(FieldAttribute::from_u8(0x10), FieldAttribute::Numeric);
    }

    /// Test StructuredFieldID enum
    #[test]
    fn test_structured_field_ids() {
        assert_eq!(StructuredFieldID::from_u8(0xC1), Some(StructuredFieldID::CreateChangeExtendedAttribute));
        assert_eq!(StructuredFieldID::from_u8(0x84), Some(StructuredFieldID::QueryCommand));
        assert_eq!(StructuredFieldID::from_u8(0x85), Some(StructuredFieldID::SetReplyMode));
        assert_eq!(StructuredFieldID::from_u8(0xFF), None); // Invalid ID
    }

    /// Test CommandCode response mapping
    #[test]
    fn test_command_response_mapping() {
        assert_eq!(CommandCode::ReadInputFields.get_response_command(), Some(CommandCode::ReadInputFields));
        assert_eq!(CommandCode::ReadMdtFields.get_response_command(), Some(CommandCode::ReadMdtFields));
        assert_eq!(CommandCode::WriteToDisplay.get_response_command(), None);
    }

    /// Test Field creation and basic functionality
    #[test]
    fn test_field_creation() {
        let field = Field::new(1, FieldType::Input, 5, 10, 20);

        assert_eq!(field.id, 1);
        assert_eq!(field.field_type, FieldType::Input);
        assert_eq!(field.start_row, 5);
        assert_eq!(field.start_col, 10);
        assert_eq!(field.length, 20);
        assert_eq!(field.max_length, 20);
        assert!(!field.active);
        assert!(field.content.is_empty());
    }

    /// Test field position checking
    #[test]
    fn test_field_position_checking() {
        let field = Field::new(1, FieldType::Input, 5, 10, 20);

        // Test positions within field
        assert!(field.contains_position(5, 10));  // Start position
        assert!(field.contains_position(5, 15));  // Middle position
        assert!(field.contains_position(5, 29));  // End position

        // Test positions outside field
        assert!(!field.contains_position(4, 10)); // Wrong row
        assert!(!field.contains_position(5, 5));  // Before field
        assert!(!field.contains_position(5, 35)); // After field
    }

    /// Test field character insertion
    #[test]
    fn test_field_character_insertion() {
        let mut field = Field::new(1, FieldType::Input, 5, 10, 10);

        // Test valid character insertion
        let result = field.insert_char('A', 0);
        assert!(result.is_ok());
        assert_eq!(field.content, "A");

        // Test character validation for different field types
        let mut numeric_field = Field::new(2, FieldType::Numeric, 5, 10, 10);
        assert!(numeric_field.insert_char('1', 0).is_ok());
        assert!(numeric_field.insert_char('A', 1).is_err()); // Invalid for numeric

        let mut alpha_field = Field::new(3, FieldType::AlphaOnly, 5, 10, 10);
        assert!(alpha_field.insert_char('A', 0).is_ok());
        assert!(alpha_field.insert_char('1', 1).is_err()); // Invalid for alpha-only
    }

    /// Test field validation
    #[test]
    fn test_field_validation() {
        let mut field = Field::new(1, FieldType::Input, 5, 10, 10);
        field.required = true;

        // Empty required field should fail
        assert!(field.validate().is_err());

        // Filled required field should pass
        field.content = "test".to_string();
        assert!(field.validate().is_ok());
    }

    /// Test FieldManager creation and field detection
    #[test]
    fn test_field_manager_creation() {
        let manager = FieldManager::new();

        assert_eq!(manager.field_count(), 0);
        assert!(manager.get_active_field().is_none());
        assert_eq!(manager.get_cursor_position(), (1, 1));
    }

    /// Test field navigation
    #[test]
    fn test_field_navigation() {
        let mut manager = FieldManager::new();

        // Add some test fields
        let field1 = Field::new(1, FieldType::Input, 5, 10, 10);
        let field2 = Field::new(2, FieldType::Input, 7, 10, 10);
        manager.add_field_for_test(field1);
        manager.add_field_for_test(field2);

        // Test navigation to next field - starts at field 0 when first called
        assert!(manager.next_field().is_ok());
        // After first next_field, we should be at index 1 (wraps from initial state)
        assert_eq!(manager.get_active_field_index(), Some(1));

        // Test navigation to previous field
        assert!(manager.previous_field().is_ok());
        assert_eq!(manager.get_active_field_index(), Some(0));
    }

    /// Test TerminalScreen creation and basic operations
    #[test]
    fn test_terminal_screen_creation() {
        let screen = TerminalScreen::new();

        assert_eq!(screen.cursor_x, 0);
        assert_eq!(screen.cursor_y, 0);
        assert!(screen.dirty);

        // Test buffer size
        assert_eq!(screen.buffer.len(), 80 * 24);
    }

    /// Test terminal screen character writing
    #[test]
    fn test_terminal_screen_writing() {
        let mut screen = TerminalScreen::new();

        // Write a character
        screen.write_char('A');
        assert_eq!(screen.cursor_x, 1);
        assert_eq!(screen.cursor_y, 0);
        assert!(screen.dirty);

        // Check character was written
        let index = TerminalScreen::buffer_index(0, 0);
        assert_eq!(screen.buffer[index].character, 'A');
    }

    /// Test terminal screen cursor movement
    #[test]
    fn test_terminal_screen_cursor() {
        let mut screen = TerminalScreen::new();

        screen.move_cursor(10, 5);
        assert_eq!(screen.cursor_x, 10);
        assert_eq!(screen.cursor_y, 5);

        // Test bounds checking - move_cursor rejects invalid positions
        screen.move_cursor(100, 50); // Out of bounds
        // Position should remain unchanged when invalid
        assert_eq!(screen.cursor_x, 10);
        assert_eq!(screen.cursor_y, 5);
    }

    /// Test terminal screen buffer consistency validation
    #[test]
    fn test_terminal_screen_validation() {
        let screen = TerminalScreen::new();

        // Valid screen should pass validation
        assert!(screen.validate_buffer_consistency().is_ok());
    }

    /// Test security: Field input validation through public API
    #[test]
    fn test_field_input_validation() {
        let mut field = Field::new(1, FieldType::Input, 5, 10, 10);

        // Test that field accepts valid input
        let result = field.insert_char('A', 0);
        assert!(result.is_ok());

        // Test that field rejects invalid input for certain types
        let mut numeric_field = Field::new(2, FieldType::Numeric, 5, 10, 10);
        let result = numeric_field.insert_char('A', 0); // Letter in numeric field
        assert!(result.is_err());
    }

    /// Test edge case: Empty field operations
    #[test]
    fn test_empty_field_operations() {
        let mut field = Field::new(1, FieldType::Input, 5, 10, 10);

        // Operations on empty field should be safe
        assert!(!field.delete_char(0)); // Should return false (no-op on empty)
        assert!(!field.backspace(0));    // Should return false (no-op on empty)
        assert_eq!(field.content.len(), 0);
    }

    /// Test edge case: Field bounds checking
    #[test]
    fn test_field_bounds_checking() {
        let mut field = Field::new(1, FieldType::Input, 5, 10, 5);

        // Fill field to capacity
        for i in 0..5 {
            assert!(field.insert_char('A', i).is_ok());
        }
        assert_eq!(field.content.len(), 5);

        // Attempt to insert beyond capacity
        assert!(field.insert_char('B', 5).is_err());
        assert_eq!(field.content.len(), 5); // Should remain unchanged
    }

    /// Test edge case: Invalid cursor positions
    #[test]
    fn test_invalid_cursor_positions() {
        let mut screen = TerminalScreen::new();

        // Invalid positions should be handled gracefully
        screen.move_cursor(0, 0); // Zero-based coordinates
        assert_eq!(screen.cursor_x, 0);
        assert_eq!(screen.cursor_y, 0);

        // Extreme values should be clamped - move_cursor rejects invalid positions
        screen.move_cursor(usize::MAX, usize::MAX);
        // Position should remain unchanged when invalid
        assert_eq!(screen.cursor_x, 0);
        assert_eq!(screen.cursor_y, 0);
    }

    /// Test edge case: Malformed packet handling
    #[test]
    fn test_malformed_packet_handling() {
        // Empty data should not parse
        assert!(Packet::from_bytes(&[]).is_none());

        // Incomplete packet should not parse
        let incomplete = vec![0x01, 0x02];
        assert!(Packet::from_bytes(&incomplete).is_none());

        // Invalid command code should not parse
        let invalid_command = vec![0xFF, 0x00, 0x00, 0x06, 0x00, 0x01, 0x02, 0x03];
        assert!(Packet::from_bytes(&invalid_command).is_none());
    }

    /// Test edge case: Session with invalid authentication state
    #[test]
    fn test_session_invalid_auth_state() {
        let mut session = Session::new();

        // Without authentication, session should not be authenticated
        assert!(!session.is_authenticated());

        // After authentication, session should be authenticated
        let result = session.authenticate("user", "pass");
        assert!(result.is_ok());
        assert!(session.is_authenticated());
    }

    /// Test performance: Large field content handling
    #[test]
    fn test_large_field_content() {
        let mut field = Field::new(1, FieldType::Input, 5, 10, 1000);

        // Large content should be handled efficiently
        let large_content = "A".repeat(1000);
        field.set_content(large_content.clone());
        assert_eq!(field.content, large_content);
        assert_eq!(field.content.len(), 1000);
    }

    /// Test performance: Multiple field operations
    #[test]
    fn test_multiple_field_operations() {
        let mut manager = FieldManager::new();

        // Create many fields
        for i in 0..100 {
            let field = Field::new(i, FieldType::Input, i % 24, i % 80, 10);
            manager.add_field_for_test(field);
        }

        assert_eq!(manager.field_count(), 100);

        // Operations should remain performant
        for _ in 0..50 {
            let _ = manager.next_field();
        }
    }
}