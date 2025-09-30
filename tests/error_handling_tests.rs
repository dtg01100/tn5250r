//! Comprehensive tests for error handling improvements
//!
//! Tests all 6 error handling improvements:
//! 1. Sanitized error messages
//! 2. Error rate limiting
//! 3. Error recovery mechanisms
//! 4. Protocol violation detection
//! 5. Sequence number validation
//! 6. DSNR generation

use tn5250r::error::*;
use tn5250r::error_handling::*;
use tn5250r::lib5250::codes::*;
use std::time::Duration;
use std::thread;

#[test]
fn test_error_sanitization_no_sensitive_info() {
    // Test that sensitive file paths are not leaked
    let error = TN5250Error::Network(NetworkError::ConnectionRefused {
        host: "/home/user/secret/config.ini".to_string(),
        port: 23,
    });
    
    let sanitized = sanitize_error(&error);
    
    // User message should not contain file paths
    assert!(!sanitized.user_message.contains("/home/user/secret"));
    assert!(!sanitized.user_message.contains("config.ini"));
    
    // Should have proper error code
    assert_eq!(sanitized.error_code, "NET001");
    assert_eq!(sanitized.category, ErrorCategory::Network);
}

#[test]
fn test_detailed_error_contains_debug_info() {
    let error = TN5250Error::Protocol(ProtocolError::InvalidCommandCode { code: 0xFF });
    let detailed = create_detailed_error(&error);
    
    // Debug message should contain details
    assert!(detailed.debug_message.contains("InvalidCommandCode"));
    assert!(detailed.debug_message.contains("0xFF") || detailed.debug_message.contains("255"));
    
    // User message should be sanitized
    assert!(!detailed.sanitized.user_message.contains("0xFF"));
}

#[test]
fn test_error_rate_limiting_same_errors() {
    let limiter = ErrorRateLimiter::new();
    let error_type = "test_error_spam";
    
    // Should allow first 10 errors per second
    for i in 0..10 {
        assert!(
            limiter.should_log_error(error_type),
            "Should allow error {} within limit",
            i
        );
    }
    
    // Should block 11th error
    assert!(
        !limiter.should_log_error(error_type),
        "Should block error beyond limit"
    );
    
    // Wait for time window to pass
    thread::sleep(Duration::from_millis(1100));
    
    // Should allow errors again after window
    assert!(
        limiter.should_log_error(error_type),
        "Should allow errors after time window reset"
    );
}

#[test]
fn test_connection_rate_limiting() {
    let limiter = ErrorRateLimiter::new();
    
    // Should allow 5 connections per minute
    for i in 0..5 {
        assert!(
            limiter.allow_connection_attempt(),
            "Should allow connection attempt {}",
            i
        );
    }
    
    // Should block 6th connection
    assert!(
        !limiter.allow_connection_attempt(),
        "Should block connection attempt beyond limit"
    );
}

#[test]
fn test_error_rate_limiter_statistics() {
    let limiter = ErrorRateLimiter::new();
    
    // Generate some errors
    for _ in 0..5 {
        limiter.should_log_error("error_type_a");
    }
    for _ in 0..3 {
        limiter.should_log_error("error_type_b");
    }
    
    let stats = limiter.get_statistics();
    assert_eq!(stats.get("error_type_a"), Some(&5));
    assert_eq!(stats.get("error_type_b"), Some(&3));
}

#[test]
fn test_circuit_breaker_opens_after_failures() {
    let breaker = CircuitBreaker::new(3, Duration::from_secs(1));
    
    // Initially closed
    assert_eq!(breaker.get_state(), CircuitState::Closed);
    assert!(breaker.allow_request());
    
    // Record failures
    for i in 0..3 {
        breaker.record_failure();
        if i < 2 {
            assert_eq!(breaker.get_state(), CircuitState::Closed);
        }
    }
    
    // Should be open after threshold
    assert_eq!(breaker.get_state(), CircuitState::Open);
    assert!(!breaker.allow_request(), "Circuit should be open");
}

