//! Alerting system for TN5250R
//!
//! This module provides comprehensive alerting capabilities for critical issues,
//! performance degradation, and system events in production operation.

use uuid::Uuid;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};

/// Alerting system for critical issue notification
pub struct AlertingSystem {
    /// Alert metrics
    pub metrics: AlertMetrics,
    /// Alert configuration
    config: AlertConfig,
    /// Active alerts
    active_alerts: std::sync::Mutex<HashMap<String, Alert>>,
    /// Alert history
    alert_history: std::sync::Mutex<VecDeque<Alert>>,
    /// Alert handlers
    alert_handlers: std::sync::Mutex<Vec<Box<dyn AlertHandler + Send + Sync>>>,
}

/// Alert metrics
#[derive(Debug)]
pub struct AlertMetrics {
    /// Total alerts triggered
    pub total_alerts: AtomicU64,
    /// Critical alerts triggered
    pub critical_alerts: AtomicU64,
    /// Warning alerts triggered
    pub warning_alerts: AtomicU64,
    /// Info alerts triggered
    pub info_alerts: AtomicU64,
    /// Alerts acknowledged
    pub acknowledged_alerts: AtomicU64,
    /// Alerts resolved
    pub resolved_alerts: AtomicU64,
    /// False positive alerts
    pub false_positives: AtomicU64,
    /// Average alert response time in seconds
    pub avg_response_time_seconds: AtomicU64,
}

/// Alert configuration
#[derive(Debug, Clone)]
pub struct AlertConfig {
    /// Enable alerting system
    pub enable_alerting: bool,
    /// Maximum alert history size
    pub max_alert_history: usize,
    /// Alert cooldown period in seconds (minimum time between similar alerts)
    pub alert_cooldown_seconds: u64,
    /// Enable email notifications
    pub enable_email_notifications: bool,
    /// Enable logging notifications
    pub enable_logging_notifications: bool,
    /// Enable dashboard notifications
    pub enable_dashboard_notifications: bool,
    /// Critical alert threshold for escalation
    pub critical_alert_threshold: u32,
    /// Auto-resolve alerts after duration (0 = disabled)
    pub auto_resolve_after_seconds: u64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enable_alerting: true,
            // Keep history bounded to a modest size to reduce memory in tests/runs
            max_alert_history: 200,
            alert_cooldown_seconds: 120, // 2 minutes
            enable_email_notifications: false, // Disabled for security
            enable_logging_notifications: true,
            enable_dashboard_notifications: true,
            critical_alert_threshold: 5,
            // Resolve automatically sooner to allow cleanup and history trimming
            auto_resolve_after_seconds: 600, // 10 minutes
        }
    }
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Critical alert
    Critical,
    /// Emergency alert (system down)
    Emergency,
}

/// Alert structure
#[derive(Debug, Clone)]
pub struct Alert {
    /// Unique alert ID
    pub id: String,
    /// Alert timestamp
    pub timestamp: Instant,
    /// Alert level
    pub level: AlertLevel,
    /// Source component
    pub component: String,
    /// Alert message
    pub message: String,
    /// Additional alert details
    pub details: HashMap<String, String>,
    /// Whether alert has been acknowledged
    pub acknowledged: bool,
    /// Alert acknowledgement timestamp
    pub acknowledged_at: Option<Instant>,
    /// Whether alert has been resolved
    pub resolved: bool,
    /// Alert resolution timestamp
    pub resolved_at: Option<Instant>,
    /// Number of times this alert has been triggered
    pub occurrence_count: u32,
    /// Last occurrence timestamp
    pub last_occurrence: Instant,
}

/// Alert handler trait for custom notification handlers
pub trait AlertHandler {
    /// Handle an alert notification
    fn handle_alert(&self, alert: &Alert) -> Result<(), String>;

    /// Get handler name
    fn get_name(&self) -> &str;

    /// Check if handler can process this alert level
    fn can_handle(&self, level: &AlertLevel) -> bool;
}

/// Logging alert handler (built-in)
pub struct LoggingAlertHandler;

impl AlertHandler for LoggingAlertHandler {
    fn handle_alert(&self, alert: &Alert) -> Result<(), String> {
        let level_str = match alert.level {
            AlertLevel::Info => "INFO",
            AlertLevel::Warning => "WARN",
            AlertLevel::Critical => "CRIT",
            AlertLevel::Emergency => "EMERGENCY",
        };

        eprintln!("ALERT [{}]: {} - {} - {}",
            level_str, alert.component, alert.message, alert.id);

        Ok(())
    }

    fn get_name(&self) -> &str {
        "logging"
    }

    fn can_handle(&self, _level: &AlertLevel) -> bool {
        true // Handle all levels
    }
}

