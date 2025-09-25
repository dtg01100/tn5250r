# TN5250R Copilot Instructions

## Project Overview

TN5250R is a cross-platform IBM AS/400 terminal emulator written in Rust, implementing the IBM 5250 protocol for connecting to IBM i systems. The architecture separates GUI (egui/eframe), protocol handling, network communication, and terminal emulation into distinct layers.

## Architecture & Key Components

### Core Modules (src/)
- **`main.rs`**: egui-based GUI application with `TN5250RApp` struct managing UI state and `AsyncTerminalController`
- **`controller.rs`**: Orchestrates terminal/protocol/network layers via `TerminalController` and `AsyncTerminalController` 
- **`protocol.rs`**: RFC 2877/4777 compliant 5250 protocol implementation with `CommandCode` enum for protocol commands
- **`protocol_state.rs`**: State machine managing protocol negotiation, EBCDIC/ASCII translation, and field processing
- **`network.rs`**: TCP connection handling via `AS400Connection` with async message passing
- **`telnet_negotiation.rs`**: Telnet option negotiation for 5250 over telnet (Binary, EOR, SGA options)
- **`terminal.rs`**: Terminal emulation with 80x24 character grid, `TerminalScreen` buffer, and field attributes
- **`keyboard.rs`**: AS/400 function key mapping (F1-F24) with protocol byte conversion

### Key Patterns & Conventions

#### Protocol Implementation
- 5250 commands use hex constants: `WriteToDisplay = 0xF1`, `ReadBuffer = 0xF2`, etc.
- EBCDIC character translation via lookup tables in `protocol_state.rs`
- Field attributes: `Protected`, `Numeric`, `Intensified`, `NonDisplay` for AS/400 screen fields
- Standard terminal dimensions: `TERMINAL_WIDTH = 80`, `TERMINAL_HEIGHT = 24`

#### Error Handling
- Uses `Result<T, String>` for controller operations and `anyhow::Result` for complex errors
- Network errors converted to strings at controller boundary
- Protocol parsing returns `Ok(())` on success, descriptive error strings on failure

#### Async Architecture
- `AsyncTerminalController` wraps sync `TerminalController` with Arc<Mutex<>> for thread safety
- Network operations use `mpsc::channel` for async communication
- Background threads handle network I/O while GUI remains responsive

## Development Workflows

### Building & Running
```bash
cargo build                    # Debug build
cargo build --release        # Release build  
cargo run                     # Run with default connection dialog
cargo run -- --server HOST --port PORT  # Connect immediately to HOST:PORT
```

### Testing Protocol Changes
- Modify protocol constants in `protocol.rs` `CommandCode` enum
- Update corresponding parsing logic in `protocol_state.rs`
- Test with real AS/400 system or telnet server on port 23

### Adding New Function Keys
- Add variant to `FunctionKey` enum in `keyboard.rs`
- Implement `to_bytes()` conversion with correct 5250 protocol bytes
- Map to egui key in `map_virtual_key_to_function_key()`

## Integration Points

### GUI ↔ Controller
- `controller.connect(host, port)` returns `Result<(), String>`
- `controller.send_input(&[u8])` for raw data, `send_function_key(FunctionKey)` for special keys
- `controller.get_terminal_content()` returns formatted terminal display string

### Protocol ↔ Network  
- `AS400Connection` handles TCP with telnet negotiation
- Protocol state machine processes incoming 5250 data streams
- Outgoing data formatted according to 5250 protocol before network transmission

### Terminal ↔ Protocol
- `TerminalScreen` maintains character buffer with attributes
- Protocol writes to screen via cursor positioning and field attributes
- Screen changes trigger GUI redraw via dirty flag mechanism

## Critical Implementation Details

- **Character Encoding**: EBCDIC input converted to ASCII via translation tables before display
- **Telnet Negotiation**: Must negotiate Binary, End-of-Record, and Suppress-Go-Ahead options for proper 5250 communication
- **Function Keys**: F13-F24 map to special AS/400 functions (Field Exit, Help, Attention, etc.)
- **Connection State**: Always check `controller.is_connected()` before sending data
- **Thread Safety**: Use `Arc<Mutex<>>` pattern when sharing state between GUI thread and network threads

## Telnet Negotiation Specifics

### Required Options (RFC 2877)
```rust
// Critical telnet options for 5250 protocol
TelnetOption::Binary = 0,           // Must negotiate for 8-bit data
TelnetOption::EndOfRecord = 19,     // EOR marks end of 5250 records
TelnetOption::SuppressGoAhead = 3,  // Eliminates GA after each message
```

### Negotiation Sequence
1. Client sends `IAC WILL BINARY` and `IAC DO BINARY` 
2. Server responds with corresponding DO/WILL confirmations
3. EOR and SGA options negotiated similarly
4. Only after all options confirmed can 5250 data flow begin

## GUI State Management Patterns

### State Updates
- `TN5250RApp` holds connection state via `connected: bool` and `AsyncTerminalController`
- Terminal content cached in `terminal_content: String` for efficient redraws
- Input buffering in `input_buffer: String` before protocol conversion
- Function key visibility toggled via `function_keys_visible: bool`

### Thread Communication
```rust
// Async wrapper pattern for GUI thread safety
Arc<Mutex<TerminalController>> -> AsyncTerminalController
// Background network thread -> GUI updates via shared state
```

## Protocol Parsing Error Scenarios

### Common Parsing Failures
- **Invalid Command Codes**: `CommandCode::from_u8()` returns `None` for unknown bytes
- **Malformed Data Streams**: Incomplete structured fields cause parsing to halt
- **EBCDIC Translation Issues**: Unmapped EBCDIC codes default to null characters
- **Cursor Bounds**: Attempts to position cursor outside 80x24 grid are clamped

### Error Recovery Strategy
```rust
// Graceful degradation - continue parsing after errors
match protocol_state.process_data(buffer) {
    Ok(()) => continue_normal_operation(),
    Err(e) => {
        log_error(e);
        reset_to_known_state(); // Don't crash on bad data
    }
}
```

## Testing Strategies for AS/400 Connectivity

### Local Testing
```bash
# Test with telnet server simulation
nc -l 2323 &  # Simple TCP listener for connection testing
cargo run -- --server localhost --port 2323
```

### Real AS/400 Testing
```bash
# Production system testing (port 23 = standard telnet)
cargo run -- --server your-as400.company.com --port 23
# Alternative secure port testing
cargo run -- --server your-as400.company.com --port 992
```

### Protocol Validation
- Use Wireshark to capture 5250 data streams for analysis
- Compare byte sequences with working TN5250 implementations
- Test function key sequences (F1-F24) send correct protocol bytes
- Verify EBCDIC translation accuracy with known character sets

## Current Limitations (per IMPLEMENTATION_ROADMAP.md)

- Telnet option negotiation incomplete (needs full RFC compliance)
- Missing SSL/TLS support for secure connections  
- Environment variable negotiation (NEW-ENVIRON) not implemented
- Auto-signon with encrypted passwords not supported

When adding features, follow the existing modular pattern and ensure protocol changes maintain RFC 2877/4777 compliance.