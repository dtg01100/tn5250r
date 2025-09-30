//! Rust implementation of IBM 3270 protocol (TN3270)
//!
//! This module provides comprehensive 3270 protocol implementation following
//! RFC 1205 (TN3270) and RFC 2355 (TN3270E) specifications.
//!
//! # Overview
//!
//! The IBM 3270 protocol is a block-oriented terminal protocol used primarily
//! with IBM mainframe systems. Unlike the character-oriented 5250 protocol,
//! 3270 uses buffer addressing and structured fields for efficient data transfer.
//!
//! # Key Differences from TN5250
//!
//! - **Buffer Addressing**: 3270 uses 12-bit or 14-bit buffer addresses instead of row/column
//! - **Command Set**: Different command codes (Write, Read, Erase Write, etc.)
//! - **Field Attributes**: Extended field attributes via SFE (Start Field Extended) order
//! - **Screen Sizes**: Multiple standard sizes (24x80, 32x80, 43x80, 27x132)
//! - **WCC**: Write Control Character for screen control operations
//!
//! # Architecture
//!
//! The module is organized into several submodules:
//!
//! - [`codes`] - TN3270 command codes, order codes, and AID keys
//! - [`protocol`] - 3270 data stream parsing and command processing
//! - [`field`] - Field attribute handling and management
//! - [`display`] - Screen buffer management and display operations
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use tn5250r::lib3270::{Display3270, ProtocolProcessor3270};
//!
//! // Create a display buffer
//! let mut display = Display3270::new();
//!
//! // Create a protocol processor
//! let mut processor = ProtocolProcessor3270::new();
//!
//! // Process incoming 3270 data stream
//! // let data = receive_from_host();
//! // processor.process_data(&data, &mut display)?;
//! ```
//!
//! # Protocol Implementation Status
//!
//! ## Phase 2: Core 3270 Protocol (Current)
//! - ✅ Command codes and order codes
//! - ✅ Basic field attributes
//! - ✅ Extended field attributes (SFE)
//! - ✅ Buffer addressing (12-bit and 14-bit)
//! - ✅ Screen buffer management
//! - ✅ Multiple screen sizes
//!
//! ## Phase 3: Session Management (Future)
//! - ⏳ Telnet negotiation (TN3270E)
//! - ⏳ Device type negotiation
//! - ⏳ Session establishment
//! - ⏳ Error recovery
//!
//! ## Phase 4: Advanced Features (Future)
//! - ⏳ Structured fields
//! - ⏳ Color and highlighting
//! - ⏳ Graphics support
//! - ⏳ Printer support

pub mod codes;
pub mod display;
pub mod field;
pub mod protocol;

// Re-exports for easy access
pub use codes::*;
pub use display::{Display3270, ScreenSize};
pub use field::*;
pub use protocol::ProtocolProcessor3270;

// Re-export EBCDIC functions from protocol_common for convenience
pub use crate::protocol_common::ebcdic::{ascii_to_ebcdic, ebcdic_to_ascii};

// Entry point for lib3270 functionality