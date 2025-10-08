//! Regression Test Suite for Protocol Validation
//!
//! This test suite validates fixes for the confirmed issues and prevents regressions.
//! Tests are organized by issue category and include baseline validation.

use tn5250r::telnet_negotiation::{TelnetNegotiator, TelnetOption};
use tn5250r::lib5250::protocol::Packet;
use tn5250r::lib5250::codes::CommandCode;
use tn5250r::protocol_common::ebcdic::ebcdic_to_ascii;

// =============================================================================
// CATEGORY: Environment Variable Negotiation
// =============================================================================

#[test]
fn test_empty_environ_send_request() {
    // Issue #1: Environment variable response should send all vars for empty SEND
    let mut negotiator = TelnetNegotiator::new();
    
    // Empty SEND: IAC SB NEW-ENVIRON SEND IAC SE
    let empty_send = vec![255, 250, 39, 1, 255, 240];
    let response = negotiator.process_incoming_data(&empty_send);
    
    // Response should contain environment variables
    let response_str = String::from_utf8_lossy(&response);
    
    // REGRESSION TEST: Should contain at least these variables
    assert!(
        response_str.contains("DEVNAME"),
        "Empty SEND should include DEVNAME variable"
    );
    assert!(
        response_str.contains("CODEPAGE"),
        "Empty SEND should include CODEPAGE variable"
    );
    assert!(
        response_str.contains("USER"),
        "Empty SEND should include USER variable"
    );
    
    // Response should be substantial (> 50 bytes with all vars)
    assert!(
        response.len() > 50,
        "Empty SEND response too short: {} bytes", response.len()
    );
}

#[test]
fn test_specific_environ_variable_request() {
    // Verify specific variable requests still work
    let mut negotiator = TelnetNegotiator::new();
    
    let devname_request = vec![
        255, 250, 39, 1,  // IAC SB NEW-ENVIRON SEND
        0,                // VAR marker
        b'D', b'E', b'V', b'N', b'A', b'M', b'E',
        255, 240,         // IAC SE
    ];
    
    let response = negotiator.process_incoming_data(&devname_request);
    let response_str = String::from_utf8_lossy(&response);
    
    assert!(response_str.contains("DEVNAME"), "Should respond with requested variable");
    assert!(response_str.contains("TN5250R"), "Should include device name value");
}

#[test]
fn test_multiple_environ_variable_requests() {
    // Test requesting multiple specific variables
    let mut negotiator = TelnetNegotiator::new();
    
    let multi_request = vec![
        255, 250, 39, 1,  // IAC SB NEW-ENVIRON SEND
        0,                // VAR marker
        b'D', b'E', b'V', b'N', b'A', b'M', b'E',
        0,                // VAR marker
        b'U', b'S', b'E', b'R',
        255, 240,         // IAC SE
    ];
    
    let response = negotiator.process_incoming_data(&multi_request);
    let response_str = String::from_utf8_lossy(&response);
    
    assert!(response_str.contains("DEVNAME"), "Should include first requested variable");
    assert!(response_str.contains("USER"), "Should include second requested variable");
}

// =============================================================================
// CATEGORY: Packet Parsing
// =============================================================================

#[test]
fn test_packet_parsing_with_correct_length_format() {
    // Issue #2: Validate correct length field interpretation
    // Length field represents data payload length only (not including header)

    let packet_data_only_length = vec![0xF1, 0x01, 0x00, 0x05, 0x00, 0x40, 0x40, 0x40, 0x40, 0x40];
    let result = Packet::from_bytes(&packet_data_only_length);

    // Verify parsing succeeds
    assert!(result.is_some(), "Packet parsing should succeed with correct length format");

    let packet = result.unwrap();

    // Verify packet structure
    assert_eq!(packet.command, CommandCode::WriteToDisplay);
    assert_eq!(packet.sequence_number, 0x01);
    assert_eq!(packet.flags, 0x00);
    assert_eq!(packet.data.len(), 5, "Data length should match length field value");
    assert_eq!(packet.data, vec![0x40, 0x40, 0x40, 0x40, 0x40], "Data should contain expected payload");

    // Verify round-trip serialization
    let serialized = packet.to_bytes();
    assert_eq!(serialized, packet_data_only_length, "Round-trip serialization should preserve original bytes");
}

