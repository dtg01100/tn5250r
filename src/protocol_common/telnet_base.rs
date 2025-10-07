//! Common Telnet protocol functionality for TN5250 and TN3270
//!
//! This module provides shared telnet protocol handling that both TN5250
//! and TN3270 can use, including option negotiation, command processing,
//! and state management.

/// Telnet command codes (RFC 854)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelnetCommand {
    /// Interpret As Command - 255 (0xFF)
    IAC = 255,
    /// Don't - 254 (0xFE)
    DONT = 254,
    /// Do - 253 (0xFD)
    DO = 253,
    /// Won't - 252 (0xFC)
    WONT = 252,
    /// Will - 251 (0xFB)
    WILL = 251,
    /// Subnegotiation Begin - 250 (0xFA)
    SB = 250,
    /// Go Ahead - 249 (0xF9)
    GA = 249,
    /// Erase Line - 248 (0xF8)
    EL = 248,
    /// Erase Character - 247 (0xF7)
    EC = 247,
    /// Are You There - 246 (0xF6)
    AYT = 246,
    /// Abort Output - 245 (0xF5)
    AO = 245,
    /// Interrupt Process - 244 (0xF4)
    IP = 244,
    /// Break - 243 (0xF3)
    BRK = 243,
    /// Data Mark - 242 (0xF2)
    DM = 242,
    /// No Operation - 241 (0xF1)
    NOP = 241,
    /// Subnegotiation End - 240 (0xF0)
    SE = 240,
}

impl TelnetCommand {
    /// Convert a byte to a TelnetCommand
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            255 => Some(TelnetCommand::IAC),
            254 => Some(TelnetCommand::DONT),
            253 => Some(TelnetCommand::DO),
            252 => Some(TelnetCommand::WONT),
            251 => Some(TelnetCommand::WILL),
            250 => Some(TelnetCommand::SB),
            249 => Some(TelnetCommand::GA),
            248 => Some(TelnetCommand::EL),
            247 => Some(TelnetCommand::EC),
            246 => Some(TelnetCommand::AYT),
            245 => Some(TelnetCommand::AO),
            244 => Some(TelnetCommand::IP),
            243 => Some(TelnetCommand::BRK),
            242 => Some(TelnetCommand::DM),
            241 => Some(TelnetCommand::NOP),
            240 => Some(TelnetCommand::SE),
            _ => None,
        }
    }

    /// Check if a byte is a telnet command
    pub fn is_command(byte: u8) -> bool {
        byte >= 240
    }
}

/// Common Telnet options (RFC 855 and extensions)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelnetOption {
    /// Binary Transmission - 0
    Binary = 0,
    /// Echo - 1
    Echo = 1,
    /// Suppress Go Ahead - 3
    SuppressGoAhead = 3,
    /// Status - 5
    Status = 5,
    /// Timing Mark - 6
    TimingMark = 6,
    /// Terminal Type - 24
    TerminalType = 24,
    /// End of Record - 25
    EndOfRecord = 25,
    /// Negotiate About Window Size - 31
    NAWS = 31,
    /// Terminal Speed - 32
    TerminalSpeed = 32,
    /// Remote Flow Control - 33
    RemoteFlowControl = 33,
    /// Linemode - 34
    Linemode = 34,
    /// Environment Variables - 36
    EnvironmentVariables = 36,
    /// New Environment - 39
    NewEnvironment = 39,
    /// TN3270 Regime - 40
    TN3270Regime = 40,
}

impl TelnetOption {
    /// Convert a byte to a TelnetOption
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TelnetOption::Binary),
            1 => Some(TelnetOption::Echo),
            3 => Some(TelnetOption::SuppressGoAhead),
            5 => Some(TelnetOption::Status),
            6 => Some(TelnetOption::TimingMark),
            24 => Some(TelnetOption::TerminalType),
            25 => Some(TelnetOption::EndOfRecord),
            31 => Some(TelnetOption::NAWS),
            32 => Some(TelnetOption::TerminalSpeed),
            33 => Some(TelnetOption::RemoteFlowControl),
            34 => Some(TelnetOption::Linemode),
            36 => Some(TelnetOption::EnvironmentVariables),
            39 => Some(TelnetOption::NewEnvironment),
            40 => Some(TelnetOption::TN3270Regime),
            _ => None,
        }
    }

    /// Get the option name as a string
    pub fn name(&self) -> &str {
        match self {
            TelnetOption::Binary => "Binary",
            TelnetOption::Echo => "Echo",
            TelnetOption::SuppressGoAhead => "Suppress Go Ahead",
            TelnetOption::Status => "Status",
            TelnetOption::TimingMark => "Timing Mark",
            TelnetOption::TerminalType => "Terminal Type",
            TelnetOption::EndOfRecord => "End of Record",
            TelnetOption::NAWS => "Window Size",
            TelnetOption::TerminalSpeed => "Terminal Speed",
            TelnetOption::RemoteFlowControl => "Remote Flow Control",
            TelnetOption::Linemode => "Linemode",
            TelnetOption::EnvironmentVariables => "Environment Variables",
            TelnetOption::NewEnvironment => "New Environment",
            TelnetOption::TN3270Regime => "TN3270 Regime",
        }
    }
}

