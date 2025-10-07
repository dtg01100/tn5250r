//! Configuration Management System for TN5250R
//!
//! This module provides a comprehensive configuration system inspired by tn5250j's
//! SessionConfig architecture, with property-based configuration, change listeners,
//! and serialization support.

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::error::{ProtocolError, ConfigError, TN5250Error};

/// Configuration change event
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    pub property_name: String,
    pub old_value: Option<ConfigValue>,
    pub new_value: ConfigValue,
}

/// Configuration change listener trait
pub trait ConfigChangeListener: Send + Sync {
    fn on_config_changed(&mut self, event: &ConfigChangeEvent);
}

/// Supported configuration value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    StringArray(Vec<String>),
}

impl ConfigValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_string_array(&self) -> Option<&Vec<String>> {
        match self {
            ConfigValue::StringArray(arr) => Some(arr),
            _ => None,
        }
    }
}

impl From<String> for ConfigValue {
    fn from(value: String) -> Self {
        ConfigValue::String(value)
    }
}

impl From<&str> for ConfigValue {
    fn from(value: &str) -> Self {
        ConfigValue::String(value.to_string())
    }
}

impl From<i64> for ConfigValue {
    fn from(value: i64) -> Self {
        ConfigValue::Integer(value)
    }
}

impl From<f64> for ConfigValue {
    fn from(value: f64) -> Self {
        ConfigValue::Float(value)
    }
}

impl From<bool> for ConfigValue {
    fn from(value: bool) -> Self {
        ConfigValue::Boolean(value)
    }
}

impl From<Vec<String>> for ConfigValue {
    fn from(value: Vec<String>) -> Self {
        ConfigValue::StringArray(value)
    }
}

/// Main configuration system following tn5250j patterns
pub struct SessionConfig {
    properties: HashMap<String, ConfigValue>,
    listeners: Vec<Box<dyn ConfigChangeListener>>,
    session_name: String,
    config_resource: String,
}

impl SessionConfig {
    /// Create a new configuration instance
    pub fn new(config_resource: String, session_name: String) -> Self {
        let mut config = Self {
            properties: HashMap::new(),
            listeners: Vec::new(),
            session_name,
            config_resource,
        };
        
        // Initialize with default values
        config.set_defaults();
        config
    }

    /// Set default configuration values
    fn set_defaults(&mut self) {
        // Display settings
        self.properties.insert("display.screenSize".to_string(), "24x80".into());
        self.properties.insert("display.colorTheme".to_string(), "default".into());
        self.properties.insert("display.fontFamily".to_string(), "monospace".into());
        self.properties.insert("display.fontSize".to_string(), 12i64.into());
        
        // Keypad settings (inspired by tn5250j KeypadAttributesPanel)
        self.properties.insert("keypad.enabled".to_string(), true.into());
        self.properties.insert("keypad.fontSize".to_string(), 12.0f64.into());
        self.properties.insert("keypad.mnemonics".to_string(), 
            vec!["F1".to_string(), "F2".to_string(), "F3".to_string(), "F4".to_string(),
                 "F5".to_string(), "F6".to_string(), "F7".to_string(), "F8".to_string(),
                 "F9".to_string(), "F10".to_string(), "F11".to_string(), "F12".to_string(),
                 "ENTER".to_string(), "CLEAR".to_string(), "SYSREQ".to_string()].into());
        
        // Connection settings
        self.properties.insert("connection.host".to_string(), "".into());
        self.properties.insert("connection.port".to_string(), 23i64.into());
        self.properties.insert("connection.ssl".to_string(), false.into());
        self.properties.insert("connection.deviceName".to_string(), "IBM-3179-2".into());
        self.properties.insert("connection.protocol".to_string(), "auto".into());
        // TLS sub-options
        self.properties.insert("connection.tls.insecure".to_string(), false.into());
        self.properties.insert("connection.tls.caBundlePath".to_string(), "".into());
        
        // Session settings
        self.properties.insert("session.autoConnect".to_string(), false.into());
        self.properties.insert("session.keepAlive".to_string(), true.into());
        self.properties.insert("session.timeout".to_string(), 30i64.into());
        
        // Terminal settings
        self.properties.insert("terminal.cursorBlink".to_string(), true.into());
        self.properties.insert("terminal.insertMode".to_string(), false.into());
        self.properties.insert("terminal.mouseSupport".to_string(), true.into());
        self.properties.insert("terminal.type".to_string(), "IBM-3179-2".into());
        
        // Field settings
        self.properties.insert("fields.validateInput".to_string(), true.into());
        self.properties.insert("fields.mandatoryHighlight".to_string(), true.into());
        self.properties.insert("fields.errorHighlight".to_string(), true.into());
    }

