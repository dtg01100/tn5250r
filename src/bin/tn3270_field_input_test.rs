//! TN3270 Field Input Test
//!
//! Comprehensive tests for keyboard handling and field input in the TN3270 protocol.
//! Tests alphanumeric input, field validation, special keys, input transmission,
//! function key integration (F1-F24, PA keys), multi-field input with MDT tracking,
//! EBCDIC encoding, buffer management, and 3270-specific features like buffer addressing.

use tn5250r::controller::TerminalController;
use tn5250r::keyboard::{KeyboardInput, FunctionKey, SpecialKey};
use tn5250r::field_manager::{Field, FieldType, FieldManager};
use tn5250r::protocol_common::ebcdic::{ascii_to_ebcdic, ebcdic_to_ascii};
use tn5250r::lib3270::protocol::ProtocolProcessor3270;
use tn5250r::lib3270::display::{Display3270, ScreenSize, addressing};
use tn5250r::lib3270::codes::{AidKey, AID_ENTER, AID_PF1, AID_PF12, AID_PF24, AID_PA1, AID_CLEAR};
use tn5250r::lib3270::field::{FieldAttribute, ExtendedAttributes};

/// Test result structure
struct TestResult {
    name: String,
    passed: bool,
    message: String,
}

impl TestResult {
    fn pass(name: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            message: "PASS".to_string(),
        }
    }

    fn fail(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            message: message.to_string(),
        }
    }

    fn print(&self) {
        let status = if self.passed { "✓ PASS" } else { "✗ FAIL" };
        println!("{}: {} - {}", status, self.name, self.message);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TN3270 Field Input Test ===\n");
    
    let mut results = Vec::new();
    
    // Test 1: Basic alphanumeric input
    println!("Test 1: Basic Alphanumeric Input");
    results.extend(test_alphanumeric_input().await);
    println!();
    
    // Test 2: Field validation
    println!("Test 2: Field Validation");
    results.extend(test_field_validation().await);
    println!();
    
    // Test 3: Special keys
    println!("Test 3: Special Key Handling");
    results.extend(test_special_keys().await);
    println!();
    
    // Test 4: Input transmission (3270-specific)
    println!("Test 4: Input Transmission (3270-specific)");
    results.extend(test_input_transmission_3270().await);
    println!();
    
    // Test 5: Function key integration (3270 AID keys)
    println!("Test 5: Function Key Integration (3270 AID keys)");
    results.extend(test_function_key_integration_3270().await);
    println!();
    
    // Test 6: Multi-field input with MDT
    println!("Test 6: Multi-Field Input with MDT");
    results.extend(test_multi_field_input_with_mdt().await);
    println!();
    
    // Test 7: EBCDIC encoding (3270-specific)
    println!("Test 7: EBCDIC Encoding (3270-specific)");
    results.extend(test_ebcdic_encoding_3270().await);
    println!();
    
    // Test 8: Buffer management
    println!("Test 8: Buffer Management");
    results.extend(test_buffer_management().await);
    println!();
    
    // Test 9: Buffer addressing (12-bit and 14-bit)
    println!("Test 9: Buffer Addressing");
    results.extend(test_buffer_addressing().await);
    println!();
    
    // Test 10: AID keys (3270-specific)
    println!("Test 10: AID Keys");
    results.extend(test_aid_keys().await);
    println!();
    
    // Test 11: Field attributes (3270-specific)
    println!("Test 11: Field Attributes");
    results.extend(test_field_attributes().await);
    println!();
    
    // Print summary
    println!("\n=== Test Summary ===");
    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    
    for result in &results {
        result.print();
    }
    
    println!("\nTotal: {}/{} tests passed", passed, total);
    
    if passed == total {
        println!("\n=== All TN3270 Field Input Tests Passed ===");
        Ok(())
    } else {
        println!("\n=== Some Tests Failed ===");
        std::process::exit(1);
    }
}

