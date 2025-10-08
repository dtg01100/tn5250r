#!/bin/bash

# Fix cursor_utils.rs format strings
sed -i 's/format!("Invalid cursor position: ({}, {}) - coordinates must be 1-based", row, col)/format!("Invalid cursor position: ({row}, {col}) - coordinates must be 1-based")/g' src/cursor_utils.rs
sed -i 's/format!("Invalid cursor position: ({}, {}) - row exceeds terminal height ({})", row, col, TERMINAL_HEIGHT)/format!("Invalid cursor position: ({row}, {col}) - row exceeds terminal height ({TERMINAL_HEIGHT})")/g' src/cursor_utils.rs
sed -i 's/format!("Invalid cursor position: ({}, {}) - column exceeds terminal width ({})", row, col, TERMINAL_WIDTH)/format!("Invalid cursor position: ({row}, {col}) - column exceeds terminal width ({TERMINAL_WIDTH})")/g' src/cursor_utils.rs
sed -i 's/format!("Cursor position exceeds bounds: ({}, {}) - column >= {}", y, x, TERMINAL_WIDTH)/format!("Cursor position exceeds bounds: ({y}, {x}) - column >= {TERMINAL_WIDTH}")/g' src/cursor_utils.rs
sed -i 's/format!("Cursor position exceeds bounds: ({}, {}) - row >= {}", y, x, TERMINAL_HEIGHT)/format!("Cursor position exceeds bounds: ({y}, {x}) - row >= {TERMINAL_HEIGHT}")/g' src/cursor_utils.rs
sed -i 's/eprintln!("SECURITY: Invalid cursor position ({}, {}) - {} - out of bounds", row, col, context)/eprintln!("SECURITY: Invalid cursor position ({row}, {col}) - {context} - out of bounds")/g' src/cursor_utils.rs
sed -i 's/eprintln!("SECURITY: Attempted to access outside terminal bounds at ({}, {}) - {}", y, x, context)/eprintln!("SECURITY: Attempted to access outside terminal bounds at ({y}, {x}) - {context}")/g' src/cursor_utils.rs

# Fix component_utils.rs format strings
sed -i 's/format!("{} index {} out of bounds (max: {})", item_type, index, max)/format!("{item_type} index {index} out of bounds (max: {max})")/g' src/component_utils.rs
sed -i 's/format!("Invalid {} position: ({}, {})", position_type, row, col)/format!("Invalid {position_type} position: ({row}, {col})")/g' src/component_utils.rs
sed -i 's/format!("Component {} changed from {} to {}", component, from_state, to_state)/format!("Component {component} changed from {from_state} to {to_state}")/g' src/component_utils.rs
sed -i 's/format!("Insufficient data for {}: required {} bytes, got {}", operation, required, actual)/format!("Insufficient data for {operation}: required {required} bytes, got {actual}")/g' src/component_utils.rs
sed -i 's/format!("{} validation failed: {}", validation_type, details)/format!("{validation_type} validation failed: {details}")/g' src/component_utils.rs

# Fix lib5250/session.rs remaining format strings
sed -i 's/println!("5250: Start of output field - Length: {}", _length)/println!("5250: Start of output field - Length: {_length}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Unknown FCW type: 0x{:02X}, data: 0x{:02X}", fcw_type, fcw_data)/println!("5250: Unknown FCW type: 0x{fcw_type:02X}, data: 0x{fcw_data:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Transparent Data - processed {} bytes", length)/println!("5250: Transparent Data - processed {length} bytes")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Write Extended Attributes - {} bytes of attribute data", length)/println!("5250: Write Extended Attributes - {length} bytes of attribute data")/g' src/lib5250/session.rs
sed -i 's/format!("Extended attribute data length {} exceeds available data", attr_length)/format!("Extended attribute data length {attr_length} exceeds available data")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Write Display Structured Field - {} bytes of structured field data", length)/println!("5250: Write Display Structured Field - {length} bytes of structured field data")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Structured field type: 0x{:02X}", sf_type)/println!("5250: Structured field type: 0x{sf_type:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: SOH - Unknown header attribute byte: 0x{:02X}", attr_byte)/println!("5250: SOH - Unknown header attribute byte: 0x{attr_byte:02X}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Parsed extended attribute ID: 0x{:02X}, length: {}", attr_id, attr_length)/println!("5250: Parsed extended attribute ID: 0x{attr_id:02X}, length: {attr_length}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Defined extended attribute ID: 0x{:02X}, length: {}", attr_id, attr_length)/println!("5250: Defined extended attribute ID: 0x{attr_id:02X}, length: {attr_length}")/g' src/lib5250/session.rs
sed -i 's/println!("5250: Presentation - Unknown element type: 0x{:02X} ({} bytes)", element_type, length)/println!("5250: Presentation - Unknown element type: 0x{element_type:02X} ({length} bytes)")/g' src/lib5250/session.rs

echo "Fixed all remaining format strings"
