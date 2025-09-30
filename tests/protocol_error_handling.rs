//! Unit tests for protocol error handling
//! 
//! This module tests the comprehensive error handling for protocol mode switches
//! throughout the codebase.

use tn5250r::error::{ProtocolError, ConfigError, TN5250Error};
use tn5250r::config::{SessionConfig, parse_protocol_string};
use tn5250r::network::ProtocolMode;

#[test]
fn test_unsupported_protocol_error() {
    let error = ProtocolError::UnsupportedProtocol {
        protocol: "invalid".to_string(),
        reason: "Protocol not recognized".to_string(),
    };
    
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Unsupported protocol"));
    assert!(error_msg.contains("invalid"));
}

#[test]
fn test_protocol_mismatch_error() {
    let error = ProtocolError::ProtocolMismatch {
        configured: "tn5250".to_string(),
        detected: "tn3270".to_string(),
    };
    
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Protocol mismatch"));
    assert!(error_msg.contains("tn5250"));
    assert!(error_msg.contains("tn3270"));
}

#[test]
fn test_protocol_switch_failed_error() {
    let error = ProtocolError::ProtocolSwitchFailed {
        from: "tn5250".to_string(),
        to: "tn3270".to_string(),
        reason: "Network connection lost".to_string(),
    };
    
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Failed to switch protocol"));
    assert!(error_msg.contains("tn5250"));
    assert!(error_msg.contains("tn3270"));
    assert!(error_msg.contains("Network connection lost"));
}

#[test]
fn test_invalid_protocol_configuration_error() {
    let error = ProtocolError::InvalidProtocolConfiguration {
        parameter: "connection.protocol".to_string(),
        value: "invalid".to_string(),
        reason: "Must be 'auto', 'tn5250', or 'tn3270'".to_string(),
    };
    
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Invalid protocol configuration"));
    assert!(error_msg.contains("connection.protocol"));
    assert!(error_msg.contains("invalid"));
}

#[test]
fn test_set_protocol_mode_valid() {
    let mut config = SessionConfig::new("test.json".to_string(), "test".to_string());
    
    // Test valid protocol modes
    assert!(config.set_protocol_mode("auto").is_ok());
    assert_eq!(config.get_protocol_mode(), "auto");
    
    assert!(config.set_protocol_mode("tn5250").is_ok());
    assert_eq!(config.get_protocol_mode(), "tn5250");
    
    assert!(config.set_protocol_mode("tn3270").is_ok());
    assert_eq!(config.get_protocol_mode(), "tn3270");
}

#[test]
fn test_set_protocol_mode_invalid() {
    let mut config = SessionConfig::new("test.json".to_string(), "test".to_string());
    
    // Test invalid protocol mode
    let result = config.set_protocol_mode("invalid");
    assert!(result.is_err());
    
    if let Err(TN5250Error::Protocol(ProtocolError::InvalidProtocolConfiguration { parameter, value, reason })) = result {
        assert_eq!(parameter, "connection.protocol");
        assert_eq!(value, "invalid");
        assert!(reason.contains("Must be"));
    } else {
        panic!("Expected InvalidProtocolConfiguration error");
    }
}

#[test]
fn test_set_terminal_type_valid() {
    let mut config = SessionConfig::new("test.json".to_string(), "test".to_string());
    
    // Test valid 5250 terminal types
    assert!(config.set_terminal_type("IBM-3179-2").is_ok());
    assert!(config.set_terminal_type("IBM-3196-A1").is_ok());
    assert!(config.set_terminal_type("IBM-5251-11").is_ok());
    
    // Test valid 3270 terminal types
    assert!(config.set_terminal_type("IBM-3278-2").is_ok());
    assert!(config.set_terminal_type("IBM-3279-2").is_ok());
}

#[test]
fn test_set_terminal_type_invalid() {
    let mut config = SessionConfig::new("test.json".to_string(), "test".to_string());
    
    // Test invalid terminal type
    let result = config.set_terminal_type("INVALID-TYPE");
    assert!(result.is_err());
    
    if let Err(TN5250Error::Config(ConfigError::InvalidParameter { parameter, value, reason })) = result {
        assert_eq!(parameter, "terminal.type");
        assert_eq!(value, "INVALID-TYPE");
        assert!(reason.contains("valid"));
    } else {
        panic!("Expected InvalidParameter error");
    }
}

#[test]
fn test_validate_protocol_terminal_combination_valid() {
    let mut config = SessionConfig::new("test.json".to_string(), "test".to_string());
    
    // Auto mode accepts any terminal type
    config.set_protocol_mode("auto").unwrap();
    config.set_terminal_type("IBM-3179-2").unwrap();
    assert!(config.validate_protocol_terminal_combination().is_ok());
    
    config.set_terminal_type("IBM-3278-2").unwrap();
    assert!(config.validate_protocol_terminal_combination().is_ok());
    
    // TN5250 with 5250 terminal type
    config.set_protocol_mode("tn5250").unwrap();
    config.set_terminal_type("IBM-3179-2").unwrap();
    assert!(config.validate_protocol_terminal_combination().is_ok());
    
    // TN3270 with 3270 terminal type
    config.set_protocol_mode("tn3270").unwrap();
    config.set_terminal_type("IBM-3278-2").unwrap();
    assert!(config.validate_protocol_terminal_combination().is_ok());
}