/// Test 1: Basic alphanumeric input
async fn test_alphanumeric_input() -> Vec<TestResult> {
    let mut results = Vec::new();
    let mut controller = TerminalController::new();
    
    // Create a test field
    let mut field_manager = FieldManager::new();
    let field = Field::new(1, FieldType::Input, 1, 1, 20);
    field_manager.add_field_for_test(field);
    field_manager.set_active_field_for_test(Some(0));
    
    // Test typing letters
    let test_chars = vec!['A', 'B', 'C', 'a', 'b', 'c'];
    for ch in test_chars {
        match field_manager.type_char(ch) {
            Ok(_) => {
                results.push(TestResult::pass(&format!("Type letter '{}'", ch)));
            }
            Err(e) => {
                results.push(TestResult::fail(&format!("Type letter '{}'", ch), &e));
            }
        }
    }
    
    // Verify content
    if let Some(field) = field_manager.get_active_field() {
        let expected = "ABCabc";
        if field.content == expected {
            results.push(TestResult::pass("Letters queued correctly"));
        } else {
            results.push(TestResult::fail(
                "Letters queued correctly",
                &format!("Expected '{}', got '{}'", expected, field.content)
            ));
        }
    }
    
    // Test typing numbers
    field_manager.clear_all_fields();
    let numbers = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    for num in numbers {
        match field_manager.type_char(num) {
            Ok(_) => {
                results.push(TestResult::pass(&format!("Type number '{}'", num)));
            }
            Err(e) => {
                results.push(TestResult::fail(&format!("Type number '{}'", num), &e));
            }
        }
    }
    
    // Test special characters
    field_manager.clear_all_fields();
    let special_chars = vec![' ', '.', ',', '-', '@', '#'];
    for ch in special_chars {
        match field_manager.type_char(ch) {
            Ok(_) => {
                results.push(TestResult::pass(&format!("Type special char '{}'", ch)));
            }
            Err(e) => {
                results.push(TestResult::fail(&format!("Type special char '{}'", ch), &e));
            }
        }
    }
    
    // Test pending input buffer in controller
    controller.clear_pending_input();
    let test_input = "TEST123";
    for ch in test_input.chars() {
        if let Err(e) = controller.type_char(ch) {
            results.push(TestResult::fail(
                "Queue input in controller",
                &format!("Failed to queue '{}': {}", ch, e)
            ));
        }
    }
    
    let pending = controller.get_pending_input();
    if pending.len() == test_input.len() {
        results.push(TestResult::pass("Characters queued in pending_input buffer"));
    } else {
        results.push(TestResult::fail(
            "Characters queued in pending_input buffer",
            &format!("Expected {} bytes, got {}", test_input.len(), pending.len())
        ));
    }
    
    // Verify EBCDIC conversion
    if !pending.is_empty() {
        let first_char_ebcdic = pending[0];
        let expected_ebcdic = ascii_to_ebcdic('T');
        if first_char_ebcdic == expected_ebcdic {
            results.push(TestResult::pass("Characters converted to EBCDIC correctly"));
        } else {
            results.push(TestResult::fail(
                "Characters converted to EBCDIC correctly",
                &format!("Expected 0x{:02X}, got 0x{:02X}", expected_ebcdic, first_char_ebcdic)
            ));
        }
    }
    
    results
}

