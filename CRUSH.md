# CRUSH.md - TN5250R Project Commands & Style

## Commands

### Build
- `cargo build` (debug build)
- `cargo build --release` (optimized release build)
- `./test_fields.sh` (builds field_test binary for specific testing)

### Run
- `cargo run --bin tn5250r` (starts GUI with connection dialog)
- `cargo run --bin tn5250r -- --server pub400.com --port 23` (direct connect to test server)
- `./launch-gui.sh` (sets up GUI environment and launches; handles Xvfb if needed)
- `cargo run --bin field_test` (runs field navigation test CLI)

### Test
- `cargo test` (runs all unit/integration tests)
- `cargo test test_name --exact` (runs a single test by name; e.g., `cargo test test_cursor_position`)
- `cargo test -- --nocapture` (runs tests with output visible)
- `./test_fields.sh` (builds and runs field_test binary with real server connection; saves output to log)

### Lint & Format
- `cargo fmt` (formats code with rustfmt)
- `cargo clippy` (lints code; use `--fix` to auto-fix issues)
- `cargo check` (quick compilation check without building)

## Code Style Guidelines

### Naming Conventions
- Use snake_case for functions, variables, modules (e.g., `process_packet`, `field_manager`)
- PascalCase for types, structs, enums (e.g., `TerminalController`, `CommandCode`)
- Constants in UPPER_SNAKE_CASE (e.g., `TERMINAL_WIDTH = 80`)

### Imports & Organization
- Group imports: `use crate::module;` first, then `std::*`, then external crates (e.g., `tokio`, `egui`)
- Use `super::*` for parent module access; avoid wildcard imports where possible
- Organize logically: protocol/error imports at top, then UI/network

### Formatting
- Follow rustfmt defaults: 100-char line limit, braces on new lines, consistent indentation (4 spaces)
- No trailing whitespace; end files with newline

### Types & Error Handling
- Prefer `Result<T, String>` for simple operations (e.g., protocol parsing returns descriptive errors)
- Use `anyhow::Result` for complex error propagation (e.g., network I/O chains)
- Graceful error recovery: log errors and reset state (e.g., on malformed data); avoid panics in core logic
- Protocol data: `Vec<u8>` for raw streams; hex constants (e.g., `0xF1` for commands)

### Async & Threading
- Tokio runtime with `full` features for async (e.g., `async fn` in network/negotiation)
- Thread safety: `Arc<Mutex<T>>` for shared state (e.g., controller between GUI and network threads)
- Use `mpsc::channel` for inter-thread communication

### Protocol-Specific Patterns (from Copilot Instructions)
- EBCDIC translation: Use CP037 lookup tables (e.g., `ebcdic_to_ascii`)
- Field attributes: Bitmasks (e.g., `0x20` for protected fields)
- Telnet options: Negotiate Binary (0), EOR (19), SGA (3) per RFC 2877
- Terminal: 80x24 grid; cursor clamping for bounds

### Testing
- Unit tests in `#[cfg(test)] mod tests {}`; cover protocol parsing, field detection
- Integration: Use `pub` helpers like `add_field_for_test`; test with real AS/400 (pub400.com:23)
- Avoid external deps in tests; mock network with `tokio::test`

### Additional Notes (from Copilot Instructions)
- Modular architecture: Separate GUI (egui/eframe), protocol, network, terminal layers
- No secrets in code; check for sensitive info before commits
- Follow RFC 2877/4777 for 5250 compliance; test with Wireshark for validation
