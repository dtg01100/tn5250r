use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use std::env;

// Import the EBCDIC translation function and 5250 session
use tn5250r::lib5250::{ebcdic_to_ascii, Session};
use tn5250r::telnet_negotiation::TelnetNegotiator;
use tn5250r::ansi_processor::AnsiProcessor;
use tn5250r::terminal::TerminalScreen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TN5250R Enhanced Connection Test");
    
    // Parse command-line arguments for credentials
    let args: Vec<String> = env::args().collect();
    let username = args.iter()
        .position(|arg| arg == "--user")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.clone());
    let password = args.iter()
        .position(|arg| arg == "--password")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.clone());
    
    if let Some(ref user) = username {
        println!("Using credentials: user={}", user);
    } else {
        println!("No credentials provided (use --user <name> --password <pass>)");
    }
    
    println!("Connecting to as400.example.com:23...");

    // Attempt to connect
    let mut stream = TcpStream::connect("as400.example.com:23")?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    println!("✅ Successfully connected to as400.example.com:23");

    // Create 5250 session and telnet negotiator
    let mut session = Session::new();
    let mut negotiator = TelnetNegotiator::new();
    let mut ansi_processor = AnsiProcessor::new();
    let mut terminal_screen = TerminalScreen::new();
    let mut use_ansi_mode = false;
    
    // Set credentials if provided
    if let (Some(user), Some(pass)) = (username.clone(), password.clone()) {
        negotiator.set_credentials(&user, &pass);
        println!("✅ Credentials configured for authentication");
    }
    
    // Send initial telnet negotiation
    let initial_negotiation = negotiator.generate_initial_negotiation();
    println!("📤 Sending initial telnet negotiation ({} bytes): {:02x?}", 
           initial_negotiation.len(), &initial_negotiation);
    stream.write_all(&initial_negotiation)?;
    stream.flush()?;

    // Read initial telnet negotiation data from the server
    let mut buffer = vec![0u8; 4096];
    let mut negotiation_complete = false;
    let mut total_data_received = 0;
    let mut timeouts_after_negotiation = 0;
    const MAX_TIMEOUTS_AFTER_NEGOTIATION: u32 = 3;

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("❌ Connection closed by remote host");
                break;
            }
            Ok(n) => {
                total_data_received += n;
                println!("✅ Received {} bytes (total: {})", n, total_data_received);
                println!("   Raw data: {:02x?}", &buffer[..n]);

                // Process data based on negotiation state and data type
                let negotiation_response = if !negotiation_complete {
                    // Check if this looks like 5250 data (starts with valid command)
                    if n >= 6 && buffer[4] == 0x12 && buffer[5] == 0xA0 { // 5250 data stream marker
                        println!("📥 Detected 5250 data during negotiation, processing with session");
                        // Process as 5250 data
                        match session.process_integrated_data(&buffer[..n]) {
                            Ok(response) => {
                                if !response.is_empty() {
                                    println!("📤 Sending 5250 response ({} bytes)", response.len());
                                    stream.write_all(&response)?;
                                    stream.flush()?;
                                }
                            }
                            Err(e) => {
                                println!("❌ Failed to process 5250 data: {}", e);
                            }
                        }
                        Vec::new() // No telnet response
                    } else {
                        // Process as telnet negotiation
                        negotiator.process_incoming_data(&buffer[..n])
                    }
                } else {
                    // Negotiation complete, check if data is ANSI or 5250
                    
                    // Detect ANSI escape sequences (starts with ESC [)
                    let is_ansi = n >= 2 && buffer[0] == 0x1B && (buffer[1] == 0x5B || buffer[1] == 0x28);
                    
                    if is_ansi && !use_ansi_mode {
                        println!("🔄 Detected ANSI/VT100 data - switching to ANSI mode");
                        use_ansi_mode = true;
                    }
                    
                    if use_ansi_mode {
                        // Process as ANSI/VT100 data
                        println!("📟 Processing ANSI/VT100 data...");
                        ansi_processor.process_data(&buffer[..n], &mut terminal_screen);
                        
                        // Also print the raw printable text from the data to see what was sent
                        println!("📺 AS/400 Sign-On Screen - Raw Text Extract:");
                        let raw_text: String = buffer[..n]
                            .iter()
                            .filter(|&&b| b >= 32 && b <= 126)
                            .map(|&b| b as char)
                            .collect();
                        println!("{}", raw_text);
                        
                        Vec::new() // No response needed for ANSI data
                    } else {
                        // Process as 5250 data
                        match session.process_integrated_data(&buffer[..n]) {
                            Ok(response) => {
                                if !response.is_empty() {
                                    println!("📤 Sending 5250 response ({} bytes)", response.len());
                                    stream.write_all(&response)?;
                                    stream.flush()?;
                                }
                            }
                            Err(e) => {
                                println!("❌ Failed to process 5250 data: {}", e);
                            }
                        }
                        Vec::new() // No telnet response
                    }
                };
                if !negotiation_response.is_empty() {
                    println!("📤 Sending telnet response: {:02x?}", &negotiation_response);
                    stream.write_all(&negotiation_response)?;
                    stream.flush()?;
                }

                // Check if negotiation is complete
                println!("🔍 Negotiation complete check: {}", negotiator.is_negotiation_complete());
                if negotiator.is_negotiation_complete() && !negotiation_complete {
                    println!("✅ Telnet negotiation complete!");
                    negotiation_complete = true;
                    
                    // Mark session as authenticated - telnet negotiation serves as authentication
                    session.mark_telnet_negotiation_complete();

                    // CRITICAL: After authentication, client must send Query to indicate readiness
                    // RFC 1205 Section 5: Client initiates 5250 data stream with Query command
                    println!("� Sending initial 5250 Query command to indicate client ready...");
                    match session.send_initial_5250_data() {
                        Ok(protocol_data) => {
                            println!("📤 Sending initial 5250 data ({} bytes): {:02x?}",
                                   protocol_data.len(),
                                   &protocol_data[..protocol_data.len().min(64)]);
                            stream.write_all(&protocol_data)?;
                            stream.flush()?;
                            println!("✅ Initial 5250 Query sent successfully");
                            println!("📥 Now waiting for server to send display data...");
                        }
                        Err(e) => {
                            println!("❌ Failed to send initial 5250 data: {}", e);
                        }
                    }
                }

                // Continue reading if negotiation not complete
                if !negotiation_complete {
                    continue;
                }

                // After negotiation, continue reading to get 5250 data from server
                // Don't break here - keep reading!
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    println!("⏱️ Read timeout - checking if we should continue...");
                    if negotiation_complete {
                        timeouts_after_negotiation += 1;
                        if timeouts_after_negotiation >= MAX_TIMEOUTS_AFTER_NEGOTIATION {
                            println!("✅ Negotiation complete, no more data from server after {} timeouts", 
                                   timeouts_after_negotiation);
                            break;
                        }
                        println!("   Waiting for 5250 data... (timeout {}/{})", 
                               timeouts_after_negotiation, MAX_TIMEOUTS_AFTER_NEGOTIATION);
                        continue;
                    } else {
                        println!("⚠️ Negotiation timeout - server may not be responding");
                        break;
                    }
                } else {
                    println!("❌ Error reading from server: {}", e);
                    break;
                }
            }
        }
    }

    // Test reading 5250 data after handshake
    println!("\n🔄 Testing 5250 data exchange...");
    let mut test_buffer = vec![0u8; 1024];

    match stream.read(&mut test_buffer) {
        Ok(0) => println!("❌ Connection closed after handshake"),
        Ok(n) => {
            println!("✅ Received {} bytes of 5250 data:", n);

            // Display the data in both hex and ASCII
            print!("Hex: ");
            for i in 0..n.min(64) {
                print!("{:02x} ", test_buffer[i]);
                if i > 0 && (i + 1) % 16 == 0 {
                    println!();
                    print!("     ");
                }
            }
            println!();

            print!("ASCII: ");
            for i in 0..n.min(64) {
                let ch = test_buffer[i];
                if ch >= 32 && ch <= 126 {
                    print!("{}", ch as char);
                } else {
                    print!(".");
                }
            }
            println!();

            // Test EBCDIC translation on the received data
            println!("\nEBCDIC translation test:");
            print!("Translated: ");
            for i in 0..n.min(64) {
                let ascii_char = ebcdic_to_ascii(test_buffer[i]);
                if ascii_char.is_ascii_graphic() || ascii_char == ' ' {
                    print!("{}", ascii_char);
                } else {
                    print!(".");
                }
            }
            println!();
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::WouldBlock {
                println!("✅ Connection maintained - no data available (expected for idle connection)");
            } else {
                println!("❌ Error reading 5250 data: {}", e);
            }
        }
    }

    println!("\n🔌 Enhanced connection test complete");
    Ok(())
}