#[test]
fn test_packet_minimum_size_validation() {
    // Packets smaller than 5 bytes should be rejected
    let too_small = vec![0xF1, 0x01];
    assert!(Packet::from_bytes(&too_small).is_none());
}

#[test]
fn test_packet_maximum_size_protection() {
    // Extremely large packets should be rejected
    let mut huge_packet = vec![0xF1, 0x01, 0xFF, 0xFF, 0x00];
    huge_packet.extend(vec![0x00; 100]); // Add some data but not 65535 bytes
    
    // Should reject because length field (0xFFFF) exceeds actual data
    assert!(Packet::from_bytes(&huge_packet).is_none());
}

#[test]
fn test_packet_boundary_conditions() {
    // Test edge cases for packet parsing
    
    // Empty data packet (just header)
    let empty_data = vec![0xF1, 0x01, 0x00, 0x00, 0x00];
    let result = Packet::from_bytes(&empty_data);
    // Behavior depends on length interpretation (to be determined)
    let _ = result;
    
    // Single byte of data
    let single_byte = vec![0xF1, 0x01, 0x00, 0x01, 0x00, 0x40];
    let result = Packet::from_bytes(&single_byte);
    let _ = result;
}

// =============================================================================
// CATEGORY: EBCDIC Conversion
// =============================================================================

#[test]
fn test_ebcdic_coverage_minimum_requirement() {
    // Issue #3: At least 90% of EBCDIC characters should be mapped
    let mut mapped = 0;
    
    for ebcdic in 0u8..=255u8 {
        let ascii = ebcdic_to_ascii(ebcdic);
        // Count as mapped if it's not null/space (except for 0x00 and 0x40 which should be)
        if ascii != '\0' && ascii != ' ' {
            mapped += 1;
        } else if ebcdic == 0x00 || ebcdic == 0x40 {
            // These should map to null/space
            mapped += 1;
        }
    }
    
    let coverage = (mapped as f64 / 256.0) * 100.0;
    assert!(
        coverage >= 90.0,
        "EBCDIC coverage too low: {:.1}% (expected >= 90%)", coverage
    );
}

#[test]
fn test_ebcdic_lowercase_alphabet_complete() {
    // Lowercase a-z should all map correctly
    let expected = "abcdefghijklmnopqrstuvwxyz";
    let mut actual = String::new();
    
    // a-i: 0x81-0x89
    for ebcdic in 0x81u8..=0x89u8 {
        actual.push(ebcdic_to_ascii(ebcdic));
    }
    // j-r: 0x91-0x99
    for ebcdic in 0x91u8..=0x99u8 {
        actual.push(ebcdic_to_ascii(ebcdic));
    }
    // s-z: 0xA2-0xA9
    for ebcdic in 0xA2u8..=0xA9u8 {
        actual.push(ebcdic_to_ascii(ebcdic));
    }
    
    assert_eq!(actual, expected, "Lowercase alphabet mapping incorrect");
}

#[test]
fn test_ebcdic_uppercase_alphabet_complete() {
    // Uppercase A-Z should all map correctly
    let expected = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut actual = String::new();
    
    // A-I: 0xC1-0xC9
    for ebcdic in 0xC1u8..=0xC9u8 {
        actual.push(ebcdic_to_ascii(ebcdic));
    }
    // J-R: 0xD1-0xD9
    for ebcdic in 0xD1u8..=0xD9u8 {
        actual.push(ebcdic_to_ascii(ebcdic));
    }
    // S-Z: 0xE2-0xE9
    for ebcdic in 0xE2u8..=0xE9u8 {
        actual.push(ebcdic_to_ascii(ebcdic));
    }
    
    assert_eq!(actual, expected, "Uppercase alphabet mapping incorrect");
}

