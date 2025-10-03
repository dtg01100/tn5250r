//! TN3270E Integration Tests
//!
//! Comprehensive test suite to validate TN3270E telnet negotiation,
//! device type negotiation, session binding, and end-to-end functionality.

use std::time::Duration;
use tn5250r::telnet_negotiation::{TelnetNegotiator, TelnetOption, TelnetCommand, TN3270EDeviceType, TN3270ESessionState};
use tn5250r::network::ProtocolMode;
use tn5250r::config::SessionConfig;

/// Test TN3270E option negotiation
#[test]
fn test_tn3270e_option_negotiation() {
    let mut negotiator = TelnetNegotiator::new();

    // TN3270E should be in initial negotiation
    let initial = negotiator.generate_initial_negotiation();
    assert!(!initial.is_empty());

    // Check that TN3270E WILL is present
    let mut found_tn3270e_will = false;
    let mut pos = 0;
    while pos + 2 < initial.len() {
        if initial[pos] == TelnetCommand::IAC as u8 &&
           initial[pos + 1] == TelnetCommand::WILL as u8 &&
           initial[pos + 2] == TelnetOption::TN3270E as u8 {
            found_tn3270e_will = true;
            break;
        }
        pos += 1;
    }
    assert!(found_tn3270e_will, "TN3270E WILL should be in initial negotiation");

    // Initial state should be NotConnected
    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::NotConnected);
}

/// Test TN3270E device type negotiation flow
#[test]
fn test_tn3270e_device_type_negotiation() {
    let mut negotiator = TelnetNegotiator::new();

    // Simulate server accepting TN3270E
    let server_accept = vec![
        TelnetCommand::IAC as u8,
        TelnetCommand::DO as u8,
        TelnetOption::TN3270E as u8,
    ];
    negotiator.process_incoming_data(&server_accept);

    // Should be in TN3270ENegotiated state
    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::TN3270ENegotiated);

    // Simulate server requesting device type
    let device_request = vec![
        TelnetCommand::IAC as u8,
        TelnetCommand::SB as u8,
        TelnetOption::TN3270E as u8,
        2, // DEVICE-TYPE command
        0x82, // Model2Color
        TelnetCommand::IAC as u8,
        TelnetCommand::SE as u8,
    ];

    let response = negotiator.process_incoming_data(&device_request);

    // Should respond with IS command
    assert!(!response.is_empty());
    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::DeviceNegotiated);
    assert_eq!(negotiator.tn3270e_device_type(), Some(TN3270EDeviceType::Model2Color));
}

/// Test TN3270E session binding
#[test]
fn test_tn3270e_session_binding() {
    let mut negotiator = TelnetNegotiator::new();

    // Set up TN3270E and device type first
    let setup_sequence = vec![
        // Server accepts TN3270E
        vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::TN3270E as u8],
        // Server requests device type
        vec![TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8, 2, 0x82,
             TelnetCommand::IAC as u8, TelnetCommand::SE as u8],
    ];

    for msg in setup_sequence {
        negotiator.process_incoming_data(&msg);
    }

    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::DeviceNegotiated);

    // Now simulate BIND command with logical unit name
    let bind_command = vec![
        TelnetCommand::IAC as u8,
        TelnetCommand::SB as u8,
        TelnetOption::TN3270E as u8,
        6, // BIND command
        b'L', b'U', b'0', b'1', 0, // "LU01" + null
        TelnetCommand::IAC as u8,
        TelnetCommand::SE as u8,
    ];

    let response = negotiator.process_incoming_data(&bind_command);

    // Should be bound and have logical unit set
    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::Bound);
    assert_eq!(negotiator.logical_unit_name(), Some("LU01"));
    assert!(!response.is_empty()); // Should send BIND response
}

/// Test TN3270E session unbinding
#[test]
fn test_tn3270e_session_unbinding() {
    let mut negotiator = TelnetNegotiator::new();

    // Set up and bind session first
    let setup_and_bind = vec![
        // Server accepts TN3270E
        vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::TN3270E as u8],
        // Server requests device type
        vec![TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8, 2, 0x82,
             TelnetCommand::IAC as u8, TelnetCommand::SE as u8],
        // Server sends BIND
        vec![TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8, 6,
             b'L', b'U', b'0', b'1', 0, TelnetCommand::IAC as u8, TelnetCommand::SE as u8],
    ];

    for msg in setup_and_bind {
        negotiator.process_incoming_data(&msg);
    }

    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::Bound);
    assert_eq!(negotiator.logical_unit_name(), Some("LU01"));

    // Now simulate UNBIND command
    let unbind_command = vec![
        TelnetCommand::IAC as u8,
        TelnetCommand::SB as u8,
        TelnetOption::TN3270E as u8,
        7, // UNBIND command
        TelnetCommand::IAC as u8,
        TelnetCommand::SE as u8,
    ];

    let response = negotiator.process_incoming_data(&unbind_command);

    // Should be unbound and logical unit cleared
    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::Unbound);
    assert_eq!(negotiator.logical_unit_name(), None);
    assert!(!response.is_empty()); // Should send UNBIND response
}

