use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

// Import the EBCDIC translation function
use tn5250r::protocol_state::ebcdic_to_ascii;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("TN5250R Connection Test");
    println!("Connecting to pub400.com:23...");
    
    // Attempt to connect
    let mut stream = TcpStream::connect("pub400.com:23")?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    
    println!("✅ Successfully connected to pub400.com:23");
    
    // Read initial data from the server
    let mut buffer = vec![0u8; 1024];
    
    match stream.read(&mut buffer) {
        Ok(0) => println!("❌ Connection closed by remote host"),
        Ok(n) => {
            println!("✅ Received {} bytes from server:", n);
            
            // Display the data in both hex and ASCII
            print!("Hex: ");
            for i in 0..n {
                print!("{:02x} ", buffer[i]);
                if i > 0 && (i + 1) % 16 == 0 {
                    println!();
                    print!("     ");
                }
            }
            println!();
            
            print!("ASCII: ");
            for i in 0..n {
                let ch = buffer[i];
                if ch >= 32 && ch <= 126 {
                    print!("{}", ch as char);
                } else {
                    print!(".");
                }
            }
            println!();
            
            // Test our EBCDIC translation on the received data
            println!("\nEBCDIC translation test:");
            print!("Translated: ");
            for i in 0..n {
                let ascii_char = ebcdic_to_ascii(buffer[i]);
                if ascii_char.is_ascii_graphic() || ascii_char == ' ' {
                    print!("{}", ascii_char);
                } else {
                    print!(".");
                }
            }
            println!();
        }
        Err(e) => println!("❌ Error reading from server: {}", e),
    }
    
    println!("\n🔌 Connection test complete");
    Ok(())
}