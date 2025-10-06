use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use std::thread;

// Import the EBCDIC translation function and telnet negotiator
use tn5250r::lib5250::ebcdic_to_ascii;
use tn5250r::telnet_negotiation::{TelnetNegotiator, TelnetOption};
use tn5250r::lib5250::protocol::{ProtocolProcessor, Packet, CommandCode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TN5250R Full Protocol Test");
    println!("Connecting to as400.example.com:23...");

    // Attempt to connect
    let mut stream = TcpStream::connect("as400.example.com:23")?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    
    println!("✅ Successfully connected to pub400.com:23");
    
    // Create telnet negotiator
    let mut negotiator = TelnetNegotiator::new();

    // Create 5250 protocol processor
    let mut protocol_processor = ProtocolProcessor::new();
    
    // Send initial negotiation
    let initial_negotiation = negotiator.generate_initial_negotiation();
    if !initial_negotiation.is_empty() {
        println!("Sending initial telnet negotiation ({} bytes):", initial_negotiation.len());
        print!("Negotiation: ");
        for &byte in &initial_negotiation {
            print!("{:02x} ", byte);
        }
        println!();
        stream.write_all(&initial_negotiation)?;
    }
    
    // Read and handle telnet negotiation rounds
    let mut buffer = vec![0u8; 1024];
    let mut total_received = 0;
    let mut negotiation_phase = true;

    for round in 1..=10 {
        println!("\n--- Round {} ---", round);

        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("❌ Connection closed by remote host");
                break;
            }
            Ok(n) => {
                total_received += n;
                println!("✅ Received {} bytes from server (total: {}):", n, total_received);

                // Display the raw data
                print!("Raw hex: ");
                for i in 0..n {
                    print!("{:02x} ", buffer[i]);
                }
                println!();

                if negotiation_phase {
                    // Process through telnet negotiator
                    let responses = negotiator.process_incoming_data(&buffer[..n]);

                    // Send any telnet responses
                    if !responses.is_empty() {
                        println!("Sending telnet responses ({} bytes):", responses.len());
                        print!("Response: ");
                        for &byte in &responses {
                            print!("{:02x} ", byte);
                        }
                        println!();
                        stream.write_all(&responses)?;
                    }

                    // Check if telnet negotiation is complete
                    if negotiator.is_negotiation_complete() {
                        println!("🎉 Telnet negotiation complete! Starting 5250 protocol phase...");
                        negotiation_phase = false;

                        // Send initial 5250 protocol data to keep connection alive
                        let welcome_packet = protocol_processor.create_write_to_display_packet("TN5250R Connected\r\n");
                        let welcome_bytes = welcome_packet.to_bytes();
                        println!("Sending 5250 welcome packet ({} bytes):", welcome_bytes.len());
                        print!("5250 Packet: ");
                        for &byte in &welcome_bytes {
                            print!("{:02x} ", byte);
                        }
                        println!();
                        stream.write_all(&welcome_bytes)?;
                    }
                } else {
                    // 5250 protocol phase - process 5250 data
                    println!("🔄 Processing 5250 protocol data...");

                    // Try to parse as 5250 packet
                    if let Some(packet) = Packet::from_bytes(&buffer[..n]) {
                        println!("📦 Received 5250 packet: Command={:?}, Seq={}, DataLen={}",
                                packet.command, packet.sequence_number, packet.data.len());

                        // Process the packet and send response
                        match protocol_processor.process_packet(&packet) {
                            Ok(response_packets) => {
                                for response_packet in response_packets {
                                    let response_bytes = response_packet.to_bytes();
                                    println!("📤 Sending 5250 response ({} bytes):", response_bytes.len());
                                    print!("Response: ");
                                    for &byte in &response_bytes {
                                        print!("{:02x} ", byte);
                                    }
                                    println!();
                                    stream.write_all(&response_bytes)?;
                                }
                            }
                            Err(e) => {
                                println!("❌ Error processing 5250 packet: {}", e);
                            }
                        }
                    } else {
                        println!("📄 Received non-5250 data ({} bytes)", n);
                        // Display as regular data
                        print!("Data: ");
                        for i in 0..n {
                            let ch = buffer[i];
                            if ch >= 32 && ch <= 126 {
                                print!("{}", ch as char);
                            } else {
                                print!(".");
                            }
                        }
                        println!();
                    }
                }
                
                // Try to parse data for telnet commands
                let mut pos = 0;
                let mut non_telnet_data = Vec::new();
                
                while pos < n {
                    if buffer[pos] == 0xFF && pos + 1 < n { // IAC
                        let command = buffer[pos + 1];
                        match command {
                            0xFD => { // DONT
                                if pos + 2 < n {
                                    println!("Telnet DONT option: 0x{:02x}", buffer[pos + 2]);
                                    pos += 3;
                                } else {
                                    pos += 2;
                                }
                            }
                            0xFC => { // WONT
                                if pos + 2 < n {
                                    println!("Telnet WONT option: 0x{:02x}", buffer[pos + 2]);
                                    pos += 3;
                                } else {
                                    pos += 2;
                                }
                            }
                            0xFB => { // WILL
                                if pos + 2 < n {
                                    println!("Telnet WILL option: 0x{:02x}", buffer[pos + 2]);
                                    pos += 3;
                                } else {
                                    pos += 2;
                                }
                            }
                            0xFE => { // DO
                                if pos + 2 < n {
                                    println!("Telnet DO option: 0x{:02x}", buffer[pos + 2]);
                                    pos += 3;
                                } else {
                                    pos += 2;
                                }
                            }
                            _ => {
                                println!("Other telnet command: 0x{:02x}", command);
                                pos += 2;
                            }
                        }
                    } else {
                        non_telnet_data.push(buffer[pos]);
                        pos += 1;
                    }
                }
                
                // Display non-telnet data (potential 5250 data)
                if !non_telnet_data.is_empty() {
                    println!("Non-telnet data ({} bytes):", non_telnet_data.len());
                    
                    print!("Hex: ");
                    for &byte in &non_telnet_data {
                        print!("{:02x} ", byte);
                    }
                    println!();
                    
                    print!("EBCDIC->ASCII: ");
                    for &byte in &non_telnet_data {
                        let ascii_char = ebcdic_to_ascii(byte);
                        if ascii_char.is_ascii_graphic() || ascii_char == ' ' {
                            print!("{}", ascii_char);
                        } else {
                            print!(".");
                        }
                    }
                    println!();
                }
                
                // Short delay before next read
                thread::sleep(Duration::from_millis(1000));
            }
            Err(e) => {
                println!("❌ Error reading from server: {}", e);
                break;
            }
        }
    }
    
    println!("\n📊 Final negotiation state:");
    println!("- Binary mode active: {}", negotiator.is_option_active(TelnetOption::Binary));
    println!("- End-of-Record active: {}", negotiator.is_option_active(TelnetOption::EndOfRecord));
    println!("- Suppress-Go-Ahead active: {}", negotiator.is_option_active(TelnetOption::SuppressGoAhead));

    println!("\n🔧 5250 Protocol Status:");
    println!("- Protocol processor initialized: ✅");
    println!("- Connection phase: {}", if negotiation_phase { "Telnet Negotiation" } else { "5250 Protocol" });
    println!("- Total bytes processed: {}", total_received);
    
    println!("\n🔌 Protocol test complete");
    Ok(())
}