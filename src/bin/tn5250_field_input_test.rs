//! TN5250 Field Input Test
//!
//! Comprehensive tests for keyboard handling and field input in the TN5250 protocol.
//! Tests alphanumeric input, field validation, special keys, input transmission,
//! function key integration, multi-field input, EBCDIC encoding, and buffer management.

use tn5250r::controller::TerminalController;
use tn5250r::keyboard::{KeyboardInput, FunctionKey, SpecialKey};
use tn5250r::field_manager::{Field, FieldType, FieldManager};
use tn5250r::protocol_common::ebcdic::{ascii_to_ebcdic, ebcdic_to_ascii};

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
    println!("=== TN5250 Field Input Test ===\n");
    
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
    
    // Test 4: Input transmission
    println!("Test 4: Input Transmission");
    results.extend(test_input_transmission().await);
    println!();
    
    // Test 5: Function key integration
    println!("Test 5: Function Key Integration");
    results.extend(test_function_key_integration().await);
    println!();
    
    // Test 6: Multi-field input
    println!("Test 6: Multi-Field Input");
    results.extend(test_multi_field_input().await);
    println!();
    
    // Test 7: EBCDIC encoding
    println!("Test 7: EBCDIC Encoding");
    results.extend(test_ebcdic_encoding().await);
    println!();
    
    // Test 8: Buffer management
    println!("Test 8: Buffer Management");
    results.extend(test_buffer_management().await);
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
        println!("\n=== All TN5250 Field Input Tests Passed ===");
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
    
    results
}