#[test]
fn test_circuit_breaker_half_open_transition() {
    let breaker = CircuitBreaker::new(2, Duration::from_millis(100));
    
    // Trigger circuit open
    breaker.record_failure();
    breaker.record_failure();
    assert_eq!(breaker.get_state(), CircuitState::Open);
    
    // Wait for timeout
    thread::sleep(Duration::from_millis(150));
    
    // Should transition to half-open
    assert!(breaker.allow_request());
    assert_eq!(breaker.get_state(), CircuitState::HalfOpen);
}

#[test]
fn test_circuit_breaker_recovery() {
    let breaker = CircuitBreaker::new(2, Duration::from_millis(100));
    
    // Open circuit
    breaker.record_failure();
    breaker.record_failure();
    assert_eq!(breaker.get_state(), CircuitState::Open);
    
    // Wait and transition to half-open
    thread::sleep(Duration::from_millis(150));
    breaker.allow_request();
    
    // Record successes to close circuit
    for _ in 0..3 {
        breaker.record_success();
    }
    
    // Should be closed again
    assert_eq!(breaker.get_state(), CircuitState::Closed);
}

#[test]
fn test_retry_policy_backoff() {
    let policy = RetryPolicy::new(5);
    
    // Check delays increase exponentially
    let delay1 = policy.get_delay(0);
    let delay2 = policy.get_delay(1);
    let delay3 = policy.get_delay(2);
    
    assert!(delay2 > delay1, "Delay should increase");
    assert!(delay3 > delay2, "Delay should continue increasing");
    
    // Should respect max delay
    let delay_large = policy.get_delay(100);
    assert!(delay_large.as_secs() <= 10, "Should not exceed max delay");
}

#[test]
fn test_retry_policy_max_attempts() {
    let policy = RetryPolicy::new(3);
    
    assert!(policy.should_retry(0));
    assert!(policy.should_retry(1));
    assert!(policy.should_retry(2));
    assert!(!policy.should_retry(3));
    assert!(!policy.should_retry(4));
}

#[test]
fn test_protocol_violation_tracking() {
    let tracker = ProtocolViolationTracker::new(10);
    let conn_id = "test_connection";
    
    // Record violations below threshold
    for i in 0..9 {
        let should_disconnect = tracker.record_violation(
            conn_id,
            "invalid_command".to_string(),
            format!("Violation {}", i),
        );
        assert!(!should_disconnect, "Should not disconnect before threshold");
    }
    
    // 10th violation should trigger disconnect
    let should_disconnect = tracker.record_violation(
        conn_id,
        "invalid_command".to_string(),
        "Final violation".to_string(),
    );
    assert!(should_disconnect, "Should disconnect at threshold");
}

#[test]
fn test_protocol_violation_retrieval() {
    let tracker = ProtocolViolationTracker::new(10);
    let conn_id = "test_connection";
    
    tracker.record_violation(conn_id, "type1".to_string(), "details1".to_string());
    tracker.record_violation(conn_id, "type2".to_string(), "details2".to_string());
    
    let violations = tracker.get_violations(conn_id);
    assert_eq!(violations.len(), 2);
    assert_eq!(violations[0].violation_type, "type1");
    assert_eq!(violations[1].violation_type, "type2");
}

#[test]
fn test_protocol_violation_report_generation() {
    let tracker = ProtocolViolationTracker::new(10);
    
    tracker.record_violation("conn1", "error1".to_string(), "details1".to_string());
    tracker.record_violation("conn1", "error2".to_string(), "details2".to_string());
    tracker.record_violation("conn2", "error3".to_string(), "details3".to_string());
    
    let report = tracker.generate_report();
    
    assert!(report.contains("conn1"));
    assert!(report.contains("conn2"));
    assert!(report.contains("error1"));
    assert!(report.contains("error2"));
    assert!(report.contains("error3"));
}

#[test]
fn test_protocol_violation_clear() {
    let tracker = ProtocolViolationTracker::new(10);
    let conn_id = "test_connection";
    
    tracker.record_violation(conn_id, "error".to_string(), "details".to_string());
    assert_eq!(tracker.get_violations(conn_id).len(), 1);
    
    tracker.clear_violations(conn_id);
    assert_eq!(tracker.get_violations(conn_id).len(), 0);
}

