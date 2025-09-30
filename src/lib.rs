/// PERFORMANCE MONITORING: Global performance metrics
/// Tracks key performance indicators across the application
pub mod performance_metrics;

// Re-export PerformanceMetrics for easier access
pub use performance_metrics::PerformanceMetrics;

/// INTEGRATION: Cross-platform abstraction layer
/// Provides platform-independent operations for file I/O, networking, and system calls
pub mod platform;

pub mod lib5250;
pub mod ansi_processor;
pub mod config;
pub mod controller;
pub mod error;
pub mod field_manager;
pub mod keyboard;
pub mod monitoring;
pub mod network;
pub mod protocol_state;
pub mod telnet_negotiation;
pub mod terminal;
pub mod test_field_detection;