#[test]
fn test_ebcdic_digits_complete() {
    // Digits 0-9: 0xF0-0xF9
    let expected = "0123456789";
    let mut actual = String::new();
    
    for ebcdic in 0xF0u8..=0xF9u8 {
        actual.push(ebcdic_to_ascii(ebcdic));
    }
    
    assert_eq!(actual, expected, "Digit mapping incorrect");
}

#[test]
fn test_ebcdic_special_characters_codepage37() {
    // Verify critical special characters per Code Page 37
    let test_cases = vec![
        (0x4B, '.'),  // period
        (0x4C, '<'),  // less than
        (0x4D, '('),  // left paren
        (0x4E, '+'),  // plus
        (0x5B, '$'),  // dollar (was reported as !)
        (0x5C, '*'),  // asterisk
        (0x5D, ')'),  // right paren
        (0x6B, ','),  // comma
        (0x6C, '%'),  // percent
        (0x7C, '@'),  // at sign
    ];
    
    for (ebcdic, expected) in test_cases {
        let actual = ebcdic_to_ascii(ebcdic);
        assert_eq!(
            actual, expected,
            "EBCDIC 0x{:02X} should map to '{}' but got '{}'",
            ebcdic, expected, actual
        );
    }
}

// =============================================================================
// CATEGORY: Telnet Negotiation
// =============================================================================

#[test]
fn test_iac_escaping_correctness() {
    // Verify IAC escaping works correctly in binary mode
    let test_data = vec![0x01, 0xFF, 0x02, 0xFF, 0xFF, 0x03];
    let escaped = TelnetNegotiator::escape_iac_in_data(&test_data);
    
    // Each 0xFF should become 0xFF 0xFF
    assert_eq!(escaped.len(), test_data.len() + 3, "Should add 3 bytes for 3 IAC escapes");
    
    // Verify round-trip
    let unescaped = TelnetNegotiator::unescape_iac_in_data(&escaped);
    assert_eq!(unescaped, test_data, "Round-trip should preserve data");
}

#[test]
fn test_iac_command_state_machine() {
    let mut negotiator = TelnetNegotiator::new();
    
    // Test IAC WILL BINARY
    let will_binary = vec![255, 251, 0];
    let response = negotiator.process_incoming_data(&will_binary);
    
    assert!(!response.is_empty(), "Should generate response to WILL BINARY");
    assert!(negotiator.is_option_active(TelnetOption::Binary), "Binary should be active");
}

#[test]
fn test_concurrent_option_negotiation() {
    let mut negotiator = TelnetNegotiator::new();
    
    // Send multiple options at once
    let concurrent = vec![
        255, 253, 0,   // IAC DO BINARY
        255, 251, 19,  // IAC WILL EOR
        255, 253, 3,   // IAC DO SGA
    ];
    
    let _response = negotiator.process_incoming_data(&concurrent);
    
    // All options should be handled
    assert!(negotiator.is_option_active(TelnetOption::Binary));
    assert!(negotiator.is_option_active(TelnetOption::EndOfRecord));
    assert!(negotiator.is_option_active(TelnetOption::SuppressGoAhead));
}

#[test]
fn test_terminal_type_response() {
    let mut negotiator = TelnetNegotiator::new();
    
    // Terminal type SEND: IAC SB TERMINAL-TYPE SEND IAC SE
    let terminal_type_send = vec![255, 250, 24, 1, 255, 240];
    let response = negotiator.process_incoming_data(&terminal_type_send);
    
    // Should contain terminal type string
    let response_str = String::from_utf8_lossy(&response);
    assert!(
        response_str.contains("IBM-3179-2") || 
        response_str.contains("IBM-5555") ||
        response_str.contains("IBM-5250"),
        "Should send valid IBM terminal type"
    );
}

