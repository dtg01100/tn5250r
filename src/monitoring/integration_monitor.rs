//! Integration monitoring system for TN5250R
//!
//! This module provides component interaction validation, error tracking,
//! and integration health monitoring for production operation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use super::{HealthStatus, ComponentHealthCheck};
use super::component_signals::{set_component_critical, set_component_status};
use super::component_signals::{get_component_signal, is_headless};

/// Integration monitoring system for component interaction validation
#[derive(Debug)]
pub struct IntegrationMonitor {
    /// Integration metrics
    pub metrics: IntegrationMetrics,
    /// Integration configuration
    config: IntegrationConfig,
    /// Component status tracking
    component_status: std::sync::Mutex<HashMap<String, ComponentStatus>>,
    /// Integration event history
    event_history: std::sync::Mutex<VecDeque<IntegrationEvent>>,
}

/// Integration monitoring metrics
#[derive(Debug)]
pub struct IntegrationMetrics {
    /// Total integration events
    pub total_integration_events: AtomicU64,
    /// Component interaction count
    pub component_interactions: AtomicU64,
    /// Integration failures
    pub integration_failures: AtomicU64,
    /// Component communication errors
    pub communication_errors: AtomicU64,
    /// Dependency resolution failures
    pub dependency_failures: AtomicU64,
    /// State synchronization issues
    pub state_sync_issues: AtomicU64,
    /// Resource contention events
    pub resource_contention: AtomicU64,
    /// Deadlock detections
    pub deadlock_detections: AtomicU64,
    /// Successful integrations
    pub successful_integrations: AtomicU64,
    /// Average integration response time in microseconds
    pub avg_integration_time_us: AtomicU64,
}

impl Clone for IntegrationMetrics {
    fn clone(&self) -> Self {
        Self {
            total_integration_events: AtomicU64::new(self.total_integration_events.load(Ordering::Relaxed)),
            component_interactions: AtomicU64::new(self.component_interactions.load(Ordering::Relaxed)),
            integration_failures: AtomicU64::new(self.integration_failures.load(Ordering::Relaxed)),
            communication_errors: AtomicU64::new(self.communication_errors.load(Ordering::Relaxed)),
            dependency_failures: AtomicU64::new(self.dependency_failures.load(Ordering::Relaxed)),
            state_sync_issues: AtomicU64::new(self.state_sync_issues.load(Ordering::Relaxed)),
            resource_contention: AtomicU64::new(self.resource_contention.load(Ordering::Relaxed)),
            deadlock_detections: AtomicU64::new(self.deadlock_detections.load(Ordering::Relaxed)),
            successful_integrations: AtomicU64::new(self.successful_integrations.load(Ordering::Relaxed)),
            avg_integration_time_us: AtomicU64::new(self.avg_integration_time_us.load(Ordering::Relaxed)),
        }
    }
}

/// Integration configuration
#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    /// Enable component status tracking
    pub enable_component_tracking: bool,
    /// Enable integration event logging
    pub enable_event_logging: bool,
    /// Maximum event history size
    pub max_event_history: usize,
    /// Component health check interval in seconds
    pub health_check_interval_seconds: u64,
    /// Maximum allowed integration time in milliseconds
    pub max_integration_time_ms: u64,
    /// Enable deadlock detection
    pub enable_deadlock_detection: bool,
    /// Component timeout threshold in seconds
    pub component_timeout_threshold_seconds: u64,
    /// Dependency check interval in seconds
    pub dependency_check_interval_seconds: u64,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            enable_component_tracking: true,
            enable_event_logging: true,
            max_event_history: 500,
            health_check_interval_seconds: 60,
            max_integration_time_ms: 5000, // 5 seconds
            enable_deadlock_detection: true,
            component_timeout_threshold_seconds: 30,
            dependency_check_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Component status information
#[derive(Debug, Clone)]
pub struct ComponentStatus {
    /// Component name
    pub name: String,
    /// Component type
    pub component_type: String,
    /// Current status
    pub status: ComponentState,
    /// Last health check timestamp
    pub last_health_check: Instant,
    /// Last successful operation
    pub last_success: Option<Instant>,
    /// Error count in the last hour
    pub recent_error_count: u64,
    /// Average response time in microseconds
    pub avg_response_time_us: u64,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Is component critical for system operation
    pub is_critical: bool,
}

