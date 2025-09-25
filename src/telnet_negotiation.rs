//! Telnet Option Negotiation for RFC 2877 compliance
//! 
//! This module handles the Telnet option negotiation required for proper 5250 protocol
//! communication with IBM AS/400 systems.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TelnetOption {
    Binary = 0,
    Echo = 1,
    SuppressGoAhead = 3,
    EndOfRecord = 19,
    TerminalType = 24,
    NewEnvironment = 39,
}

impl TelnetOption {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TelnetOption::Binary),
            1 => Some(TelnetOption::Echo),
            3 => Some(TelnetOption::SuppressGoAhead),
            19 => Some(TelnetOption::EndOfRecord),
            24 => Some(TelnetOption::TerminalType),
            39 => Some(TelnetOption::NewEnvironment),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TelnetCommand {
    WILL = 251,
    WONT = 252, 
    DO = 253,
    DONT = 254,
    IAC = 255,  // Interpret As Command
    SB = 250,   // Subnegotiation Begin
    SE = 240,   // Subnegotiation End
}

impl TelnetCommand {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            251 => Some(TelnetCommand::WILL),
            252 => Some(TelnetCommand::WONT),
            253 => Some(TelnetCommand::DO),
            254 => Some(TelnetCommand::DONT),
            255 => Some(TelnetCommand::IAC),
            250 => Some(TelnetCommand::SB),
            240 => Some(TelnetCommand::SE),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NegotiationState {
    /// We haven't sent any request for this option
    Initial,
    /// We sent DO request, waiting for WILL/WONT
    RequestedDo,
    /// We sent DONT request, waiting for WONT
    RequestedDont,
    /// We sent WILL request, waiting for DO/DONT
    RequestedWill,
    /// We sent WONT request, waiting for DONT
    RequestedWont,
    /// Both sides agree we will do this option
    Active,
    /// Both sides agree we won't do this option
    Inactive,
}

#[derive(Debug)]
pub struct TelnetNegotiator {
    /// Current state of each telnet option
    negotiation_states: HashMap<TelnetOption, NegotiationState>,
    
    /// What options we are willing to negotiate
    preferred_options: Vec<TelnetOption>,
    
    /// Buffer for processing incoming data
    input_buffer: Vec<u8>,
    
    /// Pending response to send
    output_buffer: Vec<u8>,
    
    /// Whether negotiation is complete
    negotiation_complete: bool,
}

impl TelnetNegotiator {
    pub fn new() -> Self {
        let mut negotiator = Self {
            negotiation_states: HashMap::new(),
            preferred_options: vec![
                TelnetOption::Binary,
                TelnetOption::EndOfRecord,
                TelnetOption::SuppressGoAhead,
                TelnetOption::TerminalType,
                TelnetOption::NewEnvironment,
            ],
            input_buffer: Vec::new(),
            output_buffer: Vec::new(),
            negotiation_complete: false,
        };
        
        // Initialize all options to Initial state
        for &option in &negotiator.preferred_options {
            negotiator.negotiation_states.insert(option, NegotiationState::Initial);
        }
        
        negotiator
    }
    
    /// Process incoming telnet data and generate appropriate responses
    pub fn process_incoming_data(&mut self, data: &[u8]) -> Vec<u8> {
        self.input_buffer.extend_from_slice(data);
        self.output_buffer.clear();

        let mut pos = 0;
        let buffer_len = self.input_buffer.len();

        while pos < buffer_len {
            // Use slice operations for better performance
            let remaining = &self.input_buffer[pos..];

            if !remaining.is_empty() && remaining[0] == TelnetCommand::IAC as u8 {
                // Check if we have enough bytes for a complete command
                if remaining.len() >= 2 {
                    let command = remaining[1];

                    if let Some(cmd) = TelnetCommand::from_u8(command) {
                        match cmd {
                            TelnetCommand::DO | TelnetCommand::DONT | TelnetCommand::WILL | TelnetCommand::WONT => {
                                // These commands need an option byte
                                if remaining.len() >= 3 {
                                    if let Some(option) = TelnetOption::from_u8(remaining[2]) {
                                        match cmd {
                                            TelnetCommand::DO => self.handle_do_command(option),
                                            TelnetCommand::DONT => self.handle_dont_command(option),
                                            TelnetCommand::WILL => self.handle_will_command(option),
                                            TelnetCommand::WONT => self.handle_wont_command(option),
                                            _ => {} // Unreachable
                                        }
                                        pos += 3; // Skip IAC + command + option
                                        continue;
                                    }
                                }
                            },
                            TelnetCommand::SB => {
                                // Handle subnegotiation - find the end more efficiently
                                let sb_start = pos + 2;
                                if let Some(end_pos) = self.find_subnegotiation_end(sb_start) {
                                    let sub_data = self.input_buffer[sb_start..end_pos].to_vec();
                                    self.handle_subnegotiation(&sub_data);
                                    pos = end_pos + 2; // Skip to after SE
                                    continue;
                                }
                            },
                            _ => {
                                // Not a negotiation command, treat as data
                            }
                        }
                    }
                }
            }

            pos += 1;
        }

        // Remove processed data from input buffer
        if pos > 0 {
            self.input_buffer.drain(0..pos);
        }

        // Check if all required negotiations are complete
        self.check_negotiation_complete();

        self.output_buffer.clone()
    }
    
