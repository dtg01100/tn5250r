//! Common protocol functionality for TN5250 and TN3270
//!
//! This module provides shared functionality that both TN5250 and TN3270 protocols
//! can use, including:
//!
//! - EBCDIC/ASCII conversion utilities
//! - Protocol trait abstractions
//! - Common telnet protocol handling
//!
//! # Architecture
//!
//! The protocol_common module serves as a foundation for implementing multiple
//! terminal protocols (TN5250, TN3270) with shared code and consistent interfaces.
//!
//! ## Modules
//!
//! - [`ebcdic`] - EBCDIC to ASCII conversion utilities
//! - [`traits`] - Protocol trait abstractions for common operations
//! - [`telnet_base`] - Common telnet protocol functionality
//!
//! # Examples
//!
//! ## Using EBCDIC conversion
//!
//! ```
//! use tn5250r::protocol_common::ebcdic::{ebcdic_to_ascii, ascii_to_ebcdic};
//!
//! // Convert EBCDIC to ASCII
//! let ascii_char = ebcdic_to_ascii(0xC1); // 'A'
//! assert_eq!(ascii_char, 'A');
//!
//! // Convert ASCII to EBCDIC
//! let ebcdic_byte = ascii_to_ebcdic('A');
//! assert_eq!(ebcdic_byte, 0xC1);
//! ```
//!
//! ## Implementing protocol traits
//!
//! ```
//! use tn5250r::protocol_common::traits::TerminalProtocol;
//!
//! struct MyProtocol {
//!     connected: bool,
//! }
//!
//! impl TerminalProtocol for MyProtocol {
//!     fn process_data(&mut self, data: &[u8]) -> Result<(), String> {
//!         // Process incoming data
//!         Ok(())
//!     }
//!
//!     fn generate_response(&mut self) -> Option<Vec<u8>> {
//!         None
//!     }
//!
//!     fn reset(&mut self) {
//!         self.connected = false;
//!     }
//!
//!     fn protocol_name(&self) -> &str {
//!         "MyProtocol"
//!     }
//!
//!     fn is_connected(&self) -> bool {
//!         self.connected
//!     }
//!
//!     fn handle_negotiation(&mut self, _option: u8, _data: &[u8]) -> Option<Vec<u8>> {
//!         None
//!     }
//! }
//! ```

pub mod ebcdic;
pub mod traits;
pub mod telnet_base;

// Re-export commonly used items for convenience
pub use ebcdic::{ebcdic_to_ascii, ascii_to_ebcdic, ebcdic_to_ascii_string, ascii_to_ebcdic_vec};
pub use traits::{
    TerminalProtocol, ProtocolSession, DisplayBuffer, FieldManager,
    CommandProcessor, StructuredFieldProcessor,
};
pub use telnet_base::{
    TelnetCommand, TelnetOption, NegotiationState, OptionState,
    build_negotiation, build_subnegotiation, parse_telnet_stream,
};

/// Protocol version information
pub const PROTOCOL_COMMON_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the protocol common module version
pub fn version() -> &'static str {
    PROTOCOL_COMMON_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let ver = version();
        assert!(!ver.is_empty());
    }

    #[test]
    fn test_ebcdic_reexport() {
        // Test that re-exported functions work
        let ascii = ebcdic_to_ascii(0xC1);
        assert_eq!(ascii, 'A');
        
        let ebcdic = ascii_to_ebcdic('A');
        assert_eq!(ebcdic, 0xC1);
    }

    #[test]
    fn test_telnet_reexport() {
        // Test that re-exported telnet functions work
        let seq = build_negotiation(TelnetCommand::WILL, 0);
        assert_eq!(seq, vec![255, 251, 0]);
    }
}