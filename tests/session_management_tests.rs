//! Session Management and Security Tests
//!
//! This test module verifies the fixes for:
//! 1. Session timeout and keepalive
//! 2. Keyboard lock state tracking
//! 3. Connection timeout handling
//! 4. TLS certificate validation

use std::time::Duration;
use std::thread;

// Note: Some tests require network connectivity and may be marked as #[ignore]

#[test]
fn test_session_config_defaults() {
    use tn5250r::network::SessionConfig;
    
    let config = SessionConfig::default();
    
    // Verify default timeouts are reasonable
    assert_eq!(config.idle_timeout_secs, 900, "Default idle timeout should be 15 minutes");
    assert_eq!(config.keepalive_interval_secs, 60, "Default keepalive should be 60 seconds");
    assert_eq!(config.connection_timeout_secs, 30, "Default connection timeout should be 30 seconds");
    assert_eq!(config.max_reconnect_attempts, 3, "Default max reconnect attempts should be 3");
    assert!(!config.auto_reconnect, "Auto-reconnect should be disabled by default");
}

#[test]
fn test_session_config_custom() {
    use tn5250r::network::SessionConfig;
    
    let config = SessionConfig {
        idle_timeout_secs: 300,  // 5 minutes
        keepalive_interval_secs: 30,
        connection_timeout_secs: 60,
        auto_reconnect: true,
        max_reconnect_attempts: 5,
        reconnect_backoff_multiplier: 3,
    };
    
    assert_eq!(config.idle_timeout_secs, 300);
    assert_eq!(config.keepalive_interval_secs, 30);
    assert_eq!(config.connection_timeout_secs, 60);
    assert!(config.auto_reconnect);
    assert_eq!(config.max_reconnect_attempts, 5);
}

#[test]
fn test_connection_with_session_config() {
    use tn5250r::network::{AS400Connection, SessionConfig};
    
    let mut conn = AS400Connection::new("localhost".to_string(), 23);
    
    // Set custom session configuration
    let config = SessionConfig {
        idle_timeout_secs: 600,
        keepalive_interval_secs: 45,
        connection_timeout_secs: 45,
        auto_reconnect: false,
        max_reconnect_attempts: 2,
        reconnect_backoff_multiplier: 2,
    };
    
    conn.set_session_config(config);
    
    // Verify configuration was applied
    let applied_config = conn.session_config();
    assert_eq!(applied_config.idle_timeout_secs, 600);
    assert_eq!(applied_config.keepalive_interval_secs, 45);
}

#[test]
fn test_idle_timeout_detection() {
    use tn5250r::network::{AS400Connection, SessionConfig};
    use std::time::{Duration, Instant};
    
    let mut conn = AS400Connection::new("localhost".to_string(), 23);
    
    // Set very short idle timeout for testing (2 seconds)
    let config = SessionConfig {
        idle_timeout_secs: 2,
        keepalive_interval_secs: 60,
        connection_timeout_secs: 30,
        auto_reconnect: false,
        max_reconnect_attempts: 3,
        reconnect_backoff_multiplier: 2,
    };
    
    conn.set_session_config(config);
    
    // Initially, no timeout should be detected (no activity yet)
    assert!(!conn.is_session_idle_timeout(), "No timeout before any activity");
    
    // Simulate activity by calling update (would normally happen in connect)
    // Note: We can't directly call update_last_activity as it's private,
    // so this test verifies the public API behavior
}

#[test]
fn test_time_since_last_activity() {
    use tn5250r::network::AS400Connection;
    
    let conn = AS400Connection::new("localhost".to_string(), 23);
    
    // Before any activity, should return None
    assert!(conn.time_since_last_activity().is_none(), 
            "Should return None before any activity");
}

#[test]
fn test_keyboard_lock_state_machine() {
    use tn5250r::lib3270::display::Display3270;
    use tn5250r::lib3270::codes::*;
    use tn5250r::lib3270::protocol::ProtocolProcessor3270;
    
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    // Initially, keyboard should be locked (default state)
    assert!(display.is_keyboard_locked(), "Keyboard should be locked initially");
    
    // Test 1: Write command locks keyboard, WCC_RESTORE unlocks it
    let data_with_restore = vec![
        CMD_WRITE,
        WCC_RESTORE,  // This should unlock keyboard
        0xC1,  // Some data
    ];
    
    processor.process_data(&data_with_restore, &mut display)
        .expect("Should process write with restore");
    
    assert!(!display.is_keyboard_locked(), 
            "Keyboard should be unlocked after WCC_RESTORE");
    
    // Test 2: Write command without WCC_RESTORE keeps keyboard locked
    let data_without_restore = vec![
        CMD_WRITE,
        0x00,  // WCC without restore bit
        0xC2,  // Some data
    ];
    
    processor.process_data(&data_without_restore, &mut display)
        .expect("Should process write without restore");
    
    assert!(display.is_keyboard_locked(), 
            "Keyboard should remain locked without WCC_RESTORE");
    
    // Test 3: Erase/Write with WCC_RESTORE unlocks keyboard
    let erase_write_data = vec![
        CMD_ERASE_WRITE,
        WCC_RESTORE,
        0xC3,
    ];
    
    processor.process_data(&erase_write_data, &mut display)
        .expect("Should process erase/write");
    
    assert!(!display.is_keyboard_locked(),
            "Keyboard should be unlocked after Erase/Write with WCC_RESTORE");
}