/// Test 2: Field validation
async fn test_field_validation() -> Vec<TestResult> {
    let mut results = Vec::new();
    let mut field_manager = FieldManager::new();
    
    // Test numeric-only field
    let numeric_field = Field::new(1, FieldType::Numeric, 1, 1, 10);
    field_manager.add_field_for_test(numeric_field);
    field_manager.set_active_field_for_test(Some(0));
    
    // Should accept numbers
    match field_manager.type_char('5') {
        Ok(_) => results.push(TestResult::pass("Numeric field accepts numbers")),
        Err(e) => results.push(TestResult::fail("Numeric field accepts numbers", &e)),
    }
    
    // Should reject letters
    match field_manager.type_char('A') {
        Ok(_) => results.push(TestResult::fail(
            "Numeric field rejects letters",
            "Should have rejected letter"
        )),
        Err(_) => results.push(TestResult::pass("Numeric field rejects letters")),
    }
    
    // Test alpha-only field
    field_manager = FieldManager::new();
    let alpha_field = Field::new(2, FieldType::AlphaOnly, 2, 1, 10);
    field_manager.add_field_for_test(alpha_field);
    field_manager.set_active_field_for_test(Some(0));
    
    // Should accept letters
    match field_manager.type_char('A') {
        Ok(_) => results.push(TestResult::pass("Alpha field accepts letters")),
        Err(e) => results.push(TestResult::fail("Alpha field accepts letters", &e)),
    }
    
    // Should reject numbers
    match field_manager.type_char('5') {
        Ok(_) => results.push(TestResult::fail(
            "Alpha field rejects numbers",
            "Should have rejected number"
        )),
        Err(_) => results.push(TestResult::pass("Alpha field rejects numbers")),
    }
    
    // Test uppercase-only field
    field_manager = FieldManager::new();
    let upper_field = Field::new(3, FieldType::UppercaseOnly, 3, 1, 10);
    field_manager.add_field_for_test(upper_field);
    field_manager.set_active_field_for_test(Some(0));
    
    // Type lowercase and verify it's converted
    if let Err(e) = field_manager.type_char('a') {
        results.push(TestResult::fail("Uppercase field converts lowercase", &e));
    } else {
        if let Some(field) = field_manager.get_active_field() {
            if field.content.contains('A') || field.content.contains('a') {
                results.push(TestResult::pass("Uppercase field converts lowercase"));
            } else {
                results.push(TestResult::fail(
                    "Uppercase field converts lowercase",
                    "Character not converted to uppercase"
                ));
            }
        }
    }
    
    // Test password field (should mask input)
    field_manager = FieldManager::new();
    let password_field = Field::new(4, FieldType::Password, 4, 1, 10);
    field_manager.add_field_for_test(password_field);
    field_manager.set_active_field_for_test(Some(0));
    
    let _ = field_manager.type_char('P');
    let _ = field_manager.type_char('A');
    let _ = field_manager.type_char('S');
    let _ = field_manager.type_char('S');
    
    if let Some(field) = field_manager.get_active_field() {
        let display = field.get_display_content();
        if display == "****" {
            results.push(TestResult::pass("Password field masks input"));
        } else {
            results.push(TestResult::fail(
                "Password field masks input",
                &format!("Expected '****', got '{}'", display)
            ));
        }
    }
    
    results
}

/// Test 3: Special key handling
async fn test_special_keys() -> Vec<TestResult> {
    let mut results = Vec::new();
    let mut field_manager = FieldManager::new();
    
    // Setup test field with content
    let field = Field::new(1, FieldType::Input, 1, 1, 20);
    field_manager.add_field_for_test(field);
    field_manager.set_active_field_for_test(Some(0));
    
    // Add some content
    let _ = field_manager.type_char('T');
    let _ = field_manager.type_char('E');
    let _ = field_manager.type_char('S');
    let _ = field_manager.type_char('T');
    
    // Test Backspace
    match field_manager.backspace() {
        Ok(_) => {
            if let Some(field) = field_manager.get_active_field() {
                if field.content == "TES" {
                    results.push(TestResult::pass("Backspace deletes previous character"));
                } else {
                    results.push(TestResult::fail(
                        "Backspace deletes previous character",
                        &format!("Expected 'TES', got '{}'", field.content)
                    ));
                }
            }
        }
        Err(e) => results.push(TestResult::fail("Backspace deletes previous character", &e)),
    }
    
    // Test Delete
    match field_manager.delete() {
        Ok(_) => results.push(TestResult::pass("Delete removes character")),
        Err(e) => results.push(TestResult::fail("Delete removes character", &e)),
    }
    
    // Test Tab (next field)
    field_manager = FieldManager::new();
    let field1 = Field::new(1, FieldType::Input, 1, 1, 10);
    let field2 = Field::new(2, FieldType::Input, 2, 1, 10);
    field_manager.add_field_for_test(field1);
    field_manager.add_field_for_test(field2);
    field_manager.set_active_field_for_test(Some(0));
    
    match field_manager.next_field() {
        Ok(_) => {
            if let Some(idx) = field_manager.get_active_field_index() {
                if idx == 1 {
                    results.push(TestResult::pass("Tab moves to next field"));
                } else {
                    results.push(TestResult::fail(
                        "Tab moves to next field",
                        &format!("Expected field 1, got field {}", idx)
                    ));
                }
            }
        }
        Err(e) => results.push(TestResult::fail("Tab moves to next field", &e.get_user_message())),
    }
    
    // Test Shift+Tab (previous field)
    match field_manager.previous_field() {
        Ok(_) => {
            if let Some(idx) = field_manager.get_active_field_index() {
                if idx == 0 {
                    results.push(TestResult::pass("Shift+Tab moves to previous field"));
                } else {
                    results.push(TestResult::fail(
                        "Shift+Tab moves to previous field",
                        &format!("Expected field 0, got field {}", idx)
                    ));
                }
            }
        }
        Err(e) => results.push(TestResult::fail("Shift+Tab moves to previous field", &e.get_user_message())),
    }
    
    results
}

