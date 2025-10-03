use tn5250r::lib5250::session::Session;
use tn5250r::lib5250::codes::*;

const ESC: u8 = 0x04;

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

#[test]
fn session_write_structured_field_erase_reset_clear_to_null() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create Erase/Reset structured field with clear to null (0x00)
    let mut data = Vec::new();
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x07]); // Length (7 bytes total)
    data.push(0xD9); // Class
    data.push(0x5B); // Erase/Reset SF type
    data.push(0x00); // Clear to null
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Erase/Reset should return no response
    assert!(resp.is_empty());
    
    // Check that session state was reset
    assert_eq!(session.read_opcode, 0);
    assert!(!session.invited);
}

#[test]
fn session_write_structured_field_erase_reset_clear_to_blanks() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create Erase/Reset structured field with clear to blanks (0x01)
    let mut data = Vec::new();
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x07]); // Length (7 bytes total)
    data.push(0xD9); // Class
    data.push(0x5B); // Erase/Reset SF type
    data.push(0x01); // Clear to blanks
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Erase/Reset should return no response
    assert!(resp.is_empty());
}

#[test]
fn session_write_structured_field_define_pending_operations() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create Define Pending Operations structured field
    let mut data = Vec::new();
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x08]); // Length (8 bytes total)
    data.push(0xD9); // Class
    data.push(0x80); // Define Pending Operations SF type
    data.push(0x01); // Some operation data
    data.push(0x02); // More operation data
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Define Pending Operations should return no response
    assert!(resp.is_empty());
}

#[test]
fn session_write_structured_field_enable_command_recognition() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create Enable Command Recognition structured field
    let mut data = Vec::new();
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x07]); // Length (7 bytes total)
    data.push(0xD9); // Class
    data.push(0x82); // Enable Command Recognition SF type
    data.push(0x0F); // Recognition flags
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Enable Command Recognition should return no response
    assert!(resp.is_empty());
}

#[test]
fn session_write_structured_field_request_timestamp_interval() {
    let mut session = Session::new();
    session.authenticate("testuser", "testpass").unwrap();
    
    // Create Request Minimum Timestamp Interval structured field
    let mut data = Vec::new();
    data.push(ESC);
    data.push(CMD_WRITE_STRUCTURED_FIELD);
    data.extend_from_slice(&[0x00, 0x08]); // Length (8 bytes total)
    data.push(0xD9); // Class
    data.push(0x83); // Request Minimum Timestamp Interval SF type
    data.extend_from_slice(&[0x00, 0x64]); // 100ms interval (big-endian)
    
    let resp = session.process_stream(&data).expect("process ok");
    
    // Request Minimum Timestamp Interval should return no response
    assert!(resp.is_empty());
}
