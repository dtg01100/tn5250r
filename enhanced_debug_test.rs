use tn5250r::controller::TerminalController;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TN5250R Enhanced Debug Connection Test ===");
    println!("Connecting to as400.example.com:23 in 5250 mode...");
    
    // Create controller
    let mut controller = TerminalController::new();
    
    // Connect to the AS/400 system
    match controller.connect("as400.example.com".to_string(), 23) {
        Ok(_) => {
            println!("✅ Connection established successfully!");
            
            // Immediately check terminal content after connection
            let initial_content = controller.get_terminal_content();
            println!("=== Initial Content After Connection ===");
            println!("Content length: {} chars", initial_content.len());
            
            // Show a sample of the content in hex format to see what we actually have
            let content_bytes = initial_content.as_bytes();
            println!("First 100 bytes as hex:");
            for (i, &byte) in content_bytes.iter().take(100).enumerate() {
                if i % 16 == 0 {
                    print!("\n{:04x}: ", i);
                }
                print!("{:02x} ", byte);
            }
            println!();
            
            // Check if we have a standard 80x24 grid of spaces (1920 chars) plus newlines
            let total_chars = initial_content.len();
            let lines: Vec<&str> = initial_content.lines().collect();
            println!("Number of lines: {}", lines.len());
            if !lines.is_empty() {
                println!("First line length: {}", lines[0].len());
                println!("Last line length: {}", lines.get(lines.len()-1).map_or(0, |s| s.len()));
            }
            
            // Give the connection time to receive more data and process it
            for i in 1..=3 {
                thread::sleep(Duration::from_millis(1000));
                
                if controller.is_connected() {
                    // Process any incoming data
                    if let Err(e) = controller.process_incoming_data() {
                        println!("Error processing data: {}", e);
                    }
                    
                    let content = controller.get_terminal_content();
                    println!("=== Loop {} ===", i);
                    if content != initial_content {
                        println!("Content changed! New length: {}", content.len());
                        // Show what changed
                        let non_space_chars: String = content.chars().filter(|c| !c.is_whitespace()).take(100).collect();
                        if !non_space_chars.is_empty() {
                            println!("Non-whitespace content: '{}'", non_space_chars);
                            break;
                        }
                    } else {
                        println!("Content unchanged");
                    }
                } else {
                    println!("❌ Connection lost");
                    break;
                }
            }
            
            // Try requesting login screen explicitly
            println!("=== Requesting Login Screen ===");
            match controller.request_login_screen() {
                Ok(_) => {
                    println!("Login screen request sent");
                    thread::sleep(Duration::from_millis(1000));
                    
                    if let Err(e) = controller.process_incoming_data() {
                        println!("Error processing login screen data: {}", e);
                    }
                    
                    let final_content = controller.get_terminal_content();
                    if final_content != initial_content {
                        println!("Content changed after login screen request!");
                        let non_space_chars: String = final_content.chars().filter(|c| !c.is_whitespace()).take(100).collect();
                        if !non_space_chars.is_empty() {
                            println!("Final non-whitespace content: '{}'", non_space_chars);
                        } else {
                            println!("Still only whitespace content");
                        }
                    } else {
                        println!("No change after login screen request");
                    }
                }
                Err(e) => println!("Failed to request login screen: {}", e),
            }
            
            // Disconnect
            controller.disconnect();
            println!("Disconnected.");
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}