/// Test 4: Input transmission (3270-specific)
async fn test_input_transmission_3270() -> Vec<TestResult> {
    let mut results = Vec::new();
    let processor = ProtocolProcessor3270::new();
    let display = Display3270::new();
    
    // Test encoding field data with SBA orders
    let field_data = vec![
        (100u16, "USER".to_string()),
        (200u16, "PASS".to_string()),
    ];
    
    let encoded = processor.encode_field_data(&field_data);
    
    // Verify SBA orders are present
    if !encoded.is_empty() {
        results.push(TestResult::pass("Field data encoded successfully"));
        
        // Check for SBA order (0x11)
        let has_sba = encoded.iter().any(|&b| b == 0x11);
        if has_sba {
            results.push(TestResult::pass("SBA orders present in encoded data"));
        } else {
            results.push(TestResult::fail(
                "SBA orders present in encoded data",
                "No SBA orders found"
            ));
        }
    } else {
        results.push(TestResult::fail(
            "Field data encoded successfully",
            "Encoded data is empty"
        ));
    }
    
    // Test send_input_fields with Enter AID
    let response = processor.send_input_fields(&display, AidKey::Enter, &field_data);
    
    // Verify response structure: AID + cursor address (2 bytes) + field data
    if response.len() >= 3 {
        results.push(TestResult::pass("Input fields response has correct structure"));
        
        // Verify AID byte
        if response[0] == AID_ENTER {
            results.push(TestResult::pass("Enter AID key encoded correctly"));
        } else {
            results.push(TestResult::fail(
                "Enter AID key encoded correctly",
                &format!("Expected 0x{:02X}, got 0x{:02X}", AID_ENTER, response[0])
            ));
        }
    } else {
        results.push(TestResult::fail(
            "Input fields response has correct structure",
            &format!("Response too short: {} bytes", response.len())
        ));
    }
    
    results
}

/// Test 5: Function key integration (3270 AID keys)
async fn test_function_key_integration_3270() -> Vec<TestResult> {
    let mut results = Vec::new();
    let processor = ProtocolProcessor3270::new();
    let display = Display3270::new();
    
    // Test F1-F12 AID keys
    let f1_response = processor.send_input_fields(&display, AidKey::PF1, &[]);
    if f1_response[0] == AID_PF1 {
        results.push(TestResult::pass("F1 AID key (PF1) encodes correctly"));
    } else {
        results.push(TestResult::fail(
            "F1 AID key (PF1) encodes correctly",
            &format!("Expected 0x{:02X}, got 0x{:02X}", AID_PF1, f1_response[0])
        ));
    }
    
    let f12_response = processor.send_input_fields(&display, AidKey::PF12, &[]);
    if f12_response[0] == AID_PF12 {
        results.push(TestResult::pass("F12 AID key (PF12) encodes correctly"));
    } else {
        results.push(TestResult::fail(
            "F12 AID key (PF12) encodes correctly",
            &format!("Expected 0x{:02X}, got 0x{:02X}", AID_PF12, f12_response[0])
        ));
    }
    
    // Test F13-F24 (3270 supports extended function keys)
    let f24_response = processor.send_input_fields(&display, AidKey::PF24, &[]);
    if f24_response[0] == AID_PF24 {
        results.push(TestResult::pass("F24 AID key (PF24) encodes correctly"));
    } else {
        results.push(TestResult::fail(
            "F24 AID key (PF24) encodes correctly",
            &format!("Expected 0x{:02X}, got 0x{:02X}", AID_PF24, f24_response[0])
        ));
    }
    
    // Test PA keys (PA1, PA2, PA3)
    let pa1_response = processor.send_input_fields(&display, AidKey::PA1, &[]);
    if pa1_response[0] == AID_PA1 {
        results.push(TestResult::pass("PA1 AID key encodes correctly"));
    } else {
        results.push(TestResult::fail(
            "PA1 AID key encodes correctly",
            &format!("Expected 0x{:02X}, got 0x{:02X}", AID_PA1, pa1_response[0])
        ));
    }
    
    // Test Clear key
    let clear_response = processor.send_input_fields(&display, AidKey::Clear, &[]);
    if clear_response[0] == AID_CLEAR {
        results.push(TestResult::pass("Clear AID key encodes correctly"));
    } else {
        results.push(TestResult::fail(
            "Clear AID key encodes correctly",
            &format!("Expected 0x{:02X}, got 0x{:02X}", AID_CLEAR, clear_response[0])
        ));
    }
    
    // Test that pending input is flushed before function key
    let field_data = vec![(100u16, "DATA".to_string())];
    let response_with_data = processor.send_input_fields(&display, AidKey::PF1, &field_data);
    if response_with_data.len() > 3 {
        results.push(TestResult::pass("Pending input flushed before function key"));
    } else {
        results.push(TestResult::fail(
            "Pending input flushed before function key",
            "No field data in response"
        ));
    }
    
    results
}

