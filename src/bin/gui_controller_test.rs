//! Simple test to verify GUI controller login screen functionality

use std::thread;
use std::time::Duration;

use tn5250r::controller::AsyncTerminalController;

fn main() {
    println!("Testing GUI controller login screen request functionality...");
    
    let mut controller = AsyncTerminalController::new();
    
    // Test with pub400.com
    println!("Connecting to pub400.com:23...");
    match controller.connect("pub400.com".to_string(), 23) {
        Ok(()) => {
            println!("Connected successfully!");
            
            // Wait for negotiation to complete
            println!("Waiting for negotiation to complete...");
            thread::sleep(Duration::from_secs(2));
            
            // Request login screen
            println!("Requesting login screen...");
            match controller.request_login_screen() {
                Ok(()) => {
                    println!("Login screen request sent successfully!");
                }
                Err(e) => {
                    println!("Failed to request login screen: {}", e);
                }
            }
            
            // Wait for response and check terminal content
            println!("Waiting for login screen response...");
            thread::sleep(Duration::from_secs(3));
            
            match controller.get_terminal_content() {
                Ok(content) => {
                    println!("Terminal content (first 500 chars):");
                    println!("{}", "=".repeat(50));
                    let display_content = if content.len() > 500 {
                        &content[..500]
                    } else {
                        &content
                    };
                    println!("{}", display_content);
                    println!("{}", "=".repeat(50));
                    
                    // Check if it contains login screen keywords
                    let lower_content = content.to_lowercase();
                    if lower_content.contains("welcome") || 
                       lower_content.contains("login") || 
                       lower_content.contains("user") ||
                       lower_content.contains("password") ||
                       lower_content.contains("pub400") {
                        println!("✅ Login screen detected!");
                    } else {
                        println!("❌ No login screen keywords found.");
                    }
                }
                Err(e) => {
                    println!("Failed to get terminal content: {}", e);
                }
            }
            
            println!("Disconnecting...");
            controller.disconnect();
        }
        Err(e) => {
            println!("Connection failed: {}", e);
        }
    }
    
    println!("GUI controller test completed.");
}