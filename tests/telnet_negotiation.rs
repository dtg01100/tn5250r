use tn5250r::telnet_negotiation::{TelnetNegotiator, TelnetOption, TelnetCommand, NegotiationState};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_negotiation_rfc_2877_compliant() {
        let mut negotiator = TelnetNegotiator::new();
        let initial = negotiator.generate_initial_negotiation();
        
        // Should not be empty
        assert!(!initial.is_empty());
        
        // Parse the generated negotiation commands
        let mut pos = 0;
        let mut found_binary_do = false;
        let mut found_binary_will = false;
        let mut found_eor_do = false;
        let mut found_eor_will = false;
        let mut found_sga_do = false;
        let mut found_sga_will = false;
        let mut found_terminal_type_will = false;
        
        const BINARY_OPT: u8 = TelnetOption::Binary as u8;
        const EOR_OPT: u8 = TelnetOption::EndOfRecord as u8;
        const SGA_OPT: u8 = TelnetOption::SuppressGoAhead as u8;
        const TTYPE_OPT: u8 = TelnetOption::TerminalType as u8;
        const DO_CMD: u8 = TelnetCommand::DO as u8;
        const WILL_CMD: u8 = TelnetCommand::WILL as u8;
        
        while pos + 2 < initial.len() {
            if initial[pos] == TelnetCommand::IAC as u8 {
                let command = initial[pos + 1];
                let option = initial[pos + 2];
                
                match (command, option) {
                    (DO_CMD, BINARY_OPT) => {
                        found_binary_do = true;
                    }
                    (WILL_CMD, BINARY_OPT) => {
                        found_binary_will = true;
                    }
                    (DO_CMD, EOR_OPT) => {
                        found_eor_do = true;
                    }
                    (WILL_CMD, EOR_OPT) => {
                        found_eor_will = true;
                    }
                    (DO_CMD, SGA_OPT) => {
                        found_sga_do = true;
                    }
                    (WILL_CMD, SGA_OPT) => {
                        found_sga_will = true;
                    }
                    (WILL_CMD, TTYPE_OPT) => {
                        found_terminal_type_will = true;
                    }
                    _ => {}
                }
                
                pos += 3;
            } else {
                pos += 1;
            }
        }
        
        // Verify RFC 2877 required options are requested
        assert!(found_binary_do, "Should request server to enable BINARY");
        assert!(found_binary_will, "Should offer to enable BINARY");
        assert!(found_eor_do, "Should request server to enable EOR");
        assert!(found_eor_will, "Should offer to enable EOR");
        assert!(found_sga_do, "Should request server to enable SGA");
        assert!(found_sga_will, "Should offer to enable SGA");
        assert!(found_terminal_type_will, "Should offer to provide terminal type");
    }

    #[test]
    fn test_do_command_handling() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Simulate server sending DO BINARY
        let do_binary = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::DO as u8,
            TelnetOption::Binary as u8,
        ];
        
        let response = negotiator.process_incoming_data(&do_binary);
        
        // Should respond with WILL BINARY
        assert_eq!(response.len(), 3);
        assert_eq!(response[0], TelnetCommand::IAC as u8);
        assert_eq!(response[1], TelnetCommand::WILL as u8);
        assert_eq!(response[2], TelnetOption::Binary as u8);
        
        // Check state
        assert!(negotiator.is_option_active(TelnetOption::Binary));
    }

    #[test]
    fn test_will_command_handling() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Simulate server sending WILL BINARY
        let will_binary = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::WILL as u8,
            TelnetOption::Binary as u8,
        ];
        
        let response = negotiator.process_incoming_data(&will_binary);
        
        // Should respond with DO BINARY
        assert_eq!(response.len(), 3);
        assert_eq!(response[0], TelnetCommand::IAC as u8);
        assert_eq!(response[1], TelnetCommand::DO as u8);
        assert_eq!(response[2], TelnetOption::Binary as u8);
        
        // Check state
        assert!(negotiator.is_option_active(TelnetOption::Binary));
    }

    #[test]
    fn test_negotiation_completion_detection() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Initially not complete
        assert!(!negotiator.is_negotiation_complete());
        
        // First generate initial negotiation to set up attempted states
        negotiator.generate_initial_negotiation();
        
        // Simulate successful negotiation of all required options
        let negotiations = vec![
            // Server agrees to BINARY
            vec![TelnetCommand::IAC as u8, TelnetCommand::WILL as u8, TelnetOption::Binary as u8],
            vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::Binary as u8],
            // Server agrees to EOR
            vec![TelnetCommand::IAC as u8, TelnetCommand::WILL as u8, TelnetOption::EndOfRecord as u8],
            vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::EndOfRecord as u8],
            // Server agrees to SGA
            vec![TelnetCommand::IAC as u8, TelnetCommand::WILL as u8, TelnetOption::SuppressGoAhead as u8],
            vec![TelnetCommand::IAC as u8, TelnetCommand::DO as u8, TelnetOption::SuppressGoAhead as u8],
        ];
        
        for negotiation in negotiations {
            negotiator.process_incoming_data(&negotiation);
        }
        
        // Should now be complete (essential options are negotiated)
        assert!(negotiator.is_negotiation_complete());
    }

    #[test]
    fn test_terminal_type_subnegotiation() {
        let mut negotiator = TelnetNegotiator::new();
        
        // First establish TERMINAL-TYPE option
        let will_ttype = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::DO as u8,
            TelnetOption::TerminalType as u8,
        ];
        negotiator.process_incoming_data(&will_ttype);
        
        // Simulate server requesting terminal type
        let request_ttype = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::SB as u8,
            TelnetOption::TerminalType as u8,
            1, // SEND command
            TelnetCommand::IAC as u8,
            TelnetCommand::SE as u8,
        ];
        
        // Process the request - this should return the terminal type response
        let response = negotiator.process_incoming_data(&request_ttype);
        
        // Should respond with terminal type
        assert!(!response.is_empty(), "No response generated");
        
        // Debug: print the response to understand its structure
        println!("Response length: {}", response.len());
        println!("Response bytes: {:?}", response);
        
        // Look for terminal type string in the response
        let response_str = String::from_utf8_lossy(&response);
        assert!(response_str.contains("IBM-3179-2"), 
               "Terminal type response should contain IBM-3179-2. Got: {}", response_str);
    }

    #[test]
    fn test_negotiation_loop_prevention() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Establish BINARY option
        let do_binary = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::DO as u8,
            TelnetOption::Binary as u8,
        ];
        
        let response1 = negotiator.process_incoming_data(&do_binary);
        assert!(!response1.is_empty()); // Should respond with WILL
        
        // Send the same DO command again
        let response2 = negotiator.process_incoming_data(&do_binary);
        assert!(response2.is_empty()); // Should not respond again (prevent loop)
    }

    #[test]
    fn test_forced_negotiation_completion() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Generate initial negotiation to put options in attempted state
        negotiator.generate_initial_negotiation();
        
        // Initially not complete
        assert!(!negotiator.is_negotiation_complete());
        
        // Force completion should work since we attempted essential options
        negotiator.force_negotiation_complete();
        assert!(negotiator.is_negotiation_complete());
    }

    #[test]
    fn test_negotiation_status_reporting() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Get initial status
        let status = negotiator.get_negotiation_state_details();
        assert!(!status.is_empty());
        
        // All should initially be in Initial state
        for (_option, state) in &status {
            assert_eq!(*state, NegotiationState::Initial);
        }
        
        // After generating initial negotiation, states should change
        negotiator.generate_initial_negotiation();
        let status = negotiator.get_negotiation_state_details();
        
        // Some options should now be in RequestedDo or RequestedWill state
        let has_requested = status.iter().any(|(_, state)| {
            matches!(state, NegotiationState::RequestedDo | NegotiationState::RequestedWill)
        });
        assert!(has_requested);
    }
}