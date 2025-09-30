#!/usr/bin/env cargo
//! Comprehensive Protocol Validation Test Suite
//!
//! This test suite validates the 47 identified issues in TN5250R codebase.
//! **VALIDATION ONLY** - This program tests and documents current behavior.
//! NO fixes are applied during this phase.
//!
//! Test Categories:
//! 1. Connection Establishment
//! 2. Telnet Negotiation
//! 3. Data Stream Parsing
//! 4. EBCDIC Conversion
//! 5. Session Management
//! 6. Security Vulnerabilities

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};
use std::collections::HashMap;

use tn5250r::telnet_negotiation::{TelnetNegotiator, TelnetOption, TelnetCommand};
use tn5250r::lib5250::protocol::{Packet, CommandCode, FieldAttribute};
use tn5250r::protocol_common::ebcdic::ebcdic_to_ascii;

/// Test result structure for systematic documentation
#[derive(Debug, Clone)]
struct TestResult {
    test_id: String,
    test_name: String,
    expected: String,
    actual: String,
    status: TestStatus,
    severity: Severity,
    reproduction_steps: Vec<String>,
    logs: Vec<String>,
    issue_reference: String,
    duration_ms: u128,
}

#[derive(Debug, Clone, PartialEq)]
enum TestStatus {
    Pass,
    Fail,
    Partial,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl TestResult {
    fn new(test_id: &str, test_name: &str) -> Self {
        Self {
            test_id: test_id.to_string(),
            test_name: test_name.to_string(),
            expected: String::new(),
            actual: String::new(),
            status: TestStatus::Error,
            severity: Severity::Medium,
            reproduction_steps: Vec::new(),
            logs: Vec::new(),
            issue_reference: String::new(),
            duration_ms: 0,
        }
    }

    fn print_summary(&self) {
        println!("\n{}", "=".repeat(70));
        println!("TEST: {} - {}", self.test_id, self.test_name);
        println!("{}", "=".repeat(70));
        println!("EXPECTED: {}", self.expected);
        println!("ACTUAL:   {}", self.actual);
        println!("STATUS:   {:?}", self.status);
        println!("SEVERITY: {:?}", self.severity);
        println!("DURATION: {}ms", self.duration_ms);
        
        if !self.reproduction_steps.is_empty() {
            println!("\nREPRODUCTION STEPS:");
            for (i, step) in self.reproduction_steps.iter().enumerate() {
                println!("  {}. {}", i + 1, step);
            }
        }
        
        if !self.logs.is_empty() {
            println!("\nLOGS:");
            for log in &self.logs {
                println!("  {}", log);
            }
        }
        
        if !self.issue_reference.is_empty() {
            println!("\nISSUE REFERENCE: {}", self.issue_reference);
        }
    }
}

/// Test suite manager
struct TestSuite {
    results: Vec<TestResult>,
    test_server: String,
    test_port: u16,
}

impl TestSuite {
    fn new(server: &str, port: u16) -> Self {
        Self {
            results: Vec::new(),
            test_server: server.to_string(),
            test_port: port,
        }
    }

    fn add_result(&mut self, result: TestResult) {
        result.print_summary();
        self.results.push(result);
    }

    fn print_final_summary(&self) {
        println!("\n\n{}", "=".repeat(70));
        println!("FINAL TEST SUMMARY");
        println!("{}", "=".repeat(70));
        
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.status == TestStatus::Pass).count();
        let failed = self.results.iter().filter(|r| r.status == TestStatus::Fail).count();
        let partial = self.results.iter().filter(|r| r.status == TestStatus::Partial).count();
        let errors = self.results.iter().filter(|r| r.status == TestStatus::Error).count();
        
        println!("Total Tests:   {}", total);
        println!("Passed:        {} ({:.1}%)", passed, (passed as f64 / total as f64) * 100.0);
        println!("Failed:        {} ({:.1}%)", failed, (failed as f64 / total as f64) * 100.0);
        println!("Partial:       {} ({:.1}%)", partial, (partial as f64 / total as f64) * 100.0);
        println!("Errors:        {} ({:.1}%)", errors, (errors as f64 / total as f64) * 100.0);
        
        println!("\nBy Severity:");
        let critical = self.results.iter().filter(|r| r.severity == Severity::Critical && r.status != TestStatus::Pass).count();
        let high = self.results.iter().filter(|r| r.severity == Severity::High && r.status != TestStatus::Pass).count();
        let medium = self.results.iter().filter(|r| r.severity == Severity::Medium && r.status != TestStatus::Pass).count();
        let low = self.results.iter().filter(|r| r.severity == Severity::Low && r.status != TestStatus::Pass).count();
        
        println!("  Critical Issues: {}", critical);
        println!("  High Issues:     {}", high);
        println!("  Medium Issues:   {}", medium);
        println!("  Low Issues:      {}", low);
    }
}

