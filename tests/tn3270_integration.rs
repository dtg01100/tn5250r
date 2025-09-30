//! TN3270 Integration Tests
//!
//! Comprehensive test suite to validate TN3270 protocol implementation,
//! UI component integration, and protocol switching functionality.

use tn5250r::lib3270::{
    Display3270, ProtocolProcessor3270, ScreenSize,
    CommandCode, OrderCode, AidKey,
    CMD_WRITE, CMD_ERASE_WRITE, CMD_READ_BUFFER, CMD_READ_MODIFIED,
    ORDER_SF, ORDER_SBA, ORDER_IC, ORDER_PT, ORDER_RA,
    ATTR_PROTECTED, ATTR_NUMERIC, WCC_RESTORE, WCC_ALARM,
};
use tn5250r::config::{SessionConfig, parse_protocol_string, protocol_mode_to_string};
use tn5250r::controller::ProtocolType;
use tn5250r::network::ProtocolMode;
use tn5250r::protocol_common::traits::TerminalProtocol;

/// Test TN3270 protocol processor initialization
#[test]
fn test_protocol_processor_initialization() {
    let processor = ProtocolProcessor3270::new();
    assert_eq!(processor.protocol_name(), "TN3270");
    assert!(processor.is_connected());
}

/// Test protocol processor with 14-bit addressing
#[test]
fn test_14bit_addressing_mode() {
    let mut processor = ProtocolProcessor3270::new();
    processor.set_14bit_addressing(true);
    
    let mut display = Display3270::with_size(ScreenSize::Model4);
    
    // Test with larger screen that requires 14-bit addressing
    let data = vec![
        CMD_WRITE,
        0x00, // WCC
    ];
    
    assert!(processor.process_data(&data, &mut display).is_ok());
}

/// Test protocol mode switching (TN5250 ↔ TN3270)
#[test]
fn test_protocol_mode_switching() {
    // Test ProtocolType conversion
    let tn3270 = ProtocolType::TN3270;
    assert_eq!(tn3270.to_str(), "tn3270");
    assert_eq!(tn3270.to_protocol_mode(), ProtocolMode::TN3270);
    
    let tn5250 = ProtocolType::TN5250;
    assert_eq!(tn5250.to_str(), "tn5250");
    assert_eq!(tn5250.to_protocol_mode(), ProtocolMode::TN5250);
    
    // Test parsing from string
    assert_eq!(ProtocolType::from_str("tn3270").unwrap(), ProtocolType::TN3270);
    assert_eq!(ProtocolType::from_str("3270").unwrap(), ProtocolType::TN3270);
    assert_eq!(ProtocolType::from_str("tn5250").unwrap(), ProtocolType::TN5250);
    assert_eq!(ProtocolType::from_str("5250").unwrap(), ProtocolType::TN5250);
    
    // Test invalid protocol
    assert!(ProtocolType::from_str("invalid").is_err());
}

/// Test configuration loading and validation for TN3270
#[test]
fn test_configuration_loading() {
    let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
    
    // Test setting TN3270 protocol
    assert!(config.set_protocol_mode("tn3270").is_ok());
    assert_eq!(config.get_protocol_mode(), "tn3270");
    
    // Test setting valid 3270 terminal type
    assert!(config.set_terminal_type("IBM-3278-2").is_ok());
    assert_eq!(config.get_terminal_type(), "IBM-3278-2");
    
    // Validate protocol/terminal combination
    assert!(config.validate_protocol_terminal_combination().is_ok());
}

