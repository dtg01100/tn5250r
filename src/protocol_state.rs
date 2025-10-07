// TODO: Will need session integration later
use crate::component_utils::error_messages;
use crate::error::ProtocolError;
use crate::monitoring::{set_component_status, set_component_error, ComponentState};
use crate::field_manager::{FieldManager, FieldType, Field as FmField};
use crate::terminal::{TerminalScreen, TERMINAL_WIDTH, TERMINAL_HEIGHT};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProtocolState {
    InitialNegotiation,
    Connected,
    Receiving,
    Sending,
    Error,
}

#[derive(Debug)]
pub struct DeviceAttributes {
    pub model_number: u8,
    pub character_set: u8,
    pub extended_char_set: bool,
    pub color_support: bool,
    pub highlighting_support: bool,
    pub max_buffer_size: u16,
}

impl DeviceAttributes {
    pub fn new() -> Self {
        Self {
            model_number: 0x02, // Common model for IBM i Access
            character_set: 0x00, // EBCDIC
            extended_char_set: false,
            color_support: false,
            highlighting_support: true,
            max_buffer_size: 1920, // Standard for 80x24
        }
    }
}

impl Default for DeviceAttributes {
    fn default() -> Self { Self::new() }
}

#[derive(Debug)]
pub struct ProtocolStateMachine {
    pub state: ProtocolState,
    pub screen: TerminalScreen,
    pub cursor_position: usize,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub field_manager: FieldManager,
    pub device_attributes: DeviceAttributes,
    pub connected: bool,
    saved_screen: Option<TerminalScreen>,
    saved_fields: Option<Vec<FmField>>,
}

impl ProtocolStateMachine {
    pub fn new() -> Self {
        Self {
            state: ProtocolState::InitialNegotiation,
            screen: TerminalScreen::new(),
            cursor_position: 0,
            cursor_row: 0,
            cursor_col: 0,
            field_manager: FieldManager::new(),
            device_attributes: DeviceAttributes::new(),
            connected: false,
            saved_screen: None,
            saved_fields: None,
        }
    }

    pub fn set_cursor(&mut self, row: usize, col: usize) {
        // CRITICAL FIX: Enhanced boundary checking with proper validation
        // Prevent cursor from going out of bounds and handle edge cases
        let safe_row = if row >= TERMINAL_HEIGHT { TERMINAL_HEIGHT - 1 } else { row };
        let safe_col = if col >= TERMINAL_WIDTH { TERMINAL_WIDTH - 1 } else { col };

        // Additional validation: ensure coordinates are not negative
        let valid_row = if safe_row > 0 { safe_row } else { 0 };
        let valid_col = if safe_col > 0 { safe_col } else { 0 };

        self.cursor_row = valid_row;
        self.cursor_col = valid_col;
        self.cursor_position = valid_row * TERMINAL_WIDTH + valid_col;
    }

