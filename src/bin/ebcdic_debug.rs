// TODO: EBCDIC conversion not yet implemented in new lib5250 port
// use tn5250r::lib5250::ebcdic_to_ascii;

// Placeholder EBCDIC to ASCII conversion - not accurate
fn ebcdic_to_ascii(ebcdic: u8) -> char {
    match ebcdic {
        0x40 => ' ',  // Space
        0x4a => '!',
        0x4f => '|',
        0x50..=0x59 => ('0' as u8 + (ebcdic - 0x50)) as char, // 0-9
        0x5a => '!',
        0x5b => '$',
        0x5c => '*',
        0x5d => ')',
        0x5e => ';',
        0x5f => '^',
        0x60 => '-',
        0x61 => '/',
        0x81..=0x89 => ('a' as u8 + (ebcdic - 0x81)) as char, // a-i
        0x91..=0x99 => ('j' as u8 + (ebcdic - 0x91)) as char, // j-r
        0xa2..=0xa9 => ('s' as u8 + (ebcdic - 0xa2)) as char, // s-z
        0xc1..=0xc9 => ('A' as u8 + (ebcdic - 0xc1)) as char, // A-I
        0xd1..=0xd9 => ('J' as u8 + (ebcdic - 0xd1)) as char, // J-R
        0xe2..=0xe9 => ('S' as u8 + (ebcdic - 0xe2)) as char, // S-Z
        _ => '?',  // Unknown character
    }
}

fn main() {
    println!("🔤 Debugging pub400.com EBCDIC data");
    println!("====================================");
    
    // The actual 5250 data bytes we received (excluding telnet commands)
    let data_bytes = [
        0x01, 0x03, 0x49, 0x42, 0x4d, 0x52, 0x53, 0x45, 0x45, 0x44, 0x62, 0xdd, 
        0x09, 0x24, 0x9d, 0x10, 0x6e, 0x6c, 0x00, 0x03
    ];
    
    println!("Raw hex data: {:02x?}", data_bytes);
    
    println!("\nByte-by-byte EBCDIC translation:");
    for &byte in data_bytes.iter() {
        let ascii_char = ebcdic_to_ascii(byte);
        let printable = if ascii_char.is_control() { 
            format!("\\x{:02x}", ascii_char as u8)
        } else { 
            ascii_char.to_string() 
        };
        println!("  0x{:02x} -> '{}' ({})", byte, printable, ascii_char as u8);
    }
    
    println!("\nFull translated string:");
    let translated: String = data_bytes.iter()
        .map(|&b| ebcdic_to_ascii(b))
        .collect();
    println!("'{}'", translated);
    
    println!("\nPrintable characters only:");
    let printable: String = data_bytes.iter()
        .map(|&b| ebcdic_to_ascii(b))
        .filter(|c| !c.is_control())
        .collect();
    println!("'{}'", printable);
    
    // Check some known IBM messages
    println!("\nChecking for known IBM messages:");
    
    // "SYSTEM" in EBCDIC
    let system_ebcdic = [0xE2, 0xE8, 0xE2, 0xE3, 0xC5, 0xD4];
    let system_ascii: String = system_ebcdic.iter()
        .map(|&b| ebcdic_to_ascii(b))
        .collect();
    println!("'SYSTEM' in EBCDIC: {:02x?} -> '{}'", system_ebcdic, system_ascii);
    
    // "WELCOME" in EBCDIC  
    let welcome_ebcdic = [0xE6, 0xC5, 0xD3, 0xC3, 0xD6, 0xD4, 0xC5];
    let welcome_ascii: String = welcome_ebcdic.iter()
        .map(|&b| ebcdic_to_ascii(b))
        .collect();
    println!("'WELCOME' in EBCDIC: {:02x?} -> '{}'", welcome_ebcdic, welcome_ascii);
    
    // "IBM" in EBCDIC
    let ibm_ebcdic = [0xC9, 0xC2, 0xD4];
    let ibm_ascii: String = ibm_ebcdic.iter()
        .map(|&b| ebcdic_to_ascii(b))
        .collect();
    println!("'IBM' in EBCDIC: {:02x?} -> '{}'", ibm_ebcdic, ibm_ascii);
}