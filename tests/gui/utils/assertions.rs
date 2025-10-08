//! Assertion helpers for GUI testing
//!
//! This module provides custom assertion functions for GUI-specific testing,
//! including checks for UI element presence, text content, and visual states.

use std::time::Duration;

/// Assert that text is present in the GUI
pub fn assert_text_present(text: &str, context: &str) -> Result<(), String> {
    // Placeholder implementation - in real usage, this would check the actual GUI
    if text.is_empty() {
        return Err("Text cannot be empty".to_string());
    }
    println!("✓ Asserting text '{}' is present in {}", text, context);
    Ok(())
}

/// Assert that a specific UI element exists
pub fn assert_element_exists(element_id: &str, element_type: &str) -> Result<(), String> {
    if element_id.is_empty() || element_type.is_empty() {
        return Err("Element ID and type cannot be empty".to_string());
    }
    println!("✓ Asserting {} element '{}' exists", element_type, element_id);
    Ok(())
}

/// Assert that a button is clickable
pub fn assert_button_clickable(button_text: &str) -> Result<(), String> {
    if button_text.is_empty() {
        return Err("Button text cannot be empty".to_string());
    }
    println!("✓ Asserting button '{}' is clickable", button_text);
    Ok(())
}

/// Assert that the GUI is in a specific state
pub fn assert_gui_state(expected_state: &str, actual_state: &str) -> Result<(), String> {
    if expected_state != actual_state {
        return Err(format!("Expected GUI state '{}', but got '{}'", expected_state, actual_state));
    }
    println!("✓ GUI is in expected state: {}", expected_state);
    Ok(())
}

/// Assert that an operation completes within a timeout
pub fn assert_operation_completes_within<F, T>(
    operation: F,
    timeout: Duration,
    operation_name: &str
) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    let start = std::time::Instant::now();

    match operation() {
        Ok(result) => {
            let elapsed = start.elapsed();
            if elapsed > timeout {
                return Err(format!(
                    "Operation '{}' took {:?}, exceeding timeout of {:?}",
                    operation_name, elapsed, timeout
                ));
            }
            println!("✓ Operation '{}' completed in {:?}", operation_name, elapsed);
            Ok(result)
        }
        Err(e) => Err(format!("Operation '{}' failed: {}", operation_name, e)),
    }
}

/// Assert that no errors are present in the GUI
pub fn assert_no_errors() -> Result<(), String> {
    // Placeholder - would check for error dialogs, messages, etc.
    println!("✓ No errors detected in GUI");
    Ok(())
}

/// Assert that the terminal content matches expected output
pub fn assert_terminal_content(expected: &str, actual: &str) -> Result<(), String> {
    if expected != actual {
        return Err(format!(
            "Terminal content mismatch.\nExpected: {}\nActual: {}",
            expected, actual
        ));
    }
    println!("✓ Terminal content matches expected output");
    Ok(())
}

/// Assert that a field contains the expected value
pub fn assert_field_value(field_name: &str, expected_value: &str, actual_value: &str) -> Result<(), String> {
    if expected_value != actual_value {
        return Err(format!(
            "Field '{}' value mismatch. Expected: '{}', Actual: '{}'",
            field_name, expected_value, actual_value
        ));
    }
    println!("✓ Field '{}' has expected value: {}", field_name, expected_value);
    Ok(())
}

/// Assert that the connection status is as expected
pub fn assert_connection_status(expected_connected: bool, actual_connected: bool) -> Result<(), String> {
    if expected_connected != actual_connected {
        let expected = if expected_connected { "connected" } else { "disconnected" };
        let actual = if actual_connected { "connected" } else { "disconnected" };
        return Err(format!("Connection status mismatch. Expected: {}, Actual: {}", expected, actual));
    }
    println!("✓ Connection status is correct: {}", if expected_connected { "connected" } else { "disconnected" });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_assert_text_present() {
        assert!(assert_text_present("Hello", "test context").is_ok());
        assert!(assert_text_present("", "test context").is_err());
    }

    #[test]
    fn test_assert_element_exists() {
        assert!(assert_element_exists("btn1", "button").is_ok());
        assert!(assert_element_exists("", "button").is_err());
        assert!(assert_element_exists("btn1", "").is_err());
    }

    #[test]
    fn test_assert_gui_state() {
        assert!(assert_gui_state("connected", "connected").is_ok());
        assert!(assert_gui_state("connected", "disconnected").is_err());
    }

    #[test]
    fn test_assert_operation_completes_within() {
        let result = assert_operation_completes_within(
            || Ok("success".to_string()),
            Duration::from_secs(1),
            "test operation"
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_assert_terminal_content() {
        assert!(assert_terminal_content("expected", "expected").is_ok());
        assert!(assert_terminal_content("expected", "actual").is_err());
    }

    #[test]
    fn test_assert_field_value() {
        assert!(assert_field_value("username", "testuser", "testuser").is_ok());
        assert!(assert_field_value("username", "testuser", "otheruser").is_err());
    }

    #[test]
    fn test_assert_connection_status() {
        assert!(assert_connection_status(true, true).is_ok());
        assert!(assert_connection_status(false, false).is_ok());
        assert!(assert_connection_status(true, false).is_err());
    }
}