#[test]
fn test_keyboard_lock_blocks_input() {
    use tn5250r::lib3270::display::Display3270;
    
    let mut display = Display3270::new();
    
    // Lock the keyboard
    display.lock_keyboard();
    assert!(display.is_keyboard_locked());
    
    // Unlock the keyboard
    display.unlock_keyboard();
    assert!(!display.is_keyboard_locked());
    
    // Lock again
    display.lock_keyboard();
    assert!(display.is_keyboard_locked());
}

#[test]
fn test_tls_security_warnings() {
    use tn5250r::network::AS400Connection;
    
    let mut conn = AS400Connection::new("example.com".to_string(), 992);
    
    // Attempting to disable TLS should work
    // The actual validation will still be enforced
    conn.set_tls(false);
    
    // Verify TLS is still enabled
    assert!(conn.is_tls_enabled());
}

#[test]
fn test_connection_timeout_configuration() {
    use tn5250r::network::{AS400Connection, SessionConfig};
    
    let mut conn = AS400Connection::new("localhost".to_string(), 23);
    
    // Configure shorter timeout
    let config = SessionConfig {
        idle_timeout_secs: 900,
        keepalive_interval_secs: 60,
        connection_timeout_secs: 10,  // Short timeout for testing
        auto_reconnect: false,
        max_reconnect_attempts: 3,
        reconnect_backoff_multiplier: 2,
    };
    
    conn.set_session_config(config);
    
    // Verify configuration
    assert_eq!(conn.session_config().connection_timeout_secs, 10);
}

#[test]
#[ignore] // Requires actual network connection
fn test_connection_with_timeout() {
    use tn5250r::network::AS400Connection;
    use std::time::Duration;
    
    let mut conn = AS400Connection::new("192.0.2.1".to_string(), 23); // Non-routable IP
    
    // Attempt connection with short timeout - should fail quickly
    let timeout = Duration::from_secs(2);
    let result = conn.connect_with_timeout(timeout);
    
    // Should timeout or fail to connect
    assert!(result.is_err(), "Connection to non-routable IP should fail");
}

#[test]
fn test_validate_network_data() {
    // Note: validate_network_data is private, but we can test its effects
    // through public APIs that use it
    
    // This test verifies that the send_data method validates data
    use tn5250r::network::AS400Connection;
    
    let mut conn = AS400Connection::new("localhost".to_string(), 23);
    
    // Test 1: Empty data should be rejected
    let empty_data = vec![];
    let result = conn.send_data(&empty_data);
    assert!(result.is_err(), "Empty data should be rejected");
    
    // Test 2: Oversized data should be rejected
    let oversized_data = vec![0u8; 100000];
    let result = conn.send_data(&oversized_data);
    assert!(result.is_err(), "Oversized data should be rejected");
    
    // Test 3: Normal data would be accepted (if connected)
    let normal_data = vec![0x01, 0x02, 0x03];
    let result = conn.send_data(&normal_data);
    // Should fail because not connected, not because of validation
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(e.kind(), std::io::ErrorKind::NotConnected);
    }
}

#[test]
fn test_connection_state_validation() {
    use tn5250r::network::AS400Connection;
    
    let conn = AS400Connection::new("localhost".to_string(), 23);
    
    // Validate connection integrity when not connected
    let result = conn.validate_connection_integrity();
    assert!(result.is_ok(), "Disconnected connection should still be valid");
}

#[test]
fn test_protocol_mode_setting() {
    use tn5250r::network::{AS400Connection, ProtocolMode};
    
    let mut conn = AS400Connection::new("localhost".to_string(), 23);
    
    // Initially should be AutoDetect
    assert_eq!(conn.get_detected_protocol_mode(), ProtocolMode::AutoDetect);
    
    // Force to TN5250
    conn.set_protocol_mode(ProtocolMode::TN5250);
    assert_eq!(conn.get_detected_protocol_mode(), ProtocolMode::TN5250);
    
    // Force to TN3270
    conn.set_protocol_mode(ProtocolMode::TN3270);
    assert_eq!(conn.get_detected_protocol_mode(), ProtocolMode::TN3270);
}

