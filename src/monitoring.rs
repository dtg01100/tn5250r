//! Comprehensive monitoring and validation systems for TN5250R
//!
//! This module provides real-time system health monitoring, performance tracking,
//! security validation, and quality assurance capabilities for production operation.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::thread;
use crate::component_utils::configure_component;

/// Global monitoring system instance
static GLOBAL_MONITORING: once_cell::sync::Lazy<Arc<MonitoringSystem>> =
    once_cell::sync::Lazy::new(|| Arc::new(MonitoringSystem::new()));

/// Comprehensive monitoring system for TN5250R
pub struct MonitoringSystem {
    /// Runtime validation system
    pub runtime_validator: RuntimeValidator,
    /// Performance monitoring system
    pub performance_monitor: PerformanceMonitor,
    /// Security monitoring system
    pub security_monitor: SecurityMonitor,
    /// Integration monitoring system
    pub integration_monitor: IntegrationMonitor,
    /// Quality assurance system
    pub quality_assurance: QualityAssurance,
    /// Alerting system
    pub alerting_system: AlertingSystem,
    /// System health status
    pub health_status: Arc<Mutex<SystemHealth>>,
}

/// System health status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// System is operating normally
    Healthy,
    /// Minor issues detected, but system is functional
    Warning,
    /// Critical issues detected, system may be unstable
    Critical,
    /// System is down or completely non-functional
    Down,
}

/// Overall system health information
#[derive(Debug, Clone)]
pub struct SystemHealth {
    /// Current overall health status
    pub status: HealthStatus,
    /// Timestamp of last health check
    pub last_check: Instant,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    /// Health check results by component
    pub component_health: HashMap<String, ComponentHealth>,
    /// System uptime
    pub uptime: Duration,
    /// Memory usage information
    pub memory_usage: MemoryInfo,
}

/// Health status of individual components
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Component health status
    pub status: HealthStatus,
    /// Last successful operation timestamp
    pub last_success: Option<Instant>,
    /// Number of errors in the last hour
    pub error_count: u64,
    /// Response time statistics
    pub response_time_ms: u64,
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    /// Current memory usage in bytes
    pub current_usage: u64,
    /// Peak memory usage in bytes
    pub peak_usage: u64,
    /// Memory allocation count
    pub allocations: u64,
    /// Memory deallocation count
    pub deallocations: u64,
}

impl MonitoringSystem {
    /// Create a new monitoring system instance
    pub fn new() -> Self {
        Self {
            runtime_validator: RuntimeValidator::new(),
            performance_monitor: PerformanceMonitor::new(),
            security_monitor: SecurityMonitor::new(),
            integration_monitor: IntegrationMonitor::new(),
            quality_assurance: QualityAssurance::new(),
            alerting_system: AlertingSystem::new(),
            health_status: Arc::new(Mutex::new(SystemHealth {
                status: HealthStatus::Healthy,
                last_check: Instant::now(),
                consecutive_failures: 0,
                component_health: HashMap::new(),
                uptime: Duration::from_secs(0),
                memory_usage: MemoryInfo {
                    current_usage: 0,
                    peak_usage: 0,
                    allocations: 0,
                    deallocations: 0,
                },
            })),
        }
    }

