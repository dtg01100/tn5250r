use std::sync::{Arc, Mutex};
use lib5250::session::Session;
use lib5250::display::Display;
use lib5250::protocol::{Packet, CommandCode};

fn main() {
    println!("Testing WriteToDisplay packet processing...");

    // Create a session and display
    let display = Arc::new(Mutex::new(Display::new()));
    let mut session = Session::new(display.clone());

    // Create a WriteToDisplay packet with some test data
    // This simulates a simple screen write with "HELLO" at position (0,0)
    let data = vec![
        0x11, 0x00, 0x00,  // SBA (Set Buffer Address) to (0,0)
        0xC8, 0xC5, 0xD3, 0xD3, 0xD6  // EBCDIC for "HELLO"
    ];

    let packet = Packet::new(CommandCode::WriteToDisplay, 1, data);

    // Process the packet
    match session.process_5250_data_integrated(&packet.to_bytes()) {
        Ok(_) => println!("✓ Packet processed successfully"),
        Err(e) => {
            println!("✗ Packet processing failed: {}", e);
            return;
        }
    }

    // Check if the display was updated
    let display_content = display.lock().unwrap().to_string();
    println!("Display content:\n{}", display_content);

    if display_content.contains("HELLO") {
        println!("✓ SUCCESS: Display contains expected text!");
    } else {
        println!("✗ FAILURE: Display does not contain expected text");
    }
}