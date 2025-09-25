//! Comprehensive tests for Phase 2C: Enhanced Logging and Debugging
//! 
//! Tests the logging, metrics, diagnostics, and timeout handling capabilities

#[cfg(test)]
mod tests {
    use tn5250r::telnet_negotiation::{TelnetNegotiator, LogLevel};
    use std::time::Duration;

    #[test]
    fn test_negotiator_with_logging_creation() {
        let negotiator = TelnetNegotiator::new_with_logging(LogLevel::Debug);
        
        // Test that the negotiator was created successfully with logging
        assert!(!negotiator.is_negotiation_complete());
        
        // Test that we can get logging methods
        let logger = negotiator.get_logger();
        assert!(!logger.get_log_buffer().is_empty());
        assert!(logger.get_log_buffer()[0].contains("TelnetNegotiator created"));
    }

    #[test]
    fn test_log_level_filtering() {
        let mut negotiator = TelnetNegotiator::new_with_logging(LogLevel::Warn);
        
        // Clear initial messages
        negotiator.get_logger_mut().clear_logs();
        
        // Only Warn and Error should be logged
        negotiator.get_logger_mut().debug("Debug message");
        negotiator.get_logger_mut().info("Info message");
        negotiator.get_logger_mut().warn("Warning message");
        
        let log_buffer = negotiator.get_logger().get_log_buffer();
        assert_eq!(log_buffer.len(), 1); // Only warn message
        
        assert!(log_buffer.iter().any(|entry| entry.contains("Warning message")));
        assert!(!log_buffer.iter().any(|entry| entry.contains("Debug message")));
        assert!(!log_buffer.iter().any(|entry| entry.contains("Info message")));
    }

    #[test]
    fn test_negotiation_metrics_tracking() {
        let mut negotiator = TelnetNegotiator::new_with_logging(LogLevel::Info);
        
        let initial_metrics = negotiator.get_logger().get_metrics();
        assert_eq!(initial_metrics.total_bytes_sent, 0);
        assert_eq!(initial_metrics.total_bytes_received, 0);
        assert_eq!(initial_metrics.successful_negotiations, 0);
        
        // Simulate some data processing
        let test_data = [255, 251, 0]; // IAC WILL BINARY
        let response = negotiator.process_incoming_data(&test_data);
        
        let updated_metrics = negotiator.get_logger().get_metrics();
        assert!(updated_metrics.total_bytes_received > 0);
        assert!(updated_metrics.total_bytes_sent > 0 || !response.is_empty());
    }

    #[test]
    fn test_timeout_functionality() {
        let mut negotiator = TelnetNegotiator::new_with_logging(LogLevel::Info);
        
        // Set a very short timeout for testing
        negotiator.set_negotiation_timeout(Duration::from_millis(1));
        
        // Start negotiation
        let test_data = [255, 251, 0]; // IAC WILL BINARY
        negotiator.process_incoming_data(&test_data);
        
        // Wait a bit to exceed timeout
        std::thread::sleep(Duration::from_millis(10));
        
        // Process more data to trigger timeout check
        negotiator.process_incoming_data(&[]);
        
        // Check timeout was detected
        assert!(negotiator.is_negotiation_timed_out());
        
        let log_buffer = negotiator.get_logger().get_log_buffer();
        assert!(log_buffer.iter().any(|entry| entry.contains("timeout")));
    }

    #[test]
    fn test_connection_diagnostics() {
        let mut negotiator = TelnetNegotiator::new_with_logging(LogLevel::Info);
        
        // Record connection attempt
        negotiator.record_connection_attempt("test-as400.company.com", 23);
        
        let diagnostics = negotiator.get_logger().get_diagnostics();
        assert_eq!(diagnostics.get("target_host").unwrap(), "test-as400.company.com");
        assert_eq!(diagnostics.get("target_port").unwrap(), "23");
        
        // Record success
        negotiator.record_connection_success();
        
        let log_buffer = negotiator.get_logger().get_log_buffer();
        assert!(log_buffer.iter().any(|entry| entry.contains("Attempting connection")));
        assert!(log_buffer.iter().any(|entry| entry.contains("Connection established successfully")));
    }

    #[test]
    fn test_connection_failure_diagnostics() {
        let mut negotiator = TelnetNegotiator::new_with_logging(LogLevel::Info);
        
        let error_message = "Connection refused by remote host";
        negotiator.record_connection_failure(error_message);
        
        let diagnostics = negotiator.get_logger().get_diagnostics();
        assert_eq!(diagnostics.get("connection_error").unwrap(), error_message);
        
        let log_buffer = negotiator.get_logger().get_log_buffer();
        assert!(log_buffer.iter().any(|entry| entry.contains("Connection failed")));
        assert!(log_buffer.iter().any(|entry| entry.contains(error_message)));
        
        let metrics = negotiator.get_logger().get_metrics();
        assert_eq!(metrics.failed_negotiations, 1);
    }

    #[test]
    fn test_negotiation_status_reporting() {
        let mut negotiator = TelnetNegotiator::new_with_logging(LogLevel::Info);
        
        let initial_status = negotiator.get_negotiation_status_report();
        assert_eq!(initial_status.get("negotiation_complete").unwrap(), "false");
        assert_eq!(initial_status.get("timed_out").unwrap(), "false");
        assert_eq!(initial_status.get("active_options").unwrap(), "0");
        assert_eq!(initial_status.get("pending_options").unwrap(), "0");
        
        // Start some negotiation
        let test_data = [255, 251, 0]; // IAC WILL BINARY
        negotiator.process_incoming_data(&test_data);
        
        let updated_status = negotiator.get_negotiation_status_report();
        assert!(updated_status.contains_key("duration_ms"));
        
        // Should have at least one active option now
        let active_count: i32 = updated_status.get("active_options").unwrap().parse().unwrap();
        assert!(active_count > 0);
    }

