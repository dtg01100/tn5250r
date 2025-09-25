//! Rust port of lib5250 core logic from tn5250
// This module contains comprehensive 5250 protocol implementation with RFC 2877/4777 compliance

pub mod codes;
pub mod display;
pub mod field;
// pub mod protocol;  // Temporarily disabled - uses wrong command codes
pub mod session;
pub mod telnet;

// Re-exports for easy access
pub use codes::*;
pub use display::Display;
pub use field::*;
// pub use protocol::{
//     Protocol5250, ProtocolProcessor, CommandCode, FieldAttribute, 
//     Packet, ProtocolError, ProtocolState
// };
pub use session::Session;
pub use telnet::*;

// Entry point for lib5250 functionality