#[test]
fn test_sequence_number_validation_correct_order() {
    let validator = SequenceValidator::new();
    let session_id = "test_session";
    
    // Should accept correct sequence
    assert!(validator.validate_sequence(session_id, 0).is_ok());
    assert!(validator.validate_sequence(session_id, 1).is_ok());
    assert!(validator.validate_sequence(session_id, 2).is_ok());
}

#[test]
fn test_sequence_number_validation_out_of_order() {
    let validator = SequenceValidator::new();
    let session_id = "test_session";
    
    // Start sequence
    assert!(validator.validate_sequence(session_id, 0).is_ok());
    assert!(validator.validate_sequence(session_id, 1).is_ok());
    
    // Skip ahead - should fail
    let result = validator.validate_sequence(session_id, 5);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Out-of-order"));
}

#[test]
fn test_sequence_number_wraparound() {
    let validator = SequenceValidator::new();
    let session_id = "test_session";
    
    // Manually set to near wraparound
    for i in 0..255 {
        validator.validate_sequence(session_id, i).ok();
    }
    
    // Reset for wraparound
    validator.reset_sequence(session_id);
    
    // Should start from 0 again
    assert!(validator.validate_sequence(session_id, 0).is_ok());
}

#[test]
fn test_sequence_validation_statistics() {
    let validator = SequenceValidator::new();
    let session_id = "test_session";
    
    validator.validate_sequence(session_id, 0).ok();
    validator.validate_sequence(session_id, 5).ok(); // Out of order
    validator.validate_sequence(session_id, 10).ok(); // Out of order
    
    let stats = validator.get_statistics(session_id);
    assert_eq!(stats, 2, "Should track 2 out-of-order packets");
}

#[test]
fn test_dsnr_generation_for_cursor_error() {
    let error = TN5250Error::Protocol(ProtocolError::InvalidCursorPosition {
        row: 100,
        col: 200,
    });
    
    let dsnr_code = DSNRGenerator::generate_dsnr(&error);
    assert_eq!(dsnr_code, DSNR_INVCURSPOS);
}

#[test]
fn test_dsnr_generation_for_buffer_overflow() {
    let error = TN5250Error::Protocol(ProtocolError::ScreenBufferOverflow {
        position: 5000,
        buffer_size: 1920,
    });
    
    let dsnr_code = DSNRGenerator::generate_dsnr(&error);
    assert_eq!(dsnr_code, DSNR_WRTEOD);
}

#[test]
fn test_dsnr_generation_for_field_attribute() {
    let error = TN5250Error::Protocol(ProtocolError::InvalidFieldAttribute {
        attribute: 0xFF,
    });
    
    let dsnr_code = DSNRGenerator::generate_dsnr(&error);
    assert_eq!(dsnr_code, DSNR_INVSFA);
}

#[test]
fn test_dsnr_generation_for_incomplete_data() {
    let error = TN5250Error::Protocol(ProtocolError::IncompleteData {
        expected: 100,
        received: 50,
    });
    
    let dsnr_code = DSNRGenerator::generate_dsnr(&error);
    assert_eq!(dsnr_code, DSNR_FLDEOD);
}

#[test]
fn test_dsnr_response_packet_structure() {
    let dsnr_code = DSNR_INVCURSPOS;
    let details = "Invalid cursor position at row 100, col 200";
    
    let response = DSNRGenerator::create_dsnr_response(dsnr_code, details);
    
    // Check packet structure
    assert!(!response.is_empty());
    assert_eq!(response[0], 0x04, "Should start with ESC");
    assert_eq!(response[1], 0x21, "Should be error code command");
    
    // Check error code is included
    assert!(response.contains(&dsnr_code), "Should contain DSNR code");
}

#[test]
fn test_dsnr_response_packet_length_safety() {
    let dsnr_code = DSNR_WRTEOD;
    let long_details = "x".repeat(1000); // Very long message
    
    let response = DSNRGenerator::create_dsnr_response(dsnr_code, &long_details);
    
    // Should truncate to safe length (100 chars max in implementation)
    let message_part = &response[7..]; // Skip header
    assert!(message_part.len() <= 100, "Should truncate long messages");
}