    /// Generate initial negotiation request
    pub fn generate_initial_negotiation(&mut self) -> Vec<u8> {
        let mut negotiation = Vec::new();
        
        // For each preferred option, send a DO request
        for &option in &self.preferred_options {
            if matches!(self.negotiation_states.get(&option), 
                       Some(NegotiationState::Initial)) {
                self.negotiation_states.insert(option, NegotiationState::RequestedDo);
                
                negotiation.extend_from_slice(&[
                    TelnetCommand::IAC as u8,
                    TelnetCommand::DO as u8,
                    option as u8
                ]);
            }
        }
        
        negotiation
    }
    
    /// Check if negotiation is complete
    pub fn is_negotiation_complete(&self) -> bool {
        self.negotiation_complete
    }
    
    /// Check if a specific option is active
    pub fn is_option_active(&self, option: TelnetOption) -> bool {
        matches!(self.negotiation_states.get(&option), 
                 Some(NegotiationState::Active))
    }
    
    /// Handle incoming DO command
    fn handle_do_command(&mut self, option: TelnetOption) {
        let current_state = *self.negotiation_states.get(&option).unwrap_or(&NegotiationState::Initial);
        match current_state {
            NegotiationState::RequestedWont | NegotiationState::RequestedDont => {
                // We requested it shouldn't be done, but they want us to do it
                // We'll agree to WILL if we prefer this option
                if self.preferred_options.contains(&option) {
                    self.negotiation_states.insert(option, NegotiationState::Active);
                    self.send_will(option);
                } else {
                    self.negotiation_states.insert(option, NegotiationState::Inactive);
                    self.send_wont(option);
                }
            },
            NegotiationState::Initial => {
                // They want us to do something we haven't asked for
                // If we prefer this option, WILL it, otherwise WONT it
                if self.preferred_options.contains(&option) {
                    self.negotiation_states.insert(option, NegotiationState::Active);
                    self.send_will(option);
                } else {
                    self.negotiation_states.insert(option, NegotiationState::Inactive);
                    self.send_wont(option);
                }
            },
            NegotiationState::Active | NegotiationState::Inactive => {
                // Already decided, just acknowledge
            },
            NegotiationState::RequestedWill => {
                // We asked to WILL and they responded with DO, so we're active
                self.negotiation_states.insert(option, NegotiationState::Active);
            },
            NegotiationState::RequestedDo => {
                // We asked to DO and they responded with DO, send WILL
                self.negotiation_states.insert(option, NegotiationState::Active);
                self.send_will(option);
            }
        }
    }
    
    /// Handle incoming DONT command
    fn handle_dont_command(&mut self, option: TelnetOption) {
        let current_state = *self.negotiation_states.get(&option).unwrap_or(&NegotiationState::Initial);
        match current_state {
            NegotiationState::Active | NegotiationState::RequestedWill => {
                // They don't want us to do this option
                self.negotiation_states.insert(option, NegotiationState::Inactive);
                self.send_wont(option);
            },
            NegotiationState::RequestedDo | NegotiationState::Initial => {
                // They don't want to do it, we don't want to do it - fine
                self.negotiation_states.insert(option, NegotiationState::Inactive);
            },
            NegotiationState::RequestedWont | NegotiationState::RequestedDont => {
                // We already agreed to not do it
                self.negotiation_states.insert(option, NegotiationState::Inactive);
            },
            NegotiationState::Inactive => {
                // Already inactive, just acknowledge
            }
        }
    }
    