    /// Get configuration property as string
    pub fn get_string_property(&self, key: &str) -> Option<String> {
        self.properties.get(key).and_then(|v| v.as_string().map(|s| s.to_string()))
    }

    /// Get configuration property as string with default
    pub fn get_string_property_or(&self, key: &str, default: &str) -> String {
        self.get_string_property(key).unwrap_or_else(|| default.to_string())
    }

    /// Get configuration property as integer
    pub fn get_int_property(&self, key: &str) -> Option<i64> {
        self.properties.get(key).and_then(|v| v.as_integer())
    }

    /// Get configuration property as integer with default
    pub fn get_int_property_or(&self, key: &str, default: i64) -> i64 {
        self.get_int_property(key).unwrap_or(default)
    }

    /// Get configuration property as float
    pub fn get_float_property(&self, key: &str) -> Option<f64> {
        self.properties.get(key).and_then(|v| v.as_float())
    }

    /// Get configuration property as float with default
    pub fn get_float_property_or(&self, key: &str, default: f64) -> f64 {
        self.get_float_property(key).unwrap_or(default)
    }

    /// Get configuration property as boolean
    pub fn get_boolean_property(&self, key: &str) -> Option<bool> {
        self.properties.get(key).and_then(|v| v.as_boolean())
    }

    /// Get configuration property as boolean with default
    pub fn get_boolean_property_or(&self, key: &str, default: bool) -> bool {
        self.get_boolean_property(key).unwrap_or(default)
    }

    /// Get configuration property as string array
    pub fn get_string_array_property(&self, key: &str) -> Option<&Vec<String>> {
        self.properties.get(key).and_then(|v| v.as_string_array())
    }

    /// Set configuration property and fire change event
    pub fn set_property<T: Into<ConfigValue>>(&mut self, key: &str, value: T) {
        let new_value = value.into();
        let old_value = self.properties.get(key).cloned();
        
        self.properties.insert(key.to_string(), new_value.clone());
        
        // Fire change event
        let event = ConfigChangeEvent {
            property_name: key.to_string(),
            old_value,
            new_value,
        };
        
        self.fire_change_event(&event);
    }

    /// Add a configuration change listener
    pub fn add_listener(&mut self, listener: Box<dyn ConfigChangeListener>) {
        self.listeners.push(listener);
    }

    /// Fire configuration change event to all listeners
    fn fire_change_event(&mut self, event: &ConfigChangeEvent) {
        for listener in &mut self.listeners {
            listener.on_config_changed(event);
        }
    }

    /// Get session name
    pub fn get_session_name(&self) -> &str {
        &self.session_name
    }

    /// Get configuration resource name
    pub fn get_config_resource(&self) -> &str {
        &self.config_resource
    }