/// Telnet negotiation state for an option
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegotiationState {
    /// Option is disabled
    Disabled,
    /// We want to enable the option
    Wanting,
    /// Option is enabled
    Enabled,
    /// We are disabling the option
    Disabling,
}

impl Default for NegotiationState {
    fn default() -> Self {
        NegotiationState::Disabled
    }
}

/// Telnet option negotiation tracker
#[derive(Debug, Clone, Default)]
pub struct OptionState {
    /// Local state (our side)
    pub local: NegotiationState,
    /// Remote state (their side)
    pub remote: NegotiationState,
}

impl OptionState {
    /// Create a new option state with both sides disabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the option is fully enabled on both sides
    pub fn is_enabled(&self) -> bool {
        self.local == NegotiationState::Enabled && self.remote == NegotiationState::Enabled
    }
}

/// Tuple type for parsed telnet command info
pub type TelnetCommandInfo = (TelnetCommand, Option<u8>, Option<Vec<u8>>);

/// Build a telnet negotiation sequence
///
/// # Arguments
///
/// * `command` - The telnet command (WILL, WONT, DO, DONT)
/// * `option` - The option code
///
/// # Returns
///
/// A vector containing the IAC command sequence
///
/// # Examples
///
/// ```
/// use tn5250r::protocol_common::telnet_base::{build_negotiation, TelnetCommand};
///
/// // Build "IAC WILL BINARY"
/// let seq = build_negotiation(TelnetCommand::WILL, 0);
/// assert_eq!(seq, vec![255, 251, 0]);
/// ```
pub fn build_negotiation(command: TelnetCommand, option: u8) -> Vec<u8> {
    vec![TelnetCommand::IAC as u8, command as u8, option]
}

/// Build a telnet subnegotiation sequence
///
/// # Arguments
///
/// * `option` - The option code
/// * `data` - The subnegotiation data
///
/// # Returns
///
/// A vector containing the complete subnegotiation sequence
///
/// # Examples
///
/// ```
/// use tn5250r::protocol_common::telnet_base::build_subnegotiation;
///
/// // Build "IAC SB TERMINAL-TYPE IS IBM-3278-2 IAC SE"
/// let data = b"IBM-3278-2";
/// let seq = build_subnegotiation(24, data);
/// ```
pub fn build_subnegotiation(option: u8, data: &[u8]) -> Vec<u8> {
    let mut result = vec![TelnetCommand::IAC as u8, TelnetCommand::SB as u8, option];
    
    // Escape any IAC bytes in the data
    for &byte in data {
        result.push(byte);
        if byte == TelnetCommand::IAC as u8 {
            result.push(TelnetCommand::IAC as u8); // Double IAC for escaping
        }
    }
    
    result.push(TelnetCommand::IAC as u8);
    result.push(TelnetCommand::SE as u8);
    result
}

