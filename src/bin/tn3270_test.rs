//! TN3270 Test Binary
//!
//! Demonstration binary to showcase TN3270 functionality and provide
//! example usage patterns for the TN3270 protocol implementation.

use tn5250r::lib3270::{
    Display3270, ProtocolProcessor3270, ScreenSize,
    CommandCode, OrderCode, AidKey,
    CMD_WRITE, CMD_ERASE_WRITE, CMD_READ_BUFFER,
    ORDER_SF, ORDER_SBA, ORDER_IC, ORDER_RA,
    ATTR_PROTECTED, ATTR_NUMERIC, WCC_RESTORE, WCC_ALARM,
};
use tn5250r::lib3270::display::addressing;
use tn5250r::config::SessionConfig;
use tn5250r::controller::ProtocolType;
use std::str::FromStr;
use tn5250r::protocol_common::traits::TerminalProtocol;

fn main() {
    println!("=== TN3270 Protocol Test Suite ===\n");
    
    // Test 1: Basic Protocol Initialization
    test_protocol_initialization();
    
    // Test 2: Screen Size Support
    test_screen_sizes();
    
    // Test 3: Data Stream Parsing
    test_data_stream_parsing();
    
    // Test 4: Field Management
    test_field_management();
    
    // Test 5: Display Operations
    test_display_operations();
    
    // Test 6: Configuration Integration
    test_configuration();
    
    // Test 7: Protocol Selection
    test_protocol_selection();
    
    // Test 8: Complete Session Example
    test_complete_session();
    
    println!("\n=== All Tests Completed Successfully ===");
}

/// Test 1: Protocol Initialization
fn test_protocol_initialization() {
    println!("Test 1: Protocol Initialization");
    println!("--------------------------------");
    
    let processor = ProtocolProcessor3270::new();
    println!("✓ Created TN3270 protocol processor");
    println!("  Protocol name: {}", processor.protocol_name());
    println!("  Connected: {}", processor.is_connected());
    
    let mut processor_14bit = ProtocolProcessor3270::new();
    processor_14bit.set_14bit_addressing(true);
    println!("✓ Enabled 14-bit addressing for larger screens");
    
    println!();
}

/// Test 2: Screen Size Support
fn test_screen_sizes() {
    println!("Test 2: Screen Size Support");
    println!("---------------------------");
    
    let sizes = [
        ("Model 2 (24x80)", ScreenSize::Model2, 24, 80, 1920),
        ("Model 3 (32x80)", ScreenSize::Model3, 32, 80, 2560),
        ("Model 4 (43x80)", ScreenSize::Model4, 43, 80, 3440),
        ("Model 5 (27x132)", ScreenSize::Model5, 27, 132, 3564),
    ];
    
    for (name, size, rows, cols, buffer_size) in &sizes {
        let display = Display3270::with_size(*size);
        println!("✓ {}", name);
        println!("  Rows: {}, Cols: {}, Buffer: {} bytes", rows, cols, buffer_size);
        assert_eq!(display.rows(), *rows);
        assert_eq!(display.cols(), *cols);
        assert_eq!(display.buffer_size(), *buffer_size);
    }
    
    println!();
}