    /// Serialize configuration to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.properties)
    }

    /// Load configuration from JSON
    pub fn from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let loaded_properties: HashMap<String, ConfigValue> = serde_json::from_str(json)?;
        
        // Update properties and fire change events
        for (key, value) in loaded_properties {
            let old_value = self.properties.get(&key).cloned();
            self.properties.insert(key.clone(), value.clone());
            
            let event = ConfigChangeEvent {
                property_name: key,
                old_value,
                new_value: value,
            };
            
            self.fire_change_event(&event);
        }
        
        Ok(())
    }

    /// Get all property keys
    pub fn get_all_keys(&self) -> Vec<String> {
        self.properties.keys().cloned().collect()
    }

    /// Check if property exists
    pub fn has_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }

    /// Remove property and fire change event
    pub fn remove_property(&mut self, key: &str) -> Option<ConfigValue> {
        if let Some(old_value) = self.properties.remove(key) {
            let event = ConfigChangeEvent {
                property_name: key.to_string(),
                old_value: Some(old_value.clone()),
                new_value: ConfigValue::String("".to_string()), // Placeholder for removed value
            };
            
            self.fire_change_event(&event);
            Some(old_value)
        } else {
            None
        }
    }

    /// Get protocol mode from configuration
    /// Returns "auto", "tn5250", or "tn3270"
    pub fn get_protocol_mode(&self) -> String {
        self.get_string_property_or("connection.protocol", "auto")
    }
    
    /// Set protocol mode in configuration
    /// Valid values: "auto", "tn5250", "tn3270"
    pub fn set_protocol_mode(&mut self, mode: &str) -> Result<(), TN5250Error> {
        match mode {
            "auto" | "tn5250" | "tn3270" => {
                self.set_property("connection.protocol", mode);
                Ok(())
            }
            _ => Err(TN5250Error::Protocol(ProtocolError::InvalidProtocolConfiguration {
                parameter: "connection.protocol".to_string(),
                value: mode.to_string(),
                reason: "Must be 'auto', 'tn5250', or 'tn3270'".to_string(),
            }))
        }
    }
    
    /// Get terminal type from configuration
    pub fn get_terminal_type(&self) -> String {
        self.get_string_property_or("terminal.type", "IBM-3179-2")
    }
    
    /// Set terminal type in configuration
    /// Valid 5250 types: IBM-3179-2, IBM-3196-A1, IBM-5251-11, IBM-5291-1, IBM-5292-2
    /// Valid 3270 types: IBM-3278-2, IBM-3279-2, IBM-3279-3, IBM-3278-3, IBM-3278-4, IBM-3278-5
    pub fn set_terminal_type(&mut self, terminal_type: &str) -> Result<(), TN5250Error> {
        // Validate terminal type
        let valid_5250_types = ["IBM-3179-2", "IBM-3196-A1", "IBM-5251-11", "IBM-5291-1", "IBM-5292-2"];
        let valid_3270_types = ["IBM-3278-2", "IBM-3279-2", "IBM-3279-3", "IBM-3278-3", "IBM-3278-4", "IBM-3278-5"];
        
        if valid_5250_types.contains(&terminal_type) || valid_3270_types.contains(&terminal_type) {
            self.set_property("terminal.type", terminal_type);
            Ok(())
        } else {
            Err(TN5250Error::Config(ConfigError::InvalidParameter {
                parameter: "terminal.type".to_string(),
                value: terminal_type.to_string(),
                reason: "Must be a valid 5250 or 3270 terminal type".to_string(),
            }))
        }
    }
    
    /// Validate protocol and terminal type combination
    pub fn validate_protocol_terminal_combination(&self) -> Result<(), TN5250Error> {
        let protocol = self.get_protocol_mode();
        let terminal_type = self.get_terminal_type();
        
        // Auto mode accepts any terminal type
        if protocol == "auto" {
            return Ok(());
        }
        
        let valid_5250_types = ["IBM-3179-2", "IBM-3196-A1", "IBM-5251-11", "IBM-5291-1", "IBM-5292-2"];
        let valid_3270_types = ["IBM-3278-2", "IBM-3279-2", "IBM-3279-3", "IBM-3278-3", "IBM-3278-4", "IBM-3278-5"];
        
        match protocol.as_str() {
            "tn5250" => {
                if !valid_5250_types.contains(&terminal_type.as_str()) {
                    return Err(TN5250Error::Protocol(ProtocolError::ProtocolMismatch {
                        configured: format!("TN5250 with terminal type '{}'", terminal_type),
                        detected: format!("Terminal type incompatible with TN5250. Valid types: {:?}", valid_5250_types),
                    }));
                }
            }
            "tn3270" => {
                if !valid_3270_types.contains(&terminal_type.as_str()) {
                    return Err(TN5250Error::Protocol(ProtocolError::ProtocolMismatch {
                        configured: format!("TN3270 with terminal type '{}'", terminal_type),
                        detected: format!("Terminal type incompatible with TN3270. Valid types: {:?}", valid_3270_types),
                    }));
                }
            }
            _ => {}
        }
        
        Ok(())
    }
}

/// Parse protocol string to network ProtocolMode
pub fn parse_protocol_string(protocol: &str) -> Result<crate::network::ProtocolMode, TN5250Error> {
    match protocol.to_lowercase().as_str() {
        "auto" => Ok(crate::network::ProtocolMode::AutoDetect),
        "tn5250" | "5250" => Ok(crate::network::ProtocolMode::TN5250),
        "tn3270" | "3270" => Ok(crate::network::ProtocolMode::TN3270),
        "nvt" => Ok(crate::network::ProtocolMode::NVT),
        _ => Err(TN5250Error::Protocol(ProtocolError::UnsupportedProtocol {
            protocol: protocol.to_string(),
            reason: "Must be 'auto', 'tn5250', 'tn3270', or 'nvt'".to_string(),
        }))
    }
}

