//! Connection management for TN5250R
//!
//! This module handles connection establishment, disconnection, and connection string parsing.

use crate::app_state::TN5250RApp;
use crate::monitoring;
use uuid;

impl TN5250RApp {
    pub fn parse_connection_string(&self) -> (String, u16) {
        if let Some((host, port_str)) = self.connection_string.rsplit_once(':') {
            let host = host.to_string();
            if let Ok(port) = port_str.parse::<u16>() {
                (host, port)
            } else {
                (host, 23) // Default telnet port
            }
        } else {
            (self.connection_string.clone(), 23) // Default telnet port
        }
    }

    pub fn do_connect(&mut self) {
        // Clear any previous error message
        self.error_message = None;

        // Parse host and port from connection string
        let (host, port) = self.parse_connection_string();
        self.host = host;
        self.port = port;

        // Configure credentials before connecting (RFC 4777 authentication)
        if !self.username.is_empty() && !self.password.is_empty() {
            self.controller.set_credentials(&self.username, &self.password);
            println!("GUI: Configured credentials for user: {}", self.username);
        } else {
            // Clear credentials if fields are empty
            self.controller.clear_credentials();
        }

        // Use non-blocking connect to avoid UI hang
        self.connecting = true;
        self.terminal_content = format!("Connecting to {}:{}...\n", self.host, self.port);
        // Read TLS settings from config (non-blocking)
        let (use_tls, insecure, ca_opt) = {
            if let Ok(cfg) = self.config.try_lock() {
                let use_tls = cfg.get_boolean_property_or("connection.ssl", self.port == 992);
                let insecure = cfg.get_boolean_property_or("connection.tls.insecure", false);
                let ca = cfg.get_string_property_or("connection.tls.caBundlePath", "");
                let ca_opt = if ca.trim().is_empty() { None } else { Some(ca) };
                (use_tls, insecure, ca_opt)
            } else {
                // Config locked, use safe defaults
                (self.port == 992, false, None)
            }
        };

        if let Err(e) = self.controller.connect_async_with_tls_options(self.host.clone(), self.port, Some(use_tls), Some(insecure), ca_opt) {
            self.terminal_content = format!("Connection failed to start: {e}\n");
            self.connecting = false;

            // Record connection failure in monitoring
            let monitoring = monitoring::MonitoringSystem::global();
            let alert = monitoring::Alert {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: std::time::Instant::now(),
                level: monitoring::AlertLevel::Warning,
                component: "network".to_string(),
                message: format!("Connection failed to {}:{}", self.host, self.port),
                details: std::collections::HashMap::new(),
                acknowledged: false,
                acknowledged_at: None,
                resolved: false,
                resolved_at: None,
                occurrence_count: 1,
                last_occurrence: std::time::Instant::now(),
            };
            monitoring.alerting_system.trigger_alert(alert);
        } else {
            self.connection_time = Some(std::time::Instant::now());
            self.login_screen_requested = false;
        }
    }

    pub fn do_disconnect(&mut self) {
        self.controller.disconnect();
        self.connected = false;
        self.login_screen_requested = false;
        self.connection_time = None;
        self.terminal_content = "Disconnected from AS/400 system\nReady for new connection...\n".to_string();
        self.error_message = None;  // Clear any error on disconnect

        // Record disconnection in monitoring
        let monitoring = monitoring::MonitoringSystem::global();
        monitoring.integration_monitor.record_integration_event(monitoring::IntegrationEvent {
            timestamp: std::time::Instant::now(),
            event_type: monitoring::IntegrationEventType::ComponentInteraction,
            source_component: "main".to_string(),
            target_component: Some("network".to_string()),
            description: "User initiated disconnection".to_string(),
            details: std::collections::HashMap::new(),
            duration_us: None,
            success: true,
        });
    }
}