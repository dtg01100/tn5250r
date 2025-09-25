use std::env;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use tn5250r::controller::TerminalController;
use tn5250r::keyboard::FunctionKey;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <host> <port>", args[0]);
        std::process::exit(1);
    }

    let host = &args[1];
    let port: u16 = args[2].parse().expect("Invalid port number");

    println!("TN5250R CLI Test Client");
    println!("Connecting to {}:{}...", host, port);

    let mut controller = TerminalController::new();
    
    match controller.connect(host.clone(), port) {
        Ok(()) => {
            println!("✅ Connected successfully!");
            
            // Give it time to receive initial data
            thread::sleep(Duration::from_millis(500));
            
            // Display initial terminal content
            let content = controller.get_terminal_content();
            println!("\n📺 Terminal Content:");
            println!("{}", content);
            
            println!("\n📋 Interactive Commands:");
            println!("  'exit' or 'quit' - Exit the program");
            println!("  'refresh' - Refresh terminal display");
            println!("  'f1', 'f2', etc. - Send function keys");
            println!("  Any other text - Send as input");
            println!();

            let stdin = io::stdin();
            loop {
                print!("TN5250R> ");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                if stdin.read_line(&mut input).is_err() {
                    break;
                }
                
                let input = input.trim();
                
                if input.is_empty() {
                    continue;
                }
                
                if input == "exit" || input == "quit" {
                    break;
                }
                
                if input == "refresh" {
                    let content = controller.get_terminal_content();
                    println!("📺 Terminal Content:");
                    println!("{}", content);
                    continue;
                }
                
                // Check for function keys
                if input.starts_with('f') || input.starts_with('F') {
                    if let Ok(key_num) = input[1..].parse::<u8>() {
                        if key_num >= 1 && key_num <= 24 {
                            let func_key = match key_num {
                                1 => FunctionKey::F1,
                                2 => FunctionKey::F2,
                                3 => FunctionKey::F3,
                                4 => FunctionKey::F4,
                                5 => FunctionKey::F5,
                                6 => FunctionKey::F6,
                                7 => FunctionKey::F7,
                                8 => FunctionKey::F8,
                                9 => FunctionKey::F9,
                                10 => FunctionKey::F10,
                                11 => FunctionKey::F11,
                                12 => FunctionKey::F12,
                                13 => FunctionKey::F13,
                                14 => FunctionKey::F14,
                                15 => FunctionKey::F15,
                                16 => FunctionKey::F16,
                                17 => FunctionKey::F17,
                                18 => FunctionKey::F18,
                                19 => FunctionKey::F19,
                                20 => FunctionKey::F20,
                                21 => FunctionKey::F21,
                                22 => FunctionKey::F22,
                                23 => FunctionKey::F23,
                                24 => FunctionKey::F24,
                                _ => continue,
                            };
                            
                            match controller.send_function_key(func_key) {
                                Ok(()) => {
                                    println!("✅ Sent function key F{}", key_num);
                                    // Give time for response
                                    thread::sleep(Duration::from_millis(200));
                                    let content = controller.get_terminal_content();
                                    println!("📺 Updated Terminal Content:");
                                    println!("{}", content);
                                }
                                Err(e) => println!("❌ Error sending function key: {}", e),
                            }
                            continue;
                        }
                    }
                }
                
                // Send regular text input
                match controller.send_input(input.as_bytes()) {
                    Ok(()) => {
                        println!("✅ Sent input: '{}'", input);
                        // Give time for response
                        thread::sleep(Duration::from_millis(200));
                        let content = controller.get_terminal_content();
                        println!("📺 Updated Terminal Content:");
                        println!("{}", content);
                    }
                    Err(e) => println!("❌ Error sending input: {}", e),
                }
            }
            
            println!("👋 Disconnecting...");
            controller.disconnect();
        }
        Err(e) => {
            eprintln!("❌ Failed to connect: {}", e);
            std::process::exit(1);
        }
    }
}