/// Convert network ProtocolMode to string
pub fn protocol_mode_to_string(mode: crate::network::ProtocolMode) -> String {
    match mode {
        crate::network::ProtocolMode::AutoDetect => "auto".to_string(),
        crate::network::ProtocolMode::TN5250 => "tn5250".to_string(),
        crate::network::ProtocolMode::TN3270 => "tn3270".to_string(),
        crate::network::ProtocolMode::NVT => "nvt".to_string(),
    }
}

/// Get protocol mode from configuration and parse it
pub fn get_protocol_mode_from_config(config: &SessionConfig) -> Result<crate::network::ProtocolMode, TN5250Error> {
    let protocol_str = config.get_protocol_mode();
    parse_protocol_string(&protocol_str)
}

/// Apply protocol configuration to a network connection
pub fn apply_protocol_config_to_connection(
    config: &SessionConfig,
    connection: &mut crate::network::AS400Connection
) -> Result<(), TN5250Error> {
    let protocol_mode = get_protocol_mode_from_config(config)?;
    
    // Validate protocol and terminal type combination before applying
    config.validate_protocol_terminal_combination()?;
    
    // Only set protocol mode if not auto-detect
    if protocol_mode != crate::network::ProtocolMode::AutoDetect {
        connection.set_protocol_mode(protocol_mode);
    }
    
    Ok(())
}

/// Thread-safe configuration wrapper
pub type SharedSessionConfig = Arc<Mutex<SessionConfig>>;

/// Helper function to create a shared configuration
pub fn create_shared_config(config_resource: String, session_name: String) -> SharedSessionConfig {
    Arc::new(Mutex::new(SessionConfig::new(config_resource, session_name)))
}

/// Determine a platform-appropriate default config file path.
/// Priority:
/// 1) TN5250R_CONFIG env var
/// 2) XDG config dir (Linux) ~/.config/tn5250r/session.json
/// 3) macOS: ~/Library/Application Support/tn5250r/session.json
/// 4) Windows: %APPDATA%/tn5250r/session.json
/// 5) Current directory fallback: ./session.json
pub fn default_config_path() -> PathBuf {
    // 1) Explicit override
    if let Ok(p) = std::env::var("TN5250R_CONFIG") {
        return PathBuf::from(p);
    }

    // 2/3/4) Platform-specific locations
    #[cfg(target_os = "linux")]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| Path::new(&h).join(".config"))
            })
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("tn5250r").join("session.json")
    }

    #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd"))]
    {
        let base = std::env::var_os("HOME")
            .map(|h| Path::new(&h).join("Library").join("Application Support"))
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("tn5250r").join("session.json")
    }

    #[cfg(target_os = "windows")]
    {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("tn5250r").join("session.json")
    }

    // Fallback for unsupported platforms
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return PathBuf::from("session.json");
    }
}

/// Load a shared configuration from disk if available; otherwise return defaults.
/// The config's `config_resource` will be set to the resolved path string.
pub fn load_shared_config(session_name: String) -> SharedSessionConfig {
    let path = default_config_path();
    let resource = path.to_string_lossy().to_string();
    let shared = create_shared_config(resource, session_name);

    if path.exists() {
        if let Ok(mut file) = fs::File::open(&path) {
            let mut buf = String::new();
            if let Err(e) = file.read_to_string(&mut buf) {
                eprintln!("Warning: failed to read config file {}: {}", path.display(), e);
                return shared;
            }
            let mut cfg = shared.lock().unwrap_or_else(|poisoned| {
                eprintln!("SECURITY: Config mutex poisoned during load - recovering");
                poisoned.into_inner()
            });
            if let Err(e) = cfg.from_json(&buf) {
                eprintln!("Warning: failed to parse config file {}: {}", path.display(), e);
            }
        }
    }

    shared
}

/// Save the shared configuration to disk using its `config_resource` path.
/// This function is synchronous and should NOT be called from GUI thread.
/// Use save_shared_config_async() instead for non-blocking behavior.
pub fn save_shared_config(shared: &SharedSessionConfig) -> std::io::Result<()> {
    let (path_str, json) = {
        // Use try_lock to avoid blocking if config is currently locked
        let cfg = shared.try_lock()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::WouldBlock, "Config locked"))?;
        let json = cfg.to_json().unwrap_or_else(|_| "{}".to_string());
        (cfg.get_config_resource().to_string(), json)
    };

    let path = PathBuf::from(&path_str);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut f = fs::File::create(&path)?;
    f.write_all(json.as_bytes())?;
    Ok(())
}

