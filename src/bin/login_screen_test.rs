use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tn5250r::telnet_negotiation::TelnetNegotiator;
use tn5250r::lib5250::protocol::CommandCode;

fn main() -> std::io::Result<()> {
    println!("Login Screen Test - Attempting to retrieve login screens");
    
    // Test both systems
    test_login_screen("pub400.com", 23)?;
    test_login_screen("66.189.134.90", 2323)?;
    
    Ok(())
}

fn test_login_screen(host: &str, port: u16) -> std::io::Result<()> {
    println!("\n{}", "=".repeat(60));
    println!("Testing login screen for {}:{}", host, port);
    
    let mut stream = match TcpStream::connect((host, port)) {
        Ok(s) => {
            println!("✓ Connected successfully");
            s
        },
        Err(e) => {
            println!("✗ Connection failed: {}", e);
            return Ok(());
        }
    };
    
    // Set timeouts
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    
    let mut negotiator = TelnetNegotiator::new();
    
    // 1. Complete telnet negotiation
    println!("\n1. Performing telnet negotiation...");
    let initial_negotiation = negotiator.generate_initial_negotiation();
    if !initial_negotiation.is_empty() {
        stream.write_all(&initial_negotiation)?;
        stream.flush()?;
    }
    
    // Process negotiation responses
    let mut negotiation_rounds = 0;
    while negotiation_rounds < 10 {
        let mut buffer = [0u8; 1024];
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let response = negotiator.process_incoming_data(&buffer[..n]);
                if !response.is_empty() {
                    stream.write_all(&response)?;
                    stream.flush()?;
                }
                negotiation_rounds += 1;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                break;
            }
            Err(e) => return Err(e),
        }
    }
    
    println!("   ✓ Negotiation completed");
    
    // 2. Try sending various "wake-up" commands to request initial screen
    println!("\n2. Requesting initial screen...");
    
    // Try common 5250 commands that might trigger screen display
    let wake_up_commands = vec![
        // Read Modified (common initial command)
        vec![CommandCode::ReadModified as u8],
        
        // Read Buffer (request entire screen)
        vec![CommandCode::ReadBuffer as u8],
        
        // Write to Display (might trigger initial screen)
        vec![CommandCode::WriteToDisplay as u8],
        
        // Simple ENTER key equivalent
        vec![0x0D], 
        
        // Function key F1
        vec![0x31, 0x01],
        
        // Just a newline
        vec![0x0A],
        
        // Attention key (common way to start session)
        vec![0x04], // EOT (End of Transmission)
        
        // Query command to get device capabilities
        vec![0x00, 0x01, 0x03, 0x00], // Basic query
    ];
    
    for (i, command) in wake_up_commands.iter().enumerate() {
        println!("   Trying wake-up command {}: {:02X?}", i + 1, command);
        
        // Send command
        stream.write_all(command)?;
        stream.flush()?;
        
        // Wait for response
        std::thread::sleep(Duration::from_millis(500));
        
        let mut buffer = [0u8; 2048];
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("     → Connection closed");
                break;
            }
            Ok(n) => {
                println!("     → Received {} bytes", n);
                
                // Filter out telnet negotiation to see actual data
                let clean_data = filter_telnet_data(&buffer[..n]);
                
                if !clean_data.is_empty() {
                    println!("     → Clean data ({} bytes): {:02X?}", clean_data.len(), &clean_data[..clean_data.len().min(20)]);
                    
                    // Try to interpret as EBCDIC and ASCII
                    print!("     → EBCDIC interpretation: ");
                    for &byte in &clean_data[..clean_data.len().min(50)] {
                        // Basic EBCDIC to ASCII conversion for common characters
                        let ascii_char = match byte {
                            0x40 => ' ',  // EBCDIC space
                            0x81..=0x89 => (byte - 0x81 + b'a') as char,  // a-i
                            0x91..=0x99 => (byte - 0x91 + b'j') as char,  // j-r
                            0xA2..=0xA9 => (byte - 0xA2 + b's') as char,  // s-z
                            0xC1..=0xC9 => (byte - 0xC1 + b'A') as char,  // A-I
                            0xD1..=0xD9 => (byte - 0xD1 + b'J') as char,  // J-R
                            0xE2..=0xE9 => (byte - 0xE2 + b'S') as char,  // S-Z
                            0xF0..=0xF9 => (byte - 0xF0 + b'0') as char,  // 0-9
                            _ => '.',
                        };
                        print!("{}", ascii_char);
                    }
                    println!();
                    
                    print!("     → ASCII interpretation: ");
                    for &byte in &clean_data[..clean_data.len().min(50)] {
                        if byte >= 32 && byte <= 126 {
                            print!("{}", byte as char);
                        } else {
                            print!(".");
                        }
                    }
                    println!();
                    
                    // If we got significant data, this might be our login screen
                    if clean_data.len() > 50 {
                        println!("     🎉 Potentially found login screen data!");
                        show_formatted_screen(&clean_data);
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                println!("     → Timeout (no response)");
            }
            Err(e) => {
                println!("     → Error: {}", e);
            }
        }
    }
    
    println!("\n3. Closing connection...");
    drop(stream);
    
    Ok(())
}

fn filter_telnet_data(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        if data[i] == 255 { // IAC
            if i + 1 < data.len() {
                match data[i + 1] {
                    250 => { // SB (subnegotiation) - skip to SE
                        let mut j = i + 2;
                        while j + 1 < data.len() {
                            if data[j] == 255 && data[j + 1] == 240 { // IAC SE
                                i = j + 2;
                                break;
                            }
                            j += 1;
                        }
                        if j + 1 >= data.len() {
                            break;
                        }
                        continue;
                    },
                    251..=254 => { // WILL, WONT, DO, DONT
                        if i + 2 < data.len() {
                            i += 3;
                            continue;
                        }
                    },
                    255 => { // Escaped IAC
                        result.push(255);
                        i += 2;
                        continue;
                    },
                    _ => {
                        i += 2;
                        continue;
                    }
                }
            }
        }
        
        result.push(data[i]);
        i += 1;
    }
    
    result
}

fn show_formatted_screen(data: &[u8]) {
    println!("\n📺 FORMATTED SCREEN (80x24):");
    println!("{}", "─".repeat(82));
    
    for row in 0..24 {
        print!("│");
        for col in 0..80 {
            let index = row * 80 + col;
            if index < data.len() {
                let byte = data[index];
                // Basic EBCDIC to ASCII for display
                let ascii_char = match byte {
                    0x40 => ' ',
                    0x81..=0x89 => (byte - 0x81 + b'a') as char,
                    0x91..=0x99 => (byte - 0x91 + b'j') as char,
                    0xA2..=0xA9 => (byte - 0xA2 + b's') as char,
                    0xC1..=0xC9 => (byte - 0xC1 + b'A') as char,
                    0xD1..=0xD9 => (byte - 0xD1 + b'J') as char,
                    0xE2..=0xE9 => (byte - 0xE2 + b'S') as char,
                    0xF0..=0xF9 => (byte - 0xF0 + b'0') as char,
                    _ if byte >= 32 && byte <= 126 => byte as char,
                    _ => '·',
                };
                print!("{}", ascii_char);
            } else {
                print!(" ");
            }
        }
        println!("│");
    }
    
    println!("{}", "─".repeat(82));
}