/// Component operational states
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ComponentState {
    /// Component is starting up
    Starting,
    /// Component is running normally
    #[default]
    Running,
    /// Component is in warning state
    Warning,
    /// Component is in error state
    Error,
    /// Component is stopped or unavailable
    Stopped,
    /// Component is restarting
    Restarting,
}

/// Integration event types
#[derive(Debug, Clone, PartialEq)]
pub enum IntegrationEventType {
    /// Component interaction
    ComponentInteraction,
    /// Integration failure
    IntegrationFailure,
    /// Communication error
    CommunicationError,
    /// Dependency failure
    DependencyFailure,
    /// State synchronization issue
    StateSyncIssue,
    /// Resource contention
    ResourceContention,
    /// Deadlock detected
    DeadlockDetected,
    /// Component health change
    ComponentHealthChange,
    /// Integration success
    IntegrationSuccess,
}

/// Integration event structure
#[derive(Debug, Clone)]
pub struct IntegrationEvent {
    /// Event timestamp
    pub timestamp: Instant,
    /// Event type
    pub event_type: IntegrationEventType,
    /// Source component
    pub source_component: String,
    /// Target component (if applicable)
    pub target_component: Option<String>,
    /// Event description
    pub description: String,
    /// Additional event details
    pub details: HashMap<String, String>,
    /// Operation duration in microseconds (if applicable)
    pub duration_us: Option<u64>,
    /// Was the operation successful
    pub success: bool,
}

impl IntegrationMonitor {
    /// Create a new integration monitor instance
    pub fn new() -> Self {
        let mut monitor = Self {
            metrics: IntegrationMetrics {
                total_integration_events: AtomicU64::new(0),
                component_interactions: AtomicU64::new(0),
                integration_failures: AtomicU64::new(0),
                communication_errors: AtomicU64::new(0),
                dependency_failures: AtomicU64::new(0),
                state_sync_issues: AtomicU64::new(0),
                resource_contention: AtomicU64::new(0),
                deadlock_detections: AtomicU64::new(0),
                successful_integrations: AtomicU64::new(0),
                avg_integration_time_us: AtomicU64::new(0),
            },
            config: IntegrationConfig::default(),
            component_status: std::sync::Mutex::new(HashMap::new()),
            event_history: std::sync::Mutex::new(VecDeque::new()),
        };

        monitor.register_core_components();
        monitor
    }

    /// Register core system components for monitoring
    fn register_core_components(&mut self) {
        let core_components = vec![
            ("network", "AS400Connection", true),
            ("controller", "TerminalController", true),
            ("terminal", "TerminalScreen", true),
            ("protocol", "ProtocolProcessor", true),
            ("field_manager", "FieldManager", true),
            ("telnet_negotiator", "TelnetNegotiator", true),
            ("ansi_processor", "AnsiProcessor", false),
            ("keyboard", "KeyboardHandler", false),
        ];

        if let Ok(mut components) = self.component_status.lock() {
            for (name, component_type, is_critical) in core_components {
                components.insert(name.to_string(), ComponentStatus {
                    name: name.to_string(),
                    component_type: component_type.to_string(),
                    status: ComponentState::Running,
                    last_health_check: Instant::now(),
                    last_success: Some(Instant::now()),
                    recent_error_count: 0,
                    avg_response_time_us: 0,
                    dependencies: Vec::new(),
                    is_critical,
                });

                // Seed component signals registry with initial state and criticality
                set_component_status(name, ComponentState::Running);
                set_component_critical(name, is_critical);
            }
        }
    }
}

impl Default for IntegrationMonitor {
    fn default() -> Self { Self::new() }
}

