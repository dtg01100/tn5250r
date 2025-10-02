//! Application state management for TN5250R
//!
//! This module contains the main application state structure and basic initialization.

use std::collections::HashMap;

use eframe::egui;
use crate::controller::AsyncTerminalController;
use crate::field_manager::FieldDisplayInfo;
use crate::config;
use crate::lib3270::display::ScreenSize;
use crate::network::ProtocolMode;

/// Main application structure
pub struct TN5250RApp {
    pub controller: AsyncTerminalController,
    pub connection_string: String,
    pub connected: bool,
    pub host: String,
    pub port: u16,
    pub username: String,  // AS/400 username for authentication
    pub password: String,  // AS/400 password for authentication
    pub config: config::SharedSessionConfig,
    pub input_buffer: String,
    pub function_keys_visible: bool,
    pub terminal_content: String,
    pub login_screen_requested: bool,
    pub connection_time: Option<std::time::Instant>,
    pub fields_info: Vec<FieldDisplayInfo>,
    pub show_field_info: bool,
    pub tab_pressed_this_frame: bool,  // Track if Tab was pressed to prevent egui handling
    pub connecting: bool,
    pub show_monitoring_dashboard: bool,  // Show monitoring dashboard
    pub monitoring_reports: HashMap<String, String>,  // Cached monitoring reports
    pub show_advanced_settings: bool,  // Show advanced settings dialog
    pub show_settings_dialog: bool,   // Show settings dialog
    pub selected_screen_size: ScreenSize,  // Selected screen size
    pub selected_protocol_mode: ProtocolMode,  // Selected protocol mode
    pub debug_mode: bool,  // Enable debug output and data dumps
    pub show_debug_panel: bool,  // Show debug information panel
    pub raw_buffer_dump: String,  // Raw hex dump of last received data
    pub last_data_size: usize,  // Size of last data packet
}