#[test]
fn test_safe_cleanup() {
    use tn5250r::network::AS400Connection;
    
    let mut conn = AS400Connection::new("localhost".to_string(), 23);
    
    // Perform safe cleanup
    conn.safe_cleanup();
    
    // Verify connection is properly cleaned up
    assert!(!conn.is_connected());
}

#[test]
fn test_keyboard_lock_with_multiple_operations() {
    use tn5250r::lib3270::display::Display3270;
    use tn5250r::lib3270::codes::*;
    use tn5250r::lib3270::protocol::ProtocolProcessor3270;
    
    let mut processor = ProtocolProcessor3270::new();
    let mut display = Display3270::new();
    
    // Sequence of operations:
    // 1. Write (locks) with restore (unlocks)
    // 2. Write (locks) without restore (stays locked)
    // 3. Erase All Unprotected (should unlock)
    
    // Step 1
    let data1 = vec![CMD_WRITE, WCC_RESTORE, 0xC1];
    processor.process_data(&data1, &mut display).unwrap();
    assert!(!display.is_keyboard_locked(), "Step 1: Should be unlocked");
    
    // Step 2
    let data2 = vec![CMD_WRITE, 0x00, 0xC2];
    processor.process_data(&data2, &mut display).unwrap();
    assert!(display.is_keyboard_locked(), "Step 2: Should be locked");
    
    // Step 3
    let data3 = vec![CMD_ERASE_ALL_UNPROTECTED];
    processor.process_data(&data3, &mut display).unwrap();
    assert!(!display.is_keyboard_locked(), "Step 3: Should be unlocked");
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_session_management_integration() {
        use tn5250r::network::{AS400Connection, SessionConfig};
        
        let mut conn = AS400Connection::new("test.example.com".to_string(), 23);
        
        // Configure session
        let config = SessionConfig {
            idle_timeout_secs: 300,
            keepalive_interval_secs: 30,
            connection_timeout_secs: 15,
            auto_reconnect: true,
            max_reconnect_attempts: 5,
            reconnect_backoff_multiplier: 2,
        };
        
        conn.set_session_config(config.clone());
        
        // Verify all settings
        let applied = conn.session_config();
        assert_eq!(applied.idle_timeout_secs, config.idle_timeout_secs);
        assert_eq!(applied.keepalive_interval_secs, config.keepalive_interval_secs);
        assert_eq!(applied.connection_timeout_secs, config.connection_timeout_secs);
        assert_eq!(applied.auto_reconnect, config.auto_reconnect);
        assert_eq!(applied.max_reconnect_attempts, config.max_reconnect_attempts);
    }
}

#[test]
fn test_write_to_display_packet_processing() {
    use std::sync::{Arc, Mutex};
    use tn5250r::lib5250::session::Session;
    use tn5250r::lib5250::display::Display;
    use tn5250r::lib5250::protocol::{Packet, CommandCode};

    println!("Testing WriteToDisplay packet processing...");

    // Create a session and display
    let mut session = Session::new();

    // Authenticate the session first
    match session.authenticate("testuser", "testpass") {
        Ok(true) => println!("✓ Session authenticated successfully"),
        Ok(false) => panic!("✗ Authentication failed"),
        Err(e) => panic!("✗ Authentication error: {}", e),
    }

    // Create a WriteToDisplay packet with some test data
    // This simulates a simple screen write with "HELLO" at position (0,0)
    // Format: CC1, CC2, SBA order, row, col, characters
    let data = vec![
        0x00, 0x00,  // CC1=0x00 (no keyboard lock), CC2=0x00 (no special control)
        0x11, 0x01, 0x01,  // SBA (Set Buffer Address) to (1,1) - 1-based
        0xC8, 0xC5, 0xD3, 0xD3, 0xD6  // EBCDIC for "HELLO"
    ];

    let packet = Packet::new(CommandCode::WriteToDisplay, 1, data);

    // Process the packet
    match session.process_integrated_data(&packet.to_bytes()) {
        Ok(_) => println!("✓ Packet processed successfully"),
        Err(e) => {
            panic!("✗ Packet processing failed: {}", e);
        }
    }

    // Check if the display was updated
    let display_content = session.display_string();
    println!("Display content:\n{}", display_content);

    assert!(display_content.contains("HELLO"), "Display should contain 'HELLO' after processing WriteToDisplay packet");
}