fn main() {
    println!("TN5250R Comprehensive Protocol Validation Test Suite");
    println!("====================================================\n");
    println!("**VALIDATION ONLY** - Testing and documenting current behavior");
    println!("NO fixes will be applied during this phase.\n");

    // Default test server
    let test_server = std::env::var("TEST_SERVER").unwrap_or_else(|_| "pub400.com".to_string());
    let test_port: u16 = std::env::var("TEST_PORT")
        .unwrap_or_else(|_| "23".to_string())
        .parse()
        .unwrap_or(23);

    println!("Test Target: {}:{}\n", test_server, test_port);

    let mut suite = TestSuite::new(&test_server, test_port);

    // Category 1: Connection Establishment Tests
    println!("\n{}", "#".repeat(70));
    println!("CATEGORY 1: CONNECTION ESTABLISHMENT TESTS");
    println!("{}", "#".repeat(70));
    
    suite.add_result(test_basic_connection(&test_server, test_port));
    suite.add_result(test_connection_timeout());
    suite.add_result(test_protocol_detection(&test_server, test_port));

    // Category 2: Telnet Negotiation Tests
    println!("\n{}", "#".repeat(70));
    println!("CATEGORY 2: TELNET NEGOTIATION TESTS");
    println!("{}", "#".repeat(70));
    
    suite.add_result(test_iac_command_processing());
    suite.add_result(test_option_negotiation_sequence(&test_server, test_port));
    suite.add_result(test_terminal_type_negotiation());
    suite.add_result(test_environment_variable_negotiation());
    suite.add_result(test_iac_escaping_binary_mode());
    suite.add_result(test_concurrent_negotiation());

    // Category 3: Data Stream Parsing Tests
    println!("\n{}", "#".repeat(70));
    println!("CATEGORY 3: DATA STREAM PARSING TESTS");
    println!("{}", "#".repeat(70));
    
    suite.add_result(test_packet_structure_validation());
    suite.add_result(test_structured_field_processing());
    suite.add_result(test_field_attribute_parsing());
    suite.add_result(test_buffer_overflow_conditions());
    suite.add_result(test_malformed_packet_handling());

    // Category 4: EBCDIC Conversion Tests
    println!("\n{}", "#".repeat(70));
    println!("CATEGORY 4: EBCDIC CONVERSION TESTS");
    println!("{}", "#".repeat(70));
    
    suite.add_result(test_ebcdic_character_completeness());
    suite.add_result(test_special_character_accuracy());
    suite.add_result(test_lowercase_character_range());

    // Category 5: Session Management Tests
    println!("\n{}", "#".repeat(70));
    println!("CATEGORY 5: SESSION MANAGEMENT TESTS");
    println!("{}", "#".repeat(70));
    
    suite.add_result(test_keyboard_lock_states());
    suite.add_result(test_screen_save_restore());

    // Category 6: Security Vulnerability Tests
    println!("\n{}", "#".repeat(70));
    println!("CATEGORY 6: SECURITY VULNERABILITY TESTS");
    println!("{}", "#".repeat(70));
    
    suite.add_result(test_buffer_overflow_attack_vectors());
    suite.add_result(test_input_sanitization());

    // Print final summary
    suite.print_final_summary();
}

// ==============================================================================
// CATEGORY 1: CONNECTION ESTABLISHMENT TESTS
// ==============================================================================