// =============================================================================
// CATEGORY: Buffer Overflow Protection
// =============================================================================

#[test]
fn test_buffer_overflow_protection_length_field() {
    // Packet with length field larger than actual data
    let overflow_packet = vec![
        0xF1,       // Command
        0x01,       // Sequence
        0xFF, 0xFF, // Length: 65535 (huge)
        0x00,       // Flags
        0x40,       // Only 1 byte of data
    ];
    
    // Should safely reject without crash
    assert!(
        Packet::from_bytes(&overflow_packet).is_none(),
        "Should reject packet with invalid length field"
    );
}

#[test]
fn test_buffer_overflow_protection_tiny_packets() {
    // Various too-small packets
    let test_cases = vec![
        vec![],                    // Empty
        vec![0xF1],                // Just command
        vec![0xF1, 0x01],          // Command + sequence
        vec![0xF1, 0x01, 0x00],    // Missing length byte
        vec![0xF1, 0x01, 0x00, 0x05], // Missing flags
    ];
    
    for (i, packet) in test_cases.iter().enumerate() {
        assert!(
            Packet::from_bytes(packet).is_none(),
            "Test case {}: Should reject packet with {} bytes",
            i, packet.len()
        );
    }
}

#[test]
fn test_malformed_packet_rejection() {
    // Various malformed packet structures should all be rejected safely
    let malformed_packets = vec![
        vec![0x00; 10],           // All nulls
        vec![0xFF; 10],           // All 0xFF
        vec![0xF1, 0x01, 0x00, 0x00, 0x00], // Zero length
    ];
    
    for packet in malformed_packets {
        let result = Packet::from_bytes(&packet);
        assert!(result.is_none(), "Should reject malformed packet");
    }
}

// =============================================================================
// CATEGORY: Field Attribute Parsing
// =============================================================================

#[test]
fn test_field_attribute_bit_mask_correctness() {
    use tn5250r::lib5250::protocol::FieldAttribute;
    
    // Verify bit mask 0x3C (bits 2-5) is used correctly
    let test_cases = vec![
        (0x20, FieldAttribute::Protected),
        (0x10, FieldAttribute::Numeric),
        (0x08, FieldAttribute::Skip),
        (0x0C, FieldAttribute::Mandatory),
        (0x04, FieldAttribute::DupEnable),
        (0x00, FieldAttribute::Normal),
    ];
    
    for (byte_val, expected_attr) in test_cases {
        let actual_attr = FieldAttribute::from_u8(byte_val);
        assert_eq!(
            actual_attr, expected_attr,
            "Attribute 0x{:02X} should be {:?}", byte_val, expected_attr
        );
    }
}

// =============================================================================
// CATEGORY: IAC Handling
// =============================================================================