    /// Handle incoming WILL command
    fn handle_will_command(&mut self, option: TelnetOption) {
        let current_state = *self.negotiation_states.get(&option).unwrap_or(&NegotiationState::Initial);
        match current_state {
            NegotiationState::RequestedDont | NegotiationState::RequestedWont => {
                // We asked them not to WILL but they did anyway
                // If we really don't want it, send DONT
                if !self.preferred_options.contains(&option) {
                    self.negotiation_states.insert(option, NegotiationState::Inactive);
                    self.send_dont(option);
                } else {
                    self.negotiation_states.insert(option, NegotiationState::Active);
                }
            },
            NegotiationState::Initial => {
                // They want to WILL something
                if self.preferred_options.contains(&option) {
                    self.negotiation_states.insert(option, NegotiationState::Active);
                    self.send_do(option);
                } else {
                    self.negotiation_states.insert(option, NegotiationState::Inactive);
                    self.send_dont(option);
                }
            },
            NegotiationState::RequestedDo => {
                // We asked them to DO and they WILL'd, so we need to DO back to be active
                self.negotiation_states.insert(option, NegotiationState::Active);
                self.send_do(option);
            },
            NegotiationState::Active | NegotiationState::Inactive => {
                // Already decided, just acknowledge appropriately
            },
            _ => {
                // Handle any other states
            }
        }
    }
    
    /// Handle incoming WONT command
    fn handle_wont_command(&mut self, option: TelnetOption) {
        let current_state = *self.negotiation_states.get(&option).unwrap_or(&NegotiationState::Initial);
        match current_state {
            NegotiationState::Active | NegotiationState::RequestedDo => {
                // They don't want to WILL this
                self.negotiation_states.insert(option, NegotiationState::Inactive);
                self.send_dont(option);
            },
            NegotiationState::RequestedWont | NegotiationState::Initial => {
                // They don't want to WILL something we didn't ask for
                self.negotiation_states.insert(option, NegotiationState::Inactive);
            },
            NegotiationState::RequestedWill => {
                // We asked them to WILL and they responded with WONT
                self.negotiation_states.insert(option, NegotiationState::Inactive);
            },
            NegotiationState::RequestedDont => {
                // We asked them to DONT and they responded with WONT
                self.negotiation_states.insert(option, NegotiationState::Inactive);
            },
            NegotiationState::Inactive => {
                // Already inactive, just acknowledge
            }
        }
    }
    
    /// Handle subnegotiation (like terminal type or environment variables)
    fn handle_subnegotiation(&mut self, data: &[u8]) {
        if data.len() < 2 {
            return;
        }
        
        if let Some(option) = TelnetOption::from_u8(data[0]) {
            match option {
                TelnetOption::TerminalType => {
                    // Handle terminal type negotiation
                    if data.len() > 2 && data[1] == 1 { // SEND terminal type
                        // Respond with our terminal type
                        self.send_terminal_type_response();
                    }
                },
                TelnetOption::NewEnvironment => {
                    // Handle environment variable negotiation
                    self.handle_environment_negotiation(&data[1..]);
                },
                _ => {
                    // Other subnegotiations not yet implemented
                }
            }
        }
    }
    
    /// Handle environment variable negotiation
    fn handle_environment_negotiation(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        
        let sub_command = data[0];
        match sub_command {
            1 => { // SEND command
                // Send our environment variables
                self.send_environment_variables();
            },
            2 => { // IS command - process their variables
                // Process the environment variables they sent
            },
            _ => {
                // Other sub-commands not yet implemented
            }
        }
    }
    
    /// Send a WILL command
    fn send_will(&mut self, option: TelnetOption) {
        self.output_buffer.extend_from_slice(&[
            TelnetCommand::IAC as u8,
            TelnetCommand::WILL as u8,
            option as u8
        ]);
    }
    
    /// Send a WONT command
    fn send_wont(&mut self, option: TelnetOption) {
        self.output_buffer.extend_from_slice(&[
            TelnetCommand::IAC as u8,
            TelnetCommand::WONT as u8,
            option as u8
        ]);
    }
    
