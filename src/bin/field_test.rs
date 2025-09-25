//! Test field navigation and input functionality

use std::thread;
use std::time::Duration;

use tn5250r::controller::AsyncTerminalController;

fn main() {
    println!("Testing field navigation and input functionality...");
    
    let mut controller = AsyncTerminalController::new();
    
    // Test with pub400.com
    println!("Connecting to pub400.com:23...");
    match controller.connect("pub400.com".to_string(), 23) {
        Ok(()) => {
            println!("Connected successfully!");
            
            // Wait for negotiation and request login screen
            println!("Waiting for login screen...");
            thread::sleep(Duration::from_secs(2));
            
            if let Err(e) = controller.request_login_screen() {
                println!("Failed to request login screen: {}", e);
                return;
            }
            
            thread::sleep(Duration::from_secs(3));
            
            // Get initial fields
            match controller.get_fields_info() {
                Ok(fields_info) => {
                    println!("Detected {} fields:", fields_info.len());
                    for (i, (label, content, active)) in fields_info.iter().enumerate() {
                        let status = if *active { " [ACTIVE]" } else { "" };
                        println!("  Field {}: {} = '{}'{}", i + 1, label, content, status);
                    }
                }
                Err(e) => {
                    println!("Failed to get fields info: {}", e);
                    return;
                }
            }
            
            // Test Tab navigation
            println!("\nTesting Tab navigation...");
            if let Err(e) = controller.next_field() {
                println!("Failed to navigate to next field: {}", e);
            } else {
                println!("Navigated to next field");
                
                // Check current field
                if let Ok(fields_info) = controller.get_fields_info() {
                    for (i, (label, _content, active)) in fields_info.iter().enumerate() {
                        if *active {
                            println!("Now on field {}: {}", i + 1, label);
                            break;
                        }
                    }
                }
            }
            
            // Test typing in current field
            println!("\nTesting text input...");
            let test_text = "testuser";
            for ch in test_text.chars() {
                if let Err(e) = controller.type_char(ch) {
                    println!("Failed to type '{}': {}", ch, e);
                } else {
                    println!("Typed '{}'", ch);
                }
            }
            
            // Check field content after typing
            if let Ok(fields_info) = controller.get_fields_info() {
                for (i, (_label, content, active)) in fields_info.iter().enumerate() {
                    if *active {
                        println!("Field {} now contains: '{}'", i + 1, content);
                        break;
                    }
                }
            }
            
            // Test backspace
            println!("\nTesting backspace...");
            if let Err(e) = controller.backspace() {
                println!("Failed to backspace: {}", e);
            } else {
                println!("Backspaced successfully");
                
                // Check field content after backspace
                if let Ok(fields_info) = controller.get_fields_info() {
                    for (i, (_label, content, active)) in fields_info.iter().enumerate() {
                        if *active {
                            println!("Field {} after backspace: '{}'", i + 1, content);
                            break;
                        }
                    }
                }
            }
            
            // Test Shift+Tab (previous field)
            println!("\nTesting Shift+Tab (previous field)...");
            if let Err(e) = controller.previous_field() {
                println!("Failed to navigate to previous field: {}", e);
            } else {
                println!("Navigated to previous field");
                
                if let Ok(fields_info) = controller.get_fields_info() {
                    for (i, (label, _content, active)) in fields_info.iter().enumerate() {
                        if *active {
                            println!("Now on field {}: {}", i + 1, label);
                            break;
                        }
                    }
                }
            }
            
            println!("Disconnecting...");
            controller.disconnect();
        }
        Err(e) => {
            println!("Connection failed: {}", e);
        }
    }
    
    println!("Field test completed.");
}