/// Test TN3270E negotiation completion detection
#[test]
fn test_tn3270e_negotiation_completion() {
    let mut negotiator = TelnetNegotiator::new();

    // Initially not complete
    assert!(!negotiator.is_negotiation_complete());

    // Set up basic telnet negotiation (Binary, EOR, SGA)
    let basic_negotiation = vec![
        vec![TelnetCommand::IAC as u8, TelnetCommand::WILL as u8, TelnetOption::Binary as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::Binary as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::WILL as u8, TelnetOption::EndOfRecord as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::EndOfRecord as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::WILL as u8, TelnetOption::SuppressGoAhead as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::SuppressGoAhead as u8],
    ];

    for msg in basic_negotiation {
        negotiator.process_incoming_data(&msg);
    }

    // Still not complete - missing TN3270E binding
    assert!(!negotiator.is_negotiation_complete());

    // Now complete TN3270E negotiation
    let tn3270e_negotiation = vec![
        vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::TN3270E as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8, 2, 0x82,
             TelnetCommand::IAC as u8, TelnetCommand::SE as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8, 6,
             b'L', b'U', b'0', b'1', 0, TelnetCommand::IAC as u8, TelnetCommand::SE as u8],
    ];

    for msg in tn3270e_negotiation {
        negotiator.process_incoming_data(&msg);
    }

    // Now should be complete
    assert!(negotiator.is_negotiation_complete());
}

/// Test TN3270E device type screen dimensions
#[test]
fn test_tn3270e_device_screen_dimensions() {
    let mut negotiator = TelnetNegotiator::new();

    // Set up TN3270E and negotiate different device types
    let device_types = vec![
        (0x02, TN3270EDeviceType::Model2, (24, 80)),
        (0x03, TN3270EDeviceType::Model3, (32, 80)),
        (0x04, TN3270EDeviceType::Model4, (43, 80)),
        (0x05, TN3270EDeviceType::Model5, (27, 132)),
        (0x82, TN3270EDeviceType::Model2Color, (24, 80)),
        (0x83, TN3270EDeviceType::Model3Color, (32, 80)),
        (0x84, TN3270EDeviceType::Model4Color, (43, 80)),
        (0x85, TN3270EDeviceType::Model5Color, (27, 132)),
    ];

    for (device_code, expected_type, expected_dims) in device_types {
        let mut test_negotiator = TelnetNegotiator::new();

        // Set up TN3270E
        test_negotiator.process_incoming_data(&vec![
            TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::TN3270E as u8
        ]);

        // Negotiate device type
        test_negotiator.process_incoming_data(&vec![
            TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
            2, device_code, TelnetCommand::IAC as u8, TelnetCommand::SE as u8
        ]);

        // Bind session
        test_negotiator.process_incoming_data(&vec![
            TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
            6, b'L', b'U', b'0', b'1', 0, TelnetCommand::IAC as u8, TelnetCommand::SE as u8
        ]);

        // Verify device type and dimensions
        assert_eq!(test_negotiator.tn3270e_device_type(), Some(expected_type));
        assert_eq!(test_negotiator.get_screen_dimensions(), Some(expected_dims));
    }
}

/// Test TN3270E color support detection
#[test]
fn test_tn3270e_color_support() {
    let mut negotiator = TelnetNegotiator::new();

    // Set up TN3270E
    negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::TN3270E as u8
    ]);

    // Test monochrome device
    negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
        2, 0x02, TelnetCommand::IAC as u8, TelnetCommand::SE as u8 // Model2
    ]);
    negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
        6, b'L', b'U', b'0', b'1', 0, TelnetCommand::IAC as u8, TelnetCommand::SE as u8
    ]);

    assert!(!negotiator.supports_color());

    // Test color device
    let mut color_negotiator = TelnetNegotiator::new();
    color_negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::TN3270E as u8
    ]);
    color_negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
        2, 0x82, TelnetCommand::IAC as u8, TelnetCommand::SE as u8 // Model2Color
    ]);
    color_negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
        6, b'L', b'U', b'0', b'1', 0, TelnetCommand::IAC as u8, TelnetCommand::SE as u8
    ]);

    assert!(color_negotiator.supports_color());
}

