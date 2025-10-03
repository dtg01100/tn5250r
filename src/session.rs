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
    pub fn get_cursor_position(&self) -> Option<(usize, usize)> {
        // This would query the controller for cursor position
        // Implementation depends on controller interface
        None
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

    /// Connect the session
    pub fn connect(&mut self) {
        self.connecting = true;
        self.connection_time = Some(std::time::Instant::now());
        // Implementation would call controller.connect()
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