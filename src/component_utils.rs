//! Component configuration and monitoring utilities for TN5250R
//!
//! This module provides centralized functions for component configuration,
//! health check construction, and standardized logging patterns.

use std::collections::HashMap;
use crate::monitoring::{
    HealthStatus, ComponentHealthCheck, ComponentState,
    set_component_status, set_component_critical, set_component_error
};

/// Configure a component with status and criticality in a single operation
/// 
/// This function consolidates the common pattern of setting component status
/// and criticality, reducing code duplication across the monitoring system.
/// 
/// # Arguments
/// * `name` - The component name
/// * `state` - The component's operational state
/// * `is_critical` - Whether the component is critical for system operation
/// 
/// # Examples
/// ```
/// use tn5250r::component_utils::configure_component;
/// use tn5250r::monitoring::integration_monitor::ComponentState;
/// 
/// configure_component("network", ComponentState::Running, true);
/// configure_component("ansi_processor", ComponentState::Running, false);
/// ```
pub fn configure_component(name: &str, state: ComponentState, is_critical: bool) {
    set_component_status(name, state);
    set_component_critical(name, is_critical);
}

/// Configure a component with an initial error state
/// 
/// # Arguments
/// * `name` - The component name
/// * `state` - The component's operational state
/// * `is_critical` - Whether the component is critical for system operation
/// * `error` - Optional error message to attach
pub fn configure_component_with_error(
    name: &str, 
    state: ComponentState, 
    is_critical: bool, 
    error: Option<&str>
) {
    set_component_status(name, state);
    set_component_critical(name, is_critical);
    if let Some(err_msg) = error {
        set_component_error(name, Some(err_msg));
    }
}

/// Builder for ComponentHealthCheck objects
/// 
/// Provides a fluent interface for constructing health check results
/// with common patterns and reasonable defaults.
pub struct ComponentHealthCheckBuilder {
    status: HealthStatus,
    message: String,
    details: HashMap<String, String>,
}

impl Default for ComponentHealthCheckBuilder {
    fn default() -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: String::new(),
            details: HashMap::new(),
        }
    }
}

impl ComponentHealthCheckBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the health status
    pub fn status(mut self, status: HealthStatus) -> Self {
        self.status = status;
        self
    }
    
    /// Set the status message
    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = message.into();
        self
    }
    
    /// Add a detail key-value pair
    pub fn detail<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }
    
    /// Add multiple details from an iterator
    pub fn details<I, K, V>(mut self, details: I) -> Self 
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (key, value) in details {
            self.details.insert(key.into(), value.into());
        }
        self
    }
    
    /// Build the ComponentHealthCheck
    pub fn build(self) -> ComponentHealthCheck {
        ComponentHealthCheck {
            status: self.status,
            message: self.message,
            details: self.details,
        }
    }
}

impl ComponentHealthCheck {
    /// Create a new builder
    pub fn builder() -> ComponentHealthCheckBuilder {
        ComponentHealthCheckBuilder::new()
    }
    
    /// Create a healthy status with a message
    pub fn healthy<S: Into<String>>(message: S) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: message.into(),
            details: HashMap::new(),
        }
    }
    
    /// Create a warning status with a message
    pub fn warning<S: Into<String>>(message: S) -> Self {
        Self {
            status: HealthStatus::Warning,
            message: message.into(),
            details: HashMap::new(),
        }
    }
    
    /// Create a critical/error status with a message
    pub fn critical<S: Into<String>>(message: S) -> Self {
        Self {
            status: HealthStatus::Critical,
            message: message.into(),
            details: HashMap::new(),
        }
    }
    
    /// Create a down status with a message
    pub fn down<S: Into<String>>(message: S) -> Self {
        Self {
            status: HealthStatus::Down,
            message: message.into(),
            details: HashMap::new(),
        }
    }
}

/// Standardized logging macros for consistent messaging patterns
/// 
/// These macros provide consistent formatting for security, monitoring,
/// and error messages throughout the application.
/// Log a security-related message
/// 
/// # Examples
/// ```
/// use tn5250r::component_utils::security_log;
/// 
/// security_log!("Invalid cursor position ({}, {}) - out of bounds", row, col);
/// security_log!("Authentication attempt failed for user: {}", username);
/// ```
#[macro_export]
macro_rules! security_log {
    ($($arg:tt)*) => {
        eprintln!("SECURITY: {}", format!($($arg)*));
    };
}

