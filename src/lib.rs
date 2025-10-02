/// PERFORMANCE MONITORING: Global performance metrics
/// Tracks key performance indicators across the application
pub mod performance_metrics;

// Re-export PerformanceMetrics for easier access
pub use performance_metrics::PerformanceMetrics;

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
}

/// Application constants
pub mod constants;