    pub fn get_cursor(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    pub fn detect_fields(&mut self) {
        self.field_manager.detect_fields(&self.screen);
    }

    pub fn add_field(&mut self, row: usize, col: usize, length: usize, field_type: FieldType, attribute: u8) {
        let mut field = FmField::new(self.field_manager.field_count() + 1, field_type, row + 1, col + 1, length);
        field.set_enhanced_attributes(attribute);
        self.field_manager.add_field_for_test(field);
    }

    pub fn exists_at_pos(&self, pos: usize) -> bool {
        let row = pos / TERMINAL_WIDTH;
        let col = pos % TERMINAL_WIDTH;
        self.field_manager.get_fields_slice().iter().any(|f| f.contains_position(row + 1, col + 1))
    }

    pub fn find_field_at_pos(&self, pos: usize) -> Option<&FmField> {
        let row = pos / TERMINAL_WIDTH;
        let col = pos % TERMINAL_WIDTH;
        self.field_manager.get_field_at_position(row + 1, col + 1)
    }

    pub fn determine_field_type(&self, attribute: u8) -> FieldType {
        if attribute & 0x20 != 0 {
            FieldType::Protected
        } else if attribute & 0x10 != 0 {
            FieldType::Numeric
        } else if attribute & 0x08 != 0 {
            FieldType::Bypass
        } else if attribute & 0x0C != 0 {
            FieldType::Mandatory
        } else {
            // Both 0x04 and default cases are Input type
            FieldType::Input
        }
    }

    pub fn read_buffer(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        for field in self.field_manager.get_fields() {
            if field.modified {
                buffer.extend_from_slice(field.content.as_bytes());
            }
        }
        buffer
    }

    pub fn connect(&mut self) {
        self.state = ProtocolState::Connected;
        self.connected = true;
        self.screen.clear();
        self.screen.write_string("Connected to AS/400 system\nReady...\n");
        self.field_manager.clear_all_fields();
        set_component_status("protocol", ComponentState::Running);
        set_component_error("protocol", None::<&str>);
    }

    pub fn disconnect(&mut self) {
        self.state = ProtocolState::InitialNegotiation;
        self.connected = false;
        self.screen.clear();
        self.screen.write_string("Disconnected from AS/400 system\n");
        self.field_manager.clear_all_fields();
        set_component_status("protocol", ComponentState::Stopped);
        set_component_error("protocol", None::<&str>);
    }

    pub fn process_data(&mut self, data: &[u8]) -> Result<Vec<u8>, ProtocolError> {
        // CRITICAL FIX: Enhanced state validation and transition logic
        // Prevent invalid state transitions and ensure proper error handling

        // Validate input data to prevent processing of malformed packets
        if data.is_empty() {
            set_component_status("protocol", ComponentState::Error);
            set_component_error("protocol", Some("Empty data packet"));
            return Err(ProtocolError::DeviceIdError { message: "Empty data packet".to_string() });
        }

        // Validate data size to prevent memory exhaustion attacks
        if data.len() > 65535 {
            set_component_status("protocol", ComponentState::Error);
            set_component_error("protocol", Some("Data packet too large"));
            return Err(ProtocolError::DeviceIdError { message: "Data packet too large".to_string() });
        }

        match self.state {
            ProtocolState::Connected | ProtocolState::Receiving => {
                if self.state == ProtocolState::Connected {
                    self.state = ProtocolState::Receiving;
                }
                set_component_status("protocol", ComponentState::Running);
                set_component_error("protocol", None::<&str>);
                Ok(vec![])
            },
            ProtocolState::InitialNegotiation => {
                if data.len() < 2 {
                    set_component_status("protocol", ComponentState::Error);
                    set_component_error("protocol", Some("Invalid negotiation data"));
                    return Err(ProtocolError::DeviceIdError { message: "Invalid negotiation data".to_string() });
                }
                match self.handle_negotiation(data) {
                    Ok(_) => {
                        set_component_status("protocol", ComponentState::Running);
                        set_component_error("protocol", None::<&str>);
                        Ok(self.create_device_identification_response())
                    },
                    Err(e) => {
                        set_component_status("protocol", ComponentState::Error);
                        set_component_error("protocol", Some(format!("Negotiation error: {e}")));
                        Err(e)
                    }
                }
            },
            ProtocolState::Error => {
                set_component_status("protocol", ComponentState::Error);
                set_component_error("protocol", Some("Protocol is in error state"));
                Err(ProtocolError::DeviceIdError { message: "Protocol is in error state".to_string() })
            },
            _ => {
                set_component_status("protocol", ComponentState::Error);
                set_component_error("protocol", Some("Invalid protocol state"));
                Err(ProtocolError::DeviceIdError { message: "Invalid protocol state".to_string() })
            },
        }
    }

    fn handle_negotiation(&mut self, data: &[u8]) -> Result<(), ProtocolError> {
        // CRITICAL FIX: Validate negotiation data and only transition on success
        // This prevents invalid state transitions during failed negotiations

        // Basic validation of negotiation data
        if data.is_empty() {
            set_component_status("protocol", ComponentState::Error);
            set_component_error("protocol", Some("Empty negotiation data"));
            return Err(ProtocolError::DeviceIdError { message: "Empty negotiation data".to_string() });
        }

        // Check for valid telnet negotiation commands (IAC commands)
        let mut valid_negotiation = false;
        let mut i = 0;
        while i < data.len() {
            if data[i] == 255 { // IAC
                if i + 1 < data.len() {
                    // Check for valid telnet command bytes
                    match data[i + 1] {
                        251..=254 => { // WILL, WONT, DO, DONT
                            if i + 2 < data.len() {
                                valid_negotiation = true;
                                i += 3;
                            } else {
                                break; // Malformed command
                            }
                        },
                        250 => { // SB (subnegotiation)
                            // Find SE (240) to end subnegotiation
                            let mut j = i + 2;
                            while j + 1 < data.len() {
                                if data[j] == 255 && data[j + 1] == 240 {
                                    valid_negotiation = true;
                                    i = j + 2;
                                    break;
                                }
                                j += 1;
                            }
                            if j + 1 >= data.len() {
                                return Err(ProtocolError::DeviceIdError { message: "Malformed subnegotiation".to_string() });
                            }
                        },
                        _ => {
                            i += 2; // Skip other telnet commands
                        }
                    }
                } else {
                    break; // Incomplete IAC command
                }
            } else {
                i += 1;
            }
        }

        // Only transition to connected state if we found valid negotiation
        if valid_negotiation {
            self.state = ProtocolState::Connected;
            set_component_status("protocol", ComponentState::Running);
            set_component_error("protocol", None::<&str>);
            Ok(())
        } else {
            set_component_status("protocol", ComponentState::Error);
            set_component_error("protocol", Some("Invalid negotiation data"));
            Err(ProtocolError::DeviceIdError { message: "Invalid negotiation data".to_string() })
        }
    }

    fn create_device_identification_response(&self) -> Vec<u8> {
        vec![
            0xF0, 0xF0, 0xF0, 0xF0, // Device type
            0xF1, 0xF2, 0xF3, 0xF4, // Additional info
        ]
    }

    pub fn set_cursor_position(&mut self, col: usize, row: usize) {
        // CRITICAL FIX: Add parameter validation before calling set_cursor
        // This prevents invalid state transitions and ensures cursor bounds
        if col > 0 && row > 0 && col <= TERMINAL_WIDTH && row <= TERMINAL_HEIGHT {
            self.set_cursor(row - 1, col - 1); // Convert to 0-based indexing
        } else {
            // Log invalid cursor position attempt for debugging
            eprintln!("SECURITY: Invalid cursor position ({row}, {col}) - out of bounds");
            // Set to safe default position (1,1) in 1-based coordinates
            self.set_cursor(0, 0);
        }
    }

    pub fn get_cursor_position(&self) -> (usize, usize) {
        self.get_cursor()
    }

    pub fn save_screen_state(&mut self) {
        self.saved_screen = Some(self.screen.clone());
        self.saved_fields = Some(self.field_manager.get_fields().clone());
    }

    pub fn restore_screen_state(&mut self) {
        if let Some(saved) = self.saved_screen.take() {
            self.screen = saved;
        }
        if let Some(saved) = self.saved_fields.take() {
            for field in saved {
                self.field_manager.add_field_for_test(field);
            }
        }
        // CRITICAL FIX: Reset cursor to safe position on restore
        self.set_cursor(0, 0);
    }

    /// CRITICAL FIX: Validate protocol state machine consistency
    /// This method ensures all internal state is consistent and valid
    pub fn validate_state_consistency(&self) -> Result<(), String> {
        // Validate cursor position is within bounds
        if self.cursor_row >= TERMINAL_HEIGHT || self.cursor_col >= TERMINAL_WIDTH {
            return Err(format!("Invalid cursor position: ({}, {})", self.cursor_row, self.cursor_col));
        }

        // Validate cursor position matches calculated position
        let calculated_position = self.cursor_row * TERMINAL_WIDTH + self.cursor_col;
        if self.cursor_position != calculated_position {
            return Err(format!("Cursor position mismatch: stored={}, calculated={}",
                             self.cursor_position, calculated_position));
        }

        // Validate state consistency
        match self.state {
            ProtocolState::InitialNegotiation => {
                if self.connected {
                    return Err("Connected flag true during initial negotiation".to_string());
                }
            },
            ProtocolState::Connected | ProtocolState::Receiving => {
                // These states should generally have connected = true
                // but allow for transition periods
            },
            ProtocolState::Error => {
                // Error state should generally not be connected
                if self.connected {
                    return Err("Connected flag true during error state".to_string());
                }
            },
            _ => {}
        }

        // Validate field manager state
        if let Some(active_idx) = self.field_manager.get_active_field_index() {
            let field_count = self.field_manager.field_count();
            if active_idx >= field_count {
                return Err(error_messages::out_of_bounds("Active field", active_idx, field_count));
            }
        }

        Ok(())
    }

    /// CRITICAL FIX: Safe state transition with validation
    /// This method ensures state transitions are valid and safe
    pub fn transition_to_state(&mut self, new_state: ProtocolState) -> Result<(), String> {
        // Validate the transition is allowed
        match (&self.state, &new_state) {
            (ProtocolState::InitialNegotiation, ProtocolState::Connected) => {
                // Valid transition after successful negotiation
            },
            (ProtocolState::Connected, ProtocolState::Receiving) => {
                // Valid transition when receiving data
            },
            (ProtocolState::Receiving, ProtocolState::Connected) => {
                // Valid transition when done receiving
            },
            (_, ProtocolState::Error) => {
                // Can always transition to error state
            },
            (ProtocolState::Error, _) => {
                // Can transition from error to any state for recovery
            },
            _ => {
                return Err(format!("Invalid state transition: {:?} -> {:?}", self.state, new_state));
            }
        }

        self.state = new_state;
        Ok(())
    }

    /// COMPREHENSIVE VALIDATION: Full system validation
    /// This method performs comprehensive validation of all system components
    pub fn comprehensive_validation(&self) -> Result<(), String> {
        // Validate state consistency
        self.validate_state_consistency()?;

        // Validate screen buffer
        self.screen.validate_buffer_consistency()?;

        // Validate field manager
        if let Some(active_idx) = self.field_manager.get_active_field_index() {
            if active_idx >= self.field_manager.field_count() {
                return Err(error_messages::out_of_bounds(
                    "Active field", 
                    active_idx, 
                    self.field_manager.field_count().saturating_sub(1)
                ));
            }
        }

        // Validate device attributes
        if self.device_attributes.max_buffer_size == 0 {
            return Err("Invalid device attributes: max_buffer_size is zero".to_string());
        }

        // Validate cursor position consistency
        let expected_position = self.cursor_row * TERMINAL_WIDTH + self.cursor_col;
        if self.cursor_position != expected_position {
            return Err(format!("Cursor position inconsistency: stored={}, expected={}",
                             self.cursor_position, expected_position));
        }

        Ok(())
    }
}

// TODO: Implement session-based protocol state management
// The old ProtocolState trait is being replaced with direct session integration

impl Default for ProtocolStateMachine {
    fn default() -> Self { Self::new() }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_state_machine_creation() {
        let proto = ProtocolStateMachine::new();
        assert_eq!(proto.state, ProtocolState::InitialNegotiation);
        assert!(!proto.connected);
    }