    #[test]
    fn test_diagnostic_report_generation() {
        let mut negotiator = TelnetNegotiator::new_with_logging(LogLevel::Debug);
        
        // Add some diagnostics
        negotiator.add_connection_diagnostic("server_type".to_string(), "IBM AS/400".to_string());
        negotiator.add_connection_diagnostic("client_version".to_string(), "TN5250R v0.1.0".to_string());
        
        // Process some data
        let test_data = [255, 251, 0, 255, 251, 19]; // IAC WILL BINARY, IAC WILL EOR
        negotiator.process_incoming_data(&test_data);
        
        let report = negotiator.generate_connection_report();
        
        // Verify report contains key sections
        assert!(report.contains("Telnet Negotiation Diagnostic Report"));
        assert!(report.contains("Connection Diagnostics"));
        assert!(report.contains("Negotiation State Summary"));
        assert!(report.contains("Recent Log Entries"));
        assert!(report.contains("server_type: IBM AS/400"));
        assert!(report.contains("client_version: TN5250R v0.1.0"));
        assert!(report.contains("Negotiation complete:"));
        assert!(report.contains("Active negotiation duration:"));
    }

    #[test]
    fn test_negotiation_duration_tracking() {
        let mut negotiator = TelnetNegotiator::new_with_logging(LogLevel::Info);
        
        // Initially no duration should be available
        assert!(negotiator.get_negotiation_duration().is_none());
        
        // Start negotiation
        let test_data = [255, 251, 0]; // IAC WILL BINARY
        negotiator.process_incoming_data(&test_data);
        
        // Now should have duration
        assert!(negotiator.get_negotiation_duration().is_some());
        let duration = negotiator.get_negotiation_duration().unwrap();
        assert!(duration.as_micros() > 0);
        
        // Wait a bit and check duration increases
        std::thread::sleep(Duration::from_millis(5));
        let later_duration = negotiator.get_negotiation_duration().unwrap();
        assert!(later_duration > duration);
    }

    #[test]
    fn test_comprehensive_logging_during_negotiation() {
        let mut negotiator = TelnetNegotiator::new_with_logging(LogLevel::Debug);
        negotiator.get_logger_mut().clear_logs(); // Clear initial messages
        
        // Simulate comprehensive negotiation sequence
        let binary_will = [255, 251, 0]; // IAC WILL BINARY
        let eor_will = [255, 251, 19]; // IAC WILL EOR
        let terminal_type_do = [255, 253, 24]; // IAC DO TERMINAL-TYPE
        
        negotiator.process_incoming_data(&binary_will);
        negotiator.process_incoming_data(&eor_will);
        negotiator.process_incoming_data(&terminal_type_do);
        
        let log_buffer = negotiator.get_logger().get_log_buffer();
        
        // Verify comprehensive logging
        assert!(log_buffer.iter().any(|entry| entry.contains("Starting telnet negotiation")));
        assert!(log_buffer.iter().any(|entry| entry.contains("Received WILL Binary")));
        assert!(log_buffer.iter().any(|entry| entry.contains("Received WILL EndOfRecord")));
        assert!(log_buffer.iter().any(|entry| entry.contains("bytes of telnet data")));
        
        // Check metrics were updated
        let metrics = negotiator.get_logger().get_metrics();
        assert!(metrics.total_bytes_received > 0);
        assert!(negotiator.is_option_active(tn5250r::telnet_negotiation::TelnetOption::Binary));
        assert!(negotiator.is_option_active(tn5250r::telnet_negotiation::TelnetOption::EndOfRecord));
    }

    #[test]
    fn test_error_handling_and_logging() {
        let mut negotiator = TelnetNegotiator::new_with_logging(LogLevel::Debug);
        
        // Test malformed data handling
        let malformed_data = [255, 251]; // Incomplete IAC WILL (missing option)
        negotiator.process_incoming_data(&malformed_data);
        
        // Negotiator should handle gracefully and log appropriately
        let log_buffer = negotiator.get_logger().get_log_buffer();
        assert!(!log_buffer.is_empty());
        
        // Test timeout scenario
        negotiator.set_negotiation_timeout(Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(10));
        negotiator.process_incoming_data(&[]);
        
        let log_buffer_after = negotiator.get_logger().get_log_buffer();
        let timeout_logged = log_buffer_after.iter().any(|entry| entry.contains("timeout"));
        assert!(timeout_logged);
    }

    #[test]
    fn test_performance_metrics_accuracy() {
        let mut negotiator = TelnetNegotiator::new_with_logging(LogLevel::Info);
        
        let test_data_1 = [255, 251, 0]; // 3 bytes
        let test_data_2 = [255, 251, 19]; // 3 bytes
        
        negotiator.process_incoming_data(&test_data_1);
        negotiator.process_incoming_data(&test_data_2);
        
        let metrics = negotiator.get_logger().get_metrics();
        assert_eq!(metrics.total_bytes_received, 6); // 3 + 3 bytes
        
        // Should have sent some responses
        assert!(metrics.total_bytes_sent > 0);
        
        // Check negotiation timing
        assert!(metrics.start_time.elapsed().as_micros() > 0);
    }
}
