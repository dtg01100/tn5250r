//! Protocol trait abstractions for TN5250 and TN3270
//!
//! This module defines common traits that both TN5250 and TN3270 protocols
//! must implement, enabling protocol-agnostic code and shared functionality.

use std::io;

/// Terminal protocol trait defining core protocol operations
///
/// This trait abstracts the common operations that both TN5250 and TN3270
/// protocols must support, allowing for protocol-agnostic terminal handling.
pub trait TerminalProtocol {
    /// Process incoming data from the host
    ///
    /// # Arguments
    ///
    /// * `data` - Raw bytes received from the host
    ///
    /// # Returns
    ///
    /// Result indicating success or an error message
    fn process_data(&mut self, data: &[u8]) -> Result<(), String>;

    /// Generate response data to send to the host
    ///
    /// # Returns
    ///
    /// A vector of bytes to send, or None if no response is needed
    fn generate_response(&mut self) -> Option<Vec<u8>>;

    /// Reset the protocol state to initial conditions
    fn reset(&mut self);

    /// Get the protocol name (e.g., "TN5250", "TN3270")
    fn protocol_name(&self) -> &str;

    /// Check if the protocol is in a connected state
    fn is_connected(&self) -> bool;

    /// Handle protocol-specific negotiation
    ///
    /// # Arguments
    ///
    /// * `option` - The telnet option being negotiated
    /// * `data` - Additional negotiation data
    ///
    /// # Returns
    ///
    /// Response bytes to send, or None if no response needed
    fn handle_negotiation(&mut self, option: u8, data: &[u8]) -> Option<Vec<u8>>;
}

/// Protocol session management trait
///
/// This trait handles session-level operations like connection establishment,
/// authentication, and session termination.
pub trait ProtocolSession {
    /// Establish a new session with the host
    ///
    /// # Arguments
    ///
    /// * `host` - The hostname or IP address
    /// * `port` - The port number
    ///
    /// # Returns
    ///
    /// Result indicating success or an IO error
    fn connect(&mut self, host: &str, port: u16) -> io::Result<()>;

    /// Terminate the current session
    fn disconnect(&mut self) -> io::Result<()>;

    /// Send data to the host
    ///
    /// # Arguments
    ///
    /// * `data` - The bytes to send
    ///
    /// # Returns
    ///
    /// Result indicating success or an IO error
    fn send(&mut self, data: &[u8]) -> io::Result<()>;

    /// Receive data from the host
    ///
    /// # Arguments
    ///
    /// * `buffer` - Buffer to store received data
    ///
    /// # Returns
    ///
    /// Number of bytes received, or an IO error
    fn receive(&mut self, buffer: &mut [u8]) -> io::Result<usize>;

    /// Check if the session is currently active
    fn is_active(&self) -> bool;

    /// Get the current session state as a string
    fn session_state(&self) -> String;
}

/// Display buffer operations trait
///
/// This trait abstracts screen buffer operations that are common to both
/// TN5250 and TN3270 protocols, such as cursor positioning, character writing,
/// and screen clearing.
pub trait DisplayBuffer {
    /// Get the screen dimensions (rows, columns)
    fn dimensions(&self) -> (usize, usize);

    /// Set the cursor position
    ///
    /// # Arguments
    ///
    /// * `row` - The row position (0-based)
    /// * `col` - The column position (0-based)
    ///
    /// # Returns
    ///
    /// Result indicating success or an error if position is invalid
    fn set_cursor(&mut self, row: usize, col: usize) -> Result<(), String>;

    /// Get the current cursor position
    ///
    /// # Returns
    ///
    /// A tuple of (row, column) positions
    fn get_cursor(&self) -> (usize, usize);

    /// Write a character at the current cursor position
    ///
    /// # Arguments
    ///
    /// * `ch` - The character to write
    /// * `attribute` - Display attribute (color, intensity, etc.)
    fn write_char(&mut self, ch: char, attribute: u8);

    /// Write a string at the current cursor position
    ///
    /// # Arguments
    ///
    /// * `s` - The string to write
    /// * `attribute` - Display attribute for all characters
    fn write_string(&mut self, s: &str, attribute: u8);

    /// Read a character at a specific position
    ///
    /// # Arguments
    ///
    /// * `row` - The row position
    /// * `col` - The column position
    ///
    /// # Returns
    ///
    /// The character and its attribute, or None if position is invalid
    fn read_char(&self, row: usize, col: usize) -> Option<(char, u8)>;