/// Test configuration validation with invalid combinations
#[test]
fn test_configuration_validation_errors() {
    let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
    
    // Set TN3270 protocol with 5250 terminal type (should fail)
    config.set_protocol_mode("tn3270").unwrap();
    config.set_terminal_type("IBM-3179-2").unwrap(); // 5250 terminal
    assert!(config.validate_protocol_terminal_combination().is_err());
    
    // Set TN5250 protocol with 3270 terminal type (should fail)
    config.set_protocol_mode("tn5250").unwrap();
    config.set_terminal_type("IBM-3278-2").unwrap(); // 3270 terminal
    assert!(config.validate_protocol_terminal_combination().is_err());
    
    // Auto mode should accept any terminal type
    config.set_protocol_mode("auto").unwrap();
    config.set_terminal_type("IBM-3278-2").unwrap();
    assert!(config.validate_protocol_terminal_combination().is_ok());
    config.set_terminal_type("IBM-3179-2").unwrap();
    assert!(config.validate_protocol_terminal_combination().is_ok());
}

/// Test protocol detection with 3270 data streams
#[test]
fn test_protocol_detection_3270() {
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    // Create a typical 3270 data stream
    let data = vec![
        CMD_WRITE,
        WCC_RESTORE,
        ORDER_SF,
        ATTR_PROTECTED,
        0xC1, 0xC2, 0xC3, // ABC in EBCDIC
    ];
    
    assert!(processor.process_data(&data, &mut display).is_ok());
    assert!(!display.is_keyboard_locked());
}

/// Test error handling for invalid configurations
#[test]
fn test_invalid_configuration_handling() {
    let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
    
    // Test invalid protocol mode
    assert!(config.set_protocol_mode("invalid_protocol").is_err());
    
    // Test invalid terminal type
    assert!(config.set_terminal_type("INVALID-TERMINAL").is_err());
    
    // Test parse_protocol_string with invalid input
    assert!(parse_protocol_string("invalid").is_err());
}

/// Test terminal type validation for 3270
#[test]
fn test_terminal_type_validation() {
    let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
    
    // Valid 3270 terminal types
    let valid_3270_types = [
        "IBM-3278-2", "IBM-3279-2", "IBM-3279-3",
        "IBM-3278-3", "IBM-3278-4", "IBM-3278-5"
    ];
    
    for terminal_type in &valid_3270_types {
        assert!(config.set_terminal_type(terminal_type).is_ok());
        assert_eq!(config.get_terminal_type(), *terminal_type);
    }
    
    // Valid 5250 terminal types should also work
    let valid_5250_types = [
        "IBM-3179-2", "IBM-3196-A1", "IBM-5251-11",
        "IBM-5291-1", "IBM-5292-2"
    ];
    
    for terminal_type in &valid_5250_types {
        assert!(config.set_terminal_type(terminal_type).is_ok());
        assert_eq!(config.get_terminal_type(), *terminal_type);
    }
}

/// Test display buffer operations for different screen sizes
#[test]
fn test_display_buffer_operations() {
    // Test Model 2 (24x80)
    let display2 = Display3270::with_size(ScreenSize::Model2);
    assert_eq!(display2.rows(), 24);
    assert_eq!(display2.cols(), 80);
    assert_eq!(display2.buffer_size(), 1920);
    
    // Test Model 3 (32x80)
    let display3 = Display3270::with_size(ScreenSize::Model3);
    assert_eq!(display3.rows(), 32);
    assert_eq!(display3.cols(), 80);
    assert_eq!(display3.buffer_size(), 2560);
    
    // Test Model 4 (43x80)
    let display4 = Display3270::with_size(ScreenSize::Model4);
    assert_eq!(display4.rows(), 43);
    assert_eq!(display4.cols(), 80);
    assert_eq!(display4.buffer_size(), 3440);
    
    // Test Model 5 (27x132)
    let display5 = Display3270::with_size(ScreenSize::Model5);
    assert_eq!(display5.rows(), 27);
    assert_eq!(display5.cols(), 132);
    assert_eq!(display5.buffer_size(), 3564);
}

/// Test Write command processing
#[test]
fn test_write_command() {
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    let data = vec![
        CMD_WRITE,
        WCC_RESTORE | WCC_ALARM,
        0xC1, 0xC2, 0xC3, // ABC in EBCDIC
    ];
    
    assert!(processor.process_data(&data, &mut display).is_ok());
    assert!(!display.is_keyboard_locked());
    assert!(display.is_alarm());
}

