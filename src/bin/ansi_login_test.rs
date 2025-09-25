use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use tn5250r::telnet_negotiation::TelnetNegotiator;
use tn5250r::ansi_processor::AnsiProcessor;
use tn5250r::terminal::TerminalScreen;

fn main() -> std::io::Result<()> {
    println!("ANSI Login Screen Test - Displaying actual login screens");
    
    // Test both systems
    test_ansi_login("pub400.com", 23)?;
    test_ansi_login("66.189.134.90", 2323)?;
    
    Ok(())
}

fn test_ansi_login(host: &str, port: u16) -> std::io::Result<()> {
    println!("\n{}", "=".repeat(70));
    println!("Connecting to {}:{}", host, port);
    
    let mut stream = match TcpStream::connect((host, port)) {
        Ok(s) => {
            println!("✓ Connected successfully");
            s
        },
        Err(e) => {
            println!("✗ Connection failed: {}", e);
            return Ok(());
        }
    };
    
    // Set timeouts
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    
    let mut negotiator = TelnetNegotiator::new();
    let mut ansi_processor = AnsiProcessor::new();
    let mut screen = TerminalScreen::new();
    
    // Perform telnet negotiation
    println!("\n1. Performing enhanced telnet negotiation...");
    let initial_negotiation = negotiator.generate_initial_negotiation();
    if !initial_negotiation.is_empty() {
        stream.write_all(&initial_negotiation)?;
        stream.flush()?;
    }
    
    // Process negotiation and initial screen data
    let mut rounds = 0;
    let max_rounds = 15;
    let mut screen_requested = false;
    
    while rounds < max_rounds {
        let mut buffer = [0u8; 2048];
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("   Connection closed");
                break;
            }
            Ok(n) => {
                rounds += 1;
                println!("   Round {}: Received {} bytes", rounds, n);
                
                // Process telnet negotiation
                let negotiation_response = negotiator.process_incoming_data(&buffer[..n]);
                if !negotiation_response.is_empty() {
                    stream.write_all(&negotiation_response)?;
                    stream.flush()?;
                }
                
                // Check if this looks like ANSI data
                let has_ansi = buffer[..n].windows(2).any(|w| w == [0x1B, b'[']);
                
                if has_ansi {
                    println!("   → Processing as ANSI terminal data");
                    ansi_processor.process_data(&buffer[..n], &mut screen);
                    
                    // If we've got substantial content, show the screen
                    if rounds >= 3 {
                        display_login_screen(&screen, host, port);
                    }
                } else if n > 10 {
                    // Might be other terminal data
                    println!("   → Non-ANSI data: {:02X?}", &buffer[..n.min(20)]);
                }
                
                // After negotiation is done, request the initial screen
                if !screen_requested && rounds >= 2 {
                    println!("   → Requesting initial screen...");
                    // Send ReadModified command to request screen
                    let read_cmd = [0xFB]; // ReadModified command
                    stream.write_all(&read_cmd)?;
                    stream.flush()?;
                    screen_requested = true;
                }
                
                // Small delay
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                println!("   Read timeout");
                break;
            }
            Err(e) => {
                println!("   Read error: {}", e);
                break;
            }
        }
    }
    
    // Final screen display
    display_login_screen(&screen, host, port);
    
    println!("\n4. Closing connection...");
    drop(stream);
    
    Ok(())
}

fn display_login_screen(screen: &TerminalScreen, host: &str, port: u16) {
    println!("\n🖥️  LOGIN SCREEN FOR {}:{}", host, port);
    println!("{}", "═".repeat(82));
    
    let screen_content = screen.to_string();
    let lines: Vec<&str> = screen_content.lines().collect();
    
    for (i, line) in lines.iter().enumerate() {
        if i < 24 { // Only show 24 lines
            // Ensure line is exactly 80 characters
            let display_line = if line.len() >= 80 {
                &line[..80]
            } else {
                line
            };
            
            println!("║{}{}║", 
                display_line,
                " ".repeat(80_usize.saturating_sub(display_line.len()))
            );
        }
    }
    
    // Fill remaining lines if needed
    for _ in lines.len()..24 {
        println!("║{}║", " ".repeat(80));
    }
    
    println!("{}", "═".repeat(82));
    
    // Note: cursor position tracking would need ansi_processor reference
    
    // Show some statistics
    let text_content: String = screen_content.chars().filter(|c| !c.is_control() && *c != '\n').collect();
    println!("Screen contains {} visible characters", text_content.len());
    
    // Check for login-related keywords
    let content_lower = screen_content.to_lowercase();
    let login_keywords = ["user", "password", "sign on", "login", "welcome", "system"];
    let found_keywords: Vec<&str> = login_keywords.iter()
        .filter(|&&keyword| content_lower.contains(keyword))
        .copied()
        .collect();
    
    if !found_keywords.is_empty() {
        println!("✅ Login screen detected! Found keywords: {}", found_keywords.join(", "));
    } else {
        println!("⚠️  No typical login keywords detected");
    }
}