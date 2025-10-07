use tn5250r::controller::TerminalController;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TN5250R Debug Connection Test ===");
    println!("Connecting to as400.example.com:23 in 5250 mode...");
    
    // Create controller
    let mut controller = TerminalController::new();
    
    // Connect to the AS/400 system
    match controller.connect("as400.example.com".to_string(), 23) {
        Ok(_) => {
            println!("✅ Connection established successfully!");
            
            // Give the connection time to receive data and process it
            for i in 1..=10 {
                thread::sleep(Duration::from_millis(500));
                
                // Check if connected
                if controller.is_connected() {
                    // Process any incoming data
                    if let Err(e) = controller.process_incoming_data() {
                        println!("Error processing data: {}", e);
                    }
                    
                    // Get the current terminal content
                    let content = controller.get_terminal_content();
                    println!("=== Loop {} - Terminal Content ===", i);
                    println!("Content length: {} chars", content.len());
                    if !content.is_empty() && !content.trim().is_empty() {
                        println!("First 200 chars: '{}'", 
                            content.chars().take(200).collect::<String>());
                        // Also check if there's actual non-space content
                        let non_space_chars: String = content.chars().filter(|c| !c.is_whitespace()).take(50).collect();
                        if !non_space_chars.is_empty() {
                            println!("Non-whitespace content found: '{}'", non_space_chars);
                            break;
                        } else {
                            println!("Content is mostly whitespace");
                        }
                    } else {
                        println!("No meaningful content yet, waiting...");
                    }
                } else {
                    println!("❌ Connection lost at iteration {}", i);
                    break;
                }
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