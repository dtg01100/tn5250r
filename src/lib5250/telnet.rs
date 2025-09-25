/// Telnet negotiation logic for 5250 protocol (lib5250 port)

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelnetError {
    InvalidCommand(u8),
    InvalidOption(u8),
    MalformedSubnegotiation,
    InvalidEnvironmentData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TelnetOption {
    Binary = 0,
    EndOfRecord = 19,
    SuppressGoAhead = 3,
    TerminalType = 24,
    NewEnviron = 39,
    // Add more as needed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalType {
    IBM5250,
    IBM5250W,
    IBM5555C01,
    IBM5555B01,
    Custom(&'static str),
}

impl TerminalType {
    pub fn as_str(&self) -> &str {
        match self {
            TerminalType::IBM5250 => "IBM-5250",
            TerminalType::IBM5250W => "IBM-5250-W",
            TerminalType::IBM5555C01 => "IBM-5555-C01",
            TerminalType::IBM5555B01 => "IBM-5555-B01",
            TerminalType::Custom(s) => s,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelnetCommand {
    Will = 251,
    Wont = 252,
    Do = 253,
    Dont = 254,
    Subnegotiation = 250,
    SubnegotiationEnd = 240,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NegotiationState {
    NotNegotiated,
    WillSent,
    DoSent,
    Enabled,
    Disabled,
}

/// Telnet negotiation state tracker
pub struct TelnetNegotiator {
    pub option_states: std::collections::HashMap<TelnetOption, NegotiationState>,
    pub terminal_type: Option<String>,
    pub environment_vars: std::collections::HashMap<String, String>,
    pub configured_terminal_type: TerminalType,
}

impl TelnetNegotiator {
    pub fn new() -> Self {
        Self::with_terminal_type(TerminalType::IBM5250)
    }

    pub fn with_terminal_type(terminal_type: TerminalType) -> Self {
        let mut option_states = std::collections::HashMap::new();
        // Initialize required options
        option_states.insert(TelnetOption::Binary, NegotiationState::NotNegotiated);
        option_states.insert(TelnetOption::EndOfRecord, NegotiationState::NotNegotiated);
        option_states.insert(TelnetOption::SuppressGoAhead, NegotiationState::NotNegotiated);

        Self {
            option_states,
            terminal_type: None,
            environment_vars: std::collections::HashMap::new(),
            configured_terminal_type: terminal_type,
        }
    }

    /// Process incoming telnet command and return response bytes
    pub fn process_command(&mut self, command: u8, option: u8) -> Result<Option<Vec<u8>>, TelnetError> {
        let telnet_option = TelnetOption::from_u8(option).ok_or(TelnetError::InvalidOption(option))?;
        let telnet_command = TelnetCommand::from_u8(command).ok_or(TelnetError::InvalidCommand(command))?;

        match telnet_command {
            TelnetCommand::Will => {
                // Server wants to enable option
                if self.should_accept_option(telnet_option) {
                    self.option_states.insert(telnet_option, NegotiationState::Enabled);
                    Ok(Some(vec![TelnetCommand::Do as u8, option]))
                } else {
                    self.option_states.insert(telnet_option, NegotiationState::Disabled);
                    Ok(Some(vec![TelnetCommand::Dont as u8, option]))
                }
            }
            TelnetCommand::Wont => {
                // Server doesn't want to enable option
                self.option_states.insert(telnet_option, NegotiationState::Disabled);
                Ok(Some(vec![TelnetCommand::Dont as u8, option]))
            }
            TelnetCommand::Do => {
                // Server wants us to enable option
                if self.should_offer_option(telnet_option) {
                    self.option_states.insert(telnet_option, NegotiationState::Enabled);
                    Ok(Some(vec![TelnetCommand::Will as u8, option]))
                } else {
                    self.option_states.insert(telnet_option, NegotiationState::Disabled);
                    Ok(Some(vec![TelnetCommand::Wont as u8, option]))
                }
            }
            TelnetCommand::Dont => {
                // Server doesn't want us to enable option
                self.option_states.insert(telnet_option, NegotiationState::Disabled);
                Ok(Some(vec![TelnetCommand::Wont as u8, option]))
            }
            _ => Ok(None),
        }
    }

    /// Process subnegotiation data
    pub fn process_subnegotiation(&mut self, option: u8, data: &[u8]) -> Result<Option<Vec<u8>>, TelnetError> {
        let telnet_option = TelnetOption::from_u8(option).ok_or(TelnetError::InvalidOption(option))?;

        match telnet_option {
            TelnetOption::TerminalType => {
                if data.is_empty() {
                    return Err(TelnetError::MalformedSubnegotiation);
                }
                match data[0] {
                    1 => {
                        // Send terminal type
                        let term_type = self.configured_terminal_type.as_str().as_bytes();
                        let mut response = vec![TelnetCommand::Subnegotiation as u8, option, 0];
                        response.extend_from_slice(term_type);
                        response.push(TelnetCommand::SubnegotiationEnd as u8);
                        self.terminal_type = Some(self.configured_terminal_type.as_str().to_string());
                        Ok(Some(response))
                    }
                    _ => Ok(None),
                }
            }
            TelnetOption::NewEnviron => {
                // Parse environment variables (simplified)
                self.parse_environment_vars(data);
                Ok(None) // No response needed for environment
            }
            _ => Ok(None),
        }
    }

    /// Check if negotiation is complete for required options
    pub fn is_negotiation_complete(&self) -> bool {
        let required = [TelnetOption::Binary, TelnetOption::EndOfRecord, TelnetOption::SuppressGoAhead];
        required.iter().all(|opt| {
            matches!(self.option_states.get(opt), Some(NegotiationState::Enabled))
        })
    }

    /// Get active options
    pub fn get_active_options(&self) -> Vec<TelnetOption> {
        self.option_states
            .iter()
            .filter(|(_, state)| matches!(state, NegotiationState::Enabled))
            .map(|(opt, _)| *opt)
            .collect()
    }

    /// Set environment variable for auto-signon
    pub fn set_environment_var(&mut self, name: &str, value: &str) {
        self.environment_vars.insert(name.to_string(), value.to_string());
    }

    /// Get environment variables for sending
    pub fn get_environment_vars(&self) -> &std::collections::HashMap<String, String> {
        &self.environment_vars
    }

    fn should_accept_option(&self, option: TelnetOption) -> bool {
        matches!(option, TelnetOption::Binary | TelnetOption::EndOfRecord | TelnetOption::SuppressGoAhead | TelnetOption::TerminalType | TelnetOption::NewEnviron)
    }

    fn should_offer_option(&self, option: TelnetOption) -> bool {
        matches!(option, TelnetOption::Binary | TelnetOption::EndOfRecord | TelnetOption::SuppressGoAhead | TelnetOption::TerminalType)
    }

    fn parse_environment_vars(&mut self, data: &[u8]) {
        // Parse NEW-ENVIRON subnegotiation data according to RFC 1572
        // Format: [IAC SB NEW-ENVIRON IS|INFO] [VAR|VALUE] name [VALUE] value ... [IAC SE]
        let mut i = 0;
        while i < data.len() {
            match data[i] {
                0 => {
                    // VAR command - variable name follows
                    i += 1;
                    if i < data.len() {
                        if let Some((name, new_i)) = self.extract_null_terminated_string(data, i) {
                            // Store variable name, next byte should be VALUE (1) or next VAR (0)
                            if new_i < data.len() && data[new_i] == 1 {
                                // VALUE command follows
                                i = new_i + 1;
                                if i < data.len() {
                                    if let Some((value, new_i)) = self.extract_null_terminated_string(data, i) {
                                        self.environment_vars.insert(name, value);
                                        i = new_i;
                                    } else {
                                        i = new_i;
                                    }
                                }
                            } else {
                                // No value provided, just set to empty string
                                self.environment_vars.insert(name, String::new());
                                i = new_i;
                            }
                        }
                    }
                }
                1 => {
                    // VALUE command - should not appear at this level in well-formed data
                    i += 1;
                }
                2 => {
                    // ESC command - escaped character follows
                    i += 2; // Skip ESC and the escaped character
                }
                _ => {
                    // Unknown command or data, skip
                    i += 1;
                }
            }
        }
    }

    fn extract_null_terminated_string(&self, data: &[u8], start: usize) -> Option<(String, usize)> {
        let mut end = start;
        while end < data.len() && data[end] != 0 {
            end += 1;
        }
        if end < data.len() {
            // Found null terminator
            let string_data = &data[start..end];
            match String::from_utf8(string_data.to_vec()) {
                Ok(s) => Some((s, end + 1)),
                Err(_) => {
                    // Invalid UTF-8, convert lossy
                    let s = String::from_utf8_lossy(string_data).to_string();
                    Some((s, end + 1))
                }
            }
        } else {
            None
        }
    }
}

impl TelnetOption {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TelnetOption::Binary),
            19 => Some(TelnetOption::EndOfRecord),
            3 => Some(TelnetOption::SuppressGoAhead),
            24 => Some(TelnetOption::TerminalType),
            39 => Some(TelnetOption::NewEnviron),
            _ => None,
        }
    }
}

impl TelnetCommand {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            251 => Some(TelnetCommand::Will),
            252 => Some(TelnetCommand::Wont),
            253 => Some(TelnetCommand::Do),
            254 => Some(TelnetCommand::Dont),
            250 => Some(TelnetCommand::Subnegotiation),
            240 => Some(TelnetCommand::SubnegotiationEnd),
            _ => None,
        }
    }
}

/// Negotiate required telnet options for 5250 protocol (legacy function)
pub fn negotiate_options(options: &[TelnetOption]) -> bool {
    let required = [TelnetOption::Binary, TelnetOption::EndOfRecord, TelnetOption::SuppressGoAhead];
    required.iter().all(|req| options.contains(req))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negotiate_all_required_options() {
        let options = [TelnetOption::Binary, TelnetOption::EndOfRecord, TelnetOption::SuppressGoAhead];
        assert!(negotiate_options(&options));
    }

    #[test]
    fn test_missing_required_option() {
        let options = [TelnetOption::Binary, TelnetOption::EndOfRecord];
        assert!(!negotiate_options(&options));
    }

    #[test]
    fn test_telnet_negotiator_creation() {
        let negotiator = TelnetNegotiator::new();
        assert!(!negotiator.is_negotiation_complete());
    }

    #[test]
    fn test_complete_negotiation() {
        let mut negotiator = TelnetNegotiator::new();

        // Process all required WILL commands
        let _ = negotiator.process_command(TelnetCommand::Will as u8, TelnetOption::Binary as u8);
        let _ = negotiator.process_command(TelnetCommand::Will as u8, TelnetOption::EndOfRecord as u8);
        let _ = negotiator.process_command(TelnetCommand::Will as u8, TelnetOption::SuppressGoAhead as u8);

        // Now negotiation should be complete
        assert!(negotiator.is_negotiation_complete());

        // Check all required options are enabled
        assert_eq!(negotiator.option_states.get(&TelnetOption::Binary), Some(&NegotiationState::Enabled));
        assert_eq!(negotiator.option_states.get(&TelnetOption::EndOfRecord), Some(&NegotiationState::Enabled));
        assert_eq!(negotiator.option_states.get(&TelnetOption::SuppressGoAhead), Some(&NegotiationState::Enabled));
    }

    #[test]
    fn test_terminal_type_subnegotiation() {
        let mut negotiator = TelnetNegotiator::new();
        let response = negotiator.process_subnegotiation(TelnetOption::TerminalType as u8, &[1]).unwrap();
        assert!(response.is_some());
        assert!(negotiator.terminal_type.is_some());
        assert_eq!(negotiator.terminal_type.as_ref().unwrap(), "IBM-5250");
    }

    #[test]
    fn test_environment_var_setting() {
        let mut negotiator = TelnetNegotiator::new();
        negotiator.set_environment_var("USER", "testuser");
        assert_eq!(negotiator.get_environment_vars().get("USER"), Some(&"testuser".to_string()));
    }

    #[test]
    fn test_terminal_type_configuration() {
        let negotiator = TelnetNegotiator::with_terminal_type(TerminalType::IBM5250W);
        assert_eq!(negotiator.configured_terminal_type, TerminalType::IBM5250W);
    }

    #[test]
    fn test_different_terminal_types() {
        let mut negotiator = TelnetNegotiator::with_terminal_type(TerminalType::IBM5555C01);
        let response = negotiator.process_subnegotiation(TelnetOption::TerminalType as u8, &[1]).unwrap();
        assert!(response.is_some());
        assert_eq!(negotiator.terminal_type.as_ref().unwrap(), "IBM-5555-C01");
    }

    #[test]
    fn test_custom_terminal_type() {
        let mut negotiator = TelnetNegotiator::with_terminal_type(TerminalType::Custom("MY-TERMINAL"));
        let response = negotiator.process_subnegotiation(TelnetOption::TerminalType as u8, &[1]).unwrap();
        assert!(response.is_some());
        assert_eq!(negotiator.terminal_type.as_ref().unwrap(), "MY-TERMINAL");
    }

    #[test]
    fn test_new_environ_parsing_rfc1572() {
        let mut negotiator = TelnetNegotiator::new();

        // Test RFC 1572 format: VAR "USER" VALUE "testuser"
        let data = &[0, b'U', b'S', b'E', b'R', 0, 1, b't', b'e', b's', b't', b'u', b's', b'e', b'r', 0];
        negotiator.parse_environment_vars(data);

        assert_eq!(negotiator.get_environment_vars().get("USER"), Some(&"testuser".to_string()));
    }

    #[test]
    fn test_new_environ_multiple_vars() {
        let mut negotiator = TelnetNegotiator::new();

        // VAR "USER" VALUE "testuser" VAR "PASSWORD" VALUE "secret"
        let data = &[
            0, b'U', b'S', b'E', b'R', 0, 1, b't', b'e', b's', b't', b'u', b's', b'e', b'r', 0,
            0, b'P', b'A', b'S', b'S', b'W', b'O', b'R', b'D', 0, 1, b's', b'e', b'c', b'r', b'e', b't', 0
        ];
        negotiator.parse_environment_vars(data);

        assert_eq!(negotiator.get_environment_vars().get("USER"), Some(&"testuser".to_string()));
        assert_eq!(negotiator.get_environment_vars().get("PASSWORD"), Some(&"secret".to_string()));
    }

    #[test]
    fn test_invalid_command_error() {
        let mut negotiator = TelnetNegotiator::new();
        let result = negotiator.process_command(255, TelnetOption::Binary as u8);
        assert!(matches!(result, Err(TelnetError::InvalidCommand(255))));
    }

    #[test]
    fn test_invalid_option_error() {
        let mut negotiator = TelnetNegotiator::new();
        let result = negotiator.process_command(TelnetCommand::Will as u8, 255);
        assert!(matches!(result, Err(TelnetError::InvalidOption(255))));
    }

    #[test]
    fn test_malformed_subnegotiation_error() {
        let mut negotiator = TelnetNegotiator::new();
        let result = negotiator.process_subnegotiation(TelnetOption::TerminalType as u8, &[]);
        assert!(matches!(result, Err(TelnetError::MalformedSubnegotiation)));
    }
}
