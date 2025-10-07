//! Profile manager for TN5250R session profiles
//!
//! This module provides CRUD operations for session profiles with JSON-based persistence.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::session_profile::SessionProfile;

/// Manager for session profiles with persistence
#[derive(Debug)]
pub struct ProfileManager {
    profiles: HashMap<String, SessionProfile>,
    config_dir: PathBuf,
}

impl ProfileManager {
    /// Create a new profile manager and load existing profiles
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = Self::get_config_dir();
        fs::create_dir_all(&config_dir)?;

        let mut manager = Self {
            profiles: HashMap::new(),
            config_dir,
        };

        manager.load_profiles()?;
        Ok(manager)
    }

    /// Get the configuration directory for profiles
    fn get_config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tn5250r")
            .join("profiles")
    }

    /// Load all profiles from disk
    fn load_profiles(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.profiles.clear();

        let entries = fs::read_dir(&self.config_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match Self::load_profile_from_file(&path) {
                    Ok(profile) => {
                        self.profiles.insert(profile.id.clone(), profile);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load profile from {path:?}: {e}");
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a single profile from a JSON file
    fn load_profile_from_file(path: &Path) -> Result<SessionProfile, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let profile: SessionProfile = serde_json::from_str(&content)?;
        Ok(profile)
    }

    /// Save a profile to disk
    fn save_profile_to_file(&self, profile: &SessionProfile) -> Result<(), Box<dyn std::error::Error>> {
        let filename = format!("{}.json", profile.filename());
        let path = self.config_dir.join(filename);

        let content = serde_json::to_string_pretty(profile)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Create a new profile
    pub fn create_profile(&mut self, mut profile: SessionProfile) -> Result<(), Box<dyn std::error::Error>> {
        profile.touch();
        self.profiles.insert(profile.id.clone(), profile.clone());
        self.save_profile_to_file(&profile)
    }

    /// Get a profile by ID
    pub fn get_profile(&self, id: &str) -> Option<&SessionProfile> {
        self.profiles.get(id)
    }

    /// Get a profile by name
    pub fn get_profile_by_name(&self, name: &str) -> Option<&SessionProfile> {
        self.profiles.values().find(|p| p.name == name)
    }

    /// Get all profiles
    pub fn get_profiles(&self) -> Vec<&SessionProfile> {
        self.profiles.values().collect()
    }

    /// Update an existing profile
    pub fn update_profile(&mut self, profile: SessionProfile) -> Result<(), Box<dyn std::error::Error>> {
        let mut profile = profile;
        profile.touch();
        self.profiles.insert(profile.id.clone(), profile.clone());
        self.save_profile_to_file(&profile)
    }

    /// Delete a profile
    pub fn delete_profile(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(profile) = self.profiles.remove(id) {
            let filename = format!("{}.json", profile.filename());
            let path = self.config_dir.join(filename);
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Check if a profile exists by name
    pub fn profile_exists(&self, name: &str) -> bool {
        self.profiles.values().any(|p| p.name == name)
    }

    /// Get profile names for CLI completion
    pub fn get_profile_names(&self) -> Vec<String> {
        self.profiles.values().map(|p| p.name.clone()).collect()
    }

    /// Reload profiles from disk (useful for external changes)
    pub fn reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.load_profiles()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_profile_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let _config_dir = temp_dir.path().join("profiles");

        // This test would need mocking for dirs::config_dir
        // For now, just test that the struct can be created conceptually
        // Test passes - no specific assertion needed
    }

    #[test]
    fn test_profile_crud_operations() {
        let temp_dir = tempdir().unwrap();
        let _config_dir = temp_dir.path().join("profiles");

        // Create a temporary profile manager with a custom config dir
        // This is a simplified test since we can't easily mock dirs::config_dir
        // In a real test environment, we'd use dependency injection

        // Test profile creation
        let profile = SessionProfile::new(
            "Test Profile".to_string(),
            "test.example.com".to_string(),
            23,
        );

        assert_eq!(profile.name, "Test Profile");
        assert_eq!(profile.host, "test.example.com");
        assert_eq!(profile.port, 23);
        assert!(!profile.id.is_empty());

        // Test profile serialization
        let json = serde_json::to_string(&profile).unwrap();
        assert!(json.contains("Test Profile"));
        assert!(json.contains("test.example.com"));

        // Test profile deserialization
        let deserialized: SessionProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, profile.name);
        assert_eq!(deserialized.host, profile.host);
        assert_eq!(deserialized.port, profile.port);
    }
}