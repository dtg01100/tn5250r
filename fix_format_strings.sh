#!/bin/bash

# Fix format strings in lib5250/session.rs
sed -i 's/format!("FIELD_{}", field_id)/format!("FIELD_{field_id}")/g' src/lib5250/session.rs
sed -i 's/format!("Presentation element data length {} exceeds available buffer", length)/format!("Presentation element data length {length} exceeds available buffer")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Presentation - Window definition ({} bytes)", length)/println!("5250: Presentation - Window definition ({length} bytes)")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Presentation - Menu definition ({} bytes)", length)/println!("5250: Presentation - Menu definition ({length} bytes)")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Presentation - Scrollbar definition ({} bytes)", length)/println!("5250: Presentation - Scrollbar definition ({length} bytes)")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Presentation - Selection field definition ({} bytes)", length)/println!("5250: Presentation - Selection field definition ({length} bytes)")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Presentation - Unknown element type: 0x{:02X} ({} bytes)", element_type, length)/println!("5250: Presentation - Unknown element type: 0x{element_type:02X} ({length} bytes)")/g' src/lib5250/session.rs

# Continue with more replacements
sed -i 's/println!("5250: Creating field list with {} fields", field_count)/println!("5250: Creating field list with {field_count} fields")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Insufficient data for field definition {}", i)/println!("5250: Insufficient data for field definition {i}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Field list read with {} fields", field_count)/println!("5250: Field list read with {field_count} fields")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Reading field list with {} fields", field_count)/println!("5250: Reading field list with {field_count} fields")/g' src/lib5250/session.rs

# Fix extended read operations
sed -i 's/println!("5250: Extended Read Immediate flags: 0x{:02X}", flags)/println!("5250: Extended Read Immediate flags: 0x{flags:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Read Immediate data byte: 0x{:02X}", data_byte)/println!("5250: Extended Read Immediate data byte: 0x{data_byte:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Read MDT flags: 0x{:02X}", flags)/println!("5250: Extended Read MDT flags: 0x{flags:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Read MDT data byte: 0x{:02X}", data_byte)/println!("5250: Extended Read MDT data byte: 0x{data_byte:02X}")/g' src/lib5250/session.rs

echo "Fixed format strings in lib5250/session.rs"
