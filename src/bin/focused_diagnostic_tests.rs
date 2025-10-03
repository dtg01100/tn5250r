#!/usr/bin/env cargo
//! Focused Diagnostic Tests for Confirmed Critical Issues
//!
//! This program performs deep diagnostic testing on the 5 confirmed issues:
//! 1. EOR Negotiation Failure (CRITICAL)
//! 2. Packet Parsing Failure (HIGH)
//! 3. EBCDIC Coverage Gap (HIGH)
//! 4. Environment Variable Response (HIGH)
//! 5. Special Character Mapping (MEDIUM)
//!
//! Each test includes detailed logging to pinpoint the exact root cause.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tn5250r::telnet_negotiation::{TelnetNegotiator, TelnetOption};
use tn5250r::lib5250::protocol::Packet;
use tn5250r::protocol_common::ebcdic::ebcdic_to_ascii;

fn main() {
    println!("TN5250R Focused Diagnostic Tests");
    println!("==================================\n");
    
    println!("Running diagnostic tests for 5 confirmed issues...\n");
    
    // Test 1: EOR Negotiation Failure
    diagnostic_eor_negotiation();
    
    // Test 2: Packet Parsing Failure
    diagnostic_packet_parsing();
    
    // Test 3: EBCDIC Coverage Gap
    diagnostic_ebcdic_coverage();
    
    // Test 4: Environment Variable Response
    diagnostic_environment_variables();
    
    // Test 5: Special Character Mapping
    diagnostic_special_characters();
    
    println!("\n{}", "=".repeat(70));
    println!("DIAGNOSTIC TESTS COMPLETE");
    println!("{}", "=".repeat(70));
}

// =============================================================================
// DIAGNOSTIC TEST 1: EOR Negotiation Failure
// =============================================================================