impl IntegrationMonitor {
    /// Check integration health and component status
    pub fn check_integration_health(&self) -> Result<ComponentHealthCheck, String> {
        let mut details = HashMap::new();
        let mut issues = Vec::new();
        let mut overall_status = HealthStatus::Healthy;

        // Check critical component status
        let critical_components = self.get_critical_components();
        let unhealthy_critical = critical_components.iter()
            .filter(|(_, status)| status.status != ComponentState::Running)
            .count();

        details.insert("critical_components_total".to_string(), critical_components.len().to_string());
        details.insert("unhealthy_critical_components".to_string(), unhealthy_critical.to_string());

        if unhealthy_critical > 0 {
            issues.push(format!("{} critical components unhealthy", unhealthy_critical));
            overall_status = HealthStatus::Critical;
        }

        // Check for recent integration failures
        let recent_failures = self.get_recent_failures(Duration::from_secs(300)); // Last 5 minutes
        details.insert("recent_failures_5min".to_string(), recent_failures.len().to_string());

        if recent_failures.len() > 10 {
            issues.push(format!("High failure rate: {} failures in 5 minutes", recent_failures.len()));
            if overall_status == HealthStatus::Healthy {
                overall_status = HealthStatus::Warning;
            }
        }

        // Check for communication errors
        let comm_errors = self.metrics.communication_errors.load(Ordering::Relaxed);
        details.insert("communication_errors".to_string(), comm_errors.to_string());

        if comm_errors > 5 {
            issues.push(format!("Communication errors: {}", comm_errors));
            if overall_status == HealthStatus::Healthy {
                overall_status = HealthStatus::Warning;
            }
        }

        // Check for dependency failures
        let dep_failures = self.metrics.dependency_failures.load(Ordering::Relaxed);
        details.insert("dependency_failures".to_string(), dep_failures.to_string());

        if dep_failures > 0 {
            issues.push(format!("Dependency failures: {}", dep_failures));
            if overall_status == HealthStatus::Healthy {
                overall_status = HealthStatus::Warning;
            }
        }

        // Check average integration time
        let avg_time = self.metrics.avg_integration_time_us.load(Ordering::Relaxed);
        details.insert("avg_integration_time_us".to_string(), avg_time.to_string());

        if avg_time > self.config.max_integration_time_ms * 1000 {
            issues.push(format!("Slow integration time: {} μs", avg_time));
            if overall_status == HealthStatus::Healthy {
                overall_status = HealthStatus::Warning;
            }
        }

        let message = if issues.is_empty() {
            "All integrations healthy".to_string()
        } else {
            format!("Integration issues: {}", issues.join(", "))
        };

        Ok(ComponentHealthCheck {
            status: overall_status,
            message,
            details,
        })
    }

    /// Validate component interactions and dependencies
    pub fn validate_components(&self) {
        if !self.config.enable_component_tracking {
            return;
        }

        // Check each component's health
        let components = self.get_all_components();

        for (name, mut status) in components {
            // Simulate component health check
            // In a real implementation, you would actually check each component

            let was_healthy = status.status == ComponentState::Running;
            let is_healthy = self.check_component_health(&name);

            if is_healthy && !was_healthy {
                // Component recovered
                status.status = ComponentState::Running;
                status.last_success = Some(Instant::now());
                status.recent_error_count = 0;

                self.record_integration_event(IntegrationEvent {
                    timestamp: Instant::now(),
                    event_type: IntegrationEventType::ComponentHealthChange,
                    source_component: name.clone(),
                    target_component: None,
                    description: format!("Component {} recovered", name),
                    details: HashMap::new(),
                    duration_us: None,
                    success: true,
                });
            } else if !is_healthy && was_healthy {
                // Component became unhealthy
                status.status = ComponentState::Error;
                status.recent_error_count += 1;

                self.record_integration_event(IntegrationEvent {
                    timestamp: Instant::now(),
                    event_type: IntegrationEventType::ComponentHealthChange,
                    source_component: name.clone(),
                    target_component: None,
                    description: format!("Component {} became unhealthy", name),
                    details: HashMap::new(),
                    duration_us: None,
                    success: false,
                });
            }

            // Update component status
            if let Ok(mut components) = self.component_status.lock() {
                components.insert(name, status);
            }
        }

        // Check for dependency issues
        self.validate_dependencies();

        // Check for deadlock conditions
        if self.config.enable_deadlock_detection {
            self.detect_deadlocks();
        }
    }

