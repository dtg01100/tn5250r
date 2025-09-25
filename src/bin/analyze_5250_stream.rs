use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tn5250r::telnet_negotiation::TelnetNegotiator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Raw 5250 Data Stream Analysis");
    println!("===============================");
    
    // Connect to pub400.com
    println!("Connecting to pub400.com:23...");
    let mut stream = TcpStream::connect("pub400.com:23")?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    
    println!("✅ Connected successfully!");
    
    let mut negotiator = TelnetNegotiator::new();
    let mut buffer = [0; 4096];
    let mut all_5250_data = Vec::new();
    
    println!("\n🤝 Processing telnet negotiation and collecting 5250 data...");
    
    for round in 1..=5 {
        println!("\n--- Round {} ---", round);
        
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Connection closed by server");
                break;
            },
            Ok(bytes_read) => {
                println!("📥 Received {} bytes", bytes_read);
                
                // Show first 32 bytes in hex
                print!("Raw hex: ");
                for i in 0..std::cmp::min(32, bytes_read) {
                    print!("{:02x} ", buffer[i]);
                }
                if bytes_read > 32 {
                    print!("... ({} more bytes)", bytes_read - 32);
                }
                println!();
                
                // Process telnet negotiation
                let response = negotiator.process_incoming_data(&buffer[0..bytes_read]);
                if !response.is_empty() {
                    println!("📤 Sending {} negotiation bytes", response.len());
                    stream.write_all(&response)?;
                    stream.flush()?;
                }
                
                // Extract non-telnet data (potential 5250 data)
                let clean_data = filter_telnet_data(&buffer[0..bytes_read]);
                if !clean_data.is_empty() {
                    println!("🎯 Found {} bytes of 5250 data:", clean_data.len());
                    print!("5250 hex: ");
                    for &byte in &clean_data {
                        print!("{:02x} ", byte);
                    }
                    println!();
                    
                    all_5250_data.extend_from_slice(&clean_data);
                    
                    // Try to analyze as 5250 commands
                    analyze_5250_data(&clean_data);
                }
            },
            Err(e) => {
                println!("❌ Read error: {}", e);
                break;
            }
        }
        
        std::thread::sleep(Duration::from_millis(200));
    }
    
    println!("\n📊 FINAL ANALYSIS");
    println!("=================");
    println!("Total 5250 data collected: {} bytes", all_5250_data.len());
    
    if !all_5250_data.is_empty() {
        println!("\nComplete 5250 data stream:");
        for (i, &byte) in all_5250_data.iter().enumerate() {
            if i % 16 == 0 {
                print!("\n{:04x}: ", i);
            }
            print!("{:02x} ", byte);
        }
        println!();
        
        // Full stream analysis
        println!("\n🔍 Full stream analysis:");
        analyze_5250_stream(&all_5250_data);
    }
    
    println!("\n✅ Analysis complete");
    Ok(())
}

fn filter_telnet_data(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        if data[i] == 0xFF && i + 1 < data.len() {
            match data[i + 1] {
                0xFF => {
                    result.push(0xFF);
                    i += 2;
                },
                0xFA => {
                    // Subnegotiation - extract the content between IAC SB and IAC SE
                    i += 2; // Skip IAC SB
                    let start_pos = i;
                    
                    // Find IAC SE (0xFF 0xF0)
                    while i + 1 < data.len() && !(data[i] == 0xFF && data[i + 1] == 0xF0) {
                        i += 1;
                    }
                    
                    // Extract subnegotiation content (this might contain 5250 data)
                    if i > start_pos {
                        let subneg_data = &data[start_pos..i];
                        println!("   📦 Subnegotiation data ({} bytes): {:02x?}", subneg_data.len(), subneg_data);
                        
                        // For terminal type subnegotiation, there might be 5250 data
                        if subneg_data.len() > 2 {
                            // Skip the option byte and check for 5250 data
                            result.extend_from_slice(&subneg_data[1..]);
                        }
                    }
                    
                    i += 2; // Skip IAC SE
                },
                0xFB..=0xFE => {
                    i += 3; // Skip Will/Won't/Do/Don't
                },
                _ => {
                    i += 2; // Skip other commands
                }
            }
        } else {
            result.push(data[i]);
            i += 1;
        }
    }
    
    result
}

fn analyze_5250_data(data: &[u8]) {
    if data.is_empty() {
        return;
    }
    
    println!("   📋 5250 Command Analysis:");
    
    // Look for common 5250 command codes
    for (i, &byte) in data.iter().enumerate() {
        match byte {
            0xF1 => println!("      Position {}: Write to Display (0xF1)", i),
            0xF2 => println!("      Position {}: Read Buffer (0xF2)", i), 
            0xF3 => println!("      Position {}: Read to Memory (0xF3)", i),
            0x11 => println!("      Position {}: Set Buffer Address (0x11)", i),
            0x12 => println!("      Position {}: Erase Unprotected (0x12)", i),
            0x13 => println!("      Position {}: Insert Cursor (0x13)", i),
            0x1D => println!("      Position {}: Start of Field (0x1D)", i),
            0x1E => println!("      Position {}: Set Field Attribute (0x1E)", i),
            0x29 => println!("      Position {}: Start of Header (0x29)", i),
            _ => {}
        }
    }
    
    // Check for text patterns
    if data.len() >= 3 {
        println!("   📝 Looking for text patterns...");
        // More analysis could go here
    }
}

fn analyze_5250_stream(data: &[u8]) {
    println!("Analyzing complete 5250 data stream:");
    
    if data.is_empty() {
        println!("  No 5250 data found - this might indicate:");
        println!("  • Telnet negotiation still in progress");
        println!("  • Server requires device identification first");
        println!("  • Different timing or connection requirements");
        return;
    }
    
    // Look for record structures
    println!("  Stream length: {} bytes", data.len());
    println!("  First 8 bytes: {:02x?}", &data[0..std::cmp::min(8, data.len())]);
    
    if data.len() >= 2 {
        println!("  Possible command: 0x{:02x}{:02x}", data[0], data[1]);
    }
}