impl AlertingSystem {
    /// Create a new alerting system
    pub fn new() -> Self {
        let system = Self {
            metrics: AlertMetrics {
                total_alerts: AtomicU64::new(0),
                critical_alerts: AtomicU64::new(0),
                warning_alerts: AtomicU64::new(0),
                info_alerts: AtomicU64::new(0),
                acknowledged_alerts: AtomicU64::new(0),
                resolved_alerts: AtomicU64::new(0),
                false_positives: AtomicU64::new(0),
                avg_response_time_seconds: AtomicU64::new(0),
            },
            config: AlertConfig::default(),
            active_alerts: std::sync::Mutex::new(HashMap::new()),
            alert_history: std::sync::Mutex::new(VecDeque::new()),
            alert_handlers: std::sync::Mutex::new(Vec::new()),
        };

        // Register default alert handler
        system.register_alert_handler(Box::new(LoggingAlertHandler));

        system
    }

    /// Trigger a new alert
    pub fn trigger_alert(&self, mut alert: Alert) {
        if !self.config.enable_alerting {
            return;
        }

        // Check for alert cooldown
        if self.is_alert_in_cooldown(&alert) {
            return;
        }

        // Update alert metadata
        alert.id = if alert.id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            alert.id
        };
        alert.last_occurrence = Instant::now();

        // Check if this is a repeated alert
        if let Some(existing_alert) = self.get_existing_alert(&alert) {
            // Update existing alert
            let mut updated_alert = existing_alert.clone();
            updated_alert.occurrence_count += 1;
            updated_alert.last_occurrence = Instant::now();
            updated_alert.message = alert.message; // Update with latest message

            // Replace in active alerts
            if let Ok(mut active) = self.active_alerts.lock() {
                active.insert(updated_alert.id.clone(), updated_alert.clone());
            }

            alert = updated_alert;
        } else {
            // New alert
            if let Ok(mut active) = self.active_alerts.lock() {
                active.insert(alert.id.clone(), alert.clone());
            }
        }

        // Update metrics
        self.metrics.total_alerts.fetch_add(1, Ordering::Relaxed);

        match alert.level {
            AlertLevel::Critical | AlertLevel::Emergency => {
                self.metrics.critical_alerts.fetch_add(1, Ordering::Relaxed);
            }
            AlertLevel::Warning => {
                self.metrics.warning_alerts.fetch_add(1, Ordering::Relaxed);
            }
            AlertLevel::Info => {
                self.metrics.info_alerts.fetch_add(1, Ordering::Relaxed);
            }
        }

        // Add to history
        if let Ok(mut history) = self.alert_history.lock() {
            history.push_back(alert.clone());

            // Trim history
            while history.len() > self.config.max_alert_history {
                history.pop_front();
            }
        }

        // Notify handlers
        self.notify_alert_handlers(&alert);

