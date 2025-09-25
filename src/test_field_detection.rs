use crate::field_manager::{FieldManager, FieldType as FMFieldType};
use crate::terminal::TerminalScreen;
use crate::protocol_state::{ProtocolStateMachine, Field, FieldType as PSFieldType};

/// Test the field detection with sample AS/400 login screen content
pub fn test_field_detection() {
    let mut field_manager = FieldManager::new();
    let mut terminal = TerminalScreen::new();
    
    // Simulate typical AS/400 login screen content
    let login_screen_lines = vec![
        "                                 Sign On                                     ",
        "                                                                            ",
        "System  . . . . . . . . . . . : ________________                          ",
        "User ID . . . . . . . . . . . : __________                                ",
        "Password  . . . . . . . . . . : __________                                ",
        "                                                                            ",
        "Program/procedure . . . . . . : __________                                ",
        "Menu  . . . . . . . . . . . . : __________                                ",
        "Current library . . . . . . . : __________                                ",
        "                                                                            ",
        "                                                                            ",
        "(C) COPYRIGHT IBM CORP. 1980, 2013.                                        ",
    ];
    
    println!("Testing field detection with AS/400 login screen:");
    println!("================================================");
    
    // Set up the terminal screen with the sample content
    for (row, line) in login_screen_lines.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if row < terminal.buffer.len() && col < terminal.buffer[row].len() {
                terminal.buffer[row][col].character = ch;
            }
        }
    }
    
    // Test field detection
    field_manager.detect_fields(&terminal);
    
    println!("\nDetected fields:");
    println!("===============");
    for field in field_manager.get_fields() {
        println!("Field ID: {}, Type: {:?}, Position: ({}, {}), Length: {}", 
                field.id, field.field_type, field.start_row, field.start_col, field.length);
        if let Some(ref label) = field.label {
            println!("  Label: '{}'", label);
        }
    }
    
    if field_manager.field_count() == 0 {
        println!("No fields detected. This suggests the field detection needs improvement.");
    } else {
        println!("\nField detection working! Found {} fields.", field_manager.field_count());
    }
    
    // Test Tab navigation
    println!("\nTesting Tab navigation:");
    println!("======================");
    if field_manager.field_count() > 0 {
        let initial_active = field_manager.get_active_field_index();
        println!("Initial active field: {:?}", initial_active);
        
        field_manager.next_field();
        let after_tab = field_manager.get_active_field_index();
        println!("After Tab: {:?}", after_tab);
        
        if initial_active != after_tab {
            println!("Tab navigation working!");
        } else {
            println!("Tab navigation may not be working properly.");
        }
    }

    // Test the protocol-based field detection approach
    test_protocol_field_detection();
}

/// Test the protocol-based field detection approach
pub fn test_protocol_field_detection() {
    println!("\n\nTesting Protocol-Based Field Detection:");
    println!("======================================");
    
    let mut protocol_state = ProtocolStateMachine::new();
    
    // Directly test the field management capabilities without requiring network connection
    // This simulates what would happen when processing actual 5250 protocol data
    
    // Create fields that match our test scenario (AS/400 login screen)
    let test_fields = vec![
        Field::new(3, 32, 16, PSFieldType::Input),     // System field
        Field::new(4, 32, 10, PSFieldType::Input),     // User ID field  
        Field::new(5, 32, 10, PSFieldType::Password),  // Password field
        Field::new(7, 32, 10, PSFieldType::Input),     // Program/procedure field
        Field::new(8, 32, 10, PSFieldType::Input),     // Menu field
        Field::new(9, 32, 10, PSFieldType::Input),     // Current library field
    ];
    
    // Add each field and test duplicate prevention
    println!("Adding fields and testing duplicate prevention:");
    for (i, field) in test_fields.iter().enumerate() {
        println!("  Adding field {} at ({}, {})", i + 1, field.start_row(), field.start_col());
        protocol_state.add_field_object(field.clone());
    }
    
    // Try to add a duplicate field to test prevention
    let duplicate_field = Field::new(4, 32, 10, PSFieldType::Input); // Same as User ID field
    println!("  Attempting to add duplicate field at (4, 32)...");
    protocol_state.add_field_object(duplicate_field);
    
    let fields = protocol_state.get_fields();
    println!("\nProtocol-based field detection found {} fields:", fields.len());
    
    for (i, field) in fields.iter().enumerate() {
        println!("  Field {}: Position ({}, {}) -> {}, Length: {}, Type: {:?}", 
                 i + 1, 
                 field.start_row(), 
                 field.start_col(),
                 field.end_position(),
                 field.length, 
                 field.field_type);
    }
    
    // Test field boundary methods
    if !fields.is_empty() {
        let test_field = &fields[1]; // User ID field
        println!("\nTesting field boundary methods on User ID field:");
        println!("  Start: ({}, {})", test_field.start_row(), test_field.start_col());
        println!("  End position: {}", test_field.end_position());
        
        // Convert row,col to position for within_field method
        let pos_35 = 4 * 80 + 35; // Row 4, Col 35
        let pos_45 = 4 * 80 + 45; // Row 4, Col 45
        println!("  Position (4, 35) [{}] within field: {}", pos_35, test_field.within_field(pos_35));
        println!("  Position (4, 45) [{}] within field: {}", pos_45, test_field.within_field(pos_45));
    }
    
    // Verify no duplicate fields were created
    if fields.len() == 6 {
        println!("✅ No duplicate fields detected! Expected 6, got {}", fields.len());
    } else {
        println!("❌ Field count mismatch! Expected 6, got {}", fields.len());
    }
    
    println!("✅ Protocol-based field detection test completed!");
}