/// Test Erase/Write command
#[test]
fn test_erase_write_command() {
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    // Write some data first
    display.write_char(0xC1);
    assert_eq!(display.cursor_address(), 1);
    
    // Erase/Write should clear the buffer
    let data = vec![
        CMD_ERASE_WRITE,
        0x00, // WCC
        0xC2, // B in EBCDIC
    ];
    
    assert!(processor.process_data(&data, &mut display).is_ok());
    assert_eq!(display.cursor_address(), 1);
    assert_eq!(display.read_char_at(0), Some(0xC2));
}

/// Test Set Buffer Address (SBA) order
#[test]
fn test_set_buffer_address() {
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    use tn5250r::lib3270::display::addressing;
    let (b1, b2) = addressing::encode_12bit_address(100);
    
    let data = vec![
        CMD_WRITE,
        0x00,      // WCC
        ORDER_SBA, // Set Buffer Address
        b1, b2,    // Address bytes
        0xC1,      // A in EBCDIC
    ];
    
    assert!(processor.process_data(&data, &mut display).is_ok());
    assert_eq!(display.read_char_at(100), Some(0xC1));
}

/// Test Start Field (SF) order
#[test]
fn test_start_field_order() {
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    let data = vec![
        CMD_WRITE,
        0x00,     // WCC
        ORDER_SF, // Start Field
        ATTR_PROTECTED | ATTR_NUMERIC,
    ];
    
    assert!(processor.process_data(&data, &mut display).is_ok());
    assert_eq!(display.field_manager().fields().len(), 1);
}

/// Test Read Buffer response generation
#[test]
fn test_read_buffer_response() {
    let processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    // Write some data
    display.write_char(0xC1);
    display.write_char(0xC2);
    
    let response = processor.create_read_buffer_response(&display, AidKey::Enter);
    
    // Response should have AID + cursor address (2 bytes) + buffer data
    assert!(response.len() >= 3);
    assert_eq!(response[0], AidKey::Enter.to_u8());
}

/// Test Read Modified response generation
#[test]
fn test_read_modified_response() {
    let processor = ProtocolProcessor3270::new();
    let display = Display3270::new();
    
    let response = processor.create_read_modified_response(&display, AidKey::Enter);
    
    // Response should have AID + cursor address (2 bytes)
    assert!(response.len() >= 3);
    assert_eq!(response[0], AidKey::Enter.to_u8());
}

/// Test protocol string parsing
#[test]
fn test_protocol_string_parsing() {
    assert_eq!(parse_protocol_string("auto").unwrap(), ProtocolMode::AutoDetect);
    assert_eq!(parse_protocol_string("tn3270").unwrap(), ProtocolMode::TN3270);
    assert_eq!(parse_protocol_string("3270").unwrap(), ProtocolMode::TN3270);
    assert_eq!(parse_protocol_string("tn5250").unwrap(), ProtocolMode::TN5250);
    assert_eq!(parse_protocol_string("5250").unwrap(), ProtocolMode::TN5250);
    assert_eq!(parse_protocol_string("nvt").unwrap(), ProtocolMode::NVT);
    
    // Case insensitive
    assert_eq!(parse_protocol_string("TN3270").unwrap(), ProtocolMode::TN3270);
    assert_eq!(parse_protocol_string("AUTO").unwrap(), ProtocolMode::AutoDetect);
}

/// Test protocol mode to string conversion
#[test]
fn test_protocol_mode_to_string() {
    assert_eq!(protocol_mode_to_string(ProtocolMode::AutoDetect), "auto");
    assert_eq!(protocol_mode_to_string(ProtocolMode::TN3270), "tn3270");
    assert_eq!(protocol_mode_to_string(ProtocolMode::TN5250), "tn5250");
    assert_eq!(protocol_mode_to_string(ProtocolMode::NVT), "nvt");
}