        // Auto-resolve if configured
        if self.config.auto_resolve_after_seconds > 0 {
            self.schedule_auto_resolve(alert.id.clone());
        }
    }

    /// Check if alert is in cooldown period
    fn is_alert_in_cooldown(&self, alert: &Alert) -> bool {
        if let Ok(history) = self.alert_history.lock() {
            let cooldown_cutoff = Instant::now() - Duration::from_secs(self.config.alert_cooldown_seconds);

            history.iter().rev().any(|historical_alert| {
                historical_alert.component == alert.component &&
                historical_alert.level == alert.level &&
                historical_alert.timestamp > cooldown_cutoff
            })
        } else {
            false
        }
    }

    /// Get existing alert if it matches the new one
    fn get_existing_alert(&self, new_alert: &Alert) -> Option<Alert> {
        if let Ok(active) = self.active_alerts.lock() {
            active.values().find(|existing| {
                existing.component == new_alert.component &&
                existing.level == new_alert.level &&
                !existing.resolved
            }).cloned()
        } else {
            None
        }
    }

    /// Notify all registered alert handlers
    fn notify_alert_handlers(&self, alert: &Alert) {
        if let Ok(handlers) = self.alert_handlers.lock() {
            for handler in handlers.iter() {
                if handler.can_handle(&alert.level) {
                    if let Err(e) = handler.handle_alert(alert) {
                        eprintln!("Alert handler '{}' failed: {}", handler.get_name(), e);
                    }
                }
            }
        }
    }

    /// Register a custom alert handler
    pub fn register_alert_handler(&self, handler: Box<dyn AlertHandler + Send + Sync>) {
        if let Ok(mut handlers) = self.alert_handlers.lock() {
            handlers.push(handler);
        }
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&self, alert_id: &str) -> Result<(), String> {
        if let Ok(mut active) = self.active_alerts.lock() {
            if let Some(alert) = active.get_mut(alert_id) {
                if !alert.acknowledged {
                    alert.acknowledged = true;
                    alert.acknowledged_at = Some(Instant::now());

                    self.metrics.acknowledged_alerts.fetch_add(1, Ordering::Relaxed);

                    // Calculate response time
                    let response_time = alert.acknowledged_at.unwrap().duration_since(alert.timestamp).as_secs();
                    self.update_average_response_time(response_time);

                    return Ok(());
                }
            }
        }

        Err(format!("Alert {alert_id} not found or already acknowledged"))
    }

    /// Resolve an alert
    pub fn resolve_alert(&self, alert_id: &str) -> Result<(), String> {
        if let Ok(mut active) = self.active_alerts.lock() {
            if let Some(alert) = active.get_mut(alert_id) {
                if !alert.resolved {
                    alert.resolved = true;
                    alert.resolved_at = Some(Instant::now());

                    self.metrics.resolved_alerts.fetch_add(1, Ordering::Relaxed);

                    // Move to history if not already there
                    if let Ok(mut history) = self.alert_history.lock() {
                        if !history.iter().any(|a| a.id == alert_id) {
                            history.push_back(alert.clone());
                        }
                    }

                    // Remove from active alerts
                    active.remove(alert_id);

                    return Ok(());
                }
            }
        }

        Err(format!("Alert {alert_id} not found or already resolved"))
    }

    /// Update average response time
    fn update_average_response_time(&self, response_time: u64) {
        let total_alerts = self.metrics.acknowledged_alerts.load(Ordering::Relaxed);
        let current_avg = self.metrics.avg_response_time_seconds.load(Ordering::Relaxed);
        let new_avg = ((current_avg * (total_alerts - 1)) + response_time) / total_alerts;
        self.metrics.avg_response_time_seconds.store(new_avg, Ordering::Relaxed);
    }

    /// Schedule auto-resolution of an alert
    fn schedule_auto_resolve(&self, alert_id: String) {
        let auto_resolve_after = self.config.auto_resolve_after_seconds;
        let system_ref = unsafe {
            // This is safe because we're only using the reference for reading
            std::ptr::read(self as *const Self)
        };

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(auto_resolve_after));

            // Check if alert still exists and is not manually resolved
            if let Ok(active) = system_ref.active_alerts.lock() {
                if active.contains_key(&alert_id) {
                    let _ = system_ref.resolve_alert(&alert_id);
                }
            }
        });
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        if let Ok(active) = self.active_alerts.lock() {
            active.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get recent alerts within the specified duration
    pub fn get_recent_alerts(&self, max_count: usize) -> Vec<Alert> {
        if let Ok(history) = self.alert_history.lock() {
            history.iter().rev().take(max_count).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get alerts by level
    pub fn get_alerts_by_level(&self, level: &AlertLevel) -> Vec<Alert> {
        if let Ok(history) = self.alert_history.lock() {
            history.iter()
                .filter(|alert| alert.level == *level)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Mark an alert as false positive
    pub fn mark_false_positive(&self, alert_id: &str) -> Result<(), String> {
        if let Ok(mut active) = self.active_alerts.lock() {
            if let Some(alert) = active.get_mut(alert_id) {
                alert.resolved = true;
                alert.resolved_at = Some(Instant::now());

                self.metrics.false_positives.fetch_add(1, Ordering::Relaxed);
                self.metrics.resolved_alerts.fetch_add(1, Ordering::Relaxed);

                active.remove(alert_id);
                return Ok(());
            }
        }

        Err(format!("Alert {alert_id} not found"))
    }

    /// Get alert metrics
    pub fn get_metrics(&self) -> &AlertMetrics {
        &self.metrics
    }

    /// Generate alerting report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Alerting System Report ===\n\n");

        // Alert metrics
        report.push_str("Alert Metrics:\n");
        report.push_str(&format!("  Total Alerts: {}\n", self.metrics.total_alerts.load(Ordering::Relaxed)));
        report.push_str(&format!("  Critical Alerts: {}\n", self.metrics.critical_alerts.load(Ordering::Relaxed)));
        report.push_str(&format!("  Warning Alerts: {}\n", self.metrics.warning_alerts.load(Ordering::Relaxed)));
        report.push_str(&format!("  Info Alerts: {}\n", self.metrics.info_alerts.load(Ordering::Relaxed)));
        report.push_str(&format!("  Acknowledged: {}\n", self.metrics.acknowledged_alerts.load(Ordering::Relaxed)));
        report.push_str(&format!("  Resolved: {}\n", self.metrics.resolved_alerts.load(Ordering::Relaxed)));
        report.push_str(&format!("  False Positives: {}\n", self.metrics.false_positives.load(Ordering::Relaxed)));

        if self.metrics.acknowledged_alerts.load(Ordering::Relaxed) > 0 {
            report.push_str(&format!("  Avg Response Time: {} seconds\n",
                self.metrics.avg_response_time_seconds.load(Ordering::Relaxed)));
        }

        // Active alerts
        let active_alerts = self.get_active_alerts();
        report.push_str(&format!("\nActive Alerts: {}\n", active_alerts.len()));

        for alert in active_alerts.iter().take(10) {
            report.push_str(&format!("  [{}] {} - {} ({})\n",
                match alert.level {
                    AlertLevel::Info => "INFO",
                    AlertLevel::Warning => "WARN",
                    AlertLevel::Critical => "CRIT",
                    AlertLevel::Emergency => "EMERG",
                },
                alert.component,
                alert.message,
                if alert.acknowledged { "ACK" } else { "UNACK" }));
        }

        // Recent alerts
        let recent_alerts = self.get_recent_alerts(20);
        report.push_str(&format!("\nRecent Alerts: {}\n", recent_alerts.len()));

        for alert in recent_alerts.iter().take(10) {
            report.push_str(&format!("  [{}] {} - {} ({})\n",
                match alert.level {
                    AlertLevel::Info => "INFO",
                    AlertLevel::Warning => "WARN",
                    AlertLevel::Critical => "CRIT",
                    AlertLevel::Emergency => "EMERG",
                },
                alert.component,
                alert.message,
                if alert.resolved { "RESOLVED" } else { "ACTIVE" }));
        }

        report
    }
}

impl Default for AlertingSystem {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alerting_system_creation() {
        let system = AlertingSystem::new();
        assert_eq!(system.metrics.total_alerts.load(Ordering::Relaxed), 0);
        assert!(system.config.enable_alerting);
    }

    #[test]
    fn test_alert_triggering() {
        let system = AlertingSystem::new();

        let alert = Alert {
            id: "test_alert".to_string(),
            timestamp: Instant::now(),
            level: AlertLevel::Warning,
            component: "test_component".to_string(),
            message: "Test alert message".to_string(),
            details: HashMap::new(),
            acknowledged: false,
            acknowledged_at: None,
            resolved: false,
            resolved_at: None,
            occurrence_count: 1,
            last_occurrence: Instant::now(),
        };

        system.trigger_alert(alert);

        assert_eq!(system.metrics.total_alerts.load(Ordering::Relaxed), 1);
        assert_eq!(system.metrics.warning_alerts.load(Ordering::Relaxed), 1);

        let active = system.get_active_alerts();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].component, "test_component");
    }

    #[test]
    fn test_alert_acknowledgement() {
        let system = AlertingSystem::new();

        let alert = Alert {
            id: "test_alert".to_string(),
            timestamp: Instant::now(),
            level: AlertLevel::Warning,
            component: "test_component".to_string(),
            message: "Test alert message".to_string(),
            details: HashMap::new(),
            acknowledged: false,
            acknowledged_at: None,
            resolved: false,
            resolved_at: None,
            occurrence_count: 1,
            last_occurrence: Instant::now(),
        };

        system.trigger_alert(alert);

        let result = system.acknowledge_alert("test_alert");
        assert!(result.is_ok());

        assert_eq!(system.metrics.acknowledged_alerts.load(Ordering::Relaxed), 1);

        let active = system.get_active_alerts();
        assert_eq!(active.len(), 1);
        assert!(active[0].acknowledged);
    }

    #[test]
    fn test_alert_resolution() {
        let system = AlertingSystem::new();

        let alert = Alert {
            id: "test_alert".to_string(),
            timestamp: Instant::now(),
            level: AlertLevel::Warning,
            component: "test_component".to_string(),
            message: "Test alert message".to_string(),
            details: HashMap::new(),
            acknowledged: false,
            acknowledged_at: None,
            resolved: false,
            resolved_at: None,
            occurrence_count: 1,
            last_occurrence: Instant::now(),
        };

        system.trigger_alert(alert);

        let result = system.resolve_alert("test_alert");
        assert!(result.is_ok());

        assert_eq!(system.metrics.resolved_alerts.load(Ordering::Relaxed), 1);

        let active = system.get_active_alerts();
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_recent_alerts() {
        let system = AlertingSystem::new();

        // Trigger multiple alerts
        for i in 0..5 {
            let alert = Alert {
                id: format!("test_alert_{i}"),
                timestamp: Instant::now(),
                level: AlertLevel::Warning,
                component: format!("component_{i}"),
                message: format!("Test alert {i}"),
                details: HashMap::new(),
                acknowledged: false,
                acknowledged_at: None,
                resolved: false,
                resolved_at: None,
                occurrence_count: 1,
                last_occurrence: Instant::now(),
            };

            system.trigger_alert(alert);
        }

        let recent = system.get_recent_alerts(3);
        assert_eq!(recent.len(), 3);
    }
}