#[test]
fn test_structured_logger_severity_filtering() {
    let logger = StructuredLogger::new(LogSeverity::Warning);
    
    let debug_error = DetailedError::new(
        ErrorCategory::Internal,
        "Debug message".to_string(),
        "Debug details".to_string(),
        "DBG001".to_string(),
        LogSeverity::Debug,
    );
    
    let warning_error = DetailedError::new(
        ErrorCategory::Protocol,
        "Warning message".to_string(),
        "Warning details".to_string(),
        "WARN001".to_string(),
        LogSeverity::Warning,
    );
    
    // Logger should filter based on severity
    // (In real test would capture output, here we just verify no panic)
    logger.log_detailed(&debug_error);
    logger.log_detailed(&warning_error);
}

#[test]
fn test_error_category_assignment() {
    let net_error = TN5250Error::Network(NetworkError::ConnectionRefused {
        host: "localhost".to_string(),
        port: 23,
    });
    let sanitized = sanitize_error(&net_error);
    assert_eq!(sanitized.category, ErrorCategory::Network);
    
    let proto_error = TN5250Error::Protocol(ProtocolError::InvalidCommandCode { code: 0xFF });
    let sanitized = sanitize_error(&proto_error);
    assert_eq!(sanitized.category, ErrorCategory::Protocol);
}

#[test]
fn test_error_context_tracking() {
    let mut detailed = DetailedError::new(
        ErrorCategory::Protocol,
        "Error occurred".to_string(),
        "Full details".to_string(),
        "ERR001".to_string(),
        LogSeverity::Error,
    );
    
    detailed.add_context("Processing command 0x11".to_string());
    detailed.add_context("At position 42".to_string());
    
    assert_eq!(detailed.context.len(), 2);
    assert!(detailed.context[0].contains("command 0x11"));
    assert!(detailed.context[1].contains("position 42"));
}

#[test]
fn test_integration_all_error_handling_features() {
    // This test validates that all 6 features work together
    
    // 1. Create error and sanitize
    let error = TN5250Error::Protocol(ProtocolError::InvalidCursorPosition {
        row: 100,
        col: 200,
    });
    let sanitized = sanitize_error(&error);
    assert_eq!(sanitized.category, ErrorCategory::Protocol);
    
    // 2. Check rate limiting
    let limiter = ErrorRateLimiter::new();
    assert!(limiter.should_log_error("cursor_error"));
    
    // 3. Use circuit breaker for recovery
    let breaker = CircuitBreaker::new(3, Duration::from_secs(1));
    assert!(breaker.allow_request());
    breaker.record_failure();
    
    // 4. Track protocol violations
    let tracker = ProtocolViolationTracker::new(10);
    let should_disconnect = tracker.record_violation(
        "test_conn",
        "invalid_cursor".to_string(),
        "Row 100 exceeds screen bounds".to_string(),
    );
    assert!(!should_disconnect);
    
    // 5. Validate sequence numbers
    let validator = SequenceValidator::new();
    assert!(validator.validate_sequence("test_session", 0).is_ok());
    
    // 6. Generate DSNR
    let dsnr_code = DSNRGenerator::generate_dsnr(&error);
    assert_eq!(dsnr_code, DSNR_INVCURSPOS);
    let response = DSNRGenerator::create_dsnr_response(dsnr_code, "Cursor error");
    assert!(!response.is_empty());
}

#[test]
fn test_error_recovery_with_retry_and_circuit_breaker() {
    let retry_policy = RetryPolicy::new(3);
    let breaker = CircuitBreaker::new(2, Duration::from_millis(100));
    
    let mut attempts = 0;
    let mut recovered = false;
    
    while retry_policy.should_retry(attempts) && breaker.allow_request() {
        attempts += 1;
        
        // Simulate failure
        if attempts < 2 {
            breaker.record_failure();
        } else {
            // Recover on 2nd attempt
            breaker.record_success();
            recovered = true;
            break;
        }
        
        thread::sleep(retry_policy.get_delay(attempts));
    }
    
    assert!(recovered, "Should recover within retry limit");
    assert_eq!(breaker.get_state(), CircuitState::Closed);
}