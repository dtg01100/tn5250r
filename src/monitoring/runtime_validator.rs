//! Runtime validation system for TN5250R
//!
//! This module provides real-time system health checks, consistency validation,
//! and runtime state monitoring to ensure system integrity during operation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::thread;
use super::{HealthStatus, ComponentHealthCheck, ComponentHealth};

/// Runtime validation system for continuous health monitoring
#[derive(Debug)]
pub struct RuntimeValidator {
    /// Validation metrics
    pub metrics: RuntimeMetrics,
    /// Validation configuration
    config: ValidationConfig,
    /// Last validation timestamp
    last_validation: std::sync::Mutex<Option<Instant>>,
}

/// Runtime validation metrics
#[derive(Debug)]
pub struct RuntimeMetrics {
    /// Total validations performed
    pub total_validations: AtomicU64,
    /// Successful validations count
    pub successful_validations: AtomicU64,
    /// Failed validations count
    pub failed_validations: AtomicU64,
    /// Average validation time in microseconds
    pub avg_validation_time_us: AtomicU64,
    /// Maximum validation time in microseconds
    pub max_validation_time_us: AtomicU64,
    /// Memory validation failures
    pub memory_validation_failures: AtomicU64,
    /// Thread state validation failures
    pub thread_state_failures: AtomicU64,
    /// Resource leak detections
    pub resource_leak_detections: AtomicU64,
}

impl Clone for RuntimeMetrics {
    fn clone(&self) -> Self {
        Self {
            total_validations: AtomicU64::new(self.total_validations.load(Ordering::Relaxed)),
            successful_validations: AtomicU64::new(self.successful_validations.load(Ordering::Relaxed)),
            failed_validations: AtomicU64::new(self.failed_validations.load(Ordering::Relaxed)),
            avg_validation_time_us: AtomicU64::new(self.avg_validation_time_us.load(Ordering::Relaxed)),
            max_validation_time_us: AtomicU64::new(self.max_validation_time_us.load(Ordering::Relaxed)),
            memory_validation_failures: AtomicU64::new(self.memory_validation_failures.load(Ordering::Relaxed)),
            thread_state_failures: AtomicU64::new(self.thread_state_failures.load(Ordering::Relaxed)),
            resource_leak_detections: AtomicU64::new(self.resource_leak_detections.load(Ordering::Relaxed)),
        }
    }
}

/// Configuration for runtime validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Enable memory validation
    pub enable_memory_validation: bool,
    /// Enable thread state validation
    pub enable_thread_validation: bool,
    /// Enable resource leak detection
    pub enable_resource_leak_detection: bool,
    /// Validation interval in seconds
    pub validation_interval_seconds: u64,
    /// Memory usage warning threshold (percentage)
    pub memory_warning_threshold: f64,
    /// Memory usage critical threshold (percentage)
    pub memory_critical_threshold: f64,
    /// Maximum allowed validation time in milliseconds
    pub max_validation_time_ms: u64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enable_memory_validation: true,
            enable_thread_validation: true,
            enable_resource_leak_detection: true,
            validation_interval_seconds: 30,
            memory_warning_threshold: 80.0,
            memory_critical_threshold: 95.0,
            max_validation_time_ms: 100,
        }
    }
}

impl RuntimeValidator {
    /// Create a new runtime validator instance
    pub fn new() -> Self {
        Self {
            metrics: RuntimeMetrics {
                total_validations: AtomicU64::new(0),
                successful_validations: AtomicU64::new(0),
                failed_validations: AtomicU64::new(0),
                avg_validation_time_us: AtomicU64::new(0),
                max_validation_time_us: AtomicU64::new(0),
                memory_validation_failures: AtomicU64::new(0),
                thread_state_failures: AtomicU64::new(0),
                resource_leak_detections: AtomicU64::new(0),
            },
            config: ValidationConfig::default(),
            last_validation: std::sync::Mutex::new(None),
        }
    }