/// Test TN3270E configuration integration
#[test]
fn test_tn3270e_configuration_integration() {
    let mut config = SessionConfig::new("test.json".to_string(), "tn3270e_session".to_string());

    // Set TN3270 protocol mode
    assert!(config.set_protocol_mode("tn3270").is_ok());
    assert_eq!(config.get_protocol_mode(), "tn3270");

    // Test TN3270E terminal types
    let tn3270e_types = [
        "IBM-3278-2", "IBM-3279-2", "IBM-3279-3",
        "IBM-3278-3", "IBM-3278-4", "IBM-3278-5"
    ];

    for terminal_type in &tn3270e_types {
        assert!(config.set_terminal_type(terminal_type).is_ok());
        assert!(config.validate_protocol_terminal_combination().is_ok());
    }
}

/// Test TN3270E session state transitions
#[test]
fn test_tn3270e_session_state_transitions() {
    let mut negotiator = TelnetNegotiator::new();

    // Start: NotConnected
    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::NotConnected);

    // TN3270E negotiated: TN3270ENegotiated
    negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::TN3270E as u8
    ]);
    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::TN3270ENegotiated);

    // Device negotiated: DeviceNegotiated
    negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
        2, 0x82, TelnetCommand::IAC as u8, TelnetCommand::SE as u8
    ]);
    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::DeviceNegotiated);

    // Session bound: Bound
    negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
        6, b'L', b'U', b'0', b'1', 0, TelnetCommand::IAC as u8, TelnetCommand::SE as u8
    ]);
    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::Bound);

    // Session unbound: Unbound
    negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
        7, TelnetCommand::IAC as u8, TelnetCommand::SE as u8
    ]);
    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::Unbound);
}

/// Test TN3270E error handling
#[test]
fn test_tn3270e_error_handling() {
    let mut negotiator = TelnetNegotiator::new();

    // Test invalid device type
    negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::TN3270E as u8
    ]);

    let response = negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
        2, 0xFF, TelnetCommand::IAC as u8, TelnetCommand::SE as u8 // Invalid device
    ]);

    // Should still respond but not change device type
    assert!(!response.is_empty());
    assert_eq!(negotiator.tn3270e_device_type(), None);

    // Test malformed BIND command
    let response2 = negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
        6, TelnetCommand::IAC as u8, TelnetCommand::SE as u8 // Empty BIND
    ]);

    // Should handle gracefully
    assert!(!response2.is_empty());
}

/// Integration test: Complete TN3270E session establishment
#[test]
fn test_complete_tn3270e_session_establishment() {
    let mut negotiator = TelnetNegotiator::new();

    // 1. Initial telnet negotiation
    let initial_negotiation = negotiator.generate_initial_negotiation();
    assert!(!initial_negotiation.is_empty());

    // 2. Server accepts essential options + TN3270E
    let server_responses = vec![
        vec![TelnetCommand::IAC as u8, TelnetCommand::WILL as u8, TelnetOption::Binary as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::Binary as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::WILL as u8, TelnetOption::EndOfRecord as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::EndOfRecord as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::WILL as u8, TelnetOption::SuppressGoAhead as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::SuppressGoAhead as u8],
        vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::TN3270E as u8],
    ];

    for response in server_responses {
        negotiator.process_incoming_data(&response);
    }

    // 3. Device type negotiation
    let device_response = negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
        2, 0x82, TelnetCommand::IAC as u8, TelnetCommand::SE as u8 // Model2Color
    ]);
    assert!(!device_response.is_empty());

    // 4. Session binding
    let bind_response = negotiator.process_incoming_data(&vec![
        TelnetCommand::IAC as u8, TelnetCommand::SB as u8, TelnetOption::TN3270E as u8,
        6, b'L', b'U', b'0', b'1', 0, TelnetCommand::IAC as u8, TelnetCommand::SE as u8
    ]);
    assert!(!bind_response.is_empty());

    // 5. Verify complete session
    assert!(negotiator.is_negotiation_complete());
    assert_eq!(negotiator.tn3270e_session_state(), TN3270ESessionState::Bound);
    assert_eq!(negotiator.tn3270e_device_type(), Some(TN3270EDeviceType::Model2Color));
    assert_eq!(negotiator.logical_unit_name(), Some("LU01"));
    assert_eq!(negotiator.get_screen_dimensions(), Some((24, 80)));
    assert!(negotiator.supports_color());
}