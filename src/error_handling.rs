//! Enhanced Error Handling and Logging Infrastructure
//! 
//! This module provides comprehensive error handling improvements:
//! 1. Sanitized error messages that don't leak sensitive information
//! 2. Error rate limiting to prevent spam and DoS
//! 3. Error recovery mechanisms (retry, circuit breaker)
//! 4. Protocol violation detection and tracking
//! 5. Sequence number validation
//! 6. DSNR (Data Stream Negative Response) generation

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::error::*;
use crate::lib5250::codes::*;

/// Severity levels for structured logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogSeverity {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
    Critical = 4,
}

/// Error categories for better handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    Network,
    Protocol,
    Authentication,
    RateLimit,
    Validation,
    Internal,
}

/// Sanitized error message for user display
#[derive(Debug, Clone)]
pub struct SanitizedError {
    /// User-facing message (no sensitive info)
    pub user_message: String,
    /// Error category
    pub category: ErrorCategory,
    /// Error code for documentation lookup
    pub error_code: String,
    /// Timestamp
    pub timestamp: Instant,
}

impl SanitizedError {
    pub fn new(category: ErrorCategory, user_message: String, error_code: String) -> Self {
        Self {
            user_message,
            category,
            error_code,
            timestamp: Instant::now(),
        }
    }
}

/// Detailed error information for debug logging
#[derive(Debug, Clone)]
pub struct DetailedError {
    /// Sanitized error for users
    pub sanitized: SanitizedError,
    /// Full error details (for debug logs only)
    pub debug_message: String,
    /// Stack trace or context
    pub context: Vec<String>,
    /// Severity level
    pub severity: LogSeverity,
}

impl DetailedError {
    pub fn new(
        category: ErrorCategory,
        user_message: String,
        debug_message: String,
        error_code: String,
        severity: LogSeverity,
    ) -> Self {
        Self {
            sanitized: SanitizedError::new(category, user_message, error_code),
            debug_message,
            context: Vec::new(),
            severity,
        }
    }

    pub fn add_context(&mut self, context: String) {
        self.context.push(context);
    }
}

/// Error rate limiter to prevent error spam
pub struct ErrorRateLimiter {
    /// Error counts by type within time window
    error_counts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    /// Maximum errors per type per time window
    max_errors_per_window: usize,
    /// Time window for rate limiting
    time_window: Duration,
    /// Connection attempt tracking
    connection_attempts: Arc<Mutex<Vec<Instant>>>,
    /// Max connection attempts per minute
    max_connection_attempts: usize,
}

impl ErrorRateLimiter {
    pub fn new() -> Self {
        Self {
            error_counts: Arc::new(Mutex::new(HashMap::new())),
            max_errors_per_window: 10, // 10 same errors per second
            time_window: Duration::from_secs(1),
            connection_attempts: Arc::new(Mutex::new(Vec::new())),
            max_connection_attempts: 5, // 5 connection attempts per minute
        }
    }

    /// Check if error should be logged (not rate limited)
    pub fn should_log_error(&self, error_type: &str) -> bool {
        let mut counts = self.error_counts.lock().unwrap();
        let now = Instant::now();
        
        // Get or create entry for this error type
        let timestamps = counts.entry(error_type.to_string()).or_default();
        
        // Remove old timestamps outside the window
        timestamps.retain(|&ts| now.duration_since(ts) < self.time_window);
        
        // Check if we're within limit
        if timestamps.len() >= self.max_errors_per_window {
            false
        } else {
            timestamps.push(now);
            true
        }
    }

    /// Check if connection attempt is allowed
    pub fn allow_connection_attempt(&self) -> bool {
        let mut attempts = self.connection_attempts.lock().unwrap();
        let now = Instant::now();
        
        // Remove attempts older than 1 minute
        attempts.retain(|&ts| now.duration_since(ts) < Duration::from_secs(60));
        
        // Check if we're within limit
        if attempts.len() >= self.max_connection_attempts {
            false
        } else {
            attempts.push(now);
            true
        }
    }