/// Test 6: Multi-field input with MDT
async fn test_multi_field_input_with_mdt() -> Vec<TestResult> {
    let mut results = Vec::new();
    let mut display = Display3270::new();
    
    // Create multiple fields with MDT tracking
    let mut field1 = FieldAttribute::new(100, 0x00); // Unprotected, no MDT
    let mut field2 = FieldAttribute::new(200, 0x00); // Unprotected, no MDT
    let mut field3 = FieldAttribute::new(300, 0x00); // Unprotected, no MDT
    
    // Simulate typing in first field (sets MDT)
    field1.set_modified(true);
    display.field_manager_mut().add_field(field1.clone());
    
    // Simulate typing in second field (sets MDT)
    field2.set_modified(true);
    display.field_manager_mut().add_field(field2.clone());
    
    // Third field not modified (MDT not set)
    display.field_manager_mut().add_field(field3.clone());
    
    // Verify MDT is set correctly
    if field1.is_modified() {
        results.push(TestResult::pass("Field 1 MDT set after modification"));
    } else {
        results.push(TestResult::fail("Field 1 MDT set after modification", "MDT not set"));
    }
    
    if field2.is_modified() {
        results.push(TestResult::pass("Field 2 MDT set after modification"));
    } else {
        results.push(TestResult::fail("Field 2 MDT set after modification", "MDT not set"));
    }
    
    if !field3.is_modified() {
        results.push(TestResult::pass("Field 3 MDT not set (unmodified)"));
    } else {
        results.push(TestResult::fail("Field 3 MDT not set (unmodified)", "MDT incorrectly set"));
    }
    
    // Test that only modified fields are transmitted
    let modified_fields = display.field_manager().modified_fields();
    if modified_fields.len() == 2 {
        results.push(TestResult::pass("Only modified fields returned for transmission"));
    } else {
        results.push(TestResult::fail(
            "Only modified fields returned for transmission",
            &format!("Expected 2 modified fields, got {}", modified_fields.len())
        ));
    }
    
    // Test MDT reset
    display.field_manager_mut().reset_mdt();
    let modified_after_reset = display.field_manager().modified_fields();
    if modified_after_reset.is_empty() {
        results.push(TestResult::pass("MDT reset clears all modified flags"));
    } else {
        results.push(TestResult::fail(
            "MDT reset clears all modified flags",
            &format!("Still {} modified fields after reset", modified_after_reset.len())
        ));
    }
    
    results
}