    /// Check if a specific component is healthy
    fn check_component_health(&self, component_name: &str) -> bool {
        // First, consult the component signals registry for a real state
        if let Some(sig) = get_component_signal(component_name) {
            return matches!(sig.state, ComponentState::Running);
        }

        // If no explicit signal, apply environment-aware defaults
        // In headless mode, some non-essential components should not cause failures
        let headless = is_headless();

        match component_name {
            // Core components default to healthy unless signaled otherwise
            "network" | "controller" | "terminal" | "protocol" | "field_manager" | "telnet_negotiator" => true,

            // Non-essential in headless environments: treat as healthy when TN5250R_HEADLESS=1
            "ansi_processor" | "keyboard" => headless,

            _ => false,
        }
    }

    /// Validate component dependencies
    fn validate_dependencies(&self) {
        let components = self.get_all_components();

        for (name, status) in components {
            for dependency in &status.dependencies {
                if let Some(dep_status) = self.get_component_status(dependency) {
                    if dep_status.status != ComponentState::Running {
                        self.record_integration_event(IntegrationEvent {
                            timestamp: Instant::now(),
                            event_type: IntegrationEventType::DependencyFailure,
                            source_component: name.clone(),
                            target_component: Some(dependency.clone()),
                            description: format!("Dependency {} is not running", dependency),
                            details: HashMap::new(),
                            duration_us: None,
                            success: false,
                        });
                    }
                }
            }
        }
    }

