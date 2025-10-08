#!/bin/bash

# Fix remaining format strings in lib5250/session.rs
sed -i 's/println!("5250: Extended Read MDT Alternate flags: 0x{:02X}", flags)/println!("5250: Extended Read MDT Alternate flags: 0x{flags:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Read MDT Alternate data byte: 0x{:02X}", data_byte)/println!("5250: Extended Read MDT Alternate data byte: 0x{data_byte:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Read Screen Immediate flags: 0x{:02X}", flags)/println!("5250: Extended Read Screen Immediate flags: 0x{flags:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Read Screen Immediate data byte: 0x{:02X}", data_byte)/println!("5250: Extended Read Screen Immediate data byte: 0x{data_byte:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Save Screen flags: 0x{:02X}", flags)/println!("5250: Extended Save Screen flags: 0x{flags:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Save Screen data byte: 0x{:02X}", data_byte)/println!("5250: Extended Save Screen data byte: 0x{data_byte:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Save Partial Screen flags: 0x{:02X}", flags)/println!("5250: Extended Save Partial Screen flags: 0x{flags:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Save Partial Screen data byte: 0x{:02X}", data_byte)/println!("5250: Extended Save Partial Screen data byte: 0x{data_byte:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Restore Screen flags: 0x{:02X}", flags)/println!("5250: Extended Restore Screen flags: 0x{flags:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Restore Screen data byte: 0x{:02X}", data_byte)/println!("5250: Extended Restore Screen data byte: 0x{data_byte:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Restore Partial Screen flags: 0x{:02X}", flags)/println!("5250: Extended Restore Partial Screen flags: 0x{flags:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Restore Partial Screen data byte: 0x{:02X}", data_byte)/println!("5250: Extended Restore Partial Screen data byte: 0x{data_byte:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Roll flags: 0x{:02X}", flags)/println!("5250: Extended Roll flags: 0x{flags:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Roll data byte: 0x{:02X}", data_byte)/println!("5250: Extended Roll data byte: 0x{data_byte:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Write Structured Field flags: 0x{:02X}", flags)/println!("5250: Extended Write Structured Field flags: 0x{flags:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Write Structured Field data byte: 0x{:02X}", data_byte)/println!("5250: Extended Write Structured Field data byte: 0x{data_byte:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Read Text flags: 0x{:02X}", flags)/println!("5250: Extended Read Text flags: 0x{flags:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended Read Text data byte: 0x{:02X}", data_byte)/println!("5250: Extended Read Text data byte: 0x{data_byte:02X}")/g' src/lib5250/session.rs

# Fix format strings with multiple variables
sed -i 's/format!("Extended attribute data length {} exceeds available buffer", attr_length)/format!("Extended attribute data length {attr_length} exceeds available buffer")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended attribute ID: 0x{:02X}, length: {} defined", attr_id, attr_length)/println!("5250: Extended attribute ID: 0x{attr_id:02X}, length: {attr_length} defined")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Extended attribute ID: 0x{:02X}, length: {} added to list", attr_id, attr_length)/println!("5250: Extended attribute ID: 0x{attr_id:02X}, length: {attr_length} added to list")/g' src/lib5250/session.rs

echo "Fixed remaining format strings in lib5250/session.rs"
