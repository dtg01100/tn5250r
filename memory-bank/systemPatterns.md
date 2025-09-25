# TN5250R System Patterns

## System Architecture

### Layered Architecture
```
GUI Layer (egui/eframe)
    ↓
Controller Layer (AsyncTerminalController)
    ↓
Protocol Layer (lib5250)
    ↓
Network Layer (TCP/Telnet)
```

### Component Relationships
- **TN5250R App**: Main GUI application managing UI state and user input
- **AsyncTerminalController**: Thread-safe wrapper for synchronous terminal operations
- **TerminalController**: Core business logic coordinating protocol, terminal, and network
- **lib5250**: Modular protocol implementation (protocol, field, telnet)
- **TerminalScreen**: 80x24 character buffer with field attributes
- **AS400Connection**: TCP connection with telnet negotiation

## Key Technical Decisions

### Modular Protocol Design
- **Decision**: Separate lib5250 into protocol, field, and telnet modules
- **Rationale**: Matches original lib5250 structure, enables focused testing
- **Impact**: Easier maintenance, clearer separation of concerns

### Delegation Pattern for Integration
- **Decision**: TN5250R delegates to lib5250 rather than replacing entirely
- **Rationale**: Minimizes risk, allows gradual migration
- **Impact**: Existing API preserved, incremental testing possible

### Struct-Based Parsing
- **Decision**: Use structs (ProtocolParser, Field, TelnetNegotiator) over functions
- **Rationale**: Better state management, easier testing, Rust idiomatic
- **Impact**: More maintainable than C-style function-heavy approach

## Design Patterns in Use

### Builder Pattern
```rust
let parser = ProtocolParser::new()
    .with_ebcdic_table(ebcdic_table)
    .with_screen(&terminal_screen);
```

### State Machine Pattern
- Protocol state managed through enums and match statements
- Telnet negotiation follows RFC-defined state transitions
- Field processing uses attribute flags for state

### Error Handling Pattern
- `Result<T, String>` for controller operations
- `anyhow::Result` for complex error chains
- Graceful degradation on protocol errors

### Thread Safety Pattern
- `Arc<Mutex<T>>` for shared state between GUI and network threads
- Message passing via `mpsc::channel` for async communication
- Lock minimization to prevent deadlocks

## Component Interactions

### GUI ↔ Controller
- GUI sends user input (keys, mouse) to controller
- Controller returns terminal content updates
- Connection state changes trigger UI updates

### Controller ↔ Protocol
- Controller feeds network data to protocol parser
- Protocol parser updates terminal screen
- Field changes trigger controller notifications

### Protocol ↔ Network
- Network handles raw TCP data
- Protocol processes 5250 command streams
- Telnet negotiation coordinates connection setup

## Data Flow Patterns

### Input Processing
1. User input → GUI → Controller → Protocol encoding → Network send
2. Network receive → Protocol parsing → Screen update → GUI refresh

### Screen Updates
1. Protocol receives data → Parses commands → Updates TerminalScreen
2. Controller detects changes → Notifies GUI → GUI redraws

### Error Propagation
1. Network error → Controller Result::Err → GUI error display
2. Protocol parse error → Logged, continue processing (graceful degradation)
3. Field validation error → Input rejected, user notified