#[test]
fn test_validate_protocol_terminal_combination_invalid() {
    let mut config = SessionConfig::new("test.json".to_string(), "test".to_string());
    
    // TN5250 with 3270 terminal type should fail
    config.set_protocol_mode("tn5250").unwrap();
    config.set_terminal_type("IBM-3278-2").unwrap();
    let result = config.validate_protocol_terminal_combination();
    assert!(result.is_err());
    
    if let Err(TN5250Error::Protocol(ProtocolError::ProtocolMismatch { configured, detected })) = result {
        assert!(configured.contains("TN5250"));
        assert!(detected.contains("incompatible"));
    } else {
        panic!("Expected ProtocolMismatch error");
    }
    
    // TN3270 with 5250 terminal type should fail
    config.set_protocol_mode("tn3270").unwrap();
    config.set_terminal_type("IBM-3179-2").unwrap();
    let result = config.validate_protocol_terminal_combination();
    assert!(result.is_err());
    
    if let Err(TN5250Error::Protocol(ProtocolError::ProtocolMismatch { configured, detected })) = result {
        assert!(configured.contains("TN3270"));
        assert!(detected.contains("incompatible"));
    } else {
        panic!("Expected ProtocolMismatch error");
    }
}

#[test]
fn test_parse_protocol_string_valid() {
    // Test valid protocol strings
    assert_eq!(parse_protocol_string("auto").unwrap(), ProtocolMode::AutoDetect);
    assert_eq!(parse_protocol_string("tn5250").unwrap(), ProtocolMode::TN5250);
    assert_eq!(parse_protocol_string("5250").unwrap(), ProtocolMode::TN5250);
    assert_eq!(parse_protocol_string("tn3270").unwrap(), ProtocolMode::TN3270);
    assert_eq!(parse_protocol_string("3270").unwrap(), ProtocolMode::TN3270);
    assert_eq!(parse_protocol_string("nvt").unwrap(), ProtocolMode::NVT);
    
    // Test case insensitivity
    assert_eq!(parse_protocol_string("AUTO").unwrap(), ProtocolMode::AutoDetect);
    assert_eq!(parse_protocol_string("TN5250").unwrap(), ProtocolMode::TN5250);
}

#[test]
fn test_parse_protocol_string_invalid() {
    let result = parse_protocol_string("invalid");
    assert!(result.is_err());
    
    if let Err(TN5250Error::Protocol(ProtocolError::UnsupportedProtocol { protocol, reason })) = result {
        assert_eq!(protocol, "invalid");
        assert!(reason.contains("Must be"));
    } else {
        panic!("Expected UnsupportedProtocol error");
    }
}

#[test]
fn test_error_conversion() {
    // Test that protocol errors convert to TN5250Error correctly
    let protocol_error = ProtocolError::UnsupportedProtocol {
        protocol: "test".to_string(),
        reason: "test reason".to_string(),
    };
    
    let tn5250_error: TN5250Error = protocol_error.into();
    
    match tn5250_error {
        TN5250Error::Protocol(_) => {
            // Correct conversion
        }
        _ => panic!("Expected Protocol variant"),
    }
}

#[test]
fn test_error_display_formatting() {
    // Test that error messages are user-friendly
    let error = TN5250Error::Protocol(ProtocolError::ProtocolSwitchFailed {
        from: "tn5250".to_string(),
        to: "tn3270".to_string(),
        reason: "Connection lost during switch".to_string(),
    });
    
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Protocol error"));
    assert!(error_msg.contains("Failed to switch protocol"));
    assert!(!error_msg.is_empty());
}

#[test]
fn test_protocol_validation_workflow() {
    // Test a complete workflow of protocol configuration and validation
    let mut config = SessionConfig::new("test.json".to_string(), "test".to_string());
    
    // Step 1: Set protocol mode
    config.set_protocol_mode("tn5250").expect("Should set protocol mode");
    
    // Step 2: Set compatible terminal type
    config.set_terminal_type("IBM-3179-2").expect("Should set terminal type");
    
    // Step 3: Validate combination
    config.validate_protocol_terminal_combination().expect("Should validate successfully");
    
    // Step 4: Try to set incompatible terminal type
    config.set_terminal_type("IBM-3278-2").expect("Should set terminal type");
    
    // Step 5: Validation should now fail
    let result = config.validate_protocol_terminal_combination();
    assert!(result.is_err(), "Validation should fail for incompatible combination");
}

#[test]
fn test_error_recovery_scenario() {
    // Test that errors can be recovered from
    let mut config = SessionConfig::new("test.json".to_string(), "test".to_string());
    
    // Try invalid protocol mode
    let _ = config.set_protocol_mode("invalid");
    
    // Should still be able to set valid protocol mode
    assert!(config.set_protocol_mode("tn5250").is_ok());
    assert_eq!(config.get_protocol_mode(), "tn5250");
    
    // Try invalid terminal type
    let _ = config.set_terminal_type("INVALID");
    
    // Should still be able to set valid terminal type
    assert!(config.set_terminal_type("IBM-3179-2").is_ok());
    assert_eq!(config.get_terminal_type(), "IBM-3179-2");
}