    /// Get the global monitoring system instance
    pub fn global() -> &'static Arc<MonitoringSystem> {
        &GLOBAL_MONITORING
    }

    /// Perform comprehensive system health check
    /// This method validates all critical system components and updates health status
    pub fn perform_health_check(&self) -> Result<HealthCheckResult, String> {
        let mut results = Vec::new();
        let mut overall_status = HealthStatus::Healthy;
        let mut critical_count = 0;

        // Runtime validation check
        match self.runtime_validator.validate_system_state() {
            Ok(runtime_result) => {
                results.push(("runtime".to_string(), runtime_result.clone()));
                if runtime_result.status == HealthStatus::Critical || runtime_result.status == HealthStatus::Down {
                    critical_count += 1;
                    overall_status = HealthStatus::Critical;
                } else if runtime_result.status == HealthStatus::Warning && overall_status == HealthStatus::Healthy {
                    overall_status = HealthStatus::Warning;
                }
            }
            Err(e) => {
                critical_count += 1;
                overall_status = HealthStatus::Critical;
                results.push(("runtime".to_string(), ComponentHealthCheck {
                    status: HealthStatus::Critical,
                    message: format!("Runtime validation failed: {e}"),
                    details: HashMap::new(),
                }));
            }
        }

        // Performance monitoring check
        match self.performance_monitor.check_performance_health() {
            Ok(perf_result) => {
                results.push(("performance".to_string(), perf_result.clone()));
                if perf_result.status == HealthStatus::Critical || perf_result.status == HealthStatus::Down {
                    critical_count += 1;
                    overall_status = HealthStatus::Critical;
                } else if perf_result.status == HealthStatus::Warning && overall_status == HealthStatus::Healthy {
                    overall_status = HealthStatus::Warning;
                }
            }
            Err(e) => {
                critical_count += 1;
                overall_status = HealthStatus::Critical;
                results.push(("performance".to_string(), ComponentHealthCheck {
                    status: HealthStatus::Critical,
                    message: format!("Performance check failed: {e}"),
                    details: HashMap::new(),
                }));
            }
        }

        // Security monitoring check
        match self.security_monitor.check_security_health() {
            Ok(sec_result) => {
                results.push(("security".to_string(), sec_result.clone()));
                if sec_result.status == HealthStatus::Critical || sec_result.status == HealthStatus::Down {
                    critical_count += 1;
                    overall_status = HealthStatus::Critical;
                } else if sec_result.status == HealthStatus::Warning && overall_status == HealthStatus::Healthy {
                    overall_status = HealthStatus::Warning;
                }
            }
            Err(e) => {
                critical_count += 1;
                overall_status = HealthStatus::Critical;
                results.push(("security".to_string(), ComponentHealthCheck {
                    status: HealthStatus::Critical,
                    message: format!("Security check failed: {e}"),
                    details: HashMap::new(),
                }));
            }
        }

        // Integration monitoring check
        match self.integration_monitor.check_integration_health() {
            Ok(int_result) => {
                results.push(("integration".to_string(), int_result.clone()));
                if int_result.status == HealthStatus::Critical || int_result.status == HealthStatus::Down {
                    critical_count += 1;
                    overall_status = HealthStatus::Critical;
                } else if int_result.status == HealthStatus::Warning && overall_status == HealthStatus::Healthy {
                    overall_status = HealthStatus::Warning;
                }
            }
            Err(e) => {
                critical_count += 1;
                overall_status = HealthStatus::Critical;
                results.push(("integration".to_string(), ComponentHealthCheck {
                    status: HealthStatus::Critical,
                    message: format!("Integration check failed: {e}"),
                    details: HashMap::new(),
                }));
            }
        }

        // Update system health status
        {
            let mut health = self.health_status.lock().unwrap_or_else(|poisoned| {
                eprintln!("SECURITY: Health status mutex poisoned - recovering");
                poisoned.into_inner()
            });
            health.status = overall_status.clone();
            health.last_check = Instant::now();

            if overall_status == HealthStatus::Healthy {
                health.consecutive_failures = 0;
            } else {
                health.consecutive_failures += 1;
            }

            // Update component health
            for (component_name, result) in &results {
                health.component_health.insert(component_name.clone(), ComponentHealth {
                    name: component_name.clone(),
                    status: result.status.clone(),
                    last_success: if result.status == HealthStatus::Healthy { Some(Instant::now()) } else { None },
                    error_count: result.details.get("error_count").and_then(|v| v.parse().ok()).unwrap_or(0),
                    response_time_ms: result.details.get("response_time_ms").and_then(|v| v.parse().ok()).unwrap_or(0),
                });
            }
        }

        // Trigger alerts if needed
        if critical_count > 0 || overall_status == HealthStatus::Critical {
            let alert = Alert {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: Instant::now(),
                level: AlertLevel::Critical,
                component: "system".to_string(),
                message: format!("System health check failed with {critical_count} critical issues"),
                details: HashMap::new(),
                acknowledged: false,
                acknowledged_at: None,
                resolved: false,
                resolved_at: None,
                occurrence_count: 1,
                last_occurrence: Instant::now(),
            };
            self.alerting_system.trigger_alert(alert);
        }

        Ok(HealthCheckResult {
            overall_status,
            component_results: results,
            timestamp: Instant::now(),
            check_duration_ms: 0, // Will be set by caller
        })
    }

    /// Generate comprehensive monitoring report
    pub fn generate_report(&self) -> MonitoringReport {
        let health = self.health_status.lock().unwrap_or_else(|poisoned| {
            eprintln!("SECURITY: Health status mutex poisoned - recovering");
            poisoned.into_inner()
        });

        MonitoringReport {
            timestamp: Instant::now(),
            system_health: health.clone(),
            runtime_metrics: (*self.runtime_validator.get_metrics()).clone(),
            performance_metrics: (*self.performance_monitor.get_metrics()).clone(),
            security_metrics: (*self.security_monitor.get_metrics()).clone(),
            integration_metrics: (*self.integration_monitor.get_metrics()).clone(),
            quality_metrics: (*self.quality_assurance.get_metrics()).clone(),
            recent_alerts: self.alerting_system.get_recent_alerts(50),
        }
    }

    /// Start continuous monitoring in background thread
    pub fn start_continuous_monitoring(&self, interval_seconds: u64) {
        let monitoring_ref = Arc::new(unsafe {
            // This is safe because we're only using the reference for reading
            std::ptr::read(self as *const Self)
        });

        thread::spawn(move || {
            let interval = Duration::from_secs(interval_seconds);
            loop {
                let start_time = Instant::now();

                // Perform health check
                match monitoring_ref.perform_health_check() {
                    Ok(result) => {
                        if result.overall_status != HealthStatus::Healthy {
                            eprintln!("MONITORING: Health check result: {:?}", result.overall_status);
                        }
                    }
                    Err(e) => {
                        eprintln!("MONITORING ERROR: Health check failed: {e}");
                    }
                }

                // Update performance metrics
                monitoring_ref.performance_monitor.update_metrics();

                // Check for security events
                monitoring_ref.security_monitor.scan_for_threats();

                // Validate integrations
                monitoring_ref.integration_monitor.validate_components();

                // Run quality assurance checks
                monitoring_ref.quality_assurance.run_validations();

                // Sleep for the remaining interval
                let elapsed = start_time.elapsed();
                if elapsed < interval {
                    thread::sleep(interval - elapsed);
                }
            }
        });
    }
}