    /// Detect potential deadlock conditions
    fn detect_deadlocks(&self) {
        // This is a simplified deadlock detection
        // In a real implementation, you would use more sophisticated algorithms

        // For demonstration, we'll simulate occasional deadlock detection
        if self.should_simulate_deadlock() {
            self.record_integration_event(IntegrationEvent {
                timestamp: Instant::now(),
                event_type: IntegrationEventType::DeadlockDetected,
                source_component: "system".to_string(),
                target_component: None,
                description: "Potential deadlock condition detected".to_string(),
                details: HashMap::new(),
                duration_us: None,
                success: false,
            });

            self.metrics.deadlock_detections.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Simulate deadlock detection for testing
    fn should_simulate_deadlock(&self) -> bool {
        // Very low probability for demonstration
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        Instant::now().elapsed().as_nanos().hash(&mut hasher);
        let hash = hasher.finish();
        (hash as f64 / u64::MAX as f64) < 0.0001 // 0.01% chance
    }

    /// Record an integration event
    pub fn record_integration_event(&self, event: IntegrationEvent) {
        // Update metrics
        self.metrics.total_integration_events.fetch_add(1, Ordering::Relaxed);

        match event.event_type {
            IntegrationEventType::ComponentInteraction => {
                self.metrics.component_interactions.fetch_add(1, Ordering::Relaxed);
            }
            IntegrationEventType::IntegrationFailure => {
                self.metrics.integration_failures.fetch_add(1, Ordering::Relaxed);
            }
            IntegrationEventType::CommunicationError => {
                self.metrics.communication_errors.fetch_add(1, Ordering::Relaxed);
            }
            IntegrationEventType::DependencyFailure => {
                self.metrics.dependency_failures.fetch_add(1, Ordering::Relaxed);
            }
            IntegrationEventType::StateSyncIssue => {
                self.metrics.state_sync_issues.fetch_add(1, Ordering::Relaxed);
            }
            IntegrationEventType::ResourceContention => {
                self.metrics.resource_contention.fetch_add(1, Ordering::Relaxed);
            }
            IntegrationEventType::IntegrationSuccess => {
                self.metrics.successful_integrations.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }

        // Update average integration time
        if let Some(duration) = event.duration_us {
            let total_events = self.metrics.total_integration_events.load(Ordering::Relaxed);
            let current_avg = self.metrics.avg_integration_time_us.load(Ordering::Relaxed);
            let new_avg = ((current_avg * (total_events - 1)) + duration) / total_events;
            self.metrics.avg_integration_time_us.store(new_avg, Ordering::Relaxed);
        }

        // Add to event history
        if let Ok(mut history) = self.event_history.lock() {
            history.push_back(event);

            // Trim history
            while history.len() > self.config.max_event_history {
                history.pop_front();
            }
        }
    }

    /// Get all registered components
    pub fn get_all_components(&self) -> HashMap<String, ComponentStatus> {
        if let Ok(components) = self.component_status.lock() {
            components.clone()
        } else {
            HashMap::new()
        }
    }

    /// Get status of a specific component
    pub fn get_component_status(&self, component_name: &str) -> Option<ComponentStatus> {
        if let Ok(components) = self.component_status.lock() {
            components.get(component_name).cloned()
        } else {
            None
        }
    }

    /// Get critical components only
    pub fn get_critical_components(&self) -> HashMap<String, ComponentStatus> {
        self.get_all_components()
            .into_iter()
            .filter(|(_, status)| status.is_critical)
            .collect()
    }

    /// Get recent integration failures
    pub fn get_recent_failures(&self, duration: Duration) -> Vec<IntegrationEvent> {
        let cutoff = Instant::now() - duration;

        if let Ok(history) = self.event_history.lock() {
            history.iter()
                .filter(|event| event.timestamp > cutoff && !event.success)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get integration metrics
    pub fn get_metrics(&self) -> &IntegrationMetrics {
        &self.metrics
    }

    /// Generate integration report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Integration Monitoring Report ===\n\n");

        // Component status summary
        let components = self.get_all_components();
        let running = components.values().filter(|s| s.status == ComponentState::Running).count();
        let warning = components.values().filter(|s| s.status == ComponentState::Warning).count();
        let error = components.values().filter(|s| s.status == ComponentState::Error).count();

        report.push_str("Component Status:\n");
        report.push_str(&format!("  Running: {}\n", running));
        report.push_str(&format!("  Warning: {}\n", warning));
        report.push_str(&format!("  Error: {}\n", error));
        report.push_str(&format!("  Total: {}\n", components.len()));

        // Integration metrics
        report.push_str("\nIntegration Metrics:\n");
        report.push_str(&format!("  Total Events: {}\n", self.metrics.total_integration_events.load(Ordering::Relaxed)));
        report.push_str(&format!("  Component Interactions: {}\n", self.metrics.component_interactions.load(Ordering::Relaxed)));
        report.push_str(&format!("  Integration Failures: {}\n", self.metrics.integration_failures.load(Ordering::Relaxed)));
        report.push_str(&format!("  Communication Errors: {}\n", self.metrics.communication_errors.load(Ordering::Relaxed)));
        report.push_str(&format!("  Avg Integration Time: {} μs\n", self.metrics.avg_integration_time_us.load(Ordering::Relaxed)));

        // Recent events
        let recent_events = self.get_recent_failures(Duration::from_secs(3600));
        report.push_str(&format!("\nRecent Failures (Last Hour): {}\n", recent_events.len()));

        for event in recent_events.iter().take(5) {
            report.push_str(&format!("  {} -> {}: {}\n",
                event.source_component,
                event.target_component.as_deref().unwrap_or("system"),
                event.description));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_monitor_creation() {
        let monitor = IntegrationMonitor::new();
        let components = monitor.get_all_components();
        assert!(!components.is_empty());
        assert!(components.contains_key("network"));
        assert!(components.contains_key("controller"));
    }

    #[test]
    fn test_component_registration() {
        let monitor = IntegrationMonitor::new();
        let network_status = monitor.get_component_status("network");
        assert!(network_status.is_some());

        let status = network_status.unwrap();
        assert_eq!(status.name, "network");
        assert_eq!(status.component_type, "AS400Connection");
        assert!(status.is_critical);
    }

    #[test]
    fn test_critical_components() {
        let monitor = IntegrationMonitor::new();
        let critical = monitor.get_critical_components();
        assert!(!critical.is_empty());

        // All critical components should be marked as critical
        for (_, status) in critical {
            assert!(status.is_critical);
        }
    }

    #[test]
    fn test_integration_event_recording() {
        let monitor = IntegrationMonitor::new();

        let event = IntegrationEvent {
            timestamp: Instant::now(),
            event_type: IntegrationEventType::ComponentInteraction,
            source_component: "network".to_string(),
            target_component: Some("controller".to_string()),
            description: "Test interaction".to_string(),
            details: HashMap::new(),
            duration_us: Some(1000),
            success: true,
        };

        monitor.record_integration_event(event);

        assert_eq!(monitor.metrics.total_integration_events.load(Ordering::Relaxed), 1);
        assert_eq!(monitor.metrics.component_interactions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_initial_integration_health_is_healthy() {
        let monitor = IntegrationMonitor::new();
        let result = monitor.check_integration_health().expect("health check should succeed");
        assert_eq!(result.status, HealthStatus::Healthy);
    }
}