    /// Get current error statistics
    pub fn get_statistics(&self) -> HashMap<String, usize> {
        let counts = self.error_counts.lock().unwrap();
        let now = Instant::now();
        
        counts.iter()
            .map(|(k, v)| {
                let recent_count = v.iter()
                    .filter(|&&ts| now.duration_since(ts) < self.time_window)
                    .count();
                (k.clone(), recent_count)
            })
            .collect()
    }
}

impl Default for ErrorRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Error recovery strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    Retry,
    CircuitBreaker,
    Fallback,
    Abort,
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,  // Normal operation
    Open,    // Failing, reject requests
    HalfOpen, // Testing if recovered
}

/// Circuit breaker for error recovery
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<usize>>,
    success_count: Arc<Mutex<usize>>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    failure_threshold: usize,
    success_threshold: usize,
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(Mutex::new(0)),
            success_count: Arc::new(Mutex::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            failure_threshold,
            success_threshold: 3, // 3 successes to close circuit
            timeout,
        }
    }

    /// Check if operation is allowed
    pub fn allow_request(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        let last_failure = self.last_failure_time.lock().unwrap();
        
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_time) = *last_failure {
                    if Instant::now().duration_since(last_time) > self.timeout {
                        *state = CircuitState::HalfOpen;
                        *self.success_count.lock().unwrap() = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            CircuitState::HalfOpen => true,
        }
    }

    /// Record successful operation
    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap();
        let mut success_count = self.success_count.lock().unwrap();
        let mut failure_count = self.failure_count.lock().unwrap();
        
        *success_count += 1;
        
        match *state {
            CircuitState::HalfOpen => {
                if *success_count >= self.success_threshold {
                    *state = CircuitState::Closed;
                    *failure_count = 0;
                    *success_count = 0;
                }
            },
            CircuitState::Closed => {
                *failure_count = 0;
            },
            _ => {}
        }
    }

    /// Record failed operation
    pub fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        let mut failure_count = self.failure_count.lock().unwrap();
        let mut last_failure = self.last_failure_time.lock().unwrap();
        
        *failure_count += 1;
        *last_failure = Some(Instant::now());
        
        match *state {
            CircuitState::Closed => {
                if *failure_count >= self.failure_threshold {
                    *state = CircuitState::Open;
                }
            },
            CircuitState::HalfOpen => {
                *state = CircuitState::Open;
                *self.success_count.lock().unwrap() = 0;
            },
            _ => {}
        }
    }

    /// Get current circuit state
    pub fn get_state(&self) -> CircuitState {
        *self.state.lock().unwrap()
    }
}

/// Retry policy for transient errors
pub struct RetryPolicy {
    max_attempts: usize,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
}

impl RetryPolicy {
    pub fn new(max_attempts: usize) -> Self {
        Self {
            max_attempts,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }

    /// Calculate delay for given attempt number
    pub fn get_delay(&self, attempt: usize) -> Duration {
        let delay_ms = (self.base_delay.as_millis() as f64 
            * self.backoff_multiplier.powi(attempt as i32)) as u64;
        Duration::from_millis(delay_ms.min(self.max_delay.as_millis() as u64))
    }

    /// Check if should retry
    pub fn should_retry(&self, attempt: usize) -> bool {
        attempt < self.max_attempts
    }

    pub fn max_attempts(&self) -> usize {
        self.max_attempts
    }
}

/// Protocol violation tracker
pub struct ProtocolViolationTracker {
    violations: Arc<Mutex<HashMap<String, Vec<ProtocolViolation>>>>,
    max_violations_per_connection: usize,
}

#[derive(Debug, Clone)]
pub struct ProtocolViolation {
    pub violation_type: String,
    pub timestamp: Instant,
    pub details: String,
}

impl ProtocolViolationTracker {
    pub fn new(max_violations: usize) -> Self {
        Self {
            violations: Arc::new(Mutex::new(HashMap::new())),
            max_violations_per_connection: max_violations,
        }
    }

    /// Record protocol violation
    pub fn record_violation(
        &self,
        connection_id: &str,
        violation_type: String,
        details: String,
    ) -> bool {
        let mut violations = self.violations.lock().unwrap();
        let conn_violations = violations.entry(connection_id.to_string())
            .or_default();
        
        conn_violations.push(ProtocolViolation {
            violation_type,
            timestamp: Instant::now(),
            details,
        });
        
        // Return true if threshold exceeded (should disconnect)
        conn_violations.len() >= self.max_violations_per_connection
    }

