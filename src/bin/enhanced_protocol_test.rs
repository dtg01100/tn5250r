use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tn5250r::telnet_negotiation::{TelnetNegotiator, TelnetOption};

fn main() -> std::io::Result<()> {
    println!("Enhanced Protocol Test - Testing improved telnet negotiation");
    println!("This test demonstrates the enhanced terminal type and environment negotiation");
    
    // Test with pub400.com first
    test_server("pub400.com", 23)?;
    
    // Test with the sensitive system if available
    println!("\n{}", "=".repeat(60));
    println!("Testing with sensitive system at 66.189.134.90:2323");
    test_server("66.189.134.90", 2323)?;
    
    Ok(())
}

fn test_server(host: &str, port: u16) -> std::io::Result<()> {
    println!("\n{}", "=".repeat(50));
    println!("Connecting to {}:{}", host, port);
    
    let mut stream = match TcpStream::connect((host, port)) {
        Ok(s) => {
            println!("✓ Connected successfully");
            s
        },
        Err(e) => {
            println!("✗ Connection failed: {}", e);
            return Ok(()); // Continue with next test
        }
    };
    
    // Set timeouts
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    
    let mut negotiator = TelnetNegotiator::new();
    
    // Send initial negotiation with enhanced features
    println!("\n1. Sending enhanced initial negotiation...");
    let initial_negotiation = negotiator.generate_initial_negotiation();
    
    if !initial_negotiation.is_empty() {
        println!("   Sending {} bytes of negotiation data", initial_negotiation.len());
        print!("   Negotiation sequence: ");
        for &byte in &initial_negotiation[..initial_negotiation.len().min(30)] {
            print!("{:02X} ", byte);
        }
        if initial_negotiation.len() > 30 {
            print!("...");
        }
        println!();
        
        stream.write_all(&initial_negotiation)?;
        stream.flush()?;
    }
    
    // Read and process negotiation responses
    println!("\n2. Processing negotiation responses...");
    let mut negotiation_rounds = 0;
    let max_rounds = 20;
    let mut total_5250_data = Vec::new();
    
    while negotiation_rounds < max_rounds {
        let mut buffer = [0u8; 2048];
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("   Connection closed by remote");
                break;
            }
            Ok(n) => {
                println!("   Round {}: Received {} bytes", negotiation_rounds + 1, n);
                
                // Show first few bytes for analysis
                print!("   Data: ");
                for &byte in &buffer[..n.min(20)] {
                    print!("{:02X} ", byte);
                }
                if n > 20 {
                    print!("...");
                }
                println!();
                
                // Process through our enhanced negotiator
                let response = negotiator.process_incoming_data(&buffer[..n]);
                
                if !response.is_empty() {
                    println!("   → Sending {} byte response", response.len());
                    print!("   Response: ");
                    for &byte in &response[..response.len().min(20)] {
                        print!("{:02X} ", byte);
                    }
                    if response.len() > 20 {
                        print!("...");
                    }
                    println!();
                    
                    stream.write_all(&response)?;
                    stream.flush()?;
                }
                
                // Extract any 5250 data from this packet
                let clean_data = extract_clean_5250_data(&buffer[..n]);
                if !clean_data.is_empty() {
                    println!("   → Extracted {} bytes of 5250 data", clean_data.len());
                    total_5250_data.extend_from_slice(&clean_data);
                }
                
                negotiation_rounds += 1;
                
                // Small delay between rounds
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                println!("   Read timeout - ending negotiation phase");
                break;
            }
            Err(e) => {
                println!("   Read error: {}", e);
                break;
            }
        }
    }
    
    println!("\n3. Negotiation Summary:");
    println!("   - Completed {} negotiation rounds", negotiation_rounds);
    println!("   - Negotiation complete: {}", negotiator.is_negotiation_complete());
    println!("   - Total 5250 data received: {} bytes", total_5250_data.len());
    
    // Check which options were successfully negotiated
    println!("\n4. Option Status:");
    let options = [
        (TelnetOption::Binary, "Binary"),
        (TelnetOption::EndOfRecord, "End of Record"),
        (TelnetOption::SuppressGoAhead, "Suppress Go Ahead"),
        (TelnetOption::TerminalType, "Terminal Type"),
        (TelnetOption::NewEnvironment, "New Environment"),
    ];
    
    for (option, name) in options {
        let active = negotiator.is_option_active(option);
        println!("   {} {}: {}", if active { "✓" } else { "✗" }, name, if active { "ACTIVE" } else { "inactive" });
    }
    
    // If we received 5250 data, show a sample
    if !total_5250_data.is_empty() {
        println!("\n5. Sample 5250 Data (first 100 bytes):");
        let sample_size = total_5250_data.len().min(100);
        print!("   Hex: ");
        for &byte in &total_5250_data[..sample_size] {
            print!("{:02X} ", byte);
        }
        println!();
        
        print!("   ASCII: ");
        for &byte in &total_5250_data[..sample_size] {
            if byte >= 32 && byte <= 126 {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }
        println!();
    }
    
    // Clean shutdown
    println!("\n6. Closing connection...");
    drop(stream);
    println!("   ✓ Connection closed");
    
    Ok(())
}

fn extract_clean_5250_data(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        if data[i] == 255 { // IAC
            if i + 1 < data.len() {
                match data[i + 1] {
                    251..=254 => { // WILL, WONT, DO, DONT
                        if i + 2 < data.len() {
                            i += 3; // Skip IAC + command + option
                            continue;
                        }
                    },
                    250 => { // SB (subnegotiation)
                        // Find the SE (end of subnegotiation)
                        let mut j = i + 2;
                        while j + 1 < data.len() {
                            if data[j] == 255 && data[j + 1] == 240 { // IAC SE
                                i = j + 2;
                                break;
                            }
                            j += 1;
                        }
                        if j + 1 >= data.len() {
                            break; // Incomplete subnegotiation
                        }
                        continue;
                    },
                    255 => { // Escaped IAC
                        result.push(255);
                        i += 2;
                        continue;
                    },
                    _ => {
                        i += 2; // Skip other telnet commands
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