fn diagnostic_eor_negotiation() {
    println!("\n{}", "=".repeat(70));
    println!("DIAGNOSTIC 1: EOR Negotiation Failure Analysis");
    println!("{}", "=".repeat(70));
    
    let mut negotiator = TelnetNegotiator::new();
    
    // Step 1: Check initial state
    println!("\nStep 1: Initial State");
    println!("  EOR state: {:?}", negotiator.get_negotiation_state_details().get(&TelnetOption::EndOfRecord));
    
    // Step 2: Send initial negotiation
    println!("\nStep 2: Generate Initial Negotiation");
    let initial = negotiator.generate_initial_negotiation();
    println!("  Initial negotiation bytes: {}", initial.len());
    
    // Look for EOR-related commands in initial negotiation
    let mut i = 0;
    while i + 2 < initial.len() {
        if initial[i] == 255 { // IAC
            let cmd = initial[i + 1];
            let opt = initial[i + 2];
            if opt == 19 { // EOR option
                println!("  Found EOR in initial: IAC {:?} EOR", 
                    match cmd {
                        253 => "DO",
                        254 => "DONT",
                        251 => "WILL",
                        252 => "WONT",
                        _ => "UNKNOWN",
                    });
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    
    // Step 3: Simulate server responses
    println!("\nStep 3: Simulate Server Responses");
    
    // Server sends WILL EOR
    println!("  Server sends: IAC WILL EOR");
    let server_will_eor = vec![255, 251, 19]; // IAC WILL EOR
    let response1 = negotiator.process_incoming_data(&server_will_eor);
    println!("  Client response: {:?}", response1);
    println!("  EOR state after WILL: {:?}", negotiator.get_negotiation_state_details().get(&TelnetOption::EndOfRecord));
    println!("  EOR active: {}", negotiator.is_option_active(TelnetOption::EndOfRecord));
    
    // Server sends DO EOR
    println!("\n  Server sends: IAC DO EOR");
    let server_do_eor = vec![255, 253, 19]; // IAC DO EOR
    let response2 = negotiator.process_incoming_data(&server_do_eor);
    println!("  Client response: {:?}", response2);
    println!("  EOR state after DO: {:?}", negotiator.get_negotiation_state_details().get(&TelnetOption::EndOfRecord));
    println!("  EOR active: {}", negotiator.is_option_active(TelnetOption::EndOfRecord));
    
    // Step 4: Check what's needed for activation
    println!("\nStep 4: Activation Requirements");
    println!("  Binary active: {}", negotiator.is_option_active(TelnetOption::Binary));
    println!("  EOR active: {}", negotiator.is_option_active(TelnetOption::EndOfRecord));
    println!("  SGA active: {}", negotiator.is_option_active(TelnetOption::SuppressGoAhead));
    println!("  Negotiation complete: {}", negotiator.is_negotiation_complete());
    
    // Step 5: Analysis
    println!("\nStep 5: Root Cause Analysis");
    if !negotiator.is_option_active(TelnetOption::EndOfRecord) {
        println!("  ❌ ISSUE CONFIRMED: EOR not activated despite server requests");
        println!("  Possible causes:");
        println!("    - State machine not transitioning to Active state");
        println!("    - Missing response to server's WILL or DO command");
        println!("    - Incorrect state handling in handle_will_command() or handle_do_command()");
    } else {
        println!("  ✅ EOR activated successfully");
    }
}

// =============================================================================
// DIAGNOSTIC TEST 2: Packet Parsing Failure
// =============================================================================

fn diagnostic_packet_parsing() {
    println!("\n{}", "=".repeat(70));
    println!("DIAGNOSTIC 2: Packet Parsing Failure Analysis");
    println!("{}", "=".repeat(70));
    
    // Test different length field interpretations
    println!("\nTesting different length field interpretations:");
    
    // Interpretation 1: Length = data length only
    println!("\nInterpretation 1: Length = data length (5 bytes)");
    let packet1 = vec![
        0xF1,       // Command: WriteToDisplay
        0x01,       // Sequence number
        0x00, 0x05, // Length: 5 (data length only)
        0x00,       // Flags
        0x40, 0x40, 0x40, 0x40, 0x40, // 5 bytes of data
    ];
    println!("  Packet structure: {:02X?}", packet1);
    println!("  Total packet size: {} bytes", packet1.len());
    println!("  Length field value: 5");
    println!("  Data bytes: 5");
    let result1 = Packet::from_bytes(&packet1);
    println!("  Parse result: {}", if result1.is_some() { "SUCCESS" } else { "FAILED" });
    
    // Interpretation 2: Length = total packet length
    println!("\nInterpretation 2: Length = total packet length (10 bytes)");
    let packet2 = vec![
        0xF1,       // Command: WriteToDisplay
        0x01,       // Sequence number
        0x00, 0x0A, // Length: 10 (total packet length)
        0x00,       // Flags
        0x40, 0x40, 0x40, 0x40, 0x40, // 5 bytes of data
    ];
    println!("  Packet structure: {:02X?}", packet2);
    println!("  Total packet size: {} bytes", packet2.len());
    println!("  Length field value: 10");
    println!("  Data bytes: 5");
    let result2 = Packet::from_bytes(&packet2);
    println!("  Parse result: {}", if result2.is_some() { "SUCCESS" } else { "FAILED" });
    
    // Interpretation 3: Length = data + flags length
    println!("\nInterpretation 3: Length = data + flags (6 bytes)");
    let packet3 = vec![
        0xF1,       // Command: WriteToDisplay
        0x01,       // Sequence number
        0x00, 0x06, // Length: 6 (flags + data length)
        0x00,       // Flags
        0x40, 0x40, 0x40, 0x40, 0x40, // 5 bytes of data
    ];
    println!("  Packet structure: {:02X?}", packet3);
    println!("  Total packet size: {} bytes", packet3.len());
    println!("  Length field value: 6");
    println!("  Data bytes: 5");
    let result3 = Packet::from_bytes(&packet3);
    println!("  Parse result: {}", if result3.is_some() { "SUCCESS" } else { "FAILED" });
    
    // Analysis
    println!("\nRoot Cause Analysis:");
    match (result1.is_some(), result2.is_some(), result3.is_some()) {
        (false, true, false) => {
            println!("  ✅ Parser expects: Length = total packet size");
            println!("  ❌ Our test used: Length = data length only");
            println!("  ISSUE: Documentation/test mismatch, not a parsing bug");
        }
        (true, false, false) => {
            println!("  ✅ Parser expects: Length = data length only");
            println!("  ❌ Real packets use: Length = total packet size");
            println!("  ISSUE: Parser implementation incorrect for real packets");
        }
        (false, false, true) => {
            println!("  ✅ Parser expects: Length = flags + data");
            println!("  ISSUE: Need to verify against RFC specification");
        }
        _ => {
            println!("  ⚠️ Multiple interpretations work or none work");
            println!("  Need to check RFC 2877 specification for correct format");
        }
    }
}

// =============================================================================
// DIAGNOSTIC TEST 3: EBCDIC Coverage Gap
// =============================================================================

fn diagnostic_ebcdic_coverage() {
    println!("\n{}", "=".repeat(70));
    println!("DIAGNOSTIC 3: EBCDIC Coverage Gap Analysis");
    println!("{}", "=".repeat(70));
    
    println!("\nAnalyzing unmapped EBCDIC characters:");
    
    let mut unmapped_ranges = Vec::new();
    let mut current_range_start = None;
    let mut current_range_end = None;
    
    for ebcdic in 0u8..=255u8 {
        let ascii = ebcdic_to_ascii(ebcdic);
        let is_unmapped = (ascii == '\0' || ascii == ' ') && ebcdic != 0x00 && ebcdic != 0x40;
        
        if is_unmapped {
            if current_range_start.is_none() {
                current_range_start = Some(ebcdic);
                current_range_end = Some(ebcdic);
            } else {
                current_range_end = Some(ebcdic);
            }
        } else {
            if let (Some(start), Some(end)) = (current_range_start, current_range_end) {
                unmapped_ranges.push((start, end));
                current_range_start = None;
                current_range_end = None;
            }
        }
    }
    
    // Close final range if exists
    if let (Some(start), Some(end)) = (current_range_start, current_range_end) {
        unmapped_ranges.push((start, end));
    }
    
    println!("\nUnmapped EBCDIC Ranges:");
    for (start, end) in &unmapped_ranges {
        if start == end {
            println!("  0x{:02X}: single byte", start);
        } else {
            println!("  0x{:02X}-0x{:02X}: {} bytes", start, end, end - start + 1);
        }
    }
    
    println!("\nTotal unmapped ranges: {}", unmapped_ranges.len());
    
    // Check specific important characters
    println!("\nImportant Character Mappings:");
    let important_chars = vec![
        (0x4B, '.', "period"),
        (0x4C, '<', "less than"),
        (0x4D, '(', "left paren"),
        (0x4E, '+', "plus"),
        (0x5B, '!', "exclamation"),
        (0x5C, '$', "dollar"),
        (0x5D, '*', "asterisk"),
        (0x5E, ')', "right paren"),
        (0x6B, ',', "comma"),
        (0x6C, '%', "percent"),
        (0x7C, '@', "at sign"),
    ];
    
    for (ebcdic, expected, name) in important_chars {
        let actual = ebcdic_to_ascii(ebcdic);
        let status = if actual == expected { "✓" } else { "✗" };
        println!("  0x{:02X} ({:12}): expected '{}', got '{}' {}", 
            ebcdic, name, expected, actual, status);
    }
}

// =============================================================================
// DIAGNOSTIC TEST 4: Environment Variable Response
// =============================================================================

fn diagnostic_environment_variables() {
    println!("\n{}", "=".repeat(70));
    println!("DIAGNOSTIC 4: Environment Variable Response Analysis");
    println!("{}", "=".repeat(70));
    
    let mut negotiator = TelnetNegotiator::new();
    
    // Test Case 1: Empty SEND (no specific variables requested)
    println!("\nTest Case 1: Empty SEND Request");
    println!("  Server sends: IAC SB NEW-ENVIRON SEND IAC SE");
    let empty_send = vec![255, 250, 39, 1, 255, 240];
    println!("  Raw bytes: {:02X?}", empty_send);
    
    let response1 = negotiator.process_incoming_data(&empty_send);
    println!("  Response length: {} bytes", response1.len());
    println!("  Response bytes: {:02X?}", response1);
    
    if response1.len() > 6 {
        // Parse response to show variables
        println!("\n  Analyzing response structure:");
        let mut i = 0;
        while i < response1.len() {
            if response1[i] == 255 && i + 1 < response1.len() {
                match response1[i + 1] {
                    250 => println!("    [{}] IAC SB", i),
                    240 => println!("    [{}] IAC SE", i),
                    _ => println!("    [{}] IAC {}", i, response1[i + 1]),
                }
                i += 2;
            } else if response1[i] == 39 {
                println!("    [{}] NEW-ENVIRON option", i);
                i += 1;
            } else if response1[i] == 0 {
                println!("    [{}] VAR marker", i);
                i += 1;
            } else if response1[i] == 1 {
                println!("    [{}] VALUE marker", i);
                i += 1;
            } else if response1[i] == 2 {
                println!("    [{}] IS command", i);
                i += 1;
            } else {
                if response1[i] >= 32 && response1[i] <= 126 {
                    println!("    [{}] Data: '{}'", i, response1[i] as char);
                } else {
                    println!("    [{}] Data: 0x{:02X}", i, response1[i]);
                }
                i += 1;
            }
        }
    } else {
        println!("  ❌ Response too short - no variables sent!");
    }
    
    // Test Case 2: Explicit variable request
    println!("\nTest Case 2: Request Specific Variable (DEVNAME)");
    let mut negotiator2 = TelnetNegotiator::new();
    let devname_request = vec![
        255, 250, 39, 1, // IAC SB NEW-ENVIRON SEND
        0,               // VAR marker
        b'D', b'E', b'V', b'N', b'A', b'M', b'E',
        255, 240,        // IAC SE
    ];
    println!("  Server sends: IAC SB NEW-ENVIRON SEND VAR DEVNAME IAC SE");
    println!("  Raw bytes: {:02X?}", devname_request);
    
    let response2 = negotiator2.process_incoming_data(&devname_request);
    println!("  Response length: {} bytes", response2.len());
    
    if response2.len() > 10 {
        let response_str = String::from_utf8_lossy(&response2);
        println!("  Contains DEVNAME: {}", response_str.contains("DEVNAME"));
        println!("  Contains TN5250R: {}", response_str.contains("TN5250R"));
    }
    
    // Root cause analysis
    println!("\nRoot Cause Analysis:");
    println!("  Issue in handle_environment_negotiation():");
    println!("  - Empty SEND (data.len() == 1) doesn't call send_environment_variables()");
    println!("  - Condition 'if data.len() > 1' prevents sending vars for empty SEND");
    println!("  - RFC 1572 requires sending all vars when SEND has no specific requests");
}

// =============================================================================
// DIAGNOSTIC TEST 5: Special Character Mapping
// =============================================================================

fn diagnostic_special_characters() {
    println!("\n{}", "=".repeat(70));
    println!("DIAGNOSTIC 5: Special Character Mapping Analysis");
    println!("{}", "=".repeat(70));
    
    println!("\nEBCDIC Code Page 37 Special Character Verification:");
    
    // Cross-reference with official EBCDIC Code Page 37
    let cp37_mappings = vec![
        (0x4B, '.', "period/full stop"),
        (0x4C, '<', "less-than sign"),
        (0x4D, '(', "left parenthesis"),
        (0x4E, '+', "plus sign"),
        (0x4F, '|', "vertical line"),
        (0x50, '&', "ampersand"),
        (0x5A, '!', "exclamation mark"),
        (0x5B, '$', "dollar sign"),       // ← Critical: $ not !
        (0x5C, '*', "asterisk"),          // ← This is correct
        (0x5D, ')', "right parenthesis"),
        (0x5E, ';', "semicolon"),
        (0x5F, '¬', "not sign"),          // Often mapped to different chars
        (0x60, '-', "hyphen-minus"),
        (0x61, '/', "solidus/slash"),
        (0x6B, ',', "comma"),
        (0x6C, '%', "percent sign"),
        (0x6D, '_', "low line/underscore"),
        (0x6E, '>', "greater-than sign"),
        (0x6F, '?', "question mark"),
        (0x7A, ':', "colon"),
        (0x7B, '#', "number sign"),
        (0x7C, '@', "commercial at"),
        (0x7D, '\'', "apostrophe"),
        (0x7E, '=', "equals sign"),
        (0x7F, '"', "quotation mark"),
    ];
    
    let mut mismatches = 0;
    for (ebcdic, expected, name) in cp37_mappings {
        let actual = ebcdic_to_ascii(ebcdic);
        if actual == expected {
            println!("  0x{:02X} ({:20}): '{}' ✓", ebcdic, name, actual);
        } else {
            mismatches += 1;
            println!("  0x{:02X} ({:20}): expected '{}', got '{}' ✗", 
                ebcdic, name, expected, actual);
        }
    }
    
    println!("\nMismatches found: {}", mismatches);
    
    if mismatches > 0 {
        println!("\nConclusion:");
        println!("  Some special character mappings differ from Code Page 37");
        println!("  Need to verify against RFC 2877 and actual AS/400 behavior");
        println!("  Previous test may have had incorrect expected values");
    }
}

// =============================================================================
// Actual live test against pub400.com to see EOR behavior
// =============================================================================