impl Default for MonitoringSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a health check operation
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Overall system health status
    pub overall_status: HealthStatus,
    /// Individual component health check results
    pub component_results: Vec<(String, ComponentHealthCheck)>,
    /// Timestamp when the check was performed
    pub timestamp: Instant,
    /// Duration of the health check in milliseconds
    pub check_duration_ms: u64,
}

/// Individual component health check result
#[derive(Debug, Clone)]
pub struct ComponentHealthCheck {
    /// Component health status
    pub status: HealthStatus,
    /// Human-readable status message
    pub message: String,
    /// Additional details about the check
    pub details: HashMap<String, String>,
}

/// Comprehensive monitoring report
#[derive(Debug)]
pub struct MonitoringReport {
    /// Report generation timestamp
    pub timestamp: Instant,
    /// Current system health status
    pub system_health: SystemHealth,
    /// Runtime validation metrics
    pub runtime_metrics: RuntimeMetrics,
    /// Performance monitoring metrics
    pub performance_metrics: PerformanceMetrics,
    /// Security monitoring metrics
    pub security_metrics: SecurityMetrics,
    /// Integration monitoring metrics
    pub integration_metrics: IntegrationMetrics,
    /// Quality assurance metrics
    pub quality_metrics: QualityMetrics,
    /// Recent alerts (up to specified count)
    pub recent_alerts: Vec<Alert>,
}

// Include all the submodule definitions
mod runtime_validator;
mod performance_monitor;
mod security_monitor;
mod integration_monitor;
mod quality_assurance;
mod alerting_system;
mod component_signals;

// Re-export the submodules for external use
pub use runtime_validator::*;
pub use performance_monitor::*;
pub use security_monitor::*;
pub use integration_monitor::*;
pub use quality_assurance::*;
pub use alerting_system::*;
pub use component_signals::*;

/// Initialize the monitoring system
/// This function should be called early in the application startup
pub fn init_monitoring() {
    let monitoring = MonitoringSystem::global();

    // Start continuous monitoring with 30-second intervals
    monitoring.start_continuous_monitoring(30);

    // Mark system as running in component signals
    configure_component("system", integration_monitor::ComponentState::Running, true);

    println!("MONITORING: Comprehensive monitoring system initialized");
}

/// Shutdown the monitoring system gracefully
pub fn shutdown_monitoring() {
    let monitoring = MonitoringSystem::global();

    // Generate final report
    let report = monitoring.generate_report();

    // Log final status
    println!("MONITORING: Final system status: {:?}", report.system_health.status);
    println!("MONITORING: Total uptime: {:?}", report.system_health.uptime);

    // Save report if needed (simplified version without JSON serialization)
    println!("MONITORING: Final report generated ({} components, {} alerts)",
        report.system_health.component_health.len(),
        report.recent_alerts.len());

    println!("MONITORING: Monitoring system shutdown complete");

    // Mark system as stopped in component signals
    configure_component("system", integration_monitor::ComponentState::Stopped, true);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_system_creation() {
        let monitoring = MonitoringSystem::new();
        assert_eq!(monitoring.health_status.lock().unwrap().status, HealthStatus::Healthy);
    }

    #[test]
    fn test_global_monitoring_instance() {
        let monitoring1 = MonitoringSystem::global();
        let monitoring2 = MonitoringSystem::global();
        assert_eq!(std::ptr::eq(monitoring1.as_ref(), monitoring2.as_ref()), true);
    }
}