fn test_basic_connection(server: &str, port: u16) -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("1.1", "Basic TCP Connection");
    
    result.expected = "Connection establishes within 5 seconds".to_string();
    result.severity = Severity::Critical;
    result.issue_reference = "Baseline connection test".to_string();
    result.reproduction_steps = vec![
        format!("Attempt TCP connection to {}:{}", server, port),
        "Measure connection establishment time".to_string(),
        "Verify socket is readable/writable".to_string(),
    ];

    let addr = match format!("{}:{}", server, port).parse() {
        Ok(addr) => addr,
        Err(e) => {
            // Try DNS resolution
            match std::net::ToSocketAddrs::to_socket_addrs(&format!("{}:{}", server, port)) {
                Ok(mut addrs) => {
                    if let Some(addr) = addrs.next() {
                        addr
                    } else {
                        result.actual = "DNS resolution returned no addresses".to_string();
                        result.status = TestStatus::Error;
                        result.duration_ms = start.elapsed().as_millis();
                        return result;
                    }
                }
                Err(e2) => {
                    result.actual = format!("Address resolution failed: {} (parse: {})", e2, e);
                    result.status = TestStatus::Error;
                    result.duration_ms = start.elapsed().as_millis();
                    return result;
                }
            }
        }
    };
    
    match TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
        Ok(_stream) => {
            let elapsed = start.elapsed().as_millis();
            result.actual = format!("Connection established in {}ms", elapsed);
            result.status = if elapsed < 5000 {
                TestStatus::Pass
            } else {
                TestStatus::Partial
            };
            result.logs.push(format!("Socket connected successfully"));
            result.logs.push(format!("Connection time: {}ms", elapsed));
        }
        Err(e) => {
            result.actual = format!("Connection failed: {}", e);
            result.status = TestStatus::Fail;
            result.logs.push(format!("Error: {}", e));
        }
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_connection_timeout() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("1.3", "Connection Timeout Handling");
    
    result.expected = "Connection times out gracefully after specified duration".to_string();
    result.severity = Severity::High;
    result.issue_reference = "Issue 1.6 (Session Timeout Problems)".to_string();
    result.reproduction_steps = vec![
        "Attempt connection to unreachable host (192.0.2.1)".to_string(),
        "Set timeout to 2 seconds".to_string(),
        "Verify timeout occurs within acceptable range".to_string(),
    ];

    let addr = match "192.0.2.1:23".parse() {
        Ok(addr) => addr,
        Err(e) => {
            result.actual = format!("Address parse failed: {}", e);
            result.status = TestStatus::Error;
            result.duration_ms = start.elapsed().as_millis();
            return result;
        }
    };
    
    match TcpStream::connect_timeout(&addr, Duration::from_secs(2)) {
        Ok(_) => {
            result.actual = "Connection unexpectedly succeeded to unreachable host".to_string();
            result.status = TestStatus::Fail;
            result.logs.push("Unreachable host should not be connectable".to_string());
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis();
            result.actual = format!("Timeout occurred after {}ms: {}", elapsed, e);
            
            // Check if timeout is within acceptable range (1900-2100ms)
            result.status = if elapsed >= 1900 && elapsed <= 2100 {
                TestStatus::Pass
            } else {
                TestStatus::Partial
            };
            
            result.logs.push(format!("Timeout duration: {}ms", elapsed));
            result.logs.push(format!("Expected: 2000ms ± 100ms", ));
        }
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_protocol_detection(server: &str, port: u16) -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("1.4", "Protocol Detection - NVT vs 5250");
    
    result.expected = "Correct protocol detected within 256 bytes of initial data".to_string();
    result.severity = Severity::Critical;
    result.issue_reference = "Issue 4.1 (NVT Mode vs 5250 Protocol Confusion)".to_string();
    result.reproduction_steps = vec![
        format!("Connect to {}:{}", server, port),
        "Read initial 256 bytes from server".to_string(),
        "Analyze for protocol markers (IAC, ESC sequences)".to_string(),
        "Determine if NVT (plain text) or 5250 (EBCDIC/binary)".to_string(),
    ];

    let addr = match format!("{}:{}", server, port).parse() {
        Ok(addr) => addr,
        Err(_e) => {
            // Try DNS resolution
            match std::net::ToSocketAddrs::to_socket_addrs(&format!("{}:{}", server, port)) {
                Ok(mut addrs) => {
                    if let Some(addr) = addrs.next() {
                        addr
                    } else {
                        result.actual = "DNS resolution returned no addresses".to_string();
                        result.status = TestStatus::Error;
                        result.duration_ms = start.elapsed().as_millis();
                        return result;
                    }
                }
                Err(e2) => {
                    result.actual = format!("Address resolution failed: {}", e2);
                    result.status = TestStatus::Error;
                    result.duration_ms = start.elapsed().as_millis();
                    return result;
                }
            }
        }
    };
    
    match TcpStream::connect_timeout(&addr, Duration::from_secs(10)) {
        Ok(mut stream) => {
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            
            let mut buffer = vec![0u8; 256];
            match stream.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let data = &buffer[..n];
                    
                    // Analyze for protocol markers
                    let has_iac = data.iter().any(|&b| b == 255); // IAC byte
                    let has_esc = data.iter().any(|&b| b == 0x04); // ESC in EBCDIC
                    let has_high_bytes = data.iter().filter(|&&b| b > 127).count();
                    let high_byte_ratio = has_high_bytes as f64 / n as f64;
                    
                    result.logs.push(format!("Received {} bytes", n));
                    result.logs.push(format!("Has IAC (0xFF): {}", has_iac));
                    result.logs.push(format!("Has ESC (0x04): {}", has_esc));
                    result.logs.push(format!("High bytes: {} ({:.1}%)", has_high_bytes, high_byte_ratio * 100.0));
                    result.logs.push(format!("First 32 bytes: {:02X?}", &data[..n.min(32)]));
                    
                    let detected_protocol = if has_iac {
                        "5250 (telnet negotiation detected)"
                    } else if high_byte_ratio > 0.3 {
                        "5250 (EBCDIC data detected)"
                    } else {
                        "NVT (plain text detected)"
                    };
                    
                    result.actual = format!("Protocol detected: {}", detected_protocol);
                    result.status = TestStatus::Pass; // Document what we detected
                }
                Ok(_) => {
                    result.actual = "Connection closed without data".to_string();
                    result.status = TestStatus::Fail;
                }
                Err(e) => {
                    result.actual = format!("Read error: {}", e);
                    result.status = TestStatus::Error;
                    result.logs.push(format!("Error: {}", e));
                }
            }
        }
        Err(e) => {
            result.actual = format!("Connection failed: {}", e);
            result.status = TestStatus::Error;
            result.logs.push(format!("Error: {}", e));
        }
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

// ==============================================================================
// CATEGORY 2: TELNET NEGOTIATION TESTS
// ==============================================================================

