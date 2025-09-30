use tn5250r::lib5250::session::Session;
use tn5250r::lib5250::codes::*;

#[test]
fn session_write_structured_field_5250_query() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create proper 5250 data stream: ESC + CMD_WRITE_STRUCTURED_FIELD + structured field data
    let mut data = Vec::new();
    data.push(ESC);  // Command escape
    data.push(CMD_WRITE_STRUCTURED_FIELD);  // Write Structured Field command
    data.extend_from_slice(&[0x00, 0x06]); // Length (6 bytes total)
    data.push(0xD9); // Class
    data.push(SF_5250_QUERY); // SF type (0x70)
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Should return query reply response
    assert!(!resp.is_empty());
    // Response should contain query reply data
    assert!(resp.len() > 1);
}

#[test]
fn session_write_structured_field_query_command() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create proper 5250 data stream with QueryCommand (0x84)
    let mut data = Vec::new();
    data.push(ESC);  // Command escape
    data.push(CMD_WRITE_STRUCTURED_FIELD);  // Write Structured Field command
    data.extend_from_slice(&[0x00, 0x06]); // Length (6 bytes total)
    data.push(0xD9); // Class
    data.push(SF_QUERY_COMMAND); // SF type (0x84)
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Should return SetReplyMode response (0x85)
    assert!(!resp.is_empty());
    assert_eq!(resp[0], SF_SET_REPLY_MODE); // First byte should be 0x85
    
    // Should contain basic device capability data
    assert!(resp.len() > 1);
    // Check for display capabilities (80 columns, 24 rows)
    assert!(resp.len() >= 7); // At least 7 bytes total
}

#[test]
fn session_write_structured_field_unknown_sf() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create proper 5250 data stream with unknown SF type
    let mut data = Vec::new();
    data.push(ESC);  // Command escape
    data.push(CMD_WRITE_STRUCTURED_FIELD);  // Write Structured Field command  
    data.extend_from_slice(&[0x00, 0x06]); // Length (6 bytes total)
    data.push(0xD9); // Class
    data.push(0xFF); // Unknown SF type
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Should return empty response for unknown SF
    assert!(resp.is_empty());
}

#[test]
fn session_write_structured_field_invalid_class() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create proper 5250 data stream with invalid class
    let mut data = Vec::new();
    data.push(ESC);  // Command escape
    data.push(CMD_WRITE_STRUCTURED_FIELD);  // Write Structured Field command
    data.extend_from_slice(&[0x00, 0x06]); // Length (6 bytes total)
    data.push(0xC0); // Invalid class (not 0xD9)
    data.push(SF_5250_QUERY); // SF type
    
    let result = session.process_stream(&data);
    
    // Should return error for invalid class
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid SF class"));
}

#[test]
fn session_handles_multiple_commands() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create data stream with multiple commands
    let mut data = Vec::new();
    
    // First command: QueryCommand
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x06]);
    data.push(0xD9);
    data.push(SF_QUERY_COMMAND);
    
    // Second command: 5250 Query  
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x06]);
    data.push(0xD9);
    data.push(SF_5250_QUERY);
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Should process both commands and return responses
    assert!(!resp.is_empty());
}
