//! Rust port of lib5250 core logic from tn5250
// This module contains comprehensive 5250 protocol implementation with RFC 2877/4777 compliance

pub mod codes;
pub mod display;
pub mod field;
pub mod protocol;
pub mod session;
pub mod telnet;

// Re-exports for easy access
pub use protocol::FieldAttribute;
pub use session::Session;

// Re-export EBCDIC functions from protocol_common for backward compatibility

// Entry point for lib5250 functionality