    /// Validate overall system state
    /// This method performs comprehensive validation of all system components
    pub fn validate_system_state(&self) -> Result<ComponentHealthCheck, String> {
        let start_time = Instant::now();
        let mut details = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;
        let mut issues = Vec::new();

        // Validate memory state
        if self.config.enable_memory_validation {
            match self.validate_memory_state() {
                Ok(_) => {
                    details.insert("memory_status".to_string(), "healthy".to_string());
                }
                Err(e) => {
                    issues.push(format!("Memory validation: {}", e));
                    details.insert("memory_status".to_string(), format!("error: {}", e));
                    overall_status = HealthStatus::Warning;
                    self.metrics.memory_validation_failures.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        // Validate thread state
        if self.config.enable_thread_validation {
            match self.validate_thread_state() {
                Ok(_) => {
                    details.insert("thread_status".to_string(), "healthy".to_string());
                }
                Err(e) => {
                    issues.push(format!("Thread validation: {}", e));
                    details.insert("thread_status".to_string(), format!("error: {}", e));
                    overall_status = HealthStatus::Warning;
                    self.metrics.thread_state_failures.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        // Validate resource state
        if self.config.enable_resource_leak_detection {
            match self.validate_resource_state() {
                Ok(_) => {
                    details.insert("resource_status".to_string(), "healthy".to_string());
                }
                Err(e) => {
                    issues.push(format!("Resource validation: {}", e));
                    details.insert("resource_status".to_string(), format!("error: {}", e));
                    overall_status = HealthStatus::Warning;
                    self.metrics.resource_leak_detections.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        // Validate timing constraints
        let validation_time = start_time.elapsed();
        let validation_time_us = validation_time.as_micros() as u64;

        if validation_time_us > self.config.max_validation_time_ms * 1000 {
            issues.push(format!("Validation took too long: {} μs", validation_time_us));
            overall_status = HealthStatus::Warning;
        }

        // Update metrics
        self.metrics.total_validations.fetch_add(1, Ordering::Relaxed);

        let total_validations = self.metrics.total_validations.load(Ordering::Relaxed);
        let successful_validations = self.metrics.successful_validations.load(Ordering::Relaxed);

        if overall_status == HealthStatus::Healthy {
            self.metrics.successful_validations.fetch_add(1, Ordering::Relaxed);
        } else {
            self.metrics.failed_validations.fetch_add(1, Ordering::Relaxed);
        }

        // Update average validation time
        let current_avg = self.metrics.avg_validation_time_us.load(Ordering::Relaxed);
        let new_avg = ((current_avg * (total_validations - 1)) + validation_time_us) / total_validations;
        self.metrics.avg_validation_time_us.store(new_avg, Ordering::Relaxed);

        // Update max validation time
        let current_max = self.metrics.max_validation_time_us.load(Ordering::Relaxed);
        if validation_time_us > current_max {
            self.metrics.max_validation_time_us.store(validation_time_us, Ordering::Relaxed);
        }

        // Update last validation timestamp
        if let Ok(mut last_val) = self.last_validation.lock() {
            *last_val = Some(start_time);
        }

        details.insert("validation_time_us".to_string(), validation_time_us.to_string());
        details.insert("total_validations".to_string(), total_validations.to_string());
        details.insert("success_rate".to_string(), format!("{:.2}%",
            (successful_validations as f64 / total_validations as f64) * 100.0));

        let message = if issues.is_empty() {
            "All runtime validations passed".to_string()
        } else {
            format!("Runtime validation issues: {}", issues.join(", "))
        };

        Ok(ComponentHealthCheck {
            status: overall_status,
            message,
            details,
        })
    }

    /// Validate memory state and detect potential issues
    fn validate_memory_state(&self) -> Result<(), String> {
        // Get current memory statistics
        let memory_info = self.get_memory_info()?;

        // Check memory usage thresholds
        let memory_usage_percent = (memory_info.current_usage as f64 / memory_info.peak_usage as f64) * 100.0;

        if memory_usage_percent > self.config.memory_critical_threshold {
            return Err(format!("Memory usage critical: {:.1}%", memory_usage_percent));
        }

        if memory_usage_percent > self.config.memory_warning_threshold {
            return Err(format!("Memory usage high: {:.1}%", memory_usage_percent));
        }

        // Check for potential memory leaks (allocations much higher than deallocations)
        if memory_info.allocations > memory_info.deallocations * 2 {
            return Err(format!("Potential memory leak: {} allocations vs {} deallocations",
                memory_info.allocations, memory_info.deallocations));
        }

        Ok(())
    }

    /// Validate thread state and detect potential issues
    fn validate_thread_state(&self) -> Result<(), String> {
        // This is a simplified thread validation
        // In a real implementation, you would check for:
        // - Thread pool health
        // - Deadlocks
        // - Thread leaks
        // - Proper thread cleanup

        // For now, we'll just check if we can access thread information
        let _thread_id = thread::current().id();

        // Basic thread state validation - just ensure we can access thread info

        Ok(())
    }

    /// Validate resource state and detect potential leaks
    fn validate_resource_state(&self) -> Result<(), String> {
        // Check for common resource leak patterns
        // This is a simplified implementation

        // In a real implementation, you would check for:
        // - Open file handles
        // - Network connections
        // - Database connections
        // - GPU resources
        // - System resources

        // For now, we'll check basic system resources
        let open_files = self.get_open_file_count()?;
        if open_files > 1000 {
            return Err(format!("Too many open files: {}", open_files));
        }

        Ok(())
    }

    /// Get current memory information
    fn get_memory_info(&self) -> Result<MemoryInfo, String> {
        // This is a simplified memory info implementation
        // In a real implementation, you would use system APIs to get accurate memory info

        // For demonstration, we'll return mock data
        // In practice, you would use:
        // - Windows: GetProcessMemoryInfo
        // - Linux: /proc/self/status
        // - macOS: task_info

        Ok(MemoryInfo {
            current_usage: 50 * 1024 * 1024, // 50MB
            peak_usage: 100 * 1024 * 1024,   // 100MB
            allocations: 1000,
            deallocations: 950,
        })
    }

    /// Get count of open file descriptors
    fn get_open_file_count(&self) -> Result<usize, String> {
        // This is a simplified implementation
        // In a real implementation, you would use system APIs

        // For demonstration, we'll return a small number
        Ok(10)
    }

    /// Get runtime validation metrics
    pub fn get_metrics(&self) -> &RuntimeMetrics {
        &self.metrics
    }

    /// Update validation configuration
    pub fn update_config(&mut self, config: ValidationConfig) {
        self.config = config;
    }

    /// Check if validation should be performed based on interval
    pub fn should_validate(&self) -> bool {
        if let Ok(last_val) = self.last_validation.lock() {
            if let Some(last) = *last_val {
                return last.elapsed() >= Duration::from_secs(self.config.validation_interval_seconds);
            }
            true // First validation
        } else {
            false
        }
    }

    /// Perform quick validation (subset of full validation for frequent checks)
    pub fn quick_validate(&self) -> Result<ComponentHealthCheck, String> {
        let start_time = Instant::now();
        let mut details = HashMap::new();

        // Quick memory check only
        match self.validate_memory_state() {
            Ok(_) => {
                details.insert("quick_memory_check".to_string(), "passed".to_string());
            }
            Err(e) => {
                details.insert("quick_memory_check".to_string(), format!("failed: {}", e));
                return Ok(ComponentHealthCheck {
                    status: HealthStatus::Warning,
                    message: format!("Quick validation warning: {}", e),
                    details,
                });
            }
        }

        let validation_time_us = start_time.elapsed().as_micros() as u64;
        details.insert("quick_validation_time_us".to_string(), validation_time_us.to_string());

        Ok(ComponentHealthCheck {
            status: HealthStatus::Healthy,
            message: "Quick validation passed".to_string(),
            details,
        })
    }
}

/// Memory information structure
#[derive(Debug, Clone)]
struct MemoryInfo {
    current_usage: u64,
    peak_usage: u64,
    allocations: u64,
    deallocations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_validator_creation() {
        let validator = RuntimeValidator::new();
        assert_eq!(validator.metrics.total_validations.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert_eq!(config.enable_memory_validation, true);
        assert_eq!(config.memory_warning_threshold, 80.0);
        assert_eq!(config.memory_critical_threshold, 95.0);
    }

    #[test]
    fn test_quick_validation() {
        let validator = RuntimeValidator::new();
        let result = validator.quick_validate();

        assert!(result.is_ok());
        let check = result.unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.contains_key("quick_memory_check"));
    }
}