    /// Get violations for connection
    pub fn get_violations(&self, connection_id: &str) -> Vec<ProtocolViolation> {
        let violations = self.violations.lock().unwrap();
        violations.get(connection_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Clear violations for connection
    pub fn clear_violations(&self, connection_id: &str) {
        let mut violations = self.violations.lock().unwrap();
        violations.remove(connection_id);
    }

    /// Generate violation report
    pub fn generate_report(&self) -> String {
        let violations = self.violations.lock().unwrap();
        let mut report = String::from("Protocol Violation Report:\n");
        
        for (conn_id, viols) in violations.iter() {
            report.push_str(&format!("\nConnection {}: {} violations\n", conn_id, viols.len()));
            for viol in viols {
                report.push_str(&format!(
                    "  - {}: {}\n",
                    viol.violation_type,
                    viol.details
                ));
            }
        }
        
        report
    }
}

/// Sequence number validator
pub struct SequenceValidator {
    expected_sequence: Arc<Mutex<HashMap<String, u8>>>,
    out_of_order_count: Arc<Mutex<HashMap<String, usize>>>,
}

impl SequenceValidator {
    pub fn new() -> Self {
        Self {
            expected_sequence: Arc::new(Mutex::new(HashMap::new())),
            out_of_order_count: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Validate sequence number
    pub fn validate_sequence(&self, session_id: &str, sequence: u8) -> Result<(), String> {
        let mut expected = self.expected_sequence.lock().unwrap();
        let mut out_of_order = self.out_of_order_count.lock().unwrap();
        
        let expected_seq = expected.entry(session_id.to_string()).or_insert(0);
        
        if sequence == *expected_seq {
            // Correct sequence
            *expected_seq = expected_seq.wrapping_add(1);
            Ok(())
        } else {
            // Out of order
            let count = out_of_order.entry(session_id.to_string()).or_insert(0);
            *count += 1;
            
            Err(format!(
                "Out-of-order packet: expected {}, got {} (count: {})",
                *expected_seq, sequence, *count
            ))
        }
    }

    /// Handle sequence wraparound
    pub fn reset_sequence(&self, session_id: &str) {
        let mut expected = self.expected_sequence.lock().unwrap();
        expected.insert(session_id.to_string(), 0);
    }

    /// Get out-of-order statistics
    pub fn get_statistics(&self, session_id: &str) -> usize {
        let out_of_order = self.out_of_order_count.lock().unwrap();
        *out_of_order.get(session_id).unwrap_or(&0)
    }
}

impl Default for SequenceValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// DSNR (Data Stream Negative Response) generator
pub struct DSNRGenerator;

impl DSNRGenerator {
    /// Generate DSNR code for error
    pub fn generate_dsnr(error: &TN5250Error) -> u8 {
        match error {
            TN5250Error::Protocol(ProtocolError::InvalidCursorPosition { .. }) => DSNR_INVCURSPOS,
            TN5250Error::Protocol(ProtocolError::ScreenBufferOverflow { .. }) => DSNR_WRTEOD,
            TN5250Error::Protocol(ProtocolError::InvalidFieldAttribute { .. }) => DSNR_INVSFA,
            TN5250Error::Protocol(ProtocolError::IncompleteData { .. }) => DSNR_FLDEOD,
            _ => DSNR_UNKNOWN as u8,
        }
    }

    /// Create DSNR response packet
    pub fn create_dsnr_response(error_code: u8, details: &str) -> Vec<u8> {
        let mut response = Vec::new();
        
        // ESC
        response.push(0x04);
        
        // Command code for error
        response.push(0x21); // CMD_WRITE_ERROR_CODE
        
        // Sequence number (placeholder)
        response.push(0x00);
        
        // Length (will be calculated)
        let length_pos = response.len();
        response.extend_from_slice(&[0x00, 0x00]);
        
        // Flags
        response.push(0x00);
        
        // Error code
        response.push(error_code);
        
        // Error message (truncated for safety)
        let safe_details = details.chars().take(100).collect::<String>();
        response.extend_from_slice(safe_details.as_bytes());
        
        // Calculate length
        let total_length = response.len() - length_pos + 2;
        response[length_pos] = (total_length >> 8) as u8;
        response[length_pos + 1] = (total_length & 0xFF) as u8;
        
        response
    }

    /// Log DSNR generation
    pub fn log_dsnr(error_code: u8, context: &str) {
        let message = match error_code {
            DSNR_RESEQ_ERR => EMSG_RESEQ_ERR,
            DSNR_INVCURSPOS => EMSG_INVCURSPOS,
            DSNR_RAB4WSA => EMSG_RAB4WSA,
            DSNR_INVSFA => EMSG_INVSFA,
            DSNR_FLDEOD => EMSG_FLDEOD,
            DSNR_FMTOVF => EMSG_FMTOVF,
            DSNR_WRTEOD => EMSG_WRTEOD,
            DSNR_SOHLEN => EMSG_SOHLEN,
            DSNR_ROLLPARM => EMSG_ROLLPARM,
            DSNR_NO_ESC => EMSG_NO_ESC,
            DSNR_INV_WECW => EMSG_INV_WECW,
            _ => "Unknown DSNR error",
        };
        
        eprintln!("DSNR Generated: 0x{error_code:02X} - {message} ({context})");
    }
}

/// Sanitize error for user display
pub fn sanitize_error(error: &TN5250Error) -> SanitizedError {
    match error {
        TN5250Error::Network(e) => match e {
            NetworkError::ConnectionRefused { .. } => {
                SanitizedError::new(
                    ErrorCategory::Network,
                    "Connection refused by remote server".to_string(),
                    "NET001".to_string(),
                )
            },
            NetworkError::Timeout { .. } => {
                SanitizedError::new(
                    ErrorCategory::Network,
                    "Connection timeout".to_string(),
                    "NET002".to_string(),
                )
            },
            NetworkError::DnsResolution { .. } => {
                SanitizedError::new(
                    ErrorCategory::Network,
                    "Could not resolve server address".to_string(),
                    "NET003".to_string(),
                )
            },
            NetworkError::ConnectionLost { .. } => {
                SanitizedError::new(
                    ErrorCategory::Network,
                    "Connection lost".to_string(),
                    "NET004".to_string(),
                )
            },
            _ => SanitizedError::new(
                ErrorCategory::Network,
                "Network error occurred".to_string(),
                "NET000".to_string(),
            ),
        },
        TN5250Error::Protocol(e) => match e {
            ProtocolError::InvalidCommandCode { .. } => {
                SanitizedError::new(
                    ErrorCategory::Protocol,
                    "Invalid protocol command received".to_string(),
                    "PROTO001".to_string(),
                )
            },
            ProtocolError::IncompleteData { .. } => {
                SanitizedError::new(
                    ErrorCategory::Protocol,
                    "Incomplete data received".to_string(),
                    "PROTO002".to_string(),
                )
            },
            _ => SanitizedError::new(
                ErrorCategory::Protocol,
                "Protocol error occurred".to_string(),
                "PROTO000".to_string(),
            ),
        },
        _ => SanitizedError::new(
            ErrorCategory::Internal,
            "An error occurred".to_string(),
            "ERR000".to_string(),
        ),
    }
}

/// Create detailed error for debug logging
pub fn create_detailed_error(error: &TN5250Error) -> DetailedError {
    let sanitized = sanitize_error(error);
    let debug_message = format!("{error:?}");
    let severity = match error {
        TN5250Error::Network(_) => LogSeverity::Error,
        TN5250Error::Protocol(_) => LogSeverity::Warning,
        TN5250Error::Recovery(_) => LogSeverity::Warning,
        _ => LogSeverity::Error,
    };
    
    DetailedError::new(
        sanitized.category,
        sanitized.user_message,
        debug_message,
        sanitized.error_code,
        severity,
    )
}

/// Structured logger for error handling
pub struct StructuredLogger {
    min_severity: LogSeverity,
}

impl StructuredLogger {
    pub fn new(min_severity: LogSeverity) -> Self {
        Self { min_severity }
    }

    pub fn log_sanitized(&self, error: &SanitizedError) {
        println!(
            "[ERROR] {} (Code: {})",
            error.user_message,
            error.error_code
        );
    }

    pub fn log_detailed(&self, error: &DetailedError) {
        if error.severity >= self.min_severity {
            eprintln!(
                "[{:?}] User: {} | Debug: {} | Code: {}",
                error.severity,
                error.sanitized.user_message,
                error.debug_message,
                error.sanitized.error_code
            );
            
            for ctx in &error.context {
                eprintln!("  Context: {ctx}");
            }
        }
    }

    pub fn log_recovery_attempt(&self, attempt: usize, max_attempts: usize, operation: &str) {
        eprintln!(
            "[RECOVERY] Attempt {attempt}/{max_attempts} for operation: {operation}"
        );
    }

    pub fn log_recovery_success(&self, operation: &str) {
        println!("[RECOVERY] Successfully recovered: {operation}");
    }

    pub fn log_recovery_failure(&self, operation: &str) {
        eprintln!("[RECOVERY] Failed to recover: {operation}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_rate_limiter() {
        let limiter = ErrorRateLimiter::new();
        
        // Should allow first 10 errors
        for i in 0..10 {
            assert!(limiter.should_log_error("test_error"), "Failed at attempt {i}");
        }
        
        // Should block 11th error
        assert!(!limiter.should_log_error("test_error"));
    }

    #[test]
    fn test_connection_rate_limit() {
        let limiter = ErrorRateLimiter::new();
        
        // Should allow first 5 connections
        for _ in 0..5 {
            assert!(limiter.allow_connection_attempt());
        }
        
        // Should block 6th connection
        assert!(!limiter.allow_connection_attempt());
    }

    #[test]
    fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(3, Duration::from_secs(1));
        
        // Should be closed initially
        assert_eq!(breaker.get_state(), CircuitState::Closed);
        assert!(breaker.allow_request());
        
        // Record failures
        for _ in 0..3 {
            breaker.record_failure();
        }
        
        // Should be open after threshold
        assert_eq!(breaker.get_state(), CircuitState::Open);
        assert!(!breaker.allow_request());
    }

    #[test]
    fn test_sequence_validator() {
        let validator = SequenceValidator::new();
        
        // Should accept correct sequence
        assert!(validator.validate_sequence("test", 0).is_ok());
        assert!(validator.validate_sequence("test", 1).is_ok());
        
        // Should reject out-of-order
        assert!(validator.validate_sequence("test", 5).is_err());
    }

    #[test]
    fn test_protocol_violation_tracker() {
        let tracker = ProtocolViolationTracker::new(10);
        
        // Should track violations
        for i in 0..9 {
            assert!(!tracker.record_violation(
                "conn1",
                "invalid_command".to_string(),
                format!("Violation {i}"),
            ));
        }
        
        // Should return true at threshold
        assert!(tracker.record_violation(
            "conn1",
            "invalid_command".to_string(),
            "Final violation".to_string(),
        ));
    }

    #[test]
    fn test_dsnr_generation() {
        let error = TN5250Error::Protocol(ProtocolError::InvalidCursorPosition {
            row: 100,
            col: 200,
        });
        
        let dsnr_code = DSNRGenerator::generate_dsnr(&error);
        assert_eq!(dsnr_code, DSNR_INVCURSPOS);
        
        let response = DSNRGenerator::create_dsnr_response(dsnr_code, "Test error");
        assert!(!response.is_empty());
    }

    #[test]
    fn test_error_sanitization() {
        let error = TN5250Error::Network(NetworkError::ConnectionRefused {
            host: "/secret/path/server".to_string(),
            port: 23,
        });
        
        let sanitized = sanitize_error(&error);
        
        // Should not contain sensitive path
        assert!(!sanitized.user_message.contains("/secret/path"));
        assert_eq!(sanitized.category, ErrorCategory::Network);
        assert_eq!(sanitized.error_code, "NET001");
    }
}