#[test]
fn test_iac_escaping_edge_cases() {
    // Test IAC escaping with various data patterns
    
    // Case 1: Data starting with IAC
    let data1 = vec![0xFF, 0x01, 0x02];
    let escaped1 = TelnetNegotiator::escape_iac_in_data(&data1);
    assert_eq!(escaped1, vec![0xFF, 0xFF, 0x01, 0x02]);
    
    // Case 2: Data ending with IAC
    let data2 = vec![0x01, 0x02, 0xFF];
    let escaped2 = TelnetNegotiator::escape_iac_in_data(&data2);
    assert_eq!(escaped2, vec![0x01, 0x02, 0xFF, 0xFF]);
    
    // Case 3: Multiple consecutive IACs
    let data3 = vec![0xFF, 0xFF, 0xFF];
    let escaped3 = TelnetNegotiator::escape_iac_in_data(&data3);
    assert_eq!(escaped3, vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    
    // Case 4: No IAC bytes
    let data4 = vec![0x01, 0x02, 0x03];
    let escaped4 = TelnetNegotiator::escape_iac_in_data(&data4);
    assert_eq!(escaped4, data4, "Data without IAC should be unchanged");
}

#[test]
fn test_iac_unescaping_edge_cases() {
    // Test IAC unescaping
    
    // Case 1: Escaped IACs
    let escaped1 = vec![0xFF, 0xFF, 0x01, 0x02];
    let unescaped1 = TelnetNegotiator::unescape_iac_in_data(&escaped1);
    assert_eq!(unescaped1, vec![0xFF, 0x01, 0x02]);
    
    // Case 2: Multiple escaped IACs
    let escaped2 = vec![0xFF, 0xFF, 0xFF, 0xFF];
    let unescaped2 = TelnetNegotiator::unescape_iac_in_data(&escaped2);
    assert_eq!(unescaped2, vec![0xFF, 0xFF]);
    
    // Case 3: No escaped IACs
    let escaped3 = vec![0x01, 0x02, 0x03];
    let unescaped3 = TelnetNegotiator::unescape_iac_in_data(&escaped3);
    assert_eq!(unescaped3, escaped3);
}

// =============================================================================
// CATEGORY: Protocol Baseline Validation
// =============================================================================

#[test]
fn test_telnet_negotiator_initialization() {
    let negotiator = TelnetNegotiator::new();
    
    // Should start with no options active
    assert!(!negotiator.is_option_active(TelnetOption::Binary));
    assert!(!negotiator.is_option_active(TelnetOption::EndOfRecord));
    assert!(!negotiator.is_option_active(TelnetOption::SuppressGoAhead));
    assert!(!negotiator.is_negotiation_complete());
}

#[test]
fn test_negotiation_completion_requirements() {
    let mut negotiator = TelnetNegotiator::new();
    
    // Negotiate only Binary
    let will_binary = vec![255, 251, 0];
    negotiator.process_incoming_data(&will_binary);
    assert!(!negotiator.is_negotiation_complete(), "Binary alone not sufficient");
    
    // Add EOR
    let will_eor = vec![255, 251, 19];
    negotiator.process_incoming_data(&will_eor);
    assert!(!negotiator.is_negotiation_complete(), "Binary + EOR not sufficient");
    
    // Add SGA - now should be complete
    let will_sga = vec![255, 251, 3];
    negotiator.process_incoming_data(&will_sga);
    assert!(negotiator.is_negotiation_complete(), "Binary + EOR + SGA should complete negotiation");
}

#[test]
fn test_initial_negotiation_generation() {
    let mut negotiator = TelnetNegotiator::new();
    let initial = negotiator.generate_initial_negotiation();
    
    // Should have commands for preferred options
    assert!(!initial.is_empty(), "Initial negotiation should not be empty");
    assert!(initial.len() >= 15, "Should have at least 5 options × 3 bytes each");
    
    // Verify it contains IAC commands
    assert!(initial.iter().any(|&b| b == 255), "Should contain IAC bytes");
}

// =============================================================================
// CATEGORY: Security Tests
// =============================================================================

#[test]
fn test_no_panic_on_malformed_data() {
    let mut negotiator = TelnetNegotiator::new();
    
    // Various malformed inputs should not panic
    let malformed_inputs = vec![
        vec![255],                    // Incomplete IAC
        vec![255, 251],               // Incomplete WILL
        vec![255, 250, 24],           // Incomplete subnegotiation
        vec![255, 250, 24, 1],        // Subnegotiation without SE
        vec![0; 100],                 // All nulls
        vec![255; 100],               // All IACs
    ];
    
    for input in malformed_inputs {
        let _result = negotiator.process_incoming_data(&input);
        // Should not panic, result doesn't matter for this test
    }
}

#[test]
fn test_empty_data_handling() {
    let mut negotiator = TelnetNegotiator::new();
    
    // Empty data should be handled gracefully
    let response = negotiator.process_incoming_data(&[]);
    assert_eq!(response, Vec::<u8>::new(), "Empty input should produce empty output");
}