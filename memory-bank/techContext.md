# TN5250R Technical Context

## Technologies Used

### Core Language
- **Rust 2021 Edition**: Memory-safe systems programming language
- **Cargo**: Package manager and build system
- **rustc**: Rust compiler with optimizations

### GUI Framework
- **egui**: Immediate mode GUI framework for Rust
- **eframe**: egui framework integration for desktop applications

### Networking
- **std::net::TcpStream**: Standard library TCP networking
- **std::sync::mpsc**: Multi-producer single-consumer channels for async communication

### Development Tools
- **VS Code**: Primary IDE with Rust extension
- **Git**: Version control with branching for experimental features
- **Dev Container**: Isolated development environment

## Development Setup

### Environment Requirements
- Linux (Debian-based container)
- Rust toolchain (latest stable)
- Git for version control
- VS Code with Rust extensions

### Build Configuration
```toml
[package]
name = "tn5250r"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = "0.22"
anyhow = "1.0"
```

### Test Setup
- Unit tests in each module
- Integration tests in `tests/` directory
- `cargo test` for running test suite
- Test coverage via `cargo tarpaulin` (future)

## Technical Constraints

### Memory Safety
- No unsafe code blocks allowed
- All data access bounds-checked
- Thread safety through Rust ownership system

### Performance Requirements
- Sub-second screen update times
- Minimal memory allocations during operation
- Efficient EBCDIC/ASCII translation

### Protocol Compliance
- RFC 2877: 5250 protocol over telnet
- RFC 4777: Enhanced telnet negotiation
- EBCDIC character set support

## Dependencies

### Runtime Dependencies
- **eframe**: GUI rendering and event handling
- **anyhow**: Error handling and propagation

### Development Dependencies
- **None currently**: Pure Rust implementation

### Future Dependencies
- **tokio** (potential): For advanced async networking
- **serde** (potential): For configuration serialization
- **tracing** (potential): For structured logging

## Build and Run Instructions

### Development Build
```bash
cargo build
cargo run
```

### Release Build
```bash
cargo build --release
```

### Testing
```bash
cargo test                    # All tests
cargo test --lib             # Library tests only
cargo test --bin             # Binary tests only
```

### Connection Testing
```bash
cargo run -- --server localhost --port 2323  # Test connection
```

## Platform Support

### Target Platforms
- **Linux**: Primary development and deployment platform
- **Windows**: Cross-compilation support via rustup
- **macOS**: Cross-compilation support via rustup

### Architecture Support
- **x86_64**: Primary architecture
- **ARM64**: Future Raspberry Pi support

## Code Quality Standards

### Linting
- `cargo clippy` for code quality checks
- `cargo fmt` for code formatting

### Testing Standards
- Unit test coverage for all public functions
- Integration tests for end-to-end functionality
- Property-based testing for protocol parsing (future)

### Documentation
- Rustdoc comments for all public APIs
- README.md for project overview
- lib5250_PORTING.md for implementation details