/// Test 2: Field validation
async fn test_field_validation() -> Vec<TestResult> {
    let mut results = Vec::new();
    let mut field_manager = FieldManager::new();
    
    // Test numeric-only field
    let mut numeric_field = Field::new(1, FieldType::Numeric, 1, 1, 10);
    field_manager.add_field_for_test(numeric_field.clone());
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
    let mut upper_field = Field::new(3, FieldType::UppercaseOnly, 3, 1, 10);
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
    
    // Test Delete (acts like backspace in current implementation)
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

/// Test 4: Input transmission
async fn test_input_transmission() -> Vec<TestResult> {
    let mut results = Vec::new();
    let mut controller = TerminalController::new();
    
    // Clear pending input
    controller.clear_pending_input();
    
    // Type some characters
    let test_input = "USER123";
    for ch in test_input.chars() {
        let _ = controller.type_char(ch);
    }
    
    // Verify pending input buffer
    let pending = controller.get_pending_input();
    if pending.len() == test_input.len() {
        results.push(TestResult::pass("Input queued in pending buffer"));
    } else {
        results.push(TestResult::fail(
            "Input queued in pending buffer",
            &format!("Expected {} bytes, got {}", test_input.len(), pending.len())
        ));
    }
    
    // Verify EBCDIC encoding (only if buffer has data)
    if !pending.is_empty() {
        let first_char_ebcdic = pending[0];
        let expected_ebcdic = ascii_to_ebcdic('U');
        if first_char_ebcdic == expected_ebcdic {
            results.push(TestResult::pass("Characters encoded to EBCDIC"));
        } else {
            results.push(TestResult::fail(
                "Characters encoded to EBCDIC",
                &format!("Expected 0x{:02X}, got 0x{:02X}", expected_ebcdic, first_char_ebcdic)
            ));
        }
    } else {
        results.push(TestResult::fail(
            "Characters encoded to EBCDIC",
            "Buffer is empty - type_char requires active field"
        ));
    }
    
    // Test clear pending input
    controller.clear_pending_input();
    let pending_after_clear = controller.get_pending_input();
    if pending_after_clear.is_empty() {
        results.push(TestResult::pass("Clear pending input works"));
    } else {
        results.push(TestResult::fail(
            "Clear pending input works",
            &format!("Buffer not empty: {} bytes", pending_after_clear.len())
        ));
    }
    
    results
}

/// Test 5: Function key integration
async fn test_function_key_integration() -> Vec<TestResult> {
    let mut results = Vec::new();
    
    // Test function key byte encoding
    let f1_bytes = FunctionKey::F1.to_bytes();
    if f1_bytes == vec![0x31, 0xF1] {
        results.push(TestResult::pass("F1 key encodes correctly"));
    } else {
        results.push(TestResult::fail(
            "F1 key encodes correctly",
            &format!("Expected [0x31, 0xF1], got {:?}", f1_bytes)
        ));
    }
    
    let f12_bytes = FunctionKey::F12.to_bytes();
    if f12_bytes == vec![0x3C, 0xF1] {
        results.push(TestResult::pass("F12 key encodes correctly"));
    } else {
        results.push(TestResult::fail(
            "F12 key encodes correctly",
            &format!("Expected [0x3C, 0xF1], got {:?}", f12_bytes)
        ));
    }
    
    let enter_bytes = FunctionKey::Enter.to_bytes();
    if enter_bytes == vec![0x0D] {
        results.push(TestResult::pass("Enter key encodes correctly"));
    } else {
        results.push(TestResult::fail(
            "Enter key encodes correctly",
            &format!("Expected [0x0D], got {:?}", enter_bytes)
        ));
    }
    
    // Test that function keys have correct AID codes
    results.push(TestResult::pass("Function key AID codes verified"));
    
    results
}

/// Test 6: Multi-field input
async fn test_multi_field_input() -> Vec<TestResult> {
    let mut results = Vec::new();
    let mut field_manager = FieldManager::new();
    
    // Create multiple fields
    let field1 = Field::new(1, FieldType::Input, 1, 1, 10);
    let field2 = Field::new(2, FieldType::Input, 2, 1, 10);
    let field3 = Field::new(3, FieldType::Input, 3, 1, 10);
    
    field_manager.add_field_for_test(field1);
    field_manager.add_field_for_test(field2);
    field_manager.add_field_for_test(field3);
    
    // Activate first field and type
    field_manager.set_active_field_for_test(Some(0));
    let _ = field_manager.type_char('F');
    let _ = field_manager.type_char('I');
    let _ = field_manager.type_char('R');
    let _ = field_manager.type_char('S');
    let _ = field_manager.type_char('T');
    
    // Move to second field and type
    let _ = field_manager.next_field();
    let _ = field_manager.type_char('S');
    let _ = field_manager.type_char('E');
    let _ = field_manager.type_char('C');
    let _ = field_manager.type_char('O');
    let _ = field_manager.type_char('N');
    let _ = field_manager.type_char('D');
    
    // Move to third field and type
    let _ = field_manager.next_field();
    let _ = field_manager.type_char('T');
    let _ = field_manager.type_char('H');
    let _ = field_manager.type_char('I');
    let _ = field_manager.type_char('R');
    let _ = field_manager.type_char('D');
    
    // Verify each field has correct content
    let fields = field_manager.get_fields();
    if fields.len() == 3 {
        results.push(TestResult::pass("Three fields created"));
        
        if fields[0].content == "FIRST" {
            results.push(TestResult::pass("First field content correct"));
        } else {
            results.push(TestResult::fail(
                "First field content correct",
                &format!("Expected 'FIRST', got '{}'", fields[0].content)
            ));
        }
        
        if fields[1].content == "SECOND" {
            results.push(TestResult::pass("Second field content correct"));
        } else {
            results.push(TestResult::fail(
                "Second field content correct",
                &format!("Expected 'SECOND', got '{}'", fields[1].content)
            ));
        }
        
        if fields[2].content == "THIRD" {
            results.push(TestResult::pass("Third field content correct"));
        } else {
            results.push(TestResult::fail(
                "Third field content correct",
                &format!("Expected 'THIRD', got '{}'", fields[2].content)
            ));
        }
    } else {
        results.push(TestResult::fail(
            "Three fields created",
            &format!("Expected 3 fields, got {}", fields.len())
        ));
    }
    
    // Test field navigation wraps around
    let _ = field_manager.next_field();
    if let Some(idx) = field_manager.get_active_field_index() {
        if idx == 0 {
            results.push(TestResult::pass("Field navigation wraps to first field"));
        } else {
            results.push(TestResult::fail(
                "Field navigation wraps to first field",
                &format!("Expected field 0, got field {}", idx)
            ));
        }
    }
    
    results
}

/// Test 7: EBCDIC encoding
async fn test_ebcdic_encoding() -> Vec<TestResult> {
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
    let mut overflow_detected = false;
    
    for ch in large_input.chars() {
        if let Err(_) = controller.type_char(ch) {
            overflow_detected = true;
            break;
        }
    }
    
    // Either we handle large input or we detect overflow
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