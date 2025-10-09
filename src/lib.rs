/// EBCDIC CHARACTER TRANSLATION: EBCDIC to ASCII conversion utilities
/// Consolidated EBCDIC translation functionality for IBM terminal protocols
pub mod ebcdic;

/// CURSOR VALIDATION: Cursor position validation and bounds checking utilities
/// Centralized cursor validation logic for terminal emulation security
pub mod cursor_utils;

/// COMPONENT UTILITIES: Component configuration and monitoring helpers
/// Centralized utilities for component management and health checks
pub mod component_utils;

/// BUFFER UTILITIES: Terminal buffer iteration and manipulation utilities
/// Efficient iterators and utilities for terminal screen buffer operations
pub mod buffer_utils;

/// INTEGRATION: Cross-platform abstraction layer
/// Provides platform-independent operations for file I/O, networking, and system calls
pub mod platform;

/// PROTOCOL COMMON: Shared protocol functionality for TN5250 and TN3270
/// Provides EBCDIC conversion, protocol traits, and common telnet handling
pub mod protocol_common;

/// LIB5250: IBM 5250 protocol implementation
/// Complete TN5250 protocol support for AS/400 systems
pub mod lib5250;

/// LIB3270: IBM 3270 protocol implementation
/// Complete TN3270 protocol support for mainframe systems
pub mod lib3270;
pub mod ansi_processor;
pub mod config;
pub mod controller;
pub mod error_handling;
pub mod error;
pub mod field_manager;
pub mod keyboard;
pub mod monitoring;
pub mod network;
pub mod protocol_state;
pub mod telnet_negotiation;
pub mod terminal;
pub mod test_field_detection;

/// Session profile management
pub mod session_profile;

/// Profile manager for CRUD operations
pub mod profile_manager;

/// Active session management
pub mod session;

/// Application state management
pub mod app_state;

/// Connection management
pub mod connection;

/// Terminal display rendering
pub mod terminal_display;

/// Input handling
pub mod input;

/// Main application loop
pub mod app;

/// UI components
pub mod ui {
    pub mod monitoring_ui;
    pub mod dialogs;
    pub mod function_keys;
    pub mod profile_manager_ui;
}

/// Application constants
pub mod constants;

pub mod network_platform;

/// PERFORMANCE MONITORING: Global performance metrics
/// Tracks key performance indicators across the application
pub mod performance_metrics;

// Re-export Session for easier access across modules
pub use session::Session;

// Re-export PerformanceMetrics for easier access
pub use performance_metrics::PerformanceMetrics;

/// Test modules
#[cfg(test)]
pub mod tests {
    /// Dynamic sizing behavior tests
    pub mod dynamic_sizing_tests;
}