    /// Send a DO command
    fn send_do(&mut self, option: TelnetOption) {
        self.output_buffer.extend_from_slice(&[
            TelnetCommand::IAC as u8,
            TelnetCommand::DO as u8,
            option as u8
        ]);
    }
    
    /// Send a DONT command
    fn send_dont(&mut self, option: TelnetOption) {
        self.output_buffer.extend_from_slice(&[
            TelnetCommand::IAC as u8,
            TelnetCommand::DONT as u8,
            option as u8
        ]);
    }
    
    /// Send terminal type response
    fn send_terminal_type_response(&mut self) {
        // Send terminal type: IBM-5555-C01 (for example)
        let response: Vec<u8> = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::SB as u8,
            TelnetOption::TerminalType as u8,
            0, // IS command
            b'I', b'B', b'M', b'-', b'5', b'5', b'5', b'5', b'-', b'C', b'0', b'1',
            TelnetCommand::IAC as u8,
            TelnetCommand::SE as u8,
        ];
        self.output_buffer.extend_from_slice(&response);
    }
    
    /// Send environment variables
    fn send_environment_variables(&mut self) {
        // For now, send minimal environment variables
        // In a real implementation, these would be configurable
        let response: Vec<u8> = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::SB as u8,
            TelnetOption::NewEnvironment as u8,
            2, // IS command
            1, // VAR type
            b'D', b'E', b'V', b'N', b'A', b'M', b'E', 0, // DEVNAME variable
            b'T', b'N', b'5', b'2', b'5', b'0', b'R', 0, // TN5250R value
            TelnetCommand::IAC as u8,
            TelnetCommand::SE as u8,
        ];
        self.output_buffer.extend_from_slice(&response);
    }
    
    /// Find the end of a subnegotiation (SE marker)
    fn find_subnegotiation_end(&self, start: usize) -> Option<usize> {
        let mut i = start;
        while i < self.input_buffer.len() {
            if self.input_buffer[i] == TelnetCommand::IAC as u8 {
                if i + 1 < self.input_buffer.len() && 
                   self.input_buffer[i + 1] == TelnetCommand::SE as u8 {
                    return Some(i + 2); // Position after SE
                }
            }
            i += 1;
        }
        None
    }
    
    /// Check if all required negotiations are complete
    fn check_negotiation_complete(&mut self) {
        // For now, consider negotiation complete when essential options are handled
        // In a real implementation, this would check all required options
        let essential_active = [
            TelnetOption::Binary,
            TelnetOption::EndOfRecord, 
            TelnetOption::SuppressGoAhead
        ];
        
        let all_essential_active = essential_active.iter().all(|&opt| {
            matches!(self.negotiation_states.get(&opt), Some(NegotiationState::Active))
        });
        
        if all_essential_active {
            self.negotiation_complete = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telnet_option_from_u8() {
        assert_eq!(TelnetOption::from_u8(0), Some(TelnetOption::Binary));
        assert_eq!(TelnetOption::from_u8(19), Some(TelnetOption::EndOfRecord));
        assert_eq!(TelnetOption::from_u8(99), None);
    }

    #[test]
    fn test_telnet_command_from_u8() {
        assert_eq!(TelnetCommand::from_u8(255), Some(TelnetCommand::IAC));
        assert_eq!(TelnetCommand::from_u8(251), Some(TelnetCommand::WILL));
        assert_eq!(TelnetCommand::from_u8(99), None);
    }

    #[test]
    fn test_negotiator_creation() {
        let negotiator = TelnetNegotiator::new();
        assert_eq!(negotiator.negotiation_states.len(), 5); // 5 preferred options
        assert_eq!(negotiator.is_negotiation_complete(), false);
    }

    #[test]
    fn test_initial_negotiation() {
        let mut negotiator = TelnetNegotiator::new();
        let init_data = negotiator.generate_initial_negotiation();
        
        // Should contain IAC DO commands for preferred options
        assert!(init_data.len() >= 15); // At least 5 options * 3 bytes each
        
        // Check for the pattern of IAC DO opt
        let mut i = 0;
        while i < init_data.len() {
            if i + 2 < init_data.len() && 
               init_data[i] == TelnetCommand::IAC as u8 &&
               init_data[i + 1] == TelnetCommand::DO as u8 {
                // Found a DO command
                i += 3;
            } else {
                i += 1;
            }
        }
    }
}