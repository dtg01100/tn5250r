//! Application state management for TN5250R
//!
//! This module contains the main application state structure and basic initialization.

use std::collections::HashMap;


use crate::controller::AsyncTerminalController;
use crate::field_manager::FieldDisplayInfo;
use crate::config;
use crate::lib3270::display::ScreenSize;
use crate::network::ProtocolMode;
use crate::session::Session;
use crate::session_profile::SessionProfile;
use crate::profile_manager::ProfileManager;

/// Main application structure
pub struct TN5250RApp {
    // Multi-session management
    pub sessions: HashMap<String, Session>,
    pub active_session_id: Option<String>,
    pub profile_manager: ProfileManager,

    // UI state
    pub show_profile_manager: bool,
    pub show_create_profile_dialog: bool,
    pub editing_profile: Option<SessionProfile>,

    // Legacy single-session fields (for backward compatibility during transition)
    pub controller: AsyncTerminalController,
    pub connection_string: String,
    pub connected: bool,
    pub host: String,
    pub port: u16,
    pub username: String,  // AS/400 username for authentication
    pub password: String,  // AS/400 password for authentication

    // Shared configuration and UI state
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
    pub error_message: Option<String>,  // Current error message for UI feedback
}

impl TN5250RApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_with_server(_cc, "example.system.com".to_string(), 23, false, None, None, None, None, false)
    }

    /// Create app with optional profile (for CLI profile loading)
    pub fn new_with_profile(
        _cc: &eframe::CreationContext<'_>,
        profile: Option<SessionProfile>,
        cli_protocol: Option<String>,
        debug_mode: bool,
    ) -> Self {
        let profile_manager = ProfileManager::new()
            .expect("Failed to initialize profile manager");

        let mut app = Self {
            // Multi-session fields
            sessions: HashMap::new(),
            active_session_id: None,
            profile_manager,
            show_profile_manager: true,
            show_create_profile_dialog: false,
            editing_profile: None,

            // Initialize legacy fields with defaults
            controller: AsyncTerminalController::new(),
            connection_string: String::new(),
            connected: false,
            host: String::new(),
            port: 23,
            username: String::new(),
            password: String::new(),

            // Load shared config
            config: config::load_shared_config("default".to_string()),
            input_buffer: String::new(),
            function_keys_visible: true,
            terminal_content: "TN5250R - IBM AS/400 Terminal Emulator\nReady...\n".to_string(),
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
            selected_screen_size: ScreenSize::Model2,
            selected_protocol_mode: {
                // CLI protocol override takes precedence
                if let Some(ref protocol) = cli_protocol {
                    match protocol.to_lowercase().as_str() {
                        "tn5250" | "5250" => ProtocolMode::TN5250,
                        "tn3270" | "3270" => ProtocolMode::TN3270,
                        "auto" | "autodetect" => ProtocolMode::AutoDetect,
                        _ => {
                            eprintln!("Warning: Invalid protocol '{protocol}', using TN5250");
                            ProtocolMode::TN5250
                        }
                    }
                } else {
                    ProtocolMode::TN5250  // Default when no CLI override
                }
            },
            debug_mode,
            show_debug_panel: debug_mode,
            raw_buffer_dump: String::new(),
            last_data_size: 0,
            error_message: None,
        };

        // Create session from profile if provided
        if let Some(profile) = profile {
            app.create_session_from_profile(profile);
        }

        app
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_server(
        _cc: &eframe::CreationContext<'_>,
        server: String,
        port: u16,
        auto_connect: bool,
        cli_ssl_override: Option<bool>,
        cli_username: Option<String>,
        cli_password: Option<String>,
        cli_protocol: Option<String>,
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
        let host = cfg_host.unwrap_or_else(|| server.clone());
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

        let connection_string = format!("{host}:{port}");
        let mut controller = AsyncTerminalController::new();

        // Configure credentials from CLI if provided
        let username = cli_username.unwrap_or_default();
        let password = cli_password.unwrap_or_default();

        // If auto-connect is requested, initiate connection
        let connected = if auto_connect {
            // Set credentials before connecting
            if !username.is_empty() && !password.is_empty() {
                controller.set_credentials(&username, &password);
                println!("CLI: Configured credentials for user: {username}");
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
                    eprintln!("Connection failed: {e}");
                    false
                }
            }
        } else {
            false
        };

        let terminal_content = if auto_connect && connected {
            format!("Connected to {host}:{port}\nReady...\n")
        } else if auto_connect {
            format!("Failed to connect to {host}:{port}\nReady...\n")
        } else {
            "TN5250R - IBM AS/400 Terminal Emulator\nReady...\n".to_string()
        };

        Self {
            // Multi-session fields
            sessions: HashMap::new(),
            active_session_id: None,
            profile_manager: ProfileManager::new().expect("Failed to initialize profile manager"),
            show_profile_manager: true,
            show_create_profile_dialog: false,
            editing_profile: None,

            // Legacy single-session fields (for backward compatibility during transition)
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
            error_message: None,
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
                // CLI protocol override takes precedence
                if let Some(ref protocol) = cli_protocol {
                    match protocol.to_lowercase().as_str() {
                        "tn5250" | "5250" => ProtocolMode::TN5250,
                        "tn3270" | "3270" => ProtocolMode::TN3270,
                        "auto" | "autodetect" => ProtocolMode::AutoDetect,
                        _ => {
                            eprintln!("Warning: Invalid protocol '{protocol}', using TN5250");
                            ProtocolMode::TN5250
                        }
                    }
                } else {
                    // Use config value
                    match protocol_mode_config.as_deref() {
                        Some("TN5250") => ProtocolMode::TN5250,
                        Some("TN3270") => ProtocolMode::TN3270,
                        Some("AutoDetect") => ProtocolMode::AutoDetect,
                        _ => ProtocolMode::TN5250,  // Default to TN5250
                    }
                }
            }
        }
    }

    pub fn update_terminal_content(&mut self) -> bool {
        let mut content_changed = false;

        // Check if new data has arrived (event-driven)
        let data_arrived = self.controller.check_data_arrival().unwrap_or(false);

        // Update terminal content from controller if data arrived or on first check
        if data_arrived || self.terminal_content.is_empty() {
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
            self.error_message = None;  // Clear any previous error on successful connection
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
                    format!("Connection failed: {err}\n")
                };
                self.terminal_content = message.clone();
                self.error_message = Some(message);
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
                    description: format!("Connection failed: {err}"),
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
                        eprintln!("Failed to request login screen: {e}");
                    }
                    self.login_screen_requested = true;
                }
            }
        }

        content_changed
    }

    // Multi-session management methods

    /// Create a new session from a profile
    pub fn create_session_from_profile(&mut self, profile: SessionProfile) {
        let session = Session::new(profile);
        let session_id = session.id.clone();
        self.sessions.insert(session_id.clone(), session);
        self.active_session_id = Some(session_id);
    }

    /// Get the currently active session
    pub fn get_active_session(&self) -> Option<&Session> {
        self.active_session_id
            .as_ref()
            .and_then(|id| self.sessions.get(id))
    }

    /// Get the currently active session (mutable)
    pub fn get_active_session_mut(&mut self) -> Option<&mut Session> {
        self.active_session_id
            .as_ref()
            .and_then(|id| self.sessions.get_mut(id))
    }

    /// Switch to a different session
    pub fn switch_to_session(&mut self, session_id: String) {
        if self.sessions.contains_key(&session_id) {
            self.active_session_id = Some(session_id);
        }
    }

    /// Close a session
    pub fn close_session(&mut self, session_id: &str) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.disconnect();
        }
        self.sessions.remove(session_id);

        // Update active session if it was closed
        if self.active_session_id.as_ref() == Some(&session_id.to_string()) {
            self.active_session_id = self.sessions.keys().next().cloned();
        }
    }

    /// Connect a specific session using its profile credentials
    pub fn connect_session(&mut self, session_id: &str) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.connect();
        }
    }

    /// Get all session IDs for UI
    pub fn get_session_ids(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }

    /// Show content for a specific session
    pub fn show_session_content(&mut self, ui: &mut egui::Ui, session_id: &str) {
        // Get session info needed for UI before borrowing
        let session_info = self.sessions.get(session_id).map(|s| {
            (s.profile.name.clone(), s.profile.host.clone(), s.profile.port, s.connecting)
        });

        if let Some((name, host, port, connecting)) = session_info {
            ui.heading(format!("TN5250R - {name}"));
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Host:");
                ui.label(format!("{host}:{port}"));

                if ui.button("Connect").clicked() {
                    self.connect_session(session_id);
                }

                if connecting && ui.button("Cancel").clicked() {
                    if let Some(session) = self.sessions.get_mut(session_id) {
                        session.controller.cancel_connect();
                        session.connecting = false;
                        session.connection_time = None;
                        session.terminal_content.push_str("\nConnection canceled by user.\n");
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.debug_mode
                        && ui.button("🐛 Debug").on_hover_text("Show debug information panel").clicked() {
                            self.show_debug_panel = !self.show_debug_panel;
                        }
                    if ui.button("⚙ Advanced").on_hover_text("Advanced connection settings").clicked() {
                        self.show_advanced_settings = true;
                    }
                });
            });

            // Check if monitoring dashboard should be shown for this session
            let show_monitoring = self.sessions.get(session_id).map(|s| s.show_monitoring_dashboard).unwrap_or(false);

            // Now borrow session again for the rest of the UI
            if let Some(session) = self.sessions.get_mut(session_id) {
                // Keep session model in sync with controller state
                session.update_from_controller();

                // Username and Password fields for AS/400 authentication (RFC 4777)
                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut self.username);

                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                });

                ui.separator();

                // Display terminal content with cursor and click handling
                let scroll_area_response = egui::ScrollArea::vertical()
                    .id_salt(format!("terminal_display_{}", session.id))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        TN5250RApp::draw_terminal_with_cursor_for_session(ui, &*session);
                    });

                // Handle mouse clicks on the scroll area content
                let content_rect = scroll_area_response.inner_rect;
                let response = ui.interact(content_rect, egui::Id::new(format!("terminal_area_{}", session.id)), egui::Sense::click());

                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        // Calculate position relative to the content area
                        let relative_pos = pos - content_rect.min;

                        // Get font metrics for coordinate calculation
                        let font = egui::FontId::monospace(14.0);
                        let char_width = ui.fonts(|f| f.glyph_width(&font, ' '));
                        let line_height = ui.fonts(|f| f.row_height(&font));

                        let col = (relative_pos.x / char_width).floor() as usize + 1; // Convert to 1-based
                        let row = (relative_pos.y / line_height).floor() as usize + 1; // Convert to 1-based

                        // Clamp to valid terminal bounds
                        let row = row.clamp(1, 24);
                        let col = col.clamp(1, 80);

                        // Handle session-specific click: position cursor and handle field navigation
                        if let Err(e) = session.controller.click_at_position(row, col) {
                            eprintln!("Failed to click at position ({row}, {col}): {e}");
                        }
                    }
                }

                // Display field information if available
                if !session.fields_info.is_empty() {
                    ui.separator();
                    ui.collapsing("Field Information", |ui| {
                        for (i, field) in session.fields_info.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if field.is_active {
                                    ui.colored_label(egui::Color32::GREEN, "►");
                                } else {
                                    ui.label(" ");
                                }
                                ui.label(format!("Field {}: {}", i + 1, field.label));
                                ui.label(format!("Content: '{}'", field.content));

                                // Show error if present
                                if let Some(error) = &field.error_state {
                                    ui.colored_label(egui::Color32::RED, format!("Error: {}", error.get_user_message()));
                                }

                                // Show highlight status
                                if field.highlighted {
                                    ui.colored_label(egui::Color32::YELLOW, "Highlighted");
                                }
                            });
                        }
                        ui.label("Use Tab/Shift+Tab to navigate between fields");
                    });
                }

                if ui.button("Send").clicked() && !session.input_buffer.is_empty() {
                    // Process the input when Send button is clicked
                    session.terminal_content.push_str(&format!("\n> {}", session.input_buffer));

                    // Send to controller
                    if let Err(e) = session.controller.send_input(session.input_buffer.as_bytes()) {
                        session.terminal_content.push_str(&format!("\nError: {e}"));
                    }

                    session.input_buffer.clear();
                }
                ui.separator();
                ui.collapsing("Cursor Information", |ui| {
                    let cursor_pos = session.get_cursor_position();
                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        ui.label(format!("({}, {})", cursor_pos.0, cursor_pos.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Blinking:");
                        // TODO: Implement cursor blinking detection when display state is available
                        ui.label("Disabled");
                    });
                });

                ui.separator();

                // Input area for commands
                ui.horizontal(|ui| {
                    ui.label("Input:");
                    if ui.text_edit_singleline(&mut session.input_buffer).lost_focus() &&
                        ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        // Process the input when Enter is pressed
                        if !session.input_buffer.is_empty() {
                            // Echo the input to session terminal
                            session.terminal_content.push_str(&format!("\n> {}", session.input_buffer));

                            // Send to controller
                            if let Err(e) = session.controller.send_input(session.input_buffer.as_bytes()) {
                                session.terminal_content.push_str(&format!("\nError: {e}"));
                            }

                            session.input_buffer.clear();
                        }
                    }

                    if ui.button("Send").clicked() && !session.input_buffer.is_empty() {
                        // Process the input when Send button is clicked
                        session.terminal_content.push_str(&format!("\n> {}", session.input_buffer));

                        // Send to controller
                        if let Err(e) = session.controller.send_input(session.input_buffer.as_bytes()) {
                            session.terminal_content.push_str(&format!("\nError: {e}"));
                        }

                        session.input_buffer.clear();
                    }
                });

                // Display function keys if enabled
                if self.function_keys_visible {
                    crate::ui::function_keys::render_function_keys_for_session(ui, session);
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        if session.connecting {
                            ui.colored_label(egui::Color32::YELLOW, format!("Connecting to {}:{} ... ", session.profile.host, session.profile.port));
                        } else if session.connected {
                            ui.colored_label(egui::Color32::GREEN, format!("Connected to {}:{} ", session.profile.host, session.profile.port));
                        } else {
                            ui.colored_label(egui::Color32::RED, "Disconnected");
                        }
                        ui.separator();
                        ui.label("Ready");
                    });
                });
            }

            // Display monitoring dashboard if enabled for session (after session borrow ends)
            if show_monitoring {
                ui.separator();
                self.show_monitoring_dashboard_ui(ui);
            }
        } else {
            // Session not found, show error
            ui.label("Session not found");
        }
    }

    /// Show legacy single-session content (for backward compatibility)
    pub fn show_legacy_session_content(&mut self, ui: &mut egui::Ui) {
        ui.heading("TN5250R - IBM AS/400 Terminal Emulator");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Host:");
            if ui.text_edit_singleline(&mut self.connection_string).changed() {
                // Update host and port when connection string changes
                let (host, port) = self.parse_connection_string();
                self.host = host;
                self.port = port;
                // Sync to configuration; do NOT auto-toggle TLS, keep user's persisted choice
                if let Ok(mut cfg) = self.config.try_lock() {
                    cfg.set_property("connection.host", self.host.as_str());
                    cfg.set_property("connection.port", self.port as i64);
                }
                // Persist change (async to avoid blocking GUI)
                config::save_shared_config_async(&self.config);
            }

            if ui.button("Connect").clicked() {
                self.do_connect();
            }

            if self.connecting
                && ui.button("Cancel").clicked() {
                    self.controller.cancel_connect();
                    self.connecting = false;
                    self.connection_time = None;
                    self.terminal_content.push_str("\nConnection canceled by user.\n");
                }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.debug_mode
                    && ui.button("🐛 Debug").on_hover_text("Show debug information panel").clicked() {
                        self.show_debug_panel = !self.show_debug_panel;
                    }
                if ui.button("⚙ Advanced").on_hover_text("Advanced connection settings").clicked() {
                    self.show_advanced_settings = true;
                }
            });
        });

        // Username and Password fields for AS/400 authentication (RFC 4777)
        ui.horizontal(|ui| {
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username);

            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
        });

        ui.separator();

        // Display terminal content with cursor and click handling
        let scroll_area_response = egui::ScrollArea::vertical()
            .id_salt("terminal_display")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.draw_terminal_with_cursor(ui);
            });

        // Handle mouse clicks on the scroll area content
        let content_rect = scroll_area_response.inner_rect;
        let response = ui.interact(content_rect, egui::Id::new("terminal_area"), egui::Sense::click());

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                // Calculate position relative to the content area
                let relative_pos = pos - content_rect.min;

                // Get font metrics for coordinate calculation
                let font = egui::FontId::monospace(14.0);
                let char_width = ui.fonts(|f| f.glyph_width(&font, ' '));
                let line_height = ui.fonts(|f| f.row_height(&font));

                let col = (relative_pos.x / char_width).floor() as usize + 1; // Convert to 1-based
                let row = (relative_pos.y / line_height).floor() as usize + 1; // Convert to 1-based

                // Clamp to valid terminal bounds
                let row = row.clamp(1, 24);
                let col = col.clamp(1, 80);

                if let Err(e) = self.controller.click_at_position(row, col) {
                    eprintln!("Failed to click at position ({row}, {col}): {e}");
                }
            }
        }

        // Display field information if available
        if !self.fields_info.is_empty() {
            ui.separator();
            ui.collapsing("Field Information", |ui| {
                for (i, field) in self.fields_info.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if field.is_active {
                            ui.colored_label(egui::Color32::GREEN, "►");
                        } else {
                            ui.label(" ");
                        }
                        ui.label(format!("Field {}: {}", i + 1, field.label));
                        ui.label(format!("Content: '{}'", field.content));

                        // Show error if present
                        if let Some(error) = &field.error_state {
                            ui.colored_label(egui::Color32::RED, format!("Error: {}", error.get_user_message()));
                        }

                        // Show highlight status
                        if field.highlighted {
                            ui.colored_label(egui::Color32::YELLOW, "Highlighted");
                        }
                    });
                }
                ui.label("Use Tab/Shift+Tab to navigate between fields");
            });
        }

        // Display monitoring dashboard if enabled
        if self.show_monitoring_dashboard {
            ui.separator();
            self.show_monitoring_dashboard_ui(ui);
        }

        ui.separator();

        // Input area for commands
        ui.horizontal(|ui| {
            ui.label("Input:");
            if ui.text_edit_singleline(&mut self.input_buffer).lost_focus() &&
                ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                // Process the input when Enter is pressed
                if !self.input_buffer.is_empty() {
                    // Echo the input to terminal
                    self.terminal_content.push_str(&format!("\n> {}", self.input_buffer));

                    // Send to controller
                    if let Err(e) = self.controller.send_input(self.input_buffer.as_bytes()) {
                        self.terminal_content.push_str(&format!("\nError: {e}"));
                    }

                    self.input_buffer.clear();
                }
            }

            if ui.button("Send").clicked() && !self.input_buffer.is_empty() {
                // Process the input when Send button is clicked
                self.terminal_content.push_str(&format!("\n> {}", self.input_buffer));

                // Send to controller
                if let Err(e) = self.controller.send_input(self.input_buffer.as_bytes()) {
                    self.terminal_content.push_str(&format!("\nError: {e}"));
                }

                self.input_buffer.clear();
            }
        });

        // Display function keys if enabled
        if self.function_keys_visible {
            self.render_function_keys(ui);
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.horizontal(|ui| {
                if self.connecting {
                    ui.colored_label(egui::Color32::YELLOW, format!("Connecting to {}:{} ... ", self.host, self.port));
                } else if self.connected {
                    ui.colored_label(egui::Color32::GREEN, format!("Connected to {}:{} ", self.host, self.port));
                } else {
                    ui.colored_label(egui::Color32::RED, "Disconnected");
                }
                ui.separator();

                // Show input buffer status for feedback
                if let Ok(pending_size) = self.controller.get_pending_input_size() {
                    if pending_size > 0 {
                        ui.colored_label(egui::Color32::BLUE, format!("Input buffered ({pending_size} bytes)"));
                        ui.separator();
                    }
                }

                ui.label("Ready");
            });
        });
    }

    /// Create a TN3270 protocol processor configured with current screen size settings
    pub fn create_tn3270_processor(&self) -> crate::lib3270::protocol::ProtocolProcessor3270 {
        crate::lib3270::protocol::ProtocolProcessor3270::with_screen_size(self.selected_screen_size)
    }

    /// Get screen dimensions based on current selection
    pub fn get_screen_dimensions(&self) -> (usize, usize) {
        (self.selected_screen_size.cols(), self.selected_screen_size.rows())
    }
}