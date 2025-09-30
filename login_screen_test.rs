use tn5250r::controller::TerminalController;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TN5250R Login Screen Request Test ===");
    println!("Connecting to 10.100.200.1:23...");
    
    // Create controller
    let mut controller = TerminalController::new();
    
    // Connect to the AS/400 system
    match controller.connect("10.100.200.1".to_string(), 23) {
        Ok(_) => {
            println!("✅ Connection established successfully!");
            
            // Wait a moment for the connection to stabilize
            thread::sleep(Duration::from_millis(500));
            
            // Check initial content
            let initial_content = controller.get_terminal_content();
            println!("Initial content length: {}", initial_content.len());
            
            // Immediately request login screen
            println!("=== Requesting Login Screen ===");
            match controller.request_login_screen() {
                Ok(_) => {
                    println!("Login screen request sent successfully");
                    
                    // Wait for response and process any incoming data
                    for i in 1..=5 {
                        thread::sleep(Duration::from_millis(1000));
                        
                        if let Err(e) = controller.process_incoming_data() {
                            println!("Error processing data: {}", e);
                        }
                        
                        let content = controller.get_terminal_content();
                        println!("=== Iteration {} ===", i);
                        
                        if content != initial_content {
                            println!("🎉 Content changed! New length: {}", content.len());
                            
                            // Look for non-whitespace content
                            let non_space_chars: String = content.chars()
                                .filter(|c| !c.is_whitespace())
                                .take(100)
                                .collect();
                            
                            if !non_space_chars.is_empty() {
                                println!("Non-whitespace content found: '{}'", non_space_chars);
                                
                                // Show first few lines of actual content
                                let lines: Vec<&str> = content.lines().collect();
                                println!("First 5 lines of screen:");
                                for (i, line) in lines.iter().take(5).enumerate() {
                                    println!("Line {}: '{}'", i+1, line);
                                }
                                break;
                            } else {
                                println!("Content changed but still only whitespace");
                            }
                        } else {
                            println!("Content unchanged");
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to request login screen: {}", e);
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