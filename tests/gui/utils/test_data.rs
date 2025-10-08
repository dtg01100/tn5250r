//! Test data generation utilities
//!
//! This module provides functions for generating test data used in GUI tests,
//! including mock terminal content, connection strings, and user inputs.

use std::collections::HashMap;

/// Generate mock terminal content for testing
pub fn generate_mock_terminal_content(lines: usize) -> String {
    let mut content = String::new();
    for i in 0..lines {
        content.push_str(&format!("Line {}: Mock terminal content for GUI testing\n", i + 1));
    }
    content
}

/// Generate a set of test connection strings
pub fn generate_test_connection_strings() -> Vec<String> {
    vec![
        "127.0.0.1:23".to_string(),
        "localhost:5250".to_string(),
        "test.example.com:23".to_string(),
        "192.168.1.100:5250".to_string(),
    ]
}

/// Generate test user credentials
pub fn generate_test_credentials() -> Vec<HashMap<String, String>> {
    vec![
        {
            let mut creds = HashMap::new();
            creds.insert("username".to_string(), "testuser".to_string());
            creds.insert("password".to_string(), "testpass".to_string());
            creds
        },
        {
            let mut creds = HashMap::new();
            creds.insert("username".to_string(), "admin".to_string());
            creds.insert("password".to_string(), "admin123".to_string());
            creds
        },
    ]
}

/// Generate test input sequences
pub fn generate_test_inputs() -> Vec<String> {
    vec![
        "Hello World".to_string(),
        "Test input 123".to_string(),
        "Special chars: !@#$%^&*()".to_string(),
        "".to_string(), // Empty input
        "A".repeat(100), // Long input
    ]
}

/// Generate mock field data for 5250/3270 testing
pub fn generate_mock_field_data() -> Vec<HashMap<String, String>> {
    vec![
        {
            let mut field = HashMap::new();
            field.insert("type".to_string(), "input".to_string());
            field.insert("value".to_string(), "test value".to_string());
            field.insert("row".to_string(), "5".to_string());
            field.insert("col".to_string(), "10".to_string());
            field
        },
        {
            let mut field = HashMap::new();
            field.insert("type".to_string(), "protected".to_string());
            field.insert("value".to_string(), "READ ONLY".to_string());
            field.insert("row".to_string(), "1".to_string());
            field.insert("col".to_string(), "1".to_string());
            field
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mock_terminal_content() {
        let content = generate_mock_terminal_content(3);
        assert!(content.contains("Line 1:"));
        assert!(content.contains("Line 2:"));
        assert!(content.contains("Line 3:"));
    }

    #[test]
    fn test_generate_test_connection_strings() {
        let strings = generate_test_connection_strings();
        assert_eq!(strings.len(), 4);
        assert!(strings[0].contains(":"));
    }

    #[test]
    fn test_generate_test_credentials() {
        let creds = generate_test_credentials();
        assert_eq!(creds.len(), 2);
        assert_eq!(creds[0]["username"], "testuser");
    }
}