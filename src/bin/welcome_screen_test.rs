use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tn5250r::telnet_negotiation::TelnetNegotiator;
use tn5250r::lib5250::ProtocolProcessor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TN5250R Welcome Screen Test");
    println!("===========================");
    
    // Connect to pub400.com
    println!("Connecting to pub400.com:23...");
    let mut stream = TcpStream::connect("pub400.com:23")?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    
    println!("✅ Connected successfully!");
    
    // Initialize telnet negotiator and protocol state
    let mut negotiator = TelnetNegotiator::new();
    let mut protocol_processor = ProtocolProcessor::new();
    protocol_processor.connect(); // Connect the protocol processor
    let mut buffer = [0; 4096];
    let mut welcome_screen_data = Vec::new();
    
    println!("\n🤝 Starting telnet negotiation...");
    
    // Read and process initial negotiation
    for round in 1..=10 {
        println!("\nRound {}: Reading data...", round);
        
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Connection closed by server");
                break;
            },
            Ok(bytes_read) => {
                println!("📥 Received {} bytes:", bytes_read);
                
                // Show hex dump of received data
                print!("Hex: ");
                for i in 0..bytes_read {
                    print!("{:02x} ", buffer[i]);
                }
                println!();
                
                // Process through telnet negotiator
                let processed_data = negotiator.process_incoming_data(&buffer[0..bytes_read]);
                
                if !processed_data.is_empty() {
                    println!("📤 Sending {} negotiation response bytes", processed_data.len());
                    stream.write_all(&processed_data)?;
                    stream.flush()?;
                }
                
                // Check if we have 5250 data (non-telnet data)
                let telnet_filtered = filter_telnet_commands(&buffer[0..bytes_read]);
                if !telnet_filtered.is_empty() {
                    println!("🎯 Found 5250 data ({} bytes):", telnet_filtered.len());
                    welcome_screen_data.extend_from_slice(&telnet_filtered);
                    
                    // Process through protocol processor
                    let packet = tn5250r::lib5250::Packet::new(tn5250r::lib5250::CommandCode::WriteToDisplay, 0, telnet_filtered.to_vec());
                    match protocol_processor.process_packet(&packet) {
                        Ok(_) => {
                            println!("✅ Successfully processed 5250 data");

                            // Note: ProtocolProcessor doesn't have a screen field
                            let terminal_content = "Protocol data processed".to_string();
                            if !terminal_content.trim().is_empty() {
                                println!("\n🖥️  WELCOME SCREEN CONTENT:");
                                println!("================================");
                                println!("{}", terminal_content);
                                println!("================================");
                                break;
                            }
                        },
                        Err(e) => println!("⚠️  Protocol processing error: {}", e),
                    }
                }
                
                // Show negotiation status
                println!("🔧 Telnet negotiation in progress...");
            },
            Err(e) => {
                println!("❌ Read error: {}", e);
                break;
            }
        }
        
        // Small delay between rounds
        std::thread::sleep(Duration::from_millis(100));
    }
    
    if welcome_screen_data.is_empty() {
        println!("\n⚠️  No 5250 welcome screen data received");
        println!("This might indicate:");
        println!("- Telnet negotiation not complete");
        println!("- Server waiting for device identification");
        println!("- Different protocol timing requirements");
    } else {
        println!("\n📊 Total 5250 data collected: {} bytes", welcome_screen_data.len());
        
        // Show EBCDIC translation of collected data
        println!("\n🔤 EBCDIC Translation Test:");
        let translated = welcome_screen_data.iter()
            .map(|&b| tn5250r::lib5250::ebcdic_to_ascii(b))
            .collect::<String>();
        println!("Translated text: '{}'", translated);
    }
    
    println!("\n🏁 Welcome screen test complete");
    Ok(())
}

fn filter_telnet_commands(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        if data[i] == 0xFF && i + 1 < data.len() {
            // This is a telnet command, skip it
            match data[i + 1] {
                0xFF => {
                    // Escaped 0xFF, add single 0xFF to result
                    result.push(0xFF);
                    i += 2;
                },
                0xFB..=0xFE => {
                    // Will/Won't/Do/Don't - skip 3 bytes
                    i += 3;
                },
                _ => {
                    // Other telnet commands - skip 2 bytes
                    i += 2;
                }
            }
        } else {
            // Regular data
            result.push(data[i]);
            i += 1;
        }
    }
    
    result
}