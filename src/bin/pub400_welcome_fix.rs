use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tn5250r::telnet_negotiation::TelnetNegotiator;
use tn5250r::protocol_state::ebcdic_to_ascii;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏠 TN5250R pub400.com Welcome Screen Fix");
    println!("========================================");
    
    let mut stream = TcpStream::connect("pub400.com:23")?;
    stream.set_read_timeout(Some(Duration::from_secs(15)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    
    println!("✅ Connected to pub400.com:23");
    
    let mut negotiator = TelnetNegotiator::new();
    let mut buffer = [0; 4096];
    
    // Phase 1: Complete telnet negotiation
    println!("\n🤝 Phase 1: Telnet Negotiation");
    for round in 1..=3 {
        println!("Round {}:", round);
        
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                println!("  📥 Received {} bytes", bytes_read);
                let response = negotiator.process_incoming_data(&buffer[0..bytes_read]);
                
                if !response.is_empty() {
                    println!("  📤 Sending {} response bytes", response.len());
                    stream.write_all(&response)?;
                    stream.flush()?;
                }
            },
            Ok(_) => {
                println!("  Connection closed");
                break;
            },
            Err(e) => {
                println!("  Error: {}", e);
                if round >= 3 { break; }
            }
        }
        
        std::thread::sleep(Duration::from_millis(500));
    }
    
    // Phase 2: Send 5250 device identification
    println!("\n🔧 Phase 2: 5250 Device Identification");
    
    // Send a basic 5250 terminal response
    // This tells the AS/400 what kind of terminal we are
    let device_response = create_5250_device_response();
    println!("  📤 Sending 5250 device identification ({} bytes)", device_response.len());
    stream.write_all(&device_response)?;
    stream.flush()?;
    
    // Phase 3: Read welcome screen data
    println!("\n🖥️  Phase 3: Reading Welcome Screen");
    
    for attempt in 1..=5 {
        println!("Attempt {}:", attempt);
        
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                println!("  📥 Received {} bytes", bytes_read);
                
                // Show hex dump
                print!("  Hex: ");
                for i in 0..std::cmp::min(32, bytes_read) {
                    print!("{:02x} ", buffer[i]);
                }
                if bytes_read > 32 { print!("..."); }
                println!();
                
                // Process any remaining telnet data
                let response = negotiator.process_incoming_data(&buffer[0..bytes_read]);
                if !response.is_empty() {
                    stream.write_all(&response)?;
                    stream.flush()?;
                }
                
                // Look for 5250 screen data
                let screen_data = extract_5250_screen_data(&buffer[0..bytes_read]);
                if !screen_data.is_empty() {
                    println!("  🎯 Found screen data ({} bytes)", screen_data.len());
                    display_5250_screen_data(&screen_data);
                    break;
                }
            },
            Ok(_) => {
                println!("  Connection closed");
                break;
            },
            Err(e) => {
                println!("  Waiting... ({})", e);
            }
        }
        
        std::thread::sleep(Duration::from_secs(2));
    }
    
    println!("\n✅ Welcome screen test complete");
    Ok(())
}

fn create_5250_device_response() -> Vec<u8> {
    // Create a basic 5250 terminal identification response
    // This is a simplified version - real 5250 negotiation is more complex
    vec![
        // Basic acknowledgment
        0x00, 0x00, // Length placeholder
        0x12, 0xA0, // Command: Acknowledge with device info
        // Device name: IBM-3179-2 (common terminal type)
        b'I', b'B', b'M', b'-', b'3', b'1', b'7', b'9', b'-', b'2',
    ]
}

fn extract_5250_screen_data(data: &[u8]) -> Vec<u8> {
    // This function extracts potential screen update data from 5250 stream
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        if data[i] == 0xFF {
            // Skip telnet commands
            if i + 1 < data.len() {
                match data[i + 1] {
                    0xFA => {
                        // Skip subnegotiation
                        i += 2;
                        while i + 1 < data.len() && !(data[i] == 0xFF && data[i + 1] == 0xF0) {
                            i += 1;
                        }
                        i += 2;
                    },
                    0xFB..=0xFE => i += 3,
                    _ => i += 2,
                }
            } else {
                i += 1;
            }
        } else {
            // This might be 5250 data
            result.push(data[i]);
            i += 1;
        }
    }
    
    result
}

fn display_5250_screen_data(data: &[u8]) {
    println!("  📺 Processing 5250 screen data:");
    
    if data.is_empty() {
        println!("    No screen data found");
        return;
    }
    
    // Show raw hex
    println!("    Raw: {:02x?}", data);
    
    // Try EBCDIC translation
    let translated: String = data.iter()
        .map(|&b| {
            let ascii = ebcdic_to_ascii(b);
            if ascii.is_control() { '.' } else { ascii }
        })
        .collect();
    
    println!("    EBCDIC: '{}'", translated);
    
    // Look for common 5250 commands
    for (i, &byte) in data.iter().enumerate() {
        match byte {
            0xF1 => println!("    Position {}: Write to Display command", i),
            0xF2 => println!("    Position {}: Read Buffer command", i),
            0x11 => println!("    Position {}: Set Buffer Address", i),
            0x1D => println!("    Position {}: Start of Field", i),
            0x40 => println!("    Position {}: Space character", i),
            _ => {}
        }
    }
    
    // Check for text patterns
    let printable_chars: Vec<char> = data.iter()
        .map(|&b| ebcdic_to_ascii(b))
        .filter(|&c| c.is_ascii_graphic() || c == ' ')
        .collect();
    
    if printable_chars.len() > 4 {
        let text: String = printable_chars.into_iter().collect();
        println!("    Text content: '{}'", text.trim());
    }
}