/// Log a monitoring-related message
/// 
/// # Examples
/// ```
/// use tn5250r::component_utils::monitoring_log;
/// 
/// monitoring_log!("System monitoring initialized");
/// monitoring_log!("Component {} status changed to {:?}", name, status);
/// ```
#[macro_export]
macro_rules! monitoring_log {
    ($($arg:tt)*) => {
        println!("MONITORING: {}", format!($($arg)*));
    };
}

/// Log a debug message with component context
/// 
/// # Examples
/// ```
/// use tn5250r::component_utils::debug_log;
/// 
/// debug_log!("protocol", "Processing command: 0x{:02X}", command);
/// debug_log!("network", "Connection established to {}:{}", host, port);
/// ```
#[macro_export]
macro_rules! debug_log {
    ($component:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!("DEBUG[{}]: {}", $component, format!($($arg)*));
    };
}

/// Standard error message formatters for common error patterns
pub mod error_messages {
    /// Format an "out of bounds" error message
    pub fn out_of_bounds(item_type: &str, index: usize, max: usize) -> String {
        format!("{item_type} index {index} out of bounds (max: {max})")
    }
    
    /// Format an "invalid position" error message
    pub fn invalid_position(position_type: &str, row: usize, col: usize) -> String {
        format!("Invalid {position_type} position: ({row}, {col})")
    }
    
    /// Format a "component state change" message
    pub fn component_state_change(component: &str, from_state: &str, to_state: &str) -> String {
        format!("Component {component} changed from {from_state} to {to_state}")
    }
    
    /// Format an "insufficient data" error message
    pub fn insufficient_data(operation: &str, required: usize, actual: usize) -> String {
        format!("Insufficient data for {operation}: required {required} bytes, got {actual}")
    }
    
    /// Format a "validation failed" error message
    pub fn validation_failed(validation_type: &str, details: &str) -> String {
        format!("{validation_type} validation failed: {details}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_health_check_builder() {
        let health_check = ComponentHealthCheck::builder()
            .status(HealthStatus::Warning)
            .message("Test warning")
            .detail("error_count", "5")
            .detail("response_time_ms", "100")
            .build();
            
        assert_eq!(health_check.status, HealthStatus::Warning);
        assert_eq!(health_check.message, "Test warning");
        assert_eq!(health_check.details.get("error_count"), Some(&"5".to_string()));
        assert_eq!(health_check.details.get("response_time_ms"), Some(&"100".to_string()));
    }
    
    #[test]
    fn test_component_health_check_convenience_methods() {
        let healthy = ComponentHealthCheck::healthy("All good");
        assert_eq!(healthy.status, HealthStatus::Healthy);
        assert_eq!(healthy.message, "All good");
        
        let warning = ComponentHealthCheck::warning("Some issues");
        assert_eq!(warning.status, HealthStatus::Warning);
        assert_eq!(warning.message, "Some issues");
        
        let critical = ComponentHealthCheck::critical("Critical error");
        assert_eq!(critical.status, HealthStatus::Critical);
        assert_eq!(critical.message, "Critical error");
        
        let down = ComponentHealthCheck::down("System down");
        assert_eq!(down.status, HealthStatus::Down);
        assert_eq!(down.message, "System down");
    }
    
    #[test]
    fn test_error_message_formatters() {
        assert_eq!(
            error_messages::out_of_bounds("Field", 10, 5),
            "Field index 10 out of bounds (max: 5)"
        );
        
        assert_eq!(
            error_messages::invalid_position("cursor", 25, 81),
            "Invalid cursor position: (25, 81)"
        );
        
        assert_eq!(
            error_messages::component_state_change("network", "Running", "Error"),
            "Component network changed from Running to Error"
        );
        
        assert_eq!(
            error_messages::insufficient_data("structured field", 10, 5),
            "Insufficient data for structured field: required 10 bytes, got 5"
        );
        
        assert_eq!(
            error_messages::validation_failed("Runtime", "memory leak detected"),
            "Runtime validation failed: memory leak detected"
        );
    }
}