    /// Clear the entire screen
    fn clear(&mut self);

    /// Clear a specific region of the screen
    ///
    /// # Arguments
    ///
    /// * `start_row` - Starting row
    /// * `start_col` - Starting column
    /// * `end_row` - Ending row
    /// * `end_col` - Ending column
    fn clear_region(&mut self, start_row: usize, start_col: usize, end_row: usize, end_col: usize);

    /// Get the entire screen buffer as a string
    fn get_buffer(&self) -> String;

    /// Check if the buffer has been modified since last check
    fn is_modified(&self) -> bool;

    /// Mark the buffer as unmodified
    fn clear_modified(&mut self);
}

/// Field management trait for input fields
///
/// Both TN5250 and TN3270 support field-based input, where certain regions
/// of the screen are designated as input fields with specific attributes.
pub trait FieldManager {
    /// Define a new input field
    ///
    /// # Arguments
    ///
    /// * `row` - Starting row
    /// * `col` - Starting column
    /// * `length` - Field length
    /// * `attributes` - Field attributes (protected, numeric, etc.)
    fn define_field(&mut self, row: usize, col: usize, length: usize, attributes: u8);

    /// Remove a field at the specified position
    ///
    /// # Arguments
    ///
    /// * `row` - Field row
    /// * `col` - Field column
    fn remove_field(&mut self, row: usize, col: usize);

    /// Get field information at a position
    ///
    /// # Arguments
    ///
    /// * `row` - Row position
    /// * `col` - Column position
    ///
    /// # Returns
    ///
    /// Field attributes if a field exists at that position
    fn get_field(&self, row: usize, col: usize) -> Option<u8>;

    /// Check if a position is within an input field
    ///
    /// # Arguments
    ///
    /// * `row` - Row position
    /// * `col` - Column position
    fn is_field(&self, row: usize, col: usize) -> bool;

    /// Get all modified fields
    ///
    /// # Returns
    ///
    /// A vector of (row, col, data) tuples for modified fields
    fn get_modified_fields(&self) -> Vec<(usize, usize, String)>;

    /// Clear all fields
    fn clear_fields(&mut self);
}

/// Command processing trait
///
/// This trait handles protocol-specific command processing for both
/// TN5250 and TN3270 protocols.
pub trait CommandProcessor {
    /// Process a protocol command
    ///
    /// # Arguments
    ///
    /// * `command` - The command code
    /// * `data` - Command data
    ///
    /// # Returns
    ///
    /// Result indicating success or an error message
    fn process_command(&mut self, command: u8, data: &[u8]) -> Result<(), String>;

    /// Get the list of supported commands
    fn supported_commands(&self) -> Vec<u8>;

    /// Check if a command is supported
    ///
    /// # Arguments
    ///
    /// * `command` - The command code to check
    fn is_command_supported(&self, command: u8) -> bool;
}

/// Structured field processing trait
///
/// Both protocols support structured fields for extended functionality.
pub trait StructuredFieldProcessor {
    /// Process a structured field
    ///
    /// # Arguments
    ///
    /// * `field_id` - The structured field identifier
    /// * `data` - Field data
    ///
    /// # Returns
    ///
    /// Result indicating success or an error message
    fn process_structured_field(&mut self, field_id: u8, data: &[u8]) -> Result<(), String>;

    /// Get the list of supported structured fields
    fn supported_structured_fields(&self) -> Vec<u8>;

    /// Check if a structured field is supported
    ///
    /// # Arguments
    ///
    /// * `field_id` - The field identifier to check
    fn is_structured_field_supported(&self, field_id: u8) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing trait definitions
    struct MockProtocol {
        connected: bool,
    }

    impl TerminalProtocol for MockProtocol {
        fn process_data(&mut self, _data: &[u8]) -> Result<(), String> {
            Ok(())
        }

        fn generate_response(&mut self) -> Option<Vec<u8>> {
            None
        }

        fn reset(&mut self) {
            self.connected = false;
        }

        fn protocol_name(&self) -> &str {
            "MOCK"
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        fn handle_negotiation(&mut self, _option: u8, _data: &[u8]) -> Option<Vec<u8>> {
            None
        }
    }

    #[test]
    fn test_mock_protocol() {
        let mut protocol = MockProtocol { connected: true };
        assert_eq!(protocol.protocol_name(), "MOCK");
        assert!(protocol.is_connected());
        protocol.reset();
        assert!(!protocol.is_connected());
    }
}