/// Parse telnet commands from a data stream
///
/// This function extracts telnet commands and separates them from regular data.
///
/// # Arguments
///
/// * `data` - The raw data stream
///
/// # Returns
///
/// A tuple of (regular_data, commands) where commands is a vector of
/// (command, option, subnegotiation_data) tuples
pub fn parse_telnet_stream(data: &[u8]) -> (Vec<u8>, Vec<TelnetCommandInfo>) {
    let mut regular_data = Vec::new();
    let mut commands = Vec::new();
    let mut i = 0;

    while i < data.len() {
        if data[i] == TelnetCommand::IAC as u8 {
            if i + 1 < data.len() {
                match TelnetCommand::from_u8(data[i + 1]) {
                    Some(TelnetCommand::IAC) => {
                        // Escaped IAC, treat as regular data
                        regular_data.push(TelnetCommand::IAC as u8);
                        i += 2;
                    }
                    Some(cmd @ (TelnetCommand::WILL | TelnetCommand::WONT | 
                               TelnetCommand::DO | TelnetCommand::DONT)) => {
                        // Option negotiation
                        if i + 2 < data.len() {
                            let option = data[i + 2];
                            commands.push((cmd, Some(option), None));
                            i += 3;
                        } else {
                            i += 2;
                        }
                    }
                    Some(TelnetCommand::SB) => {
                        // Subnegotiation
                        if i + 2 < data.len() {
                            let option = data[i + 2];
                            let mut sb_data = Vec::new();
                            let mut j = i + 3;
                            
                            // Find IAC SE
                            while j + 1 < data.len() {
                                if data[j] == TelnetCommand::IAC as u8 {
                                    if data[j + 1] == TelnetCommand::SE as u8 {
                                        commands.push((TelnetCommand::SB, Some(option), Some(sb_data)));
                                        i = j + 2;
                                        break;
                                    } else if data[j + 1] == TelnetCommand::IAC as u8 {
                                        // Escaped IAC in subnegotiation
                                        sb_data.push(TelnetCommand::IAC as u8);
                                        j += 2;
                                    } else {
                                        sb_data.push(data[j]);
                                        j += 1;
                                    }
                                } else {
                                    sb_data.push(data[j]);
                                    j += 1;
                                }
                            }
                            if j >= data.len() {
                                // Incomplete subnegotiation
                                i = data.len();
                            }
                        } else {
                            i += 2;
                        }
                    }
                    Some(cmd) => {
                        // Other commands without options
                        commands.push((cmd, None, None));
                        i += 2;
                    }
                    None => {
                        // Unknown command, skip
                        i += 2;
                    }
                }
            } else {
                i += 1;
            }
        } else {
            regular_data.push(data[i]);
            i += 1;
        }
    }

    (regular_data, commands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telnet_command_conversion() {
        assert_eq!(TelnetCommand::from_u8(255), Some(TelnetCommand::IAC));
        assert_eq!(TelnetCommand::from_u8(251), Some(TelnetCommand::WILL));
        assert_eq!(TelnetCommand::from_u8(253), Some(TelnetCommand::DO));
        assert!(TelnetCommand::is_command(255));
        assert!(!TelnetCommand::is_command(100));
    }

    #[test]
    fn test_telnet_option_conversion() {
        assert_eq!(TelnetOption::from_u8(0), Some(TelnetOption::Binary));
        assert_eq!(TelnetOption::from_u8(24), Some(TelnetOption::TerminalType));
        assert_eq!(TelnetOption::Binary.name(), "Binary");
    }

    #[test]
    fn test_build_negotiation() {
        let seq = build_negotiation(TelnetCommand::WILL, 0);
        assert_eq!(seq, vec![255, 251, 0]);
    }

    #[test]
    fn test_build_subnegotiation() {
        let data = b"TEST";
        let seq = build_subnegotiation(24, data);
        assert_eq!(seq[0], 255); // IAC
        assert_eq!(seq[1], 250); // SB
        assert_eq!(seq[2], 24);  // Option
        assert_eq!(&seq[3..7], b"TEST");
        assert_eq!(seq[7], 255); // IAC
        assert_eq!(seq[8], 240); // SE
    }

    #[test]
    fn test_parse_telnet_stream() {
        let data = vec![
            255, 251, 0,  // IAC WILL BINARY
            b'H', b'e', b'l', b'l', b'o',
            255, 253, 3,  // IAC DO SUPPRESS-GO-AHEAD
        ];
        
        let (regular, commands) = parse_telnet_stream(&data);
        assert_eq!(regular, b"Hello");
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].0, TelnetCommand::WILL);
        assert_eq!(commands[0].1, Some(0));
        assert_eq!(commands[1].0, TelnetCommand::DO);
        assert_eq!(commands[1].1, Some(3));
    }

    #[test]
    fn test_option_state() {
        let mut state = OptionState::new();
        assert!(!state.is_enabled());
        
        state.local = NegotiationState::Enabled;
        state.remote = NegotiationState::Enabled;
        assert!(state.is_enabled());
    }
}