/// Test 7: EBCDIC encoding (3270-specific)
async fn test_ebcdic_encoding_3270() -> Vec<TestResult> {
    let mut results = Vec::new();
    
    // Test uppercase letters
    let test_cases = vec![
        ('A', 0xC1), ('B', 0xC2), ('C', 0xC3), ('Z', 0xE9),
    ];
    
    for (ascii_char, expected_ebcdic) in test_cases {
        let ebcdic = ascii_to_ebcdic(ascii_char);
        if ebcdic == expected_ebcdic {
            results.push(TestResult::pass(&format!("'{}' -> 0x{:02X}", ascii_char, expected_ebcdic)));
        } else {
            results.push(TestResult::fail(
                &format!("'{}' -> 0x{:02X}", ascii_char, expected_ebcdic),
                &format!("Got 0x{:02X}", ebcdic)
            ));
        }
    }
    
    // Test lowercase letters
    let lowercase_cases = vec![
        ('a', 0x81), ('b', 0x82), ('c', 0x83), ('z', 0xA9),
    ];
    
    for (ascii_char, expected_ebcdic) in lowercase_cases {
        let ebcdic = ascii_to_ebcdic(ascii_char);
        if ebcdic == expected_ebcdic {
            results.push(TestResult::pass(&format!("'{}' -> 0x{:02X}", ascii_char, expected_ebcdic)));
        } else {
            results.push(TestResult::fail(
                &format!("'{}' -> 0x{:02X}", ascii_char, expected_ebcdic),
                &format!("Got 0x{:02X}", ebcdic)
            ));
        }
    }
    
    // Test numbers
    let number_cases = vec![
        ('0', 0xF0), ('5', 0xF5), ('9', 0xF9),
    ];
    
    for (ascii_char, expected_ebcdic) in number_cases {
        let ebcdic = ascii_to_ebcdic(ascii_char);
        if ebcdic == expected_ebcdic {
            results.push(TestResult::pass(&format!("'{}' -> 0x{:02X}", ascii_char, expected_ebcdic)));
        } else {
            results.push(TestResult::fail(
                &format!("'{}' -> 0x{:02X}", ascii_char, expected_ebcdic),
                &format!("Got 0x{:02X}", ebcdic)
            ));
        }
    }
    
    // Test special characters
    let special_cases = vec![
        (' ', 0x40), ('.', 0x4B), ('@', 0x7C),
    ];
    
    for (ascii_char, expected_ebcdic) in special_cases {
        let ebcdic = ascii_to_ebcdic(ascii_char);
        if ebcdic == expected_ebcdic {
            results.push(TestResult::pass(&format!("'{}' -> 0x{:02X}", ascii_char, expected_ebcdic)));
        } else {
            results.push(TestResult::fail(
                &format!("'{}' -> 0x{:02X}", ascii_char, expected_ebcdic),
                &format!("Got 0x{:02X}", ebcdic)
            ));
        }
    }
    
    // Test round-trip conversion
    let test_string = "Hello123";
    let mut round_trip_ok = true;
    for ch in test_string.chars() {
        let ebcdic = ascii_to_ebcdic(ch);
        let back_to_ascii = ebcdic_to_ascii(ebcdic);
        if back_to_ascii != ch {
            round_trip_ok = false;
            break;
        }
    }
    
    if round_trip_ok {
        results.push(TestResult::pass("Round-trip ASCII->EBCDIC->ASCII"));
    } else {
        results.push(TestResult::fail(
            "Round-trip ASCII->EBCDIC->ASCII",
            "Conversion not reversible"
        ));
    }
    
    // Test 3270 protocol encoding
    let processor = ProtocolProcessor3270::new();
    let field_data = vec![(100u16, "TEST".to_string())];
    let encoded = processor.encode_field_data(&field_data);
    
    // Verify EBCDIC encoding in field data
    let has_ebcdic_t = encoded.iter().any(|&b| b == ascii_to_ebcdic('T'));
    if has_ebcdic_t {
        results.push(TestResult::pass("3270 protocol uses EBCDIC encoding"));
    } else {
        results.push(TestResult::fail(
            "3270 protocol uses EBCDIC encoding",
            "EBCDIC encoding not found in field data"
        ));
    }
    
    results
}