impl TN5250RApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_with_server(_cc, "example.system.com".to_string(), 23, false, None, None, None, false)
    }

    pub fn new_with_server(
        _cc: &eframe::CreationContext<'_>,
        server: String,
        port: u16,
        auto_connect: bool,
        cli_ssl_override: Option<bool>,
        cli_username: Option<String>,
        cli_password: Option<String>,
        debug_mode: bool,
    ) -> Self {
        // Load persistent configuration
        let shared_config = config::load_shared_config("default".to_string());

        // Read all required config values once to avoid multiple borrows
        let (cfg_host, cfg_port, cfg_ssl, screen_size_config, protocol_mode_config) = {
            if let Ok(cfg) = shared_config.try_lock() {
                let host_val = cfg.get_string_property("connection.host");
                let port_val = cfg.get_int_property("connection.port").map(|v| v as u16);
                let ssl_val = cfg.get_boolean_property("connection.ssl").unwrap_or(port == 992);
                let screen_size_val = cfg.get_string_property("terminal.screenSize");
                let protocol_mode_val = cfg.get_string_property("terminal.protocolMode");
                (host_val, port_val, ssl_val, screen_size_val, protocol_mode_val)
            } else {
                (None, None, port == 992, None, None)
            }
        };

        // Seed host/port/TLS from config if available, otherwise from CLI/defaults
        let mut host = cfg_host.unwrap_or_else(|| server.clone());
        let port = cfg_port.unwrap_or(port);
        let config_ssl = cfg_ssl;

        // Update config with current values if config is available
        if let Ok(mut cfg) = shared_config.try_lock() {
            cfg.set_property("connection.host", host.as_str());
            cfg.set_property("connection.port", port as i64);
            if cli_ssl_override.is_none() {
                // Persist only when no CLI override to avoid surprising save of ephemeral override
                cfg.set_property("connection.ssl", config_ssl);
            }
        }

        // Save initial state so a newly created config file gets written
        if cli_ssl_override.is_none() {
            let _ = config::save_shared_config(&shared_config);
        }

        let connection_string = format!("{}:{}", host, port);
        let mut controller = AsyncTerminalController::new();

        // Configure credentials from CLI if provided
        let username = cli_username.unwrap_or_default();
        let password = cli_password.unwrap_or_default();

        // If auto-connect is requested, initiate connection
        let connected = if auto_connect {
            // Set credentials before connecting
            if !username.is_empty() && !password.is_empty() {
                controller.set_credentials(&username, &password);
                println!("CLI: Configured credentials for user: {}", username);
            }

            // Use pre-read config values to avoid borrow issues
            let config_ssl = {
                if let Ok(cfg) = shared_config.try_lock() {
                    cfg.get_boolean_property("connection.ssl").unwrap_or(port == 992)
                } else {
                    port == 992  // Default based on port
                }
            };
            let use_tls = cli_ssl_override.unwrap_or(config_ssl);
            match controller.connect_with_tls(host.clone(), port, Some(use_tls)) {
                Ok(()) => {
                    true
                },
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                    false
                }
            }
        } else {
            false
        };

        let terminal_content = if auto_connect && connected {
            format!("Connected to {}:{}\nReady...\n", host, port)
        } else if auto_connect {
            format!("Failed to connect to {}:{}\nReady...\n", host, port)
        } else {
            "TN5250R - IBM AS/400 Terminal Emulator\nReady...\n".to_string()
        };

        Self {
            connection_string,
            controller,
            connected,
            host,
            port,
            config: shared_config.clone(),
            input_buffer: String::new(),
            function_keys_visible: true,
            terminal_content,
            login_screen_requested: false,
            connection_time: None,
            fields_info: Vec::new(),
            show_field_info: true,
            tab_pressed_this_frame: false,
            connecting: false,
            show_monitoring_dashboard: false,
            monitoring_reports: HashMap::new(),
            show_advanced_settings: false,
            show_settings_dialog: false,
            username,  // Use CLI credentials or empty
            password,  // Use CLI credentials or empty
            debug_mode,  // From CLI flag
            show_debug_panel: debug_mode,  // Auto-show if debug enabled
            raw_buffer_dump: String::new(),
            last_data_size: 0,
            selected_screen_size: {
                match screen_size_config.as_deref() {
                    Some("Model2") => crate::lib3270::display::ScreenSize::Model2,
                    Some("Model3") => crate::lib3270::display::ScreenSize::Model3,
                    Some("Model4") => crate::lib3270::display::ScreenSize::Model4,
                    Some("Model5") => crate::lib3270::display::ScreenSize::Model5,
                    _ => crate::lib3270::display::ScreenSize::Model2,  // Default to 24x80
                }
            },
            selected_protocol_mode: {
                match protocol_mode_config.as_deref() {
                    Some("TN5250") => ProtocolMode::TN5250,
                    Some("TN3270") => ProtocolMode::TN3270,
                    Some("AutoDetect") => ProtocolMode::AutoDetect,
                    _ => ProtocolMode::TN5250,  // Default to TN5250
                }
            }
        }
    }

    pub fn update_terminal_content(&mut self) -> bool {
        let mut content_changed = false;

        // Update terminal content from controller
        if let Ok(content) = self.controller.get_terminal_content() {
            // Only update and log if content has actually changed
            if content != self.terminal_content {
                println!("DEBUG: Terminal content changed ({} -> {} chars)",
                    self.terminal_content.len(),
                    content.len()
                );
                self.terminal_content = content;
                content_changed = true;
            }
        }

        // Update field information (always update if available)
        if let Ok(fields) = self.controller.get_fields_info() {
            self.fields_info = fields;
        }

        // Update connection status
        let was_connected = self.connected;
        self.connected = self.controller.is_connected();
        if self.connected != was_connected {
            content_changed = true;
        }

        if self.connecting && self.connected && !was_connected {
            self.connecting = false;
            self.terminal_content = format!("Connected to {}:{}\nNegotiating...\n", self.host, self.port);
            content_changed = true;

            // Record successful connection in monitoring
            let monitoring = crate::monitoring::MonitoringSystem::global();
            monitoring.integration_monitor.record_integration_event(crate::monitoring::IntegrationEvent {
                timestamp: std::time::Instant::now(),
                event_type: crate::monitoring::IntegrationEventType::IntegrationSuccess,
                source_component: "network".to_string(),
                target_component: Some("controller".to_string()),
                description: format!("Successfully established connection to {}:{}", self.host, self.port),
                details: std::collections::HashMap::new(),
                duration_us: self.connection_time.map(|t| t.elapsed().as_micros() as u64),
                success: true,
            });
        }
        if self.connecting {
            // Check for async connect error and surface it
            if let Some(err) = self.controller.take_last_connect_error() {
                self.connecting = false;
                let message = if err.to_lowercase().contains("timed out") {
                    format!("Connection timed out to {}:{}\n", self.host, self.port)
                } else if err.to_lowercase().contains("canceled") {
                    "Connection canceled by user\n".to_string()
                } else {
                    format!("Connection failed: {}\n", err)
                };
                self.terminal_content = message;
                self.connection_time = None;
                self.login_screen_requested = false;
                content_changed = true;

                // Record connection error in monitoring
                let monitoring = crate::monitoring::MonitoringSystem::global();
                let alert_level = if err.to_lowercase().contains("timed out") {
                    crate::monitoring::AlertLevel::Warning
                } else {
                    crate::monitoring::AlertLevel::Critical
                };

                let alert = crate::monitoring::Alert {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: std::time::Instant::now(),
                    level: alert_level,
                    component: "network".to_string(),
                    message: format!("Connection error to {}:{}: {}", self.host, self.port, err),
                    details: std::collections::HashMap::new(),
                    acknowledged: false,
                    acknowledged_at: None,
                    resolved: false,
                    resolved_at: None,
                    occurrence_count: 1,
                    last_occurrence: std::time::Instant::now(),
                };
                monitoring.alerting_system.trigger_alert(alert);

                // Record integration failure
                monitoring.integration_monitor.record_integration_event(crate::monitoring::IntegrationEvent {
                    timestamp: std::time::Instant::now(),
                    event_type: crate::monitoring::IntegrationEventType::IntegrationFailure,
                    source_component: "network".to_string(),
                    target_component: Some("controller".to_string()),
                    description: format!("Connection failed: {}", err),
                    details: std::collections::HashMap::new(),
                    duration_us: self.connection_time.map(|t| t.elapsed().as_micros() as u64),
                    success: false,
                });
            }
        }

        // Request login screen if connected and enough time has passed
        if self.connected && !self.login_screen_requested {
            if let Some(connection_time) = self.connection_time {
                if connection_time.elapsed() >= std::time::Duration::from_secs(2) {
                    if let Err(e) = self.controller.request_login_screen() {
                        eprintln!("Failed to request login screen: {}", e);
                    }
                    self.login_screen_requested = true;
                }
            }
        }

        content_changed
    }
}