/// Async version that saves config in a background thread to avoid blocking GUI.
/// This is the preferred method to call from GUI code.
pub fn save_shared_config_async(shared: &SharedSessionConfig) {
    let shared = Arc::clone(shared);
    std::thread::spawn(move || {
        // Retry a few times if config is locked
        for attempt in 0..3 {
            match save_shared_config(&shared) {
                Ok(()) => {
                    if attempt > 0 {
                        eprintln!("Config saved on attempt {}", attempt + 1);
                    }
                    return;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Config locked, wait briefly and retry
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    eprintln!("Failed to save config: {}", e);
                    return;
                }
            }
        }
        eprintln!("Failed to save config after 3 attempts (config still locked)");
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestListener {
        events: Vec<ConfigChangeEvent>,
    }

    impl TestListener {
        fn new() -> Self {
            Self { events: Vec::new() }
        }
    }

    impl ConfigChangeListener for TestListener {
        fn on_config_changed(&mut self, event: &ConfigChangeEvent) {
            self.events.push(event.clone());
        }
    }

    #[test]
    fn test_config_creation() {
        let config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
        assert_eq!(config.get_session_name(), "test_session");
        assert_eq!(config.get_config_resource(), "test.json");
    }

    #[test]
    fn test_default_values() {
        let config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
        
        // Test default values
        assert_eq!(config.get_string_property_or("display.screenSize", ""), "24x80");
        assert_eq!(config.get_int_property_or("connection.port", 0), 23);
        assert_eq!(config.get_boolean_property_or("keypad.enabled", false), true);
        assert_eq!(config.get_float_property_or("keypad.fontSize", 0.0), 12.0);
    }

    #[test]
    fn test_protocol_mode_configuration() {
        let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
        
        // Test default protocol mode
        assert_eq!(config.get_protocol_mode(), "auto");
        
        // Test setting valid protocol modes
        assert!(config.set_protocol_mode("tn5250").is_ok());
        assert_eq!(config.get_protocol_mode(), "tn5250");
        
        assert!(config.set_protocol_mode("tn3270").is_ok());
        assert_eq!(config.get_protocol_mode(), "tn3270");
        
        assert!(config.set_protocol_mode("auto").is_ok());
        assert_eq!(config.get_protocol_mode(), "auto");
        
        // Test invalid protocol mode
        assert!(config.set_protocol_mode("invalid").is_err());
    }
    
    #[test]
    fn test_terminal_type_configuration() {
        let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
        
        // Test default terminal type
        assert_eq!(config.get_terminal_type(), "IBM-3179-2");
        
        // Test setting valid 5250 terminal types
        assert!(config.set_terminal_type("IBM-3196-A1").is_ok());
        assert_eq!(config.get_terminal_type(), "IBM-3196-A1");
        
        // Test setting valid 3270 terminal types
        assert!(config.set_terminal_type("IBM-3278-2").is_ok());
        assert_eq!(config.get_terminal_type(), "IBM-3278-2");
        
        // Test invalid terminal type
        assert!(config.set_terminal_type("INVALID-TYPE").is_err());
    }
    
    #[test]
    fn test_protocol_terminal_validation() {
        let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
        
        // Auto mode should accept any terminal type
        config.set_protocol_mode("auto").unwrap();
        config.set_terminal_type("IBM-3179-2").unwrap();
        assert!(config.validate_protocol_terminal_combination().is_ok());
        
        config.set_terminal_type("IBM-3278-2").unwrap();
        assert!(config.validate_protocol_terminal_combination().is_ok());
        
        // TN5250 should only accept 5250 terminal types
        config.set_protocol_mode("tn5250").unwrap();
        config.set_terminal_type("IBM-3179-2").unwrap();
        assert!(config.validate_protocol_terminal_combination().is_ok());
        
        config.set_terminal_type("IBM-3278-2").unwrap();
        assert!(config.validate_protocol_terminal_combination().is_err());
        
        // TN3270 should only accept 3270 terminal types
        config.set_protocol_mode("tn3270").unwrap();
        config.set_terminal_type("IBM-3278-2").unwrap();
        assert!(config.validate_protocol_terminal_combination().is_ok());
        
        config.set_terminal_type("IBM-3179-2").unwrap();
        assert!(config.validate_protocol_terminal_combination().is_err());
    }
    
    #[test]
    fn test_parse_protocol_string() {
        assert_eq!(parse_protocol_string("auto").unwrap(), crate::network::ProtocolMode::AutoDetect);
        assert_eq!(parse_protocol_string("tn5250").unwrap(), crate::network::ProtocolMode::TN5250);
        assert_eq!(parse_protocol_string("5250").unwrap(), crate::network::ProtocolMode::TN5250);
        assert_eq!(parse_protocol_string("tn3270").unwrap(), crate::network::ProtocolMode::TN3270);
        assert_eq!(parse_protocol_string("3270").unwrap(), crate::network::ProtocolMode::TN3270);
        assert_eq!(parse_protocol_string("nvt").unwrap(), crate::network::ProtocolMode::NVT);
        assert!(parse_protocol_string("invalid").is_err());
    }
    
    #[test]
    fn test_protocol_mode_to_string() {
        assert_eq!(protocol_mode_to_string(crate::network::ProtocolMode::AutoDetect), "auto");
        assert_eq!(protocol_mode_to_string(crate::network::ProtocolMode::TN5250), "tn5250");
        assert_eq!(protocol_mode_to_string(crate::network::ProtocolMode::TN3270), "tn3270");
        assert_eq!(protocol_mode_to_string(crate::network::ProtocolMode::NVT), "nvt");
    }
    
    #[test]
    fn test_get_protocol_mode_from_config() {
        let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
        
        // Default should be auto
        assert_eq!(get_protocol_mode_from_config(&config).unwrap(), crate::network::ProtocolMode::AutoDetect);
        
        // Set to tn5250
        config.set_protocol_mode("tn5250").unwrap();
        assert_eq!(get_protocol_mode_from_config(&config).unwrap(), crate::network::ProtocolMode::TN5250);
        
        // Set to tn3270
        config.set_protocol_mode("tn3270").unwrap();
        assert_eq!(get_protocol_mode_from_config(&config).unwrap(), crate::network::ProtocolMode::TN3270);
    }

    #[test]
    fn test_tls_defaults() {
        let config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
        assert_eq!(config.get_boolean_property_or("connection.tls.insecure", true), false);
        assert_eq!(config.get_string_property_or("connection.tls.caBundlePath", "missing"), "");
    }

    #[test]
    fn test_property_setters() {
        let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
        
        config.set_property("test.string", "hello");
        config.set_property("test.int", 42i64);
        config.set_property("test.float", std::f64::consts::PI);
        config.set_property("test.bool", true);
        
        assert_eq!(config.get_string_property("test.string"), Some("hello".to_string()));
        assert_eq!(config.get_int_property("test.int"), Some(42));
        assert_eq!(config.get_float_property("test.float"), Some(std::f64::consts::PI));
        assert_eq!(config.get_boolean_property("test.bool"), Some(true));
    }

    #[test]
    fn test_change_listeners() {
        let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
        config.add_listener(Box::new(TestListener::new()));
        config.set_property("test.key", "test.value");
        
        // Note: Due to ownership issues, we can't easily test the listener here
        // In a real implementation, we'd use Arc<Mutex<>> for listeners
        assert!(config.has_property("test.key"));
    }

    #[test]
    fn test_serialization() {
        let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
        config.set_property("custom.setting", "test_value");
        
        let json = config.to_json().expect("Serialization should work");
        assert!(json.contains("custom.setting"));
        assert!(json.contains("test_value"));
        
        let mut new_config = SessionConfig::new("test2.json".to_string(), "test_session2".to_string());
        new_config.from_json(&json).expect("Deserialization should work");
        
        assert_eq!(new_config.get_string_property("custom.setting"), Some("test_value".to_string()));
    }

    #[test]
    fn test_property_removal() {
        let mut config = SessionConfig::new("test.json".to_string(), "test_session".to_string());
        config.set_property("removable.key", "value");
        
        assert!(config.has_property("removable.key"));
        
        let removed = config.remove_property("removable.key");
        assert!(removed.is_some());
        assert!(!config.has_property("removable.key"));
    }

    #[test]
    fn test_shared_config() {
        let shared_config = create_shared_config("shared.json".to_string(), "shared_session".to_string());
        
        {
            let mut config = shared_config.lock().unwrap();
            config.set_property("shared.test", "shared_value");
        }
        
        {
            let config = shared_config.lock().unwrap();
            assert_eq!(config.get_string_property("shared.test"), Some("shared_value".to_string()));
        }
    }
}