/// Test field attribute handling
#[test]
fn test_field_attributes() {
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    // Create field with various attributes
    let data = vec![
        CMD_WRITE,
        0x00,
        ORDER_SF,
        ATTR_PROTECTED | ATTR_NUMERIC,
        0xC1, 0xC2, 0xC3, // Field data
    ];
    
    assert!(processor.process_data(&data, &mut display).is_ok());
    
    let fields = display.field_manager().fields();
    assert_eq!(fields.len(), 1);
    
    let field = &fields[0];
    assert!(field.is_protected());
    assert!(field.is_numeric());
}

/// Test cursor positioning
#[test]
fn test_cursor_positioning() {
    let mut display = Display3270::new();
    
    // Test initial position
    assert_eq!(display.cursor_address(), 0);
    assert_eq!(display.cursor_position(), (0, 0));
    
    // Test setting cursor
    display.set_cursor(100);
    assert_eq!(display.cursor_address(), 100);
    
    let (row, col) = display.cursor_position();
    assert_eq!(row, 1);
    assert_eq!(col, 20);
}

/// Test keyboard lock/unlock
#[test]
fn test_keyboard_lock() {
    let mut display = Display3270::new();
    
    // Initially locked
    assert!(display.is_keyboard_locked());
    
    // Unlock
    display.unlock_keyboard();
    assert!(!display.is_keyboard_locked());
    
    // Lock again
    display.lock_keyboard();
    assert!(display.is_keyboard_locked());
}

/// Test alarm functionality
#[test]
fn test_alarm() {
    let mut display = Display3270::new();
    
    assert!(!display.is_alarm());
    
    display.set_alarm(true);
    assert!(display.is_alarm());
    
    display.set_alarm(false);
    assert!(!display.is_alarm());
}

/// Test buffer clear operations
#[test]
fn test_buffer_clear() {
    let mut display = Display3270::new();
    
    // Write some data
    display.write_char(0xC1);
    display.write_char(0xC2);
    assert_eq!(display.cursor_address(), 2);
    
    // Clear buffer
    display.clear();
    assert_eq!(display.cursor_address(), 0);
    assert_eq!(display.read_char_at(0), Some(0x00));
    assert_eq!(display.read_char_at(1), Some(0x00));
}

/// Test repeat to address operation
#[test]
fn test_repeat_to_address() {
    let mut display = Display3270::new();
    
    display.set_cursor(10);
    display.repeat_to_address(0xC1, 20);
    
    // Check that characters were repeated
    for i in 10..=20 {
        assert_eq!(display.read_char_at(i), Some(0xC1));
    }
}

/// Test display string conversion
#[test]
fn test_display_to_string() {
    let mut display = Display3270::new();
    
    // Write some EBCDIC characters
    display.write_char(0xC1); // A
    display.write_char(0xC2); // B
    display.write_char(0xC3); // C
    
    let display_str = display.to_string();
    assert!(display_str.contains('A'));
    assert!(display_str.contains('B'));
    assert!(display_str.contains('C'));
}

/// Test row retrieval
#[test]
fn test_get_row() {
    let mut display = Display3270::new();
    
    // Write to first row
    for i in 0..10 {
        display.write_char(0xC1 + i as u8);
    }
    
    let row = display.get_row(0);
    assert!(row.is_some());
    
    let row_str = row.unwrap();
    assert!(row_str.len() == 80);
}

/// Test error handling for missing data
#[test]
fn test_error_handling_missing_data() {
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    // Write command without WCC
    let data = vec![CMD_WRITE];
    assert!(processor.process_data(&data, &mut display).is_err());
    
    // SF order without attribute byte
    let data = vec![CMD_WRITE, 0x00, ORDER_SF];
    assert!(processor.process_data(&data, &mut display).is_err());
}

