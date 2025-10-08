//! Active session management for TN5250R
//!
//! This module defines the Session struct that represents an active terminal connection
//! with its associated controller and UI state.

use crate::controller::AsyncTerminalController;
use crate::field_manager::FieldDisplayInfo;
use crate::session_profile::SessionProfile;

/// Represents an active terminal session
#[derive(Debug)]
pub struct Session {
    /// Unique session identifier
    pub id: String,
    /// The profile this session was created from
    pub profile: SessionProfile,
    /// The terminal controller handling the connection
    pub controller: AsyncTerminalController,
    /// Current terminal content for display
    pub terminal_content: String,
    /// Field information for UI highlighting
    pub fields_info: Vec<FieldDisplayInfo>,
    /// Connection status
    pub connected: bool,
    /// Connection in progress
    pub connecting: bool,
    /// Connection start time
    pub connection_time: Option<std::time::Instant>,
    /// Last error message
    pub error_message: Option<String>,
    /// Show monitoring dashboard for this session
    pub show_monitoring_dashboard: bool,
    /// Input buffer for session-specific commands
    pub input_buffer: String,
}

impl Session {
    /// Create a new session from a profile
    pub fn new(profile: SessionProfile) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            controller: AsyncTerminalController::new(),
            terminal_content: String::new(),
            fields_info: Vec::new(),
            connected: false,
            connecting: false,
            connection_time: None,
            error_message: None,
            profile,
            show_monitoring_dashboard: false,
            input_buffer: String::new(),
        }
    }

    /// Get the display name for this session
    pub fn display_name(&self) -> String {
        format!("{} ({})",
            self.profile.name,
            if self.connected { "Connected" } else { "Disconnected" }
        )
    }

    /// Check if the session needs UI updates
    pub fn needs_update(&self) -> bool {
        // This would check if terminal content or fields have changed
        // For now, return true if connected
        self.connected
    }

    /// Get the current cursor position from the controller
    pub fn get_cursor_position(&self) -> (usize, usize) {
        self.controller.get_cursor_position().unwrap_or((1, 1))
    }

    /// Send function key to the session
    pub fn send_function_key(&mut self, _key: crate::keyboard::FunctionKey) {
        // Implementation depends on controller interface
        // This would send the function key to the active session
    }

    /// Send text input to the session
    pub fn send_input(&mut self, _input: &str) {
        // Implementation depends on controller interface
        // This would send text input to the active session
    }

    /// Connect the session using profile credentials
    pub fn connect(&mut self) {
        // Clear any previous error message
        self.error_message = None;

        // Configure credentials from profile if available (RFC 4777 authentication)
        if let (Some(username), Some(password)) = (&self.profile.username, &self.profile.password) {
            self.controller.set_credentials(username, password);
            println!("Session {}: Configured credentials for user: {}", self.id, username);
        } else {
            // Clear credentials if not set in profile
            self.controller.clear_credentials();
        }

        // Set connecting state
        self.connecting = true;
        self.connection_time = Some(std::time::Instant::now());
        self.terminal_content = format!("Connecting to {}:{}...\n", self.profile.host, self.profile.port);

        // Use non-blocking connect with TLS options
        // For now, use default TLS settings (port 992 uses SSL, others don't)
        let use_tls = self.profile.port == 992;

        if let Err(e) = self.controller.connect_async_with_tls_options(
            self.profile.host.clone(),
            self.profile.port,
            Some(use_tls),
            Some(false), // insecure = false
            None, // ca_bundle_path = None
        ) {
            self.terminal_content = format!("Connection failed to start: {}\n", e);
            self.connecting = false;
            self.error_message = Some(format!("Connection failed: {}", e));
        }
    }

    /// Disconnect the session
    pub fn disconnect(&mut self) {
        self.controller.cancel_connect();
        self.connected = false;
        self.connecting = false;
        self.connection_time = None;
    }

    /// Update session state from controller
    pub fn update_from_controller(&mut self) {
        // This would poll the controller for updates
        // Update connected status, terminal content, fields, etc.
    }
}