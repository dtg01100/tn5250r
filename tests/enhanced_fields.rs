#[cfg(test)]
mod tests {
    use tn5250r::field_manager::*;
    
    #[test]
    fn test_enhanced_field_types() {
        // Test new field types
        assert_ne!(FieldType::AutoEnter, FieldType::Input);
        assert_ne!(FieldType::Mandatory, FieldType::Protected);
        assert_ne!(FieldType::DigitsOnly, FieldType::Numeric);
        assert_ne!(FieldType::AlphaOnly, FieldType::Input);
    }
    
    #[test]
    fn test_field_behavior_defaults() {
        let behavior = FieldBehavior::default();
        assert!(!behavior.auto_enter);
        assert!(!behavior.mandatory);
        assert!(!behavior.field_exit_required);
        assert!(!behavior.bypass);
        assert!(!behavior.right_adjust);
        assert!(!behavior.zero_fill);
        assert!(!behavior.uppercase_convert);
        assert!(!behavior.dup_enabled);
        assert!(behavior.cursor_progression.is_none());
    }
    
    #[test]
    fn test_field_error_types() {
        let error = FieldError::NumericOnly;
        assert_eq!(error.get_user_message(), "Numeric characters only");
        
        let error = FieldError::FieldFull;
        assert_eq!(error.get_user_message(), "Field is full");
        
        let error = FieldError::CursorProtected;
        assert_eq!(error.get_user_message(), "Cursor is in protected area");
    }
    
    #[test]
    fn test_character_validation() {
        let mut field = Field::new(1, FieldType::DigitsOnly, 1, 1, 10);
        
        // Valid digits should work
        assert!(field.validate_character('5').is_ok());
        assert!(field.validate_character('0').is_ok());
        
        // Invalid characters should fail
        assert_eq!(field.validate_character('a'), Err(FieldError::DigitsOnly));
        assert_eq!(field.validate_character(' '), Err(FieldError::DigitsOnly));
        
        // Test AlphaOnly field
        let mut alpha_field = Field::new(2, FieldType::AlphaOnly, 1, 1, 10);
        assert!(alpha_field.validate_character('a').is_ok());
        assert!(alpha_field.validate_character('Z').is_ok());
        assert!(alpha_field.validate_character(' ').is_ok());  // Space allowed
        assert_eq!(alpha_field.validate_character('5'), Err(FieldError::AlphaOnly));
    }
    
    #[test]
    fn test_enhanced_field_structure() {
        let mut field = Field::new(1, FieldType::Mandatory, 1, 1, 10);
        
        // Test default values
        assert_eq!(field.field_id, 1);
        assert!(field.next_field_id.is_none());
        assert!(field.prev_field_id.is_none());
        assert!(field.continued_group_id.is_none());
        assert!(!field.highlighted);
        assert!(field.error_state.is_none());
        assert!(!field.modified);
        assert_eq!(field.cursor_position, 0);
        
        // Test behavior modification
        let mut behavior = FieldBehavior::default();
        behavior.auto_enter = true;
        behavior.mandatory = true;
        field.set_behavior(behavior);
        
        assert!(field.behavior.auto_enter);
        assert!(field.behavior.mandatory);
    }
    
    #[test]
    fn test_field_manager_enhanced_navigation() {
        let mut manager = FieldManager::new();
        
        // Test empty field manager
        assert_eq!(manager.field_count(), 0);
        assert!(manager.get_active_field_index().is_none());
        assert!(manager.get_continued_groups().is_empty());
        assert!(manager.get_error_state().is_none());
        
        // Add some fields for testing
        let field1 = Field::new(1, FieldType::Input, 1, 1, 10);
        let field2 = Field::new(2, FieldType::Bypass, 1, 15, 10);
        let field3 = Field::new(3, FieldType::Input, 1, 30, 10);
        
        manager.add_field_for_test(field1);
        manager.add_field_for_test(field2);
        manager.add_field_for_test(field3);
        
        // Activate first field
        manager.set_active_field_for_test(Some(0));
        
        // Test navigation that should skip bypass field
        let result = manager.navigate_to_next_field();
        assert!(result.is_ok());
        assert_eq!(manager.get_active_field_index(), Some(2)); // Should skip bypass field
    }
    
    #[test]
    fn test_continued_field_groups() {
        let mut manager = FieldManager::new();
        
        // Add some fields
        let field1 = Field::new(1, FieldType::Input, 1, 1, 10);
        let field2 = Field::new(2, FieldType::Continued, 1, 15, 10);
        let field3 = Field::new(3, FieldType::Continued, 1, 30, 10);
        
        manager.add_field_for_test(field1);
        manager.add_field_for_test(field2);
        manager.add_field_for_test(field3);
        
        // Create a continued group with fields 1 and 2
        manager.add_field_to_continued_group(1, 100);
        manager.add_field_to_continued_group(2, 100);
        
        // Verify group was created
        assert!(manager.get_continued_groups().contains_key(&100));
        assert_eq!(manager.get_continued_groups()[&100].len(), 2);
        assert_eq!(manager.get_fields()[1].continued_group_id, Some(100));
        assert_eq!(manager.get_fields()[2].continued_group_id, Some(100));
    }
    
    #[test]
    fn test_insert_char_with_validation() {
        let mut field = Field::new(1, FieldType::DigitsOnly, 1, 1, 5);
        
        // Test valid character insertion
        let result = field.insert_char('5', 0);
        assert!(result.is_ok());
        assert_eq!(field.content, "5");
        assert!(field.modified);
        
        // Test invalid character insertion
        let result = field.insert_char('a', 1);
        assert!(result.is_err());
        assert_eq!(result, Err(FieldError::DigitsOnly));
        assert_eq!(field.content, "5"); // Content unchanged
        
        // Test field full error
        field.content = "12345".to_string(); // Fill to max
        let result = field.insert_char('6', 5);
        assert!(result.is_err());
        assert_eq!(result, Err(FieldError::FieldFull));
    }
}