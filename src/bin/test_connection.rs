use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

// Import the EBCDIC translation function and 5250 session
use tn5250r::lib5250::{ebcdic_to_ascii, Session};
use tn5250r::telnet_negotiation::TelnetNegotiator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TN5250R Enhanced Connection Test");
    println!("Connecting to 10.100.200.1:23...");

    // Attempt to connect
    let mut stream = TcpStream::connect("10.100.200.1:23")?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;

    println!("✅ Successfully connected to 10.100.200.1:23");

    // Create 5250 session and telnet negotiator
    let mut session = Session::new();
    let mut negotiator = TelnetNegotiator::new();

    // Read initial telnet negotiation data from the server
    let mut buffer = vec![0u8; 4096];
    let mut negotiation_complete = false;
    let mut total_data_received = 0;

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("❌ Connection closed by remote host");
                break;
            }
            Ok(n) => {
                total_data_received += n;
                println!("✅ Received {} bytes (total: {})", n, total_data_received);

                // Process telnet negotiation
                let negotiation_response = negotiator.process_incoming_data(&buffer[..n]);
                if !negotiation_response.is_empty() {
                    println!("📤 Sending telnet response: {:02x?}", &negotiation_response);
                    stream.write_all(&negotiation_response)?;
                    stream.flush()?;
                }

                // Check if negotiation is complete
                if negotiator.is_negotiation_complete() && !negotiation_complete {
                    println!("✅ Telnet negotiation complete!");
                    negotiation_complete = true;

                    // Send initial 5250 protocol data
                    match session.send_initial_5250_data() {
                        Ok(protocol_data) => {
                            println!("📤 Sending initial 5250 data ({} bytes): {:02x?}",
                                   protocol_data.len(),
                                   &protocol_data[..protocol_data.len().min(64)]);
                            stream.write_all(&protocol_data)?;
                            stream.flush()?;
                            println!("✅ Initial 5250 data sent successfully");
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

                // After negotiation, read 5250 protocol data
                println!("📥 Reading 5250 protocol data...");
                break; // For now, just test the initial handshake
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    println!("⏱️ Read timeout - checking if we should continue...");
                    if negotiation_complete {
                        println!("✅ Negotiation complete, connection maintained");
                        break;
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