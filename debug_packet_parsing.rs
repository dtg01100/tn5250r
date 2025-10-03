//! Debug packet parsing issues
//!
//! This program tests the Packet::from_bytes() method with various packet structures
//! to identify why all interpretations are failing.

use std::io::{self, Write};

fn main() {
    println!("TN5250R Packet Parsing Debug Tool");
    println!("==================================");

    // Test case 1: Simple WriteToDisplay packet
    // ESC (0x04) + WriteToDisplay (0x11) + seq (0x01) + length (0x0005) + flags (0x00) + data (0x40, 0x40, 0x40, 0x40, 0x40)
    let test_packet_1 = vec![
        0x11, // Command: WriteToDisplay
        0x01, // Sequence: 1
        0x00, 0x05, // Length: 5 (data payload length)
        0x00, // Flags: 0
        0x40, 0x40, 0x40, 0x40, 0x40 // Data: 5 spaces in EBCDIC
    ];

    println!("\nTest Case 1: WriteToDisplay packet");
    println!("Raw bytes: {:02x?}", test_packet_1);
    println!("Length: {} bytes", test_packet_1.len());

    // Try to parse with our current implementation
    match tn5250r::lib5250::protocol::Packet::from_bytes(&test_packet_1) {
        Some(packet) => {
            println!("✅ SUCCESS: Parsed packet");
            println!("  Command: {:?}", packet.command);
            println!("  Sequence: {}", packet.sequence_number);
            println!("  Flags: 0x{:02X}", packet.flags);
            println!("  Data length: {}", packet.data.len());
            println!("  Data: {:02x?}", packet.data);
        }
        None => {
            println!("❌ FAILED: Could not parse packet");
        }
    }

    // Test case 2: Query packet structure
    let test_packet_2 = vec![
        0xF3, // Command: WriteStructuredField
        0x01, // Sequence: 1
        0x00, 0x0A, // Length: 10 (data payload length)
        0x00, // Flags: 0
        0x00, 0x03, // Structured field length: 3
        0xD9, // Class: 5250
        0x70, // Type: Query
        0x80  // Query flag
    ];

    println!("\nTest Case 2: WriteStructuredField (Query) packet");
    println!("Raw bytes: {:02x?}", test_packet_2);
    println!("Length: {} bytes", test_packet_2.len());

    match tn5250r::lib5250::protocol::Packet::from_bytes(&test_packet_2) {
        Some(packet) => {
            println!("✅ SUCCESS: Parsed packet");
            println!("  Command: {:?}", packet.command);
            println!("  Sequence: {}", packet.sequence_number);
            println!("  Flags: 0x{:02X}", packet.flags);
            println!("  Data length: {}", packet.data.len());
            println!("  Data: {:02x?}", packet.data);
        }
        None => {
            println!("❌ FAILED: Could not parse packet");
        }
    }

    // Test case 3: Minimal packet
    let test_packet_3 = vec![
        0x11, // Command: WriteToDisplay
        0x01, // Sequence: 1
        0x00, 0x00, // Length: 0 (no data)
        0x00  // Flags: 0
    ];

    println!("\nTest Case 3: Minimal packet (no data)");
    println!("Raw bytes: {:02x?}", test_packet_3);
    println!("Length: {} bytes", test_packet_3.len());

    match tn5250r::lib5250::protocol::Packet::from_bytes(&test_packet_3) {
        Some(packet) => {
            println!("✅ SUCCESS: Parsed packet");
            println!("  Command: {:?}", packet.command);
            println!("  Sequence: {}", packet.sequence_number);
            println!("  Flags: 0x{:02X}", packet.flags);
            println!("  Data length: {}", packet.data.len());
        }
        None => {
            println!("❌ FAILED: Could not parse packet");
        }
    }

    println!("\nDebug complete. Press Enter to exit...");
    io::stdin().read_line(&mut String::new()).unwrap();
}