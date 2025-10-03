#!/usr/bin/env cargo
//! TN3270E Real System Validation Test
//!
//! Tests the TN3270E implementation against real TN3270E-capable systems.
//! This validates the complete TN3270E negotiation flow including:
//! - TN3270E option negotiation
//! - Device type negotiation
//! - Session binding
//! - Error handling

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use std::env;

use tn5250r::telnet_negotiation::{TelnetNegotiator, TelnetOption, TN3270ESessionState, TN3270EDeviceType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TN3270E Real System Validation Test");
    println!("===================================\n");

    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let server = args.iter()
        .position(|arg| arg == "--server")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.clone())
        .unwrap_or_else(|| "pub400.com".to_string());

    let port: u16 = args.iter()
        .position(|arg| arg == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(23);

    println!("Testing TN3270E against: {}:{}", server, port);
    println!("Note: pub400.com is NVT/VT100, not TN3270E - expect negotiation failure\n");

    // Connect to server
    println!("🔌 Connecting to {}:{}...", server, port);
    let mut stream = TcpStream::connect(format!("{}:{}", server, port))?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    println!("✅ Connected successfully");

    // Create TN3270E negotiator
    let mut negotiator = TelnetNegotiator::new();

    // Send initial TN3270E negotiation
    let initial_negotiation = negotiator.generate_initial_negotiation();
    println!("📤 Sending initial telnet negotiation ({} bytes)", initial_negotiation.len());
    println!("   Raw: {:02x?}", &initial_negotiation[..initial_negotiation.len().min(32)]);
    stream.write_all(&initial_negotiation)?;
    stream.flush()?;

    // Read server response
    let mut buffer = vec![0u8; 4096];
    let mut total_read = 0;
    let mut negotiation_rounds = 0;
    const MAX_ROUNDS: u32 = 5;

    println!("\n🔄 Starting TN3270E negotiation...");

    while negotiation_rounds < MAX_ROUNDS {
        negotiation_rounds += 1;
        println!("\n--- Round {} ---", negotiation_rounds);

        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("❌ Connection closed by server");
                break;
            }
            Ok(n) => {
                total_read += n;
                println!("📥 Received {} bytes (total: {})", n, total_read);
                println!("   Raw: {:02x?}", &buffer[..n.min(64)]);

                // Process the data
                let response = negotiator.process_incoming_data(&buffer[..n]);

                // Log current TN3270E state
                println!("   TN3270E State: {:?}", negotiator.tn3270e_session_state());
                println!("   TN3270E Device: {:?}", negotiator.tn3270e_device_type());
                println!("   Logical Unit: {:?}", negotiator.logical_unit_name());
                println!("   Negotiation Complete: {}", negotiator.is_negotiation_complete());

                // Send response if any
                if !response.is_empty() {
                    println!("📤 Sending response ({} bytes): {:02x?}", response.len(), &response[..response.len().min(32)]);
                    stream.write_all(&response)?;
                    stream.flush()?;
                } else {
                    println!("📤 No response needed");
                }

                // Check if negotiation is complete
                if negotiator.is_negotiation_complete() {
                    println!("✅ TN3270E negotiation completed successfully!");
                    break;
                }

                // If server doesn't support TN3270E, it might close or send NVT data
                if negotiation_rounds >= 2 && matches!(negotiator.tn3270e_session_state(), TN3270ESessionState::NotConnected) {
                    println!("⚠️  Server does not appear to support TN3270E (likely NVT/VT100 only)");
                    break;
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    println!("⏱️  Read timeout - server may not be responding");
                    break;
                } else {
                    println!("❌ Read error: {}", e);
                    break;
                }
            }
        }
    }

    // Final status report
    println!("\n📊 TN3270E Validation Results");
    println!("============================");

    let final_state = negotiator.tn3270e_session_state();
    let device_type = negotiator.tn3270e_device_type();
    let lu_name = negotiator.logical_unit_name();
    let is_complete = negotiator.is_negotiation_complete();

    println!("Final TN3270E Session State: {:?}", final_state);
    println!("Negotiated Device Type: {:?}", device_type);
    println!("Logical Unit Name: {:?}", lu_name);
    println!("Negotiation Complete: {}", is_complete);
    println!("Total Bytes Received: {}", total_read);
    println!("Negotiation Rounds: {}", negotiation_rounds);

    // Assessment
    println!("\n🎯 Assessment:");
    match final_state {
        TN3270ESessionState::Bound => {
            println!("✅ SUCCESS: Full TN3270E session established!");
            println!("   - TN3270E option negotiated");
            println!("   - Device type agreed upon");
            println!("   - Session bound with logical unit");
        }
        TN3270ESessionState::DeviceNegotiated => {
            println!("⚠️  PARTIAL: Device negotiated but session not bound");
            println!("   - TN3270E option negotiated");
            println!("   - Device type agreed upon");
            println!("   - Session binding may have failed");
        }
        TN3270ESessionState::TN3270ENegotiated => {
            println!("⚠️  PARTIAL: TN3270E negotiated but device type not agreed");
            println!("   - TN3270E option negotiated");
            println!("   - Device type negotiation failed");
        }
        TN3270ESessionState::NotConnected => {
            println!("❌ FAILURE: Server does not support TN3270E");
            println!("   - Likely an NVT/VT100-only server");
            println!("   - This is expected for pub400.com");
        }
        _ => {
            println!("❓ UNKNOWN: Unexpected final state");
        }
    }

    if server == "pub400.com" && matches!(final_state, TN3270ESessionState::NotConnected) {
        println!("\nℹ️  Note: pub400.com is known to be NVT/VT100 only, not TN3270E.");
        println!("   This result is expected and validates correct protocol detection.");
    }

    println!("\n🔌 Connection test complete");
    Ok(())
}