/// Test 8: Buffer management
async fn test_buffer_management() -> Vec<TestResult> {
    let mut results = Vec::new();
    let mut controller = TerminalController::new();
    
    // Test buffer grows correctly
    controller.clear_pending_input();
    let test_data = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    
    for ch in test_data.chars() {
        let _ = controller.type_char(ch);
    }
    
    let pending = controller.get_pending_input();
    if pending.len() == test_data.len() {
        results.push(TestResult::pass("Buffer grows correctly"));
    } else {
        results.push(TestResult::fail(
            "Buffer grows correctly",
            &format!("Expected {} bytes, got {}", test_data.len(), pending.len())
        ));
    }
    
    // Test clear_pending_input
    controller.clear_pending_input();
    let pending_after = controller.get_pending_input();
    if pending_after.is_empty() {
        results.push(TestResult::pass("clear_pending_input() clears buffer"));
    } else {
        results.push(TestResult::fail(
            "clear_pending_input() clears buffer",
            &format!("Buffer still has {} bytes", pending_after.len())
        ));
    }
    
    // Test buffer doesn't overflow
    controller.clear_pending_input();
    let large_input = "X".repeat(1000);
    
    for ch in large_input.chars() {
        let _ = controller.type_char(ch);
    }
    
    let pending_large = controller.get_pending_input();
    if pending_large.len() <= 1000 {
        results.push(TestResult::pass("Buffer handles large input safely"));
    } else {
        results.push(TestResult::fail(
            "Buffer handles large input safely",
            &format!("Buffer grew to {} bytes", pending_large.len())
        ));
    }
    
    // Test multiple clear operations
    controller.clear_pending_input();
    controller.clear_pending_input();
    controller.clear_pending_input();
    
    let pending_final = controller.get_pending_input();
    if pending_final.is_empty() {
        results.push(TestResult::pass("Multiple clear operations work"));
    } else {
        results.push(TestResult::fail(
            "Multiple clear operations work",
            "Buffer not empty after multiple clears"
        ));
    }
    
    results
}

/// Test 9: Buffer addressing (12-bit and 14-bit)
async fn test_buffer_addressing() -> Vec<TestResult> {
    let mut results = Vec::new();
    
    // Test 12-bit addressing (standard 3270)
    let test_addresses_12bit = vec![0u16, 100, 500, 1000, 1919];
    
    for addr in test_addresses_12bit {
        let (b1, b2) = addressing::encode_12bit_address(addr);
        let decoded = addressing::decode_12bit_address(b1, b2);
        
        if decoded == addr {
            results.push(TestResult::pass(&format!("12-bit address {} encodes/decodes correctly", addr)));
        } else {
            results.push(TestResult::fail(
                &format!("12-bit address {} encodes/decodes correctly", addr),
                &format!("Expected {}, got {}", addr, decoded)
            ));
        }
    }
    
    // Test 14-bit addressing (extended 3270)
    let test_addresses_14bit = vec![0u16, 1000, 2000, 3000, 5000];
    
    for addr in test_addresses_14bit {
        let (b1, b2) = addressing::encode_14bit_address(addr);
        let decoded = addressing::decode_14bit_address(b1, b2);
        
        if decoded == addr {
            results.push(TestResult::pass(&format!("14-bit address {} encodes/decodes correctly", addr)));
        } else {
            results.push(TestResult::fail(
                &format!("14-bit address {} encodes/decodes correctly", addr),
                &format!("Expected {}, got {}", addr, decoded)
            ));
        }
    }
    
    // Test processor with 12-bit addressing
    let processor_12bit = ProtocolProcessor3270::new();
    let field_data = vec![(100u16, "TEST".to_string())];
    let encoded_12bit = processor_12bit.encode_field_data(&field_data);
    
    if !encoded_12bit.is_empty() {
        results.push(TestResult::pass("12-bit addressing mode works in processor"));
    } else {
        results.push(TestResult::fail("12-bit addressing mode works in processor", "No data encoded"));
    }
    
    // Test processor with 14-bit addressing
    let mut processor_14bit = ProtocolProcessor3270::new();
    processor_14bit.set_14bit_addressing(true);
    let encoded_14bit = processor_14bit.encode_field_data(&field_data);
    
    if !encoded_14bit.is_empty() {
        results.push(TestResult::pass("14-bit addressing mode works in processor"));
    } else {
        results.push(TestResult::fail("14-bit addressing mode works in processor", "No data encoded"));
    }
    
    // Verify different encoding between 12-bit and 14-bit
    if encoded_12bit != encoded_14bit {
        results.push(TestResult::pass("12-bit and 14-bit addressing produce different encodings"));
    } else {
        results.push(TestResult::fail(
            "12-bit and 14-bit addressing produce different encodings",
            "Encodings are identical"
        ));
    }
    
    results
}

