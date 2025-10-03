//! Session profile management for TN5250R
//!
//! This module defines connection profiles that store reusable connection settings
//! for AS/400 and mainframe systems.

use serde::{Deserialize, Serialize};
use crate::lib3270::display::ScreenSize;
use crate::network::ProtocolMode;

/// A connection profile containing all settings needed to connect to a terminal session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProfile {
    /// Unique identifier for the profile
    pub id: String,
    /// Display name for the profile
    pub name: String,
    /// Optional description
    pub description: String,
    /// Hostname or IP address
    pub host: String,
    /// Port number (typically 23 for telnet, 992 for SSL)
    pub port: u16,
    /// Protocol type (TN5250 or TN3270)
    pub protocol: ProtocolMode,
    /// Optional username for authentication
    pub username: Option<String>,
    /// Optional password for authentication (consider secure storage in production)
    pub password: Option<String>,
    /// Terminal screen size
    pub screen_size: ScreenSize,
    /// Whether to auto-connect when profile is loaded
    pub auto_connect: bool,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modification timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl SessionProfile {
    /// Create a new session profile with default values
    pub fn new(name: String, host: String, port: u16) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: String::new(),
            host,
            port,
            protocol: ProtocolMode::TN5250,
            username: None,
            password: None,
            screen_size: ScreenSize::Model2,
            auto_connect: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a profile with a specific ID (for loading from storage)
    pub fn with_id(id: String, name: String, host: String, port: u16) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            name,
            description: String::new(),
            host,
            port,
            protocol: ProtocolMode::TN5250,
            username: None,
            password: None,
            screen_size: ScreenSize::Model2,
            auto_connect: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the profile's modification timestamp
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now();
    }

    /// Get a sanitized filename for this profile
    pub fn filename(&self) -> String {
        self.name.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
            .collect::<String>()
            .to_lowercase()
    }
}

impl Default for SessionProfile {
    fn default() -> Self {
        Self::new("Default Profile".to_string(), "localhost".to_string(), 23)
    }
}