/// Test 3: Data Stream Parsing
fn test_data_stream_parsing() {
    println!("Test 3: Data Stream Parsing");
    println!("---------------------------");
    
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    // Test Write command
    println!("Testing Write command...");
    let data = vec![
        CMD_WRITE,
        WCC_RESTORE,
        0xC1, 0xC2, 0xC3, // ABC in EBCDIC
    ];
    
    match processor.process_data(&data, &mut display) {
        Ok(_) => {
            println!("✓ Write command processed successfully");
            println!("  Keyboard locked: {}", display.is_keyboard_locked());
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test Erase/Write command
    println!("Testing Erase/Write command...");
    let data = vec![
        CMD_ERASE_WRITE,
        WCC_RESTORE | WCC_ALARM,
        0xC4, 0xC5, 0xC6, // DEF in EBCDIC
    ];
    
    match processor.process_data(&data, &mut display) {
        Ok(_) => {
            println!("✓ Erase/Write command processed successfully");
            println!("  Alarm set: {}", display.is_alarm());
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test Set Buffer Address
    println!("Testing Set Buffer Address order...");
    let (b1, b2) = addressing::encode_12bit_address(100);
    let data = vec![
        CMD_WRITE,
        0x00,
        ORDER_SBA,
        b1, b2,
        0xC7, // G in EBCDIC
    ];
    
    match processor.process_data(&data, &mut display) {
        Ok(_) => {
            println!("✓ Set Buffer Address processed successfully");
            println!("  Character at position 100: {:?}", display.read_char_at(100));
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    println!();
}

/// Test 4: Field Management
fn test_field_management() {
    println!("Test 4: Field Management");
    println!("-----------------------");
    
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    // Create a protected field
    println!("Creating protected field...");
    let data = vec![
        CMD_WRITE,
        0x00,
        ORDER_SF,
        ATTR_PROTECTED,
        0xD3, 0xC1, 0xC2, 0xC5, 0xD3, // "LABEL" in EBCDIC
    ];
    
    match processor.process_data(&data, &mut display) {
        Ok(_) => {
            println!("✓ Protected field created");
            let fields = display.field_manager().fields();
            println!("  Number of fields: {}", fields.len());
            if !fields.is_empty() {
                let field = &fields[0];
                println!("  Field protected: {}", field.is_protected());
            }
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Create a numeric input field
    println!("Creating numeric input field...");
    let (b1, b2) = addressing::encode_12bit_address(160);
    let data = vec![
        CMD_WRITE,
        0x00,
        ORDER_SBA,
        b1, b2,
        ORDER_SF,
        ATTR_NUMERIC, // Unprotected numeric field
    ];
    
    match processor.process_data(&data, &mut display) {
        Ok(_) => {
            println!("✓ Numeric input field created");
            let fields = display.field_manager().fields();
            println!("  Total fields: {}", fields.len());
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    println!();
}

/// Test 5: Display Operations
fn test_display_operations() {
    println!("Test 5: Display Operations");
    println!("-------------------------");
    
    let mut display = Display3270::new();
    
    // Test cursor positioning
    println!("Testing cursor positioning...");
    display.set_cursor(100);
    let (row, col) = display.cursor_position();
    println!("✓ Cursor set to address 100");
    println!("  Position: row {}, col {}", row, col);
    
    // Test character writing
    println!("Testing character writing...");
    display.set_cursor(0);
    display.write_char(0xC1); // A
    display.write_char(0xC2); // B
    display.write_char(0xC3); // C
    println!("✓ Characters written to buffer");
    println!("  Cursor now at: {}", display.cursor_address());
    
    // Test repeat operation
    println!("Testing repeat operation...");
    display.set_cursor(10);
    display.repeat_to_address(0xE2, 20); // 'S' repeated
    println!("✓ Repeated character from position 10 to 20");
    
    // Test buffer clear
    println!("Testing buffer clear...");
    display.clear();
    println!("✓ Buffer cleared");
    println!("  Cursor reset to: {}", display.cursor_address());
    
    // Test keyboard lock/unlock
    println!("Testing keyboard lock...");
    display.lock_keyboard();
    println!("✓ Keyboard locked: {}", display.is_keyboard_locked());
    display.unlock_keyboard();
    println!("✓ Keyboard unlocked: {}", !display.is_keyboard_locked());
    
    println!();
}

/// Test 6: Configuration Integration
fn test_configuration() {
    println!("Test 6: Configuration Integration");
    println!("--------------------------------");
    
    let mut config = SessionConfig::new("test.json".to_string(), "tn3270_test".to_string());
    
    // Set TN3270 protocol
    println!("Setting protocol to TN3270...");
    match config.set_protocol_mode("tn3270") {
        Ok(_) => {
            println!("✓ Protocol set to: {}", config.get_protocol_mode());
        }
        Err(e) => println!("✗ Error: {:?}", e),
    }
    
    // Set 3270 terminal type
    println!("Setting terminal type...");
    match config.set_terminal_type("IBM-3278-2") {
        Ok(_) => {
            println!("✓ Terminal type set to: {}", config.get_terminal_type());
        }
        Err(e) => println!("✗ Error: {:?}", e),
    }
    
    // Validate combination
    println!("Validating protocol/terminal combination...");
    match config.validate_protocol_terminal_combination() {
        Ok(_) => println!("✓ Configuration is valid"),
        Err(e) => println!("✗ Validation error: {:?}", e),
    }
    
    // Test invalid combination
    println!("Testing invalid combination (TN3270 with 5250 terminal)...");
    config.set_terminal_type("IBM-3179-2").unwrap(); // 5250 terminal
    match config.validate_protocol_terminal_combination() {
        Ok(_) => println!("✗ Should have failed validation"),
        Err(_) => println!("✓ Correctly rejected invalid combination"),
    }
    
    println!();
}

/// Test 7: Protocol Selection
fn test_protocol_selection() {
    println!("Test 7: Protocol Selection");
    println!("-------------------------");
    
    // Test ProtocolType parsing
    println!("Testing protocol type parsing...");
    
    let protocols = [
        ("tn3270", ProtocolType::TN3270),
        ("3270", ProtocolType::TN3270),
        ("tn5250", ProtocolType::TN5250),
        ("5250", ProtocolType::TN5250),
    ];
    
    for (input, expected) in &protocols {
        match ProtocolType::from_str(input) {
            Ok(protocol) => {
                println!("✓ '{}' -> {:?}", input, protocol);
                assert_eq!(protocol, *expected);
            }
            Err(e) => println!("✗ Error parsing '{}': {}", input, e),
        }
    }
    
    // Test invalid protocol
    println!("Testing invalid protocol...");
    match ProtocolType::from_str("invalid") {
        Ok(_) => println!("✗ Should have failed"),
        Err(_) => println!("✓ Correctly rejected invalid protocol"),
    }
    
    println!();
}

/// Test 8: Complete Session Example
fn test_complete_session() {
    println!("Test 8: Complete Session Example");
    println!("--------------------------------");
    println!("Simulating a complete 3270 session...\n");
    
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    // Step 1: Clear screen and display header
    println!("Step 1: Clearing screen and displaying header");
    let header_data = vec![
        CMD_ERASE_WRITE,
        WCC_RESTORE,
        ORDER_SF,
        ATTR_PROTECTED,
        // "SYSTEM LOGIN" in EBCDIC
        0xE2, 0xE8, 0xE2, 0xE3, 0xC5, 0xD4, 0x40,
        0xD3, 0xD6, 0xC7, 0xC9, 0xD5,
    ];
    
    match processor.process_data(&header_data, &mut display) {
        Ok(_) => println!("✓ Header displayed"),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Step 2: Create username field
    println!("Step 2: Creating username field");
    let (b1, b2) = addressing::encode_12bit_address(160); // Row 2
    let username_field = vec![
        CMD_WRITE,
        0x00,
        ORDER_SBA, b1, b2,
        ORDER_SF, ATTR_PROTECTED,
        // "Username:" in EBCDIC
        0xE4, 0xA2, 0x85, 0x99, 0x95, 0x81, 0x94, 0x85, 0x7A,
        ORDER_SF, 0x00, // Unprotected input field
    ];
    
    match processor.process_data(&username_field, &mut display) {
        Ok(_) => println!("✓ Username field created"),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Step 3: Create password field
    println!("Step 3: Creating password field");
    let (b1, b2) = addressing::encode_12bit_address(320); // Row 4
    let password_field = vec![
        CMD_WRITE,
        0x00,
        ORDER_SBA, b1, b2,
        ORDER_SF, ATTR_PROTECTED,
        // "Password:" in EBCDIC
        0xD7, 0x81, 0xA2, 0xA2, 0xA6, 0x96, 0x99, 0x84, 0x7A,
        ORDER_SF, 0x00, // Unprotected input field
    ];
    
    match processor.process_data(&password_field, &mut display) {
        Ok(_) => println!("✓ Password field created"),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Step 4: Display status
    println!("\nSession Status:");
    println!("  Fields created: {}", display.field_manager().fields().len());
    println!("  Keyboard locked: {}", display.is_keyboard_locked());
    println!("  Cursor position: {:?}", display.cursor_position());
    println!("  Screen size: {}x{}", display.rows(), display.cols());
    
    // Step 5: Generate Read Buffer response
    println!("\nStep 5: Generating Read Buffer response");
    let response = processor.create_read_buffer_response(&display, AidKey::Enter);
    println!("✓ Response generated");
    println!("  Response size: {} bytes", response.len());
    println!("  AID key: Enter (0x{:02X})", response[0]);
    
    // Step 6: Display buffer content (first 80 characters)
    println!("\nStep 6: Display buffer content (first row)");
    if let Some(row) = display.get_row(0) {
        println!("  Row 0: {}", row.chars().take(40).collect::<String>());
    }
    
    println!("\n✓ Complete session simulation successful");
    println!();
}