/// Test protocol reset
#[test]
fn test_protocol_reset() {
    use tn5250r::protocol_common::traits::TerminalProtocol;
    
    let mut processor = ProtocolProcessor3270::new();
    
    assert!(processor.is_connected());
    
    processor.reset();
    assert!(processor.is_connected());
}

/// Test configuration serialization with TN3270 settings
#[test]
fn test_configuration_serialization() {
    let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
    
    config.set_protocol_mode("tn3270").unwrap();
    config.set_terminal_type("IBM-3278-2").unwrap();
    
    let json = config.to_json().expect("Serialization should work");
    assert!(json.contains("tn3270"));
    assert!(json.contains("IBM-3278-2"));
    
    let mut new_config = SessionConfig::new("test2.json".to_string(), "test_session2".to_string());
    new_config.from_json(&json).expect("Deserialization should work");
    
    assert_eq!(new_config.get_protocol_mode(), "tn3270");
    assert_eq!(new_config.get_terminal_type(), "IBM-3278-2");
}

/// Test backward compatibility with TN5250
#[test]
fn test_backward_compatibility() {
    // Ensure TN5250 configuration still works
    let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
    
    config.set_protocol_mode("tn5250").unwrap();
    config.set_terminal_type("IBM-3179-2").unwrap();
    assert!(config.validate_protocol_terminal_combination().is_ok());
    
    // Ensure protocol switching works
    config.set_protocol_mode("tn3270").unwrap();
    config.set_terminal_type("IBM-3278-2").unwrap();
    assert!(config.validate_protocol_terminal_combination().is_ok());
}

/// Test multiple screen size support
#[test]
fn test_multiple_screen_sizes() {
    let sizes = [
        (ScreenSize::Model2, 24, 80, 1920),
        (ScreenSize::Model3, 32, 80, 2560),
        (ScreenSize::Model4, 43, 80, 3440),
        (ScreenSize::Model5, 27, 132, 3564),
    ];
    
    for (size, rows, cols, buffer_size) in &sizes {
        let display = Display3270::with_size(*size);
        assert_eq!(display.rows(), *rows);
        assert_eq!(display.cols(), *cols);
        assert_eq!(display.buffer_size(), *buffer_size);
    }
}

/// Test address coordinate conversion
#[test]
fn test_address_coordinate_conversion() {
    let size = ScreenSize::Model2;
    
    // Test various positions
    assert_eq!(size.address_to_coords(0), (0, 0));
    assert_eq!(size.address_to_coords(80), (1, 0));
    assert_eq!(size.address_to_coords(81), (1, 1));
    assert_eq!(size.address_to_coords(160), (2, 0));
    
    // Test reverse conversion
    assert_eq!(size.coords_to_address(0, 0), 0);
    assert_eq!(size.coords_to_address(1, 0), 80);
    assert_eq!(size.coords_to_address(1, 1), 81);
    assert_eq!(size.coords_to_address(2, 0), 160);
}

/// Integration test: Complete 3270 session simulation
#[test]
fn test_complete_3270_session() {
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    // 1. Erase screen and write header
    let data1 = vec![
        CMD_ERASE_WRITE,
        WCC_RESTORE,
        ORDER_SF,
        ATTR_PROTECTED,
        0xD7, 0xD9, 0xD6, 0xC4, 0xE4, 0xC3, 0xE3, // "PRODUCT" in EBCDIC
    ];
    assert!(processor.process_data(&data1, &mut display).is_ok());
    
    // 2. Position cursor and create input field
    use tn5250r::lib3270::display::addressing;
    let (b1, b2) = addressing::encode_12bit_address(160); // Row 2
    let data2 = vec![
        CMD_WRITE,
        0x00,
        ORDER_SBA,
        b1, b2,
        ORDER_SF,
        0x00, // Unprotected field
    ];
    assert!(processor.process_data(&data2, &mut display).is_ok());
    
    // 3. Verify state
    assert!(!display.is_keyboard_locked());
    assert!(display.field_manager().fields().len() >= 2);
}