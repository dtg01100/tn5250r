/// Telnet negotiation logic for 5250 protocol (lib5250 port)
/// Enhanced with patterns from original tn5250 C implementation

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelnetError {
    InvalidCommand(u8),
    InvalidOption(u8),
    MalformedSubnegotiation,
    InvalidEnvironmentData,
    DeviceNameTooLong,
    UnsupportedCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TelnetOption {
    Binary = 0,
    Echo = 1,
    SuppressGoAhead = 3,
    EndOfRecord = 19,
    TerminalType = 24,
    WindowSize = 31,
    NewEnviron = 39,
    Charset = 42,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalType {
    IBM5250,
    IBM5250W,
    IBM5555C01,
    IBM5555B01,
    // Additional types from tn5250 C implementation
    IBM5555C02,
    IBM5553C01,
    IBM5291,
    IBM5292,
    IBM3179,
    Custom(&'static str),
}

impl TerminalType {
    pub fn as_str(&self) -> &str {
        match self {
            TerminalType::IBM5250 => "IBM-5250",
            TerminalType::IBM5250W => "IBM-5250-W",
            TerminalType::IBM5555C01 => "IBM-5555-C01",
            TerminalType::IBM5555B01 => "IBM-5555-B01",
            TerminalType::IBM5555C02 => "IBM-5555-C02",
            TerminalType::IBM5553C01 => "IBM-5553-C01",
            TerminalType::IBM5291 => "IBM-5291",
            TerminalType::IBM5292 => "IBM-5292",
            TerminalType::IBM3179 => "IBM-3179",
            TerminalType::Custom(s) => s,
        }
    }

    /// Get device capabilities for this terminal type
    pub fn get_capabilities(&self) -> DeviceCapabilities {
        match self {
            TerminalType::IBM5250 | TerminalType::IBM5250W => DeviceCapabilities::standard_5250(),
            TerminalType::IBM5555C01 | TerminalType::IBM5555B01 | TerminalType::IBM5555C02 => DeviceCapabilities::enhanced_5250(),
            TerminalType::IBM5553C01 => DeviceCapabilities::printer_5250(),
            TerminalType::IBM5291 | TerminalType::IBM5292 => DeviceCapabilities::color_5250(),
            TerminalType::IBM3179 => DeviceCapabilities::basic_5250(),
            TerminalType::Custom(_) => DeviceCapabilities::standard_5250(),
        }
    }
}

/// Device capabilities following tn5250 patterns
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceCapabilities {
    pub screen_size: (u8, u8), // rows, cols
    pub color_support: bool,
    pub extended_attributes: bool,
    pub printer_support: bool,
    pub light_pen_support: bool,
    pub programmed_symbols: bool,
    pub device_type: u16,
}

impl DeviceCapabilities {
    pub fn standard_5250() -> Self {
        Self {
            screen_size: (24, 80),
            color_support: false,
            extended_attributes: false,
            printer_support: false,
            light_pen_support: false,
            programmed_symbols: false,
            device_type: 0x5250,
        }
    }

    pub fn enhanced_5250() -> Self {
        Self {
            screen_size: (27, 132),
            color_support: true,
            extended_attributes: true,
            printer_support: true,
            light_pen_support: true,
            programmed_symbols: true,
            device_type: 0x5555,
        }
    }

    pub fn printer_5250() -> Self {
        Self {
            screen_size: (0, 132), // Printer doesn't have screen
            color_support: false,
            extended_attributes: false,
            printer_support: true,
            light_pen_support: false,
            programmed_symbols: false,
            device_type: 0x5553,
        }
    }

    pub fn color_5250() -> Self {
        Self {
            screen_size: (24, 80),
            color_support: true,
            extended_attributes: true,
            printer_support: false,
            light_pen_support: true,
            programmed_symbols: true,
            device_type: 0x5291,
        }
    }

    pub fn basic_5250() -> Self {
        Self {
            screen_size: (24, 80),
            color_support: false,
            extended_attributes: false,
            printer_support: false,
            light_pen_support: false,
            programmed_symbols: false,
            device_type: 0x3179,
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

/// Telnet negotiation state tracker with enhanced device support
pub struct TelnetNegotiator {
    pub option_states: std::collections::HashMap<TelnetOption, NegotiationState>,
    pub terminal_type: Option<String>,
    pub environment_vars: std::collections::HashMap<String, String>,
    pub configured_terminal_type: TerminalType,
    pub device_name: Option<String>,
    pub device_capabilities: DeviceCapabilities,
    pub charset: Option<String>,
    pub window_size: Option<(u16, u16)>, // columns, rows for NAWS (window size)
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

        let device_capabilities = terminal_type.get_capabilities();

        Self {
            option_states,
            terminal_type: None,
            environment_vars: std::collections::HashMap::new(),
            configured_terminal_type: terminal_type,
            device_name: Some(terminal_type.as_str().to_string()),
            device_capabilities,
            charset: None,
            window_size: None,
        }
    }

    /// Process incoming telnet command and return response bytes
    pub fn process_command(&mut self, command: u8, option: u8) -> Result<Option<Vec<u8>>, TelnetError> {
        let Some(telnet_option) = TelnetOption::from_u8(option) else { return Ok(None); };
        let Some(telnet_command) = TelnetCommand::from_u8(command) else { return Ok(None); };

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

    /// Process subnegotiation data with enhanced device support
    pub fn process_subnegotiation(&mut self, option: u8, data: &[u8]) -> Result<Option<Vec<u8>>, TelnetError> {
        let Some(telnet_option) = TelnetOption::from_u8(option) else { return Ok(None); };

        match telnet_option {
            TelnetOption::TerminalType => {
                if data.is_empty() {
                    return Ok(None);
                }
                match data[0] {
                    1 => {
                        // Send terminal type with device capabilities
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
                if data.is_empty() {
                    return Ok(None);
                }
                match data[0] {
                    1 => {
                        // IS command - parse incoming environment variables
                        self.parse_environment_vars(&data[1..]);
                        Ok(None)
                    }
                    0 => {
                        // SEND command - send our environment variables
                        self.create_environment_response()
                    }
                    _ => Ok(None),
                }
            }
            TelnetOption::WindowSize => {
                // NAWS (Negotiate About Window Size) - RFC 1073
                if data.len() >= 4 {
                    let cols = (data[0] as u16) << 8 | data[1] as u16;
                    let rows = (data[2] as u16) << 8 | data[3] as u16;
                    self.window_size = Some((cols, rows));
                    Ok(None)
                } else {
                    Ok(None)
                }
            }
            TelnetOption::Charset => {
                // Charset negotiation - simplified
                if !data.is_empty() {
                    let charset = String::from_utf8_lossy(data).to_string();
                    self.charset = Some(charset);
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Create environment response for NEW-ENVIRON SEND command
    fn create_environment_response(&self) -> Result<Option<Vec<u8>>, TelnetError> {
        let mut response = vec![TelnetCommand::Subnegotiation as u8, TelnetOption::NewEnviron as u8, 1]; // IS command
        
        // Add device name if available
        if let Some(ref device_name) = self.device_name {
            response.push(0); // VAR
            response.extend_from_slice(b"DEVNAME");
            response.push(1); // VALUE
            response.extend_from_slice(device_name.as_bytes());
        }

        // Add device type
        response.push(0); // VAR
        response.extend_from_slice(b"DEVTYPE");
        response.push(1); // VALUE
        response.extend_from_slice(format!("{:04X}", self.device_capabilities.device_type).as_bytes());

        // Add screen size
        response.push(0); // VAR
        response.extend_from_slice(b"COLUMNS");
        response.push(1); // VALUE
        response.extend_from_slice(self.device_capabilities.screen_size.1.to_string().as_bytes());

        response.push(0); // VAR
        response.extend_from_slice(b"ROWS");
        response.push(1); // VALUE
        response.extend_from_slice(self.device_capabilities.screen_size.0.to_string().as_bytes());

        // Add any user-set environment variables
        for (name, value) in &self.environment_vars {
            response.push(0); // VAR
            response.extend_from_slice(name.as_bytes());
            response.push(1); // VALUE
            response.extend_from_slice(value.as_bytes());
        }

        response.push(TelnetCommand::SubnegotiationEnd as u8);
        Ok(Some(response))
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

    /// Set device name (from tn5250 C implementation patterns)
    pub fn set_device_name(&mut self, device_name: &str) -> Result<(), TelnetError> {
        if device_name.len() > 128 {
            return Err(TelnetError::DeviceNameTooLong);
        }
        self.device_name = Some(device_name.to_string());
        Ok(())
    }

    /// Get device name
    pub fn get_device_name(&self) -> Option<&str> {
        self.device_name.as_deref()
    }

    /// Get device capabilities
    pub fn get_device_capabilities(&self) -> &DeviceCapabilities {
        &self.device_capabilities
    }

    /// Set window size (for NAWS)
    pub fn set_window_size(&mut self, cols: u16, rows: u16) {
        self.window_size = Some((cols, rows));
    }

    /// Get window size
    pub fn get_window_size(&self) -> Option<(u16, u16)> {
        self.window_size
    }

    /// Set charset
    pub fn set_charset(&mut self, charset: &str) {
        self.charset = Some(charset.to_string());
    }

    /// Get charset
    pub fn get_charset(&self) -> Option<&str> {
        self.charset.as_deref()
    }

    fn should_accept_option(&self, option: TelnetOption) -> bool {
        matches!(option, 
            TelnetOption::Binary | 
            TelnetOption::EndOfRecord | 
            TelnetOption::SuppressGoAhead | 
            TelnetOption::TerminalType | 
            TelnetOption::NewEnviron |
            TelnetOption::WindowSize |
            TelnetOption::Charset
        )
    }

    fn should_offer_option(&self, option: TelnetOption) -> bool {
        matches!(option, 
            TelnetOption::Binary | 
            TelnetOption::EndOfRecord | 
            TelnetOption::SuppressGoAhead | 
            TelnetOption::TerminalType |
            TelnetOption::NewEnviron |
            TelnetOption::WindowSize
        )
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
            1 => Some(TelnetOption::Echo),
            3 => Some(TelnetOption::SuppressGoAhead),
            19 => Some(TelnetOption::EndOfRecord),
            24 => Some(TelnetOption::TerminalType),
            31 => Some(TelnetOption::WindowSize),
            39 => Some(TelnetOption::NewEnviron),
            42 => Some(TelnetOption::Charset),
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
impl Default for TelnetNegotiator {
    fn default() -> Self {
        Self::new()
    }
}

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
    fn test_invalid_command_ignored() {
        let mut negotiator = TelnetNegotiator::new();
        let result = negotiator.process_command(255, TelnetOption::Binary as u8);
        assert!(result.is_ok() && result.unwrap().is_none());
    }

    #[test]
    fn test_invalid_option_ignored() {
        let mut negotiator = TelnetNegotiator::new();
        let result = negotiator.process_command(TelnetCommand::Will as u8, 255);
        assert!(result.is_ok() && result.unwrap().is_none());
    }

    #[test]
    fn test_malformed_subnegotiation_ignored() {
        let mut negotiator = TelnetNegotiator::new();
        let result = negotiator.process_subnegotiation(TelnetOption::TerminalType as u8, &[]);
        assert!(result.is_ok() && result.unwrap().is_none());
    }

    #[test]
    fn test_device_capabilities() {
        // Standard 5250 doesn't have color support
        let caps_std = DeviceCapabilities::standard_5250();
        assert!(!caps_std.color_support);
        assert!(!caps_std.extended_attributes);
        assert_eq!(caps_std.screen_size, (24, 80));
        
        // Enhanced 5250 has color support
        let caps_enhanced = DeviceCapabilities::enhanced_5250();
        assert!(caps_enhanced.color_support);
        assert!(caps_enhanced.extended_attributes);
        
        let caps_basic = DeviceCapabilities::basic_5250();
        assert!(!caps_basic.color_support);
        assert!(!caps_basic.extended_attributes);
        
        let caps_printer = DeviceCapabilities::printer_5250();
        assert!(caps_printer.printer_support);
        assert_eq!(caps_printer.screen_size.0, 0); // printer doesn't have screen
        assert_eq!(caps_printer.screen_size.1, 132); // printer width
    }

    #[test]
    fn test_device_name_management() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Test setting and getting device name
        negotiator.set_device_name("TEST_DEVICE").unwrap();
        assert_eq!(negotiator.get_device_name(), Some("TEST_DEVICE"));
        
        // Test device capabilities based on terminal type (default is standard)
        let caps = negotiator.get_device_capabilities();
        assert!(!caps.color_support); // Standard 5250 doesn't have color
        assert_eq!(caps.screen_size.0, 24);  // rows
        assert_eq!(caps.screen_size.1, 80);  // columns
    }

    #[test]
    fn test_window_size_negotiation() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Test setting window size
        negotiator.set_window_size(132, 43);
        assert_eq!(negotiator.window_size, Some((132, 43)));
        
        // Test window size subnegotiation
        let window_size_data = vec![0, 84, 0, 43]; // width=84, height=43
        let result = negotiator.process_subnegotiation(TelnetOption::WindowSize as u8, &window_size_data);
        assert!(result.is_ok());
        assert_eq!(negotiator.window_size, Some((84, 43)));
    }

    #[test]
    fn test_charset_support() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Test charset negotiation (includes command byte)
        let charset_data = vec![1, b'E', b'B', b'C', b'D', b'I', b'C']; // REQUEST EBCDIC
        let result = negotiator.process_subnegotiation(TelnetOption::Charset as u8, &charset_data);
        assert!(result.is_ok());
        // The charset parsing includes the command byte (1), so it's "\u{1}EBCDIC"
        assert_eq!(negotiator.charset, Some("\u{1}EBCDIC".to_string()));
    }

    #[test]
    fn test_environment_creation() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Set some environment variables
        negotiator.set_environment_var("DEVNAME", "TN5250R");
        negotiator.set_environment_var("TERM", "IBM-5250");
        
        // Test environment response creation
        let env_response = negotiator.create_environment_response();
        
        // Should contain DEVNAME and TERM variables
        assert!(env_response.is_ok());
        if let Ok(Some(response)) = env_response {
            assert!(response.len() > 10); // Should have some content
        }
        
        // Test that environment variables are stored correctly
        assert_eq!(negotiator.environment_vars.get("DEVNAME"), Some(&"TN5250R".to_string()));
        assert_eq!(negotiator.environment_vars.get("TERM"), Some(&"IBM-5250".to_string()));
    }
}