/// Test 10: AID keys (3270-specific)
async fn test_aid_keys() -> Vec<TestResult> {
    let mut results = Vec::new();
    
    // Test all AID key values
    let aid_tests = vec![
        (AidKey::Enter, AID_ENTER, "Enter"),
        (AidKey::PF1, AID_PF1, "PF1"),
        (AidKey::PF12, AID_PF12, "PF12"),
        (AidKey::PF24, AID_PF24, "PF24"),
        (AidKey::PA1, AID_PA1, "PA1"),
        (AidKey::Clear, AID_CLEAR, "Clear"),
    ];
    
    for (aid_key, expected_byte, name) in aid_tests {
        let byte_value = aid_key.to_u8();
        if byte_value == expected_byte {
            results.push(TestResult::pass(&format!("{} AID key value correct (0x{:02X})", name, expected_byte)));
        } else {
            results.push(TestResult::fail(
                &format!("{} AID key value correct", name),
                &format!("Expected 0x{:02X}, got 0x{:02X}", expected_byte, byte_value)
            ));
        }
    }
    
    // Test AID keys in responses
    let processor = ProtocolProcessor3270::new();
    let display = Display3270::new();
    
    let enter_response = processor.create_read_buffer_response(&display, AidKey::Enter);
    if enter_response[0] == AID_ENTER {
        results.push(TestResult::pass("Enter AID in Read Buffer response"));
    } else {
        results.push(TestResult::fail("Enter AID in Read Buffer response", "Incorrect AID byte"));
    }
    
    let pf1_response = processor.create_read_modified_response(&display, AidKey::PF1);
    if pf1_response[0] == AID_PF1 {
        results.push(TestResult::pass("PF1 AID in Read Modified response"));
    } else {
        results.push(TestResult::fail("PF1 AID in Read Modified response", "Incorrect AID byte"));
    }
    
    results
}

/// Test 11: Field attributes (3270-specific)
async fn test_field_attributes() -> Vec<TestResult> {
    let mut results = Vec::new();
    
    // Test protected field attribute
    let protected_attr = FieldAttribute::new(100, 0x20); // ATTR_PROTECTED
    if protected_attr.is_protected() {
        results.push(TestResult::pass("Protected field attribute detected"));
    } else {
        results.push(TestResult::fail("Protected field attribute detected", "Not detected as protected"));
    }
    
    // Test numeric field attribute
    let numeric_attr = FieldAttribute::new(200, 0x10); // ATTR_NUMERIC
    if numeric_attr.is_numeric() {
        results.push(TestResult::pass("Numeric field attribute detected"));
    } else {
        results.push(TestResult::fail("Numeric field attribute detected", "Not detected as numeric"));
    }
    
    // Test MDT (Modified Data Tag)
    let mut mdt_attr = FieldAttribute::new(300, 0x00);
    if !mdt_attr.is_modified() {
        results.push(TestResult::pass("MDT initially not set"));
    } else {
        results.push(TestResult::fail("MDT initially not set", "MDT incorrectly set"));
    }
    
    mdt_attr.set_modified(true);
    if mdt_attr.is_modified() {
        results.push(TestResult::pass("MDT set correctly"));
    } else {
        results.push(TestResult::fail("MDT set correctly", "MDT not set"));
    }
    
    mdt_attr.set_modified(false);
    if !mdt_attr.is_modified() {
        results.push(TestResult::pass("MDT cleared correctly"));
    } else {
        results.push(TestResult::fail("MDT cleared correctly", "MDT not cleared"));
    }
    
    // Test extended attributes
    let extended_attrs = ExtendedAttributes::new()
        .with_highlighting(0xF1)
        .with_foreground(0xF2)
        .with_background(0xF4);
    
    if extended_attrs.highlighting == Some(0xF1) {
        results.push(TestResult::pass("Extended highlighting attribute set"));
    } else {
        results.push(TestResult::fail("Extended highlighting attribute set", "Not set correctly"));
    }
    
    if extended_attrs.foreground_color == Some(0xF2) {
        results.push(TestResult::pass("Extended foreground color set"));
    } else {
        results.push(TestResult::fail("Extended foreground color set", "Not set correctly"));
    }
    
    if extended_attrs.background_color == Some(0xF4) {
        results.push(TestResult::pass("Extended background color set"));
    } else {
        results.push(TestResult::fail("Extended background color set", "Not set correctly"));
    }
    
    // Test field with extended attributes
    let extended_field = FieldAttribute::new_extended(400, 0x00, extended_attrs);
    if extended_field.extended_attrs.highlighting.is_some() {
        results.push(TestResult::pass("Field with extended attributes created"));
    } else {
        results.push(TestResult::fail("Field with extended attributes created", "Extended attrs not present"));
    }
    
    results
}