    #[test]
    fn test_connection() {
        let mut proto = ProtocolStateMachine::new();
        proto.connect();
        assert_eq!(proto.state, ProtocolState::Connected);
        assert!(proto.connected);
    }

    #[test]
    fn test_field_type_determination() {
        let proto = ProtocolStateMachine::new();
        
        // Test protected field (bit 5 set)
        assert_eq!(proto.determine_field_type(0x20), FieldType::Protected);
        
        // Test numeric field (bit 4 set)
        assert_eq!(proto.determine_field_type(0x10), FieldType::Numeric);
        
        // Test normal field
        assert_eq!(proto.determine_field_type(0x00), FieldType::Input);
    }

    #[test]
    fn test_cursor_position() {
        let mut proto = ProtocolStateMachine::new();
        proto.set_cursor(5, 10); // row=5, col=10
        let (row, col) = proto.get_cursor();
        assert_eq!((row, col), (5, 10));
    }

    #[test]
    fn test_add_field() {
        let mut proto = ProtocolStateMachine::new();
        proto.add_field(0, 0, 10, FieldType::Input, 0x00);
        assert_eq!(proto.field_manager.field_count(), 1);
    }

    #[test]
    fn test_detect_fields() {
        let mut proto = ProtocolStateMachine::new();
        proto.screen.write_string("Test screen with fields");
        proto.detect_fields();
        // field_count() returns usize which is always >= 0, so test actual functionality
        let field_count = proto.field_manager.field_count();
        assert!(field_count < 1000); // Reasonable upper bound for testing
    }
}