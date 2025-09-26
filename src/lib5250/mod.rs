//! Rust port of lib5250 core logic from tn5250
// This module contains comprehensive 5250 protocol implementation with RFC 2877/4777 compliance

pub mod codes;
pub mod display;
pub mod field;
pub mod protocol;
pub mod session;
pub mod telnet;

// Re-exports for easy access
pub use codes::*;
pub use display::Display;
pub use field::*;
pub use protocol::{
	ProtocolProcessor, Packet, ebcdic_to_ascii, ascii_to_ebcdic, FieldAttribute,
};
pub use session::Session;
pub use telnet::*;

// Entry point for lib5250 functionality