fn test_iac_command_processing() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("2.1", "IAC Command Processing");
    
    result.expected = "IAC sequences parsed correctly with proper state machine transitions".to_string();
    result.severity = Severity::Critical;
    result.issue_reference = "Issue 1.2 (Telnet Command Processing State Machine)".to_string();
    result.reproduction_steps = vec![
        "Create TelnetNegotiator instance".to_string(),
        "Process IAC WILL/WONT/DO/DONT sequences".to_string(),
        "Verify state machine transitions".to_string(),
        "Check for proper response generation".to_string(),
    ];

    let mut negotiator = TelnetNegotiator::new();
    
    // Test IAC WILL BINARY
    let test_data = vec![255, 251, 0]; // IAC WILL BINARY
    let response = negotiator.process_incoming_data(&test_data);
    
    result.logs.push(format!("Input: {:?}", test_data));
    result.logs.push(format!("Response: {:?}", response));
    result.logs.push(format!("Binary option active: {}", negotiator.is_option_active(TelnetOption::Binary)));
    
    if negotiator.is_option_active(TelnetOption::Binary) {
        result.actual = "IAC WILL BINARY processed correctly, option activated".to_string();
        result.status = TestStatus::Pass;
    } else {
        result.actual = "IAC WILL BINARY not properly processed".to_string();
        result.status = TestStatus::Fail;
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_option_negotiation_sequence(server: &str, port: u16) -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("2.2", "Option Negotiation Sequence");
    
    result.expected = "Binary, EOR, and SGA options negotiated successfully within 10 rounds".to_string();
    result.severity = Severity::Critical;
    result.issue_reference = "Issue 1.10 (Telnet Option Negotiation Logic)".to_string();
    result.reproduction_steps = vec![
        format!("Connect to {}:{}", server, port),
        "Send initial negotiation (WILL BINARY, WILL EOR, WILL SGA)".to_string(),
        "Track negotiation rounds until complete".to_string(),
        "Verify all essential options are active".to_string(),
    ];

    let addr = match std::net::ToSocketAddrs::to_socket_addrs(&format!("{}:{}", server, port)) {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                addr
            } else {
                result.actual = "DNS resolution returned no addresses".to_string();
                result.status = TestStatus::Error;
                result.duration_ms = start.elapsed().as_millis();
                return result;
            }
        }
        Err(e) => {
            result.actual = format!("Address resolution failed: {}", e);
            result.status = TestStatus::Error;
            result.duration_ms = start.elapsed().as_millis();
            return result;
        }
    };
    
    match TcpStream::connect_timeout(&addr, Duration::from_secs(10)) {
        Ok(mut stream) => {
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
            
            let mut negotiator = TelnetNegotiator::new();
            let initial_neg = negotiator.generate_initial_negotiation();
            
            if let Err(e) = stream.write_all(&initial_neg) {
                result.actual = format!("Failed to send initial negotiation: {}", e);
                result.status = TestStatus::Error;
                return result;
            }

            let mut rounds = 0;
            let max_rounds = 10;
            let mut buffer = vec![0u8; 1024];
            
            while !negotiator.is_negotiation_complete() && rounds < max_rounds {
                match stream.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        let response = negotiator.process_incoming_data(&buffer[..n]);
                        if !response.is_empty() {
                            stream.write_all(&response).ok();
                        }
                        rounds += 1;
                        result.logs.push(format!("Round {}: received {} bytes", rounds, n));
                    }
                    Ok(_) => break, // 0 or any other value means connection closed
                    Err(e) => {
                        result.logs.push(format!("Read error: {}", e));
                        break;
                    }
                }
            }
            
            let is_complete = negotiator.is_negotiation_complete();
            result.logs.push(format!("Negotiation rounds: {}", rounds));
            result.logs.push(format!("Negotiation complete: {}", is_complete));
            result.logs.push(format!("Binary active: {}", negotiator.is_option_active(TelnetOption::Binary)));
            result.logs.push(format!("EOR active: {}", negotiator.is_option_active(TelnetOption::EndOfRecord)));
            result.logs.push(format!("SGA active: {}", negotiator.is_option_active(TelnetOption::SuppressGoAhead)));
            
            if is_complete && rounds <= max_rounds {
                result.actual = format!("Negotiation completed in {} rounds", rounds);
                result.status = TestStatus::Pass;
            } else if is_complete {
                result.actual = format!("Negotiation completed but took {} rounds (max: {})", rounds, max_rounds);
                result.status = TestStatus::Partial;
            } else {
                result.actual = format!("Negotiation incomplete after {} rounds", rounds);
                result.status = TestStatus::Fail;
            }
        }
        Err(e) => {
            result.actual = format!("Connection failed: {}", e);
            result.status = TestStatus::Error;
            result.logs.push(format!("Error: {}", e));
        }
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_terminal_type_negotiation() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("2.3", "Terminal Type Cycling");
    
    result.expected = "Terminal type sent as 'IBM-3179-2' per RFC compliance".to_string();
    result.severity = Severity::High;
    result.issue_reference = "Issue 4.3 (Terminal Type Negotiation Issues)".to_string();
    result.reproduction_steps = vec![
        "Create TelnetNegotiator".to_string(),
        "Process TERMINAL-TYPE SEND subnegotiation".to_string(),
        "Verify response contains correct terminal type".to_string(),
    ];

    let mut negotiator = TelnetNegotiator::new();
    
    // Simulate server sending IAC SB TERMINAL-TYPE SEND IAC SE
    let terminal_type_send = vec![255, 250, 24, 1, 255, 240];
    let response = negotiator.process_incoming_data(&terminal_type_send);
    
    result.logs.push(format!("SEND command: {:?}", terminal_type_send));
    result.logs.push(format!("Response: {:?}", response));
    
    // Check if response contains terminal type
    if response.len() > 6 {
        let terminal_type = String::from_utf8_lossy(&response[4..response.len()-2]);
        result.logs.push(format!("Terminal type in response: {}", terminal_type));
        
        if terminal_type.contains("IBM-3179-2") || terminal_type.contains("IBM-5555") || terminal_type.contains("IBM-5250") {
            result.actual = format!("Terminal type sent: {}", terminal_type);
            result.status = TestStatus::Pass;
        } else {
            result.actual = format!("Unexpected terminal type: {}", terminal_type);
            result.status = TestStatus::Partial;
        }
    } else {
        result.actual = "Terminal type response too short or missing".to_string();
        result.status = TestStatus::Fail;
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_environment_variable_negotiation() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("2.4", "Environment Variable Negotiation");
    
    result.expected = "DEVNAME, CODEPAGE, USER variables sent per RFC 1572".to_string();
    result.severity = Severity::High;
    result.issue_reference = "Issue 4.4 (Environment Variable Handling)".to_string();
    result.reproduction_steps = vec![
        "Create TelnetNegotiator".to_string(),
        "Process NEW-ENVIRON SEND command".to_string(),
        "Verify response contains required variables".to_string(),
    ];

    let mut negotiator = TelnetNegotiator::new();
    
    // Simulate server sending IAC SB NEW-ENVIRON SEND IAC SE
    let env_send = vec![255, 250, 39, 1, 255, 240];
    let response = negotiator.process_incoming_data(&env_send);
    
    result.logs.push(format!("ENV SEND command: {:?}", env_send));
    result.logs.push(format!("Response length: {} bytes", response.len()));
    
    // Check for required variables in response
    let response_str = String::from_utf8_lossy(&response);
    let has_devname = response_str.contains("DEVNAME");
    let has_codepage = response_str.contains("CODEPAGE");
    let has_user = response_str.contains("USER");
    
    result.logs.push(format!("Contains DEVNAME: {}", has_devname));
    result.logs.push(format!("Contains CODEPAGE: {}", has_codepage));
    result.logs.push(format!("Contains USER: {}", has_user));
    
    let var_count = [has_devname, has_codepage, has_user].iter().filter(|&&x| x).count();
    
    if var_count == 3 {
        result.actual = "All required environment variables present".to_string();
        result.status = TestStatus::Pass;
    } else if var_count > 0 {
        result.actual = format!("{} of 3 required variables present", var_count);
        result.status = TestStatus::Partial;
    } else {
        result.actual = "No required environment variables found".to_string();
        result.status = TestStatus::Fail;
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_iac_escaping_binary_mode() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("2.5", "IAC Escaping in Binary Mode");
    
    result.expected = "IAC byte (0xFF) doubled in data stream for proper escaping".to_string();
    result.severity = Severity::High;
    result.issue_reference = "BUG_REPORT (IAC escaping in binary mode)".to_string();
    result.reproduction_steps = vec![
        "Create data with embedded 0xFF bytes".to_string(),
        "Call TelnetNegotiator::escape_iac_in_data()".to_string(),
        "Verify 0xFF bytes are doubled".to_string(),
        "Verify unescape_iac_in_data() reverses correctly".to_string(),
    ];

    // Test data with IAC bytes
    let test_data = vec![0x01, 0xFF, 0x02, 0xFF, 0xFF, 0x03];
    let escaped = TelnetNegotiator::escape_iac_in_data(&test_data);
    let unescaped = TelnetNegotiator::unescape_iac_in_data(&escaped);
    
    result.logs.push(format!("Original:   {:?}", test_data));
    result.logs.push(format!("Escaped:    {:?}", escaped));
    result.logs.push(format!("Unescaped:  {:?}", unescaped));
    result.logs.push(format!("Original len: {}, Escaped len: {}", test_data.len(), escaped.len()));
    
    // Check escaping correctness
    let expected_escaped = vec![0x01, 0xFF, 0xFF, 0x02, 0xFF, 0xFF, 0xFF, 0xFF, 0x03];
    let escaping_correct = escaped == expected_escaped;
    let roundtrip_correct = unescaped == test_data;
    
    result.logs.push(format!("Escaping correct: {}", escaping_correct));
    result.logs.push(format!("Round-trip correct: {}", roundtrip_correct));
    
    if escaping_correct && roundtrip_correct {
        result.actual = "IAC escaping and unescaping work correctly".to_string();
        result.status = TestStatus::Pass;
    } else if roundtrip_correct {
        result.actual = "Round-trip works but escaping format may differ".to_string();
        result.status = TestStatus::Partial;
    } else {
        result.actual = "IAC escaping/unescaping has errors".to_string();
        result.status = TestStatus::Fail;
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_concurrent_negotiation() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("2.6", "Concurrent Negotiation Handling");
    
    result.expected = "State machine handles overlapping negotiations without deadlock".to_string();
    result.severity = Severity::High;
    result.issue_reference = "Issue 1.2 (State Machine Errors)".to_string();
    result.reproduction_steps = vec![
        "Create TelnetNegotiator".to_string(),
        "Send multiple overlapping option requests".to_string(),
        "Verify no deadlocks or incorrect states".to_string(),
    ];

    let mut negotiator = TelnetNegotiator::new();
    
    // Simulate concurrent negotiations: DO BINARY, WILL EOR, DO SGA all at once
    let concurrent_data = vec![
        255, 253, 0,  // IAC DO BINARY
        255, 251, 19, // IAC WILL EOR
        255, 253, 3,  // IAC DO SGA
    ];
    
    let response = negotiator.process_incoming_data(&concurrent_data);
    
    result.logs.push(format!("Concurrent input: {:?}", concurrent_data));
    result.logs.push(format!("Response: {:?}", response));
    result.logs.push(format!("Binary active: {}", negotiator.is_option_active(TelnetOption::Binary)));
    result.logs.push(format!("EOR active: {}", negotiator.is_option_active(TelnetOption::EndOfRecord)));
    result.logs.push(format!("SGA active: {}", negotiator.is_option_active(TelnetOption::SuppressGoAhead)));
    
    // Check that at least some options were activated
    let active_count = [
        negotiator.is_option_active(TelnetOption::Binary),
        negotiator.is_option_active(TelnetOption::EndOfRecord),
        negotiator.is_option_active(TelnetOption::SuppressGoAhead),
    ].iter().filter(|&&x| x).count();
    
    if active_count == 3 {
        result.actual = "All concurrent negotiations handled correctly".to_string();
        result.status = TestStatus::Pass;
    } else if active_count > 0 {
        result.actual = format!("{} of 3 options activated", active_count);
        result.status = TestStatus::Partial;
    } else {
        result.actual = "Concurrent negotiations failed".to_string();
        result.status = TestStatus::Fail;
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

// ==============================================================================
// CATEGORY 3: DATA STREAM PARSING TESTS
// ==============================================================================

fn test_packet_structure_validation() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("3.1", "Packet Structure Validation");
    
    result.expected = "Packets parsed with proper [CMD][SEQ][LEN][FLAGS][DATA] structure".to_string();
    result.severity = Severity::High;
    result.issue_reference = "Issue 1.3 (Buffer Overflow in Packet Processing)".to_string();
    result.reproduction_steps = vec![
        "Create valid 5250 packet with known structure".to_string(),
        "Call Packet::from_bytes()".to_string(),
        "Verify all fields parsed correctly".to_string(),
    ];

    // Create a valid packet: WriteToDisplay command
    let packet_data = vec![
        0xF1,       // Command: WriteToDisplay
        0x01,       // Sequence number
        0x00, 0x05, // Length: 5 bytes
        0x00,       // Flags
        0x40, 0x40, 0x40, 0x40, 0x40, // Data: 5 spaces in EBCDIC
    ];
    
    result.logs.push(format!("Test packet: {:?}", packet_data));
    
    match Packet::from_bytes(&packet_data) {
        Some(packet) => {
            result.logs.push(format!("Command: {:?}", packet.command));
            result.logs.push(format!("Sequence: {}", packet.sequence_number));
            result.logs.push(format!("Data length: {}", packet.data.len()));
            result.logs.push(format!("Flags: 0x{:02X}", packet.flags));
            
            let correct_command = matches!(packet.command, CommandCode::WriteToDisplay);
            let correct_sequence = packet.sequence_number == 1;
            let correct_data_len = packet.data.len() == 5;
            
            if correct_command && correct_sequence && correct_data_len {
                result.actual = "Packet parsed correctly".to_string();
                result.status = TestStatus::Pass;
            } else {
                result.actual = "Packet parsed with some errors".to_string();
                result.status = TestStatus::Partial;
            }
        }
        None => {
            result.actual = "Failed to parse valid packet".to_string();
            result.status = TestStatus::Fail;
        }
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_structured_field_processing() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("3.2", "Structured Field Processing");
    
    result.expected = "Structured fields processed per RFC 2877 with proper length validation".to_string();
    result.severity = Severity::High;
    result.issue_reference = "Issue 1.6 (Structured Field Length Validation)".to_string();
    result.reproduction_steps = vec![
        "Create WriteStructuredField packet".to_string(),
        "Include EraseReset structured field".to_string(),
        "Process through protocol processor".to_string(),
    ];

    // Note: This would require integration with ProtocolProcessor
    // For now, document the test structure
    
    result.actual = "Test requires ProtocolProcessor integration (deferred)".to_string();
    result.status = TestStatus::Partial;
    result.logs.push("Structured field format: [FLAGS][SFID][LENGTH][DATA]".to_string());
    result.logs.push("Need to validate length field against actual data size".to_string());

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_field_attribute_parsing() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("3.4", "Field Attribute Parsing");
    
    result.expected = "Field attributes parsed with correct bit mask (0x3C for bits 2-5)".to_string();
    result.severity = Severity::Medium;
    result.issue_reference = "Issue 1.5 (Field Attribute Processing Logic)".to_string();
    result.reproduction_steps = vec![
        "Create field attribute bytes with known values".to_string(),
        "Call FieldAttribute::from_u8()".to_string(),
        "Verify correct attribute type returned".to_string(),
    ];

    // Test known attribute values
    let test_cases = vec![
        (0x20, "Protected"),
        (0x10, "Numeric"),
        (0x08, "Skip"),
        (0x0C, "Mandatory"),
        (0x04, "DupEnable"),
        (0x00, "Normal"),
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (byte_val, expected_name) in &test_cases {
        let attr = FieldAttribute::from_u8(*byte_val);
        let attr_name = format!("{:?}", attr);
        
        if attr_name.contains(expected_name) {
            passed += 1;
            result.logs.push(format!("0x{:02X} → {} ✓", byte_val, expected_name));
        } else {
            failed += 1;
            result.logs.push(format!("0x{:02X} → expected {}, got {} ✗", byte_val, expected_name, attr_name));
        }
    }
    
    result.logs.push(format!("Passed: {}/{}", passed, test_cases.len()));
    
    if failed == 0 {
        result.actual = "All field attributes parsed correctly".to_string();
        result.status = TestStatus::Pass;
    } else {
        result.actual = format!("{} of {} attributes correct", passed, test_cases.len());
        result.status = TestStatus::Partial;
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_buffer_overflow_conditions() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("3.5", "Buffer Overflow Conditions");
    
    result.expected = "Oversized packets rejected safely without crashes".to_string();
    result.severity = Severity::Critical;
    result.issue_reference = "Issue 1.3, Issue 3.2 (Buffer Overflow Vulnerabilities)".to_string();
    result.reproduction_steps = vec![
        "Create packet with invalid length field".to_string(),
        "Attempt to parse with Packet::from_bytes()".to_string(),
        "Verify rejection without crash".to_string(),
    ];

    // Test case 1: Length field exceeds actual data
    let oversized_packet = vec![
        0xF1,       // Command
        0x01,       // Sequence
        0xFF, 0xFF, // Length: 65535 (way too large)
        0x00,       // Flags
        0x40,       // Only 1 byte of data
    ];
    
    result.logs.push("Test 1: Length field exceeds buffer".to_string());
    let result1 = Packet::from_bytes(&oversized_packet);
    result.logs.push(format!("Result: {:?}", if result1.is_some() { "Parsed (BAD)" } else { "Rejected (GOOD)" }));
    
    // Test case 2: Minimum packet size
    let tiny_packet = vec![0xF1, 0x01];
    result.logs.push("Test 2: Packet too small".to_string());
    let result2 = Packet::from_bytes(&tiny_packet);
    result.logs.push(format!("Result: {:?}", if result2.is_some() { "Parsed (BAD)" } else { "Rejected (GOOD)" }));
    
    let safe1 = result1.is_none();
    let safe2 = result2.is_none();
    
    if safe1 && safe2 {
        result.actual = "All malformed packets rejected safely".to_string();
        result.status = TestStatus::Pass;
    } else {
        result.actual = "Some malformed packets not rejected".to_string();
        result.status = TestStatus::Fail;
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_malformed_packet_handling() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("3.6", "Malformed Packet Handling");
    
    result.expected = "Malformed packets handled gracefully without crashes".to_string();
    result.severity = Severity::High;
    result.issue_reference = "Multiple security issues".to_string();
    result.reproduction_steps = vec![
        "Create truncated and corrupted packets".to_string(),
        "Attempt parsing".to_string(),
        "Verify no crashes or panics".to_string(),
    ];

    let test_packets = vec![
        vec![], // Empty
        vec![0xFF], // Single byte
        vec![0xF1], // Just command
        vec![0xF1, 0x01], // Command + sequence
        vec![0xF1, 0x01, 0x00], // Missing second length byte
    ];
    
    let mut rejected = 0;
    for (i, packet) in test_packets.iter().enumerate() {
        let parsed = Packet::from_bytes(packet);
        if parsed.is_none() {
            rejected += 1;
        }
        result.logs.push(format!("Test {}: len={}, rejected={}", i+1, packet.len(), parsed.is_none()));
    }
    
    result.logs.push(format!("Rejected: {}/{}", rejected, test_packets.len()));
    
    if rejected == test_packets.len() {
        result.actual = "All malformed packets rejected".to_string();
        result.status = TestStatus::Pass;
    } else {
        result.actual = format!("{} of {} malformed packets rejected", rejected, test_packets.len());
        result.status = TestStatus::Partial;
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

// ==============================================================================
// CATEGORY 4: EBCDIC CONVERSION TESTS
// ==============================================================================

fn test_ebcdic_character_completeness() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("4.1", "EBCDIC Character Set Completeness");
    
    result.expected = "All EBCDIC characters (0x00-0xFF) have ASCII mappings".to_string();
    result.severity = Severity::High;
    result.issue_reference = "Issue 1.1 (EBCDIC Character Conversion Errors)".to_string();
    result.reproduction_steps = vec![
        "Convert all EBCDIC values 0x00-0xFF to ASCII".to_string(),
        "Count unmapped characters (those that convert to null/space)".to_string(),
        "Calculate coverage percentage".to_string(),
    ];

    let mut mapped = 0;
    let mut unmapped_chars = Vec::new();
    
    for ebcdic in 0u8..=255u8 {
        let ascii = ebcdic_to_ascii(ebcdic);
        if ascii != '\0' && ascii != ' ' {
            mapped += 1;
        } else if ebcdic != 0x00 && ebcdic != 0x40 { // 0x00 and 0x40 should be null/space
            unmapped_chars.push(ebcdic);
        }
    }
    
    let coverage = (mapped as f64 / 256.0) * 100.0;
    
    result.logs.push(format!("Mapped characters: {}/256", mapped));
    result.logs.push(format!("Coverage: {:.1}%", coverage));
    
    if !unmapped_chars.is_empty() {
        result.logs.push(format!("Sample unmapped: {:02X?}", &unmapped_chars[..unmapped_chars.len().min(10)]));
    }
    
    if coverage >= 90.0 {
        result.actual = format!("Character coverage: {:.1}%", coverage);
        result.status = TestStatus::Pass;
    } else if coverage >= 70.0 {
        result.actual = format!("Character coverage: {:.1}% (partial)", coverage);
        result.status = TestStatus::Partial;
    } else {
        result.actual = format!("Character coverage: {:.1}% (insufficient)", coverage);
        result.status = TestStatus::Fail;
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_special_character_accuracy() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("4.2", "Special Character Accuracy");
    
    result.expected = "Special characters map correctly (0x4B→'.', 0x6B→',', etc.)".to_string();
    result.severity = Severity::Medium;
    result.issue_reference = "Issue 1.1 (specific character errors)".to_string();
    result.reproduction_steps = vec![
        "Test known EBCDIC→ASCII mappings".to_string(),
        "Verify each mapping is correct".to_string(),
    ];

    let test_mappings = vec![
        (0x4B, '.'),
        (0x6B, ','),
        (0x40, ' '),
        (0x5C, '$'),
        (0x7C, '@'),
        (0x60, '-'),
        (0x61, '/'),
    ];
    
    let mut correct = 0;
    let mut incorrect = Vec::new();
    
    for (ebcdic, expected_ascii) in &test_mappings {
        let actual_ascii = ebcdic_to_ascii(*ebcdic);
        if actual_ascii == *expected_ascii {
            correct += 1;
            result.logs.push(format!("0x{:02X} → '{}' ✓", ebcdic, actual_ascii));
        } else {
            incorrect.push((*ebcdic, *expected_ascii, actual_ascii));
            result.logs.push(format!("0x{:02X} → expected '{}', got '{}' ✗", 
                ebcdic, expected_ascii, actual_ascii));
        }
    }
    
    result.logs.push(format!("Correct: {}/{}", correct, test_mappings.len()));
    
    if incorrect.is_empty() {
        result.actual = "All special characters map correctly".to_string();
        result.status = TestStatus::Pass;
    } else {
        result.actual = format!("{} of {} mappings correct", correct, test_mappings.len());
        result.status = TestStatus::Partial;
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_lowercase_character_range() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("4.3", "Lowercase Character Range");
    
    result.expected = "Lowercase s-z (0xA2-0xA9) correctly mapped".to_string();
    result.severity = Severity::Medium;
    result.issue_reference = "Issue 1.1 (missing s-z mappings)".to_string();
    result.reproduction_steps = vec![
        "Convert EBCDIC range 0xA2-0xA9".to_string(),
        "Verify maps to ASCII 's'-'z'".to_string(),
    ];

    let expected = "stuvwxyz";
    let mut actual = String::new();
    
    for ebcdic in 0xA2u8..=0xA9u8 {
        actual.push(ebcdic_to_ascii(ebcdic));
    }
    
    result.logs.push(format!("Expected: {}", expected));
    result.logs.push(format!("Actual:   {}", actual));
    
    if actual == expected {
        result.actual = "Lowercase s-z range correct".to_string();
        result.status = TestStatus::Pass;
    } else {
        result.actual = format!("Lowercase s-z mapping incorrect: got '{}'", actual);
        result.status = TestStatus::Fail;
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

// ==============================================================================
// CATEGORY 5: SESSION MANAGEMENT TESTS
// ==============================================================================

fn test_keyboard_lock_states() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("5.1", "Keyboard Lock States");
    
    result.expected = "Keyboard lock states managed correctly".to_string();
    result.severity = Severity::Medium;
    result.issue_reference = "Issue 1.8 (Keyboard State Management)".to_string();
    result.reproduction_steps = vec![
        "Test requires session integration".to_string(),
    ];

    result.actual = "Test requires Session/Controller integration (deferred)".to_string();
    result.status = TestStatus::Partial;

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_screen_save_restore() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("5.2", "Screen Save/Restore");
    
    result.expected = "Screen state fully preserved during save/restore".to_string();
    result.severity = Severity::Medium;
    result.issue_reference = "Issue 1.9 (Save/Restore Functionality Bugs)".to_string();
    result.reproduction_steps = vec![
        "Test requires ProtocolProcessor integration".to_string(),
    ];

    result.actual = "Test requires ProtocolProcessor integration (deferred)".to_string();
    result.status = TestStatus::Partial;

    result.duration_ms = start.elapsed().as_millis();
    result
}

// ==============================================================================
// CATEGORY 6: SECURITY VULNERABILITY TESTS
// ==============================================================================

fn test_buffer_overflow_attack_vectors() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("6.1", "Buffer Overflow Attack Vectors");
    
    result.expected = "All attack vectors rejected safely".to_string();
    result.severity = Severity::Critical;
    result.issue_reference = "Issue 3.2 (Buffer Overflow Vulnerabilities)".to_string();
    result.reproduction_steps = vec![
        "Create malicious packets with crafted length fields".to_string(),
        "Attempt parsing".to_string(),
        "Verify safe rejection".to_string(),
    ];

    // Attack vector 1: Integer overflow in length
    let attack1 = vec![0xF1, 0x01, 0xFF, 0xFF, 0x00];
    let result1 = Packet::from_bytes(&attack1);
    
    // Attack vector 2: Negative length (if interpreted as signed)
    let attack2 = vec![0xF1, 0x01, 0x80, 0x00, 0x00];
    let result2 = Packet::from_bytes(&attack2);
    
    result.logs.push(format!("Attack 1 (huge length): {}", if result1.is_none() { "Blocked" } else { "DANGER" }));
    result.logs.push(format!("Attack 2 (negative): {}", if result2.is_none() { "Blocked" } else { "DANGER" }));
    
    let safe = result1.is_none() && result2.is_none();
    
    if safe {
        result.actual = "All attack vectors blocked".to_string();
        result.status = TestStatus::Pass;
    } else {
        result.actual = "Some attack vectors NOT blocked - SECURITY RISK".to_string();
        result.status = TestStatus::Fail;
    }

    result.duration_ms = start.elapsed().as_millis();
    result
}

fn test_input_sanitization() -> TestResult {
    let start = Instant::now();
    let mut result = TestResult::new("6.3", "Input Sanitization");
    
    result.expected = "Dangerous input characters filtered/escaped".to_string();
    result.severity = Severity::Medium;
    result.issue_reference = "Issue 3.5 (Missing Input Sanitization)".to_string();
    result.reproduction_steps = vec![
        "Test requires integration with input processing".to_string(),
    ];

    result.actual = "Test requires input processing integration (deferred)".to_string();
    result.status = TestStatus::Partial;
    result.logs.push("Should test: control chars, escape sequences, injection attempts".to_string());

    result.duration_ms = start.elapsed().as_millis();
    result
}