use tn5250r::telnet_negotiation::{TelnetNegotiator, TelnetCommand, TelnetOption, EnvironmentType, TerminalType};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_variable_management() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Test default environment variables
        assert!(negotiator.get_environment_variable("USER").is_some());
        assert!(negotiator.get_environment_variable("DEVNAME").is_some());
        assert_eq!(negotiator.get_environment_variable("USER").unwrap(), "GUEST");
        assert_eq!(negotiator.get_environment_variable("DEVNAME").unwrap(), "TN5250R");
        
        // Test setting new environment variable
        negotiator.set_environment_variable("NEWVAR".to_string(), "TESTVALUE".to_string());
        assert_eq!(negotiator.get_environment_variable("NEWVAR").unwrap(), "TESTVALUE");
        
        // Test getting all environment variables
        let all_vars = negotiator.get_all_environment_variables();
        assert!(all_vars.contains_key("USER"));
        assert!(all_vars.contains_key("DEVNAME"));
        assert!(all_vars.contains_key("NEWVAR"));
    }

    #[test]
    fn test_terminal_type_cycling() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Check initial terminal type
        let initial_type = negotiator.get_current_terminal_type();
        assert_eq!(*initial_type, TerminalType::Ibm3179_2);
        
        // Test getting supported terminal types
        let supported_types = negotiator.get_supported_terminal_types();
        assert_eq!(supported_types.len(), 3);
        assert!(supported_types.contains(&TerminalType::Ibm3179_2));
        assert!(supported_types.contains(&TerminalType::Ibm3180_2));
        assert!(supported_types.contains(&TerminalType::Ibm5555C01));
        
        // Test cycling through terminal types by sending requests
        let request_ttype = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::SB as u8,
            TelnetOption::TerminalType as u8,
            1, // SEND command
            TelnetCommand::IAC as u8,
            TelnetCommand::SE as u8,
        ];
        
        // First request should return IBM-3179-2
        let response1 = negotiator.process_incoming_data(&request_ttype);
        assert!(!response1.is_empty());
        let response1_str = String::from_utf8_lossy(&response1);
        assert!(response1_str.contains("IBM-3179-2"));
        
        // Second request should return IBM-3180-2
        let response2 = negotiator.process_incoming_data(&request_ttype);
        assert!(!response2.is_empty());
        let response2_str = String::from_utf8_lossy(&response2);
        assert!(response2_str.contains("IBM-3180-2"));
        
        // Third request should return IBM-5555-C01
        let response3 = negotiator.process_incoming_data(&request_ttype);
        assert!(!response3.is_empty());
        let response3_str = String::from_utf8_lossy(&response3);
        assert!(response3_str.contains("IBM-5555-C01"));
        
        // Fourth request should cycle back to IBM-3179-2
        let response4 = negotiator.process_incoming_data(&request_ttype);
        assert!(!response4.is_empty());
        let response4_str = String::from_utf8_lossy(&response4);
        assert!(response4_str.contains("IBM-3179-2"));
    }

    #[test]
    fn test_terminal_type_reset() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Cycle through some terminal types
        let request_ttype = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::SB as u8,
            TelnetOption::TerminalType as u8,
            1, // SEND command
            TelnetCommand::IAC as u8,
            TelnetCommand::SE as u8,
        ];
        
        negotiator.process_incoming_data(&request_ttype); // First cycle
        negotiator.process_incoming_data(&request_ttype); // Second cycle
        
        // Current should be at index 2 (IBM-5555-C01)
        assert_eq!(*negotiator.get_current_terminal_type(), TerminalType::Ibm5555C01);
        
        // Reset cycling
        negotiator.reset_terminal_type_cycling();
        assert_eq!(*negotiator.get_current_terminal_type(), TerminalType::Ibm3179_2);
    }

    #[test]
    fn test_environment_variable_negotiation_rfc1572() {
        let mut negotiator = TelnetNegotiator::new();
        
        // First establish NEW-ENVIRON option
        let will_environ = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::DO as u8,
            TelnetOption::NewEnvironment as u8,
        ];
        negotiator.process_incoming_data(&will_environ);
        
        // Simulate server requesting specific environment variables
        let mut request_vars = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::SB as u8,
            TelnetOption::NewEnvironment as u8,
            1, // SEND command
        ];
        
        // Request USER variable (VAR type)
        request_vars.push(EnvironmentType::Var as u8);
        request_vars.extend_from_slice(b"USER");
        
        // Request DEVNAME variable (USERVAR type)
        request_vars.push(EnvironmentType::UserVar as u8);
        request_vars.extend_from_slice(b"DEVNAME");
        
        request_vars.extend_from_slice(&[
            TelnetCommand::IAC as u8,
            TelnetCommand::SE as u8,
        ]);
        
        // Process the request
        let response = negotiator.process_incoming_data(&request_vars);
        assert!(!response.is_empty());
        
        // Should contain both USER and DEVNAME in the response
        let response_str = String::from_utf8_lossy(&response);
        println!("Environment response: {:?}", response);
        
        // Check that response contains our variables
        // Note: The response is in binary format, so we check for the presence of the values
        assert!(response.windows(5).any(|window| window == b"GUEST"));
        assert!(response.windows(7).any(|window| window == b"TN5250R"));
    }

    #[test]
    fn test_environment_variable_comprehensive_send() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Add some custom environment variables
        negotiator.set_environment_variable("PRINTER".to_string(), "PRT02".to_string());
        negotiator.set_environment_variable("SYSTEMTYPE".to_string(), "LINUX".to_string());
        
        // Simulate server requesting all environment variables
        let request_all_vars = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::SB as u8,
            TelnetOption::NewEnvironment as u8,
            1, // SEND command (no specific variables = send all)
            TelnetCommand::IAC as u8,
            TelnetCommand::SE as u8,
        ];
        
        let response = negotiator.process_incoming_data(&request_all_vars);
        assert!(!response.is_empty());
        
        // Should contain standard variables and user variables
        let response_str = String::from_utf8_lossy(&response);
        println!("Comprehensive response length: {}", response.len());
        
        // Check for presence of some key variables
        assert!(response.windows(5).any(|window| window == b"GUEST")); // USER value
        assert!(response.windows(7).any(|window| window == b"TN5250R")); // DEVNAME value
        assert!(response.windows(5).any(|window| window == b"PRT02")); // PRINTER value
    }

    #[test]
    fn test_terminal_type_string_conversion() {
        assert_eq!(TerminalType::Ibm3179_2.as_str(), "IBM-3179-2");
        assert_eq!(TerminalType::Ibm3180_2.as_str(), "IBM-3180-2");
        assert_eq!(TerminalType::Ibm5555C01.as_str(), "IBM-5555-C01");
        
        assert_eq!(TerminalType::Ibm3179_2.as_bytes(), b"IBM-3179-2");
        assert_eq!(TerminalType::Ibm3180_2.as_bytes(), b"IBM-3180-2");
        assert_eq!(TerminalType::Ibm5555C01.as_bytes(), b"IBM-5555-C01");
    }

    #[test]
    fn test_add_terminal_type() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Initial count should be 3
        assert_eq!(negotiator.get_supported_terminal_types().len(), 3);
        
        // Add a new terminal type
        negotiator.add_terminal_type(TerminalType::Ibm3180_2); // Already exists - should not add
        assert_eq!(negotiator.get_supported_terminal_types().len(), 3);
        
        // Add a duplicate should not increase count
        negotiator.add_terminal_type(TerminalType::Ibm3179_2);
        assert_eq!(negotiator.get_supported_terminal_types().len(), 3);
    }

    #[test]
    fn test_environment_type_constants() {
        assert_eq!(EnvironmentType::Var as u8, 0);
        assert_eq!(EnvironmentType::UserVar as u8, 3);
        assert_eq!(EnvironmentType::Value as u8, 1);
        assert_eq!(EnvironmentType::Esc as u8, 2);
    }

    #[test]
    fn test_malformed_environment_negotiation() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Test with malformed environment data (incomplete variable name)
        let malformed_request = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::SB as u8,
            TelnetOption::NewEnvironment as u8,
            1, // SEND command
            EnvironmentType::Var as u8,
            // Missing variable name and SE
        ];
        
        // Should not crash, should handle gracefully
        let response = negotiator.process_incoming_data(&malformed_request);
        // Response might be empty or contain default variables
        println!("Malformed request response length: {}", response.len());
    }

    #[test] 
    fn test_error_handling_short_subnegotiation() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Test very short subnegotiation that should not crash
        let short_sub = vec![
            TelnetCommand::IAC as u8,
            TelnetCommand::SB as u8,
            TelnetOption::TerminalType as u8,
            TelnetCommand::IAC as u8,
            TelnetCommand::SE as u8,
        ];
        
        let response = negotiator.process_incoming_data(&short_sub);
        // Should handle gracefully without crashing
        println!("Short subnegotiation response length: {}", response.len());
    }
}