# TN5250R

TN5250R is a cross-platform desktop terminal emulator written in Rust, designed to connect to IBM AS/400 (now known as IBM i) systems. It emulates IBM 5250-series terminals and provides a modern, safe, and efficient implementation of the protocol and terminal emulation functionality.

## Features

- Cross-platform desktop application (Windows, macOS, Linux)
- IBM 5250 terminal emulation
- Secure connections to AS/400 systems
- Customizable appearance and keyboard mapping
- Session management

## Building

To build the project, you'll need Rust and Cargo installed:

```bash
# Clone the repository
git clone https://github.com/your-username/tn5250r
cd tn5250r

# Build in debug mode
cargo build

# Build in release mode
cargo build --release
```

## Running

```bash
# Run in debug mode
cargo run

# Run the release build
cargo run --release
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the GNU General Public License v2.0 or later (GPL-2.0-or-later) - see the [LICENSE](LICENSE) file for details.