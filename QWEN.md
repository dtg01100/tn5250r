# TN5250R Project

## Project Overview
TN5250R is a cross-platform desktop terminal emulator written in Rust, designed to connect to IBM AS/400 (now known as IBM i) systems. It emulates IBM 5250-series terminals and provides a modern, safe, and efficient implementation of the protocol and terminal emulation functionality. The cross-platform nature means it should work on Windows, macOS, and Linux operating systems.

The project name suggests it's designed to be a Rust implementation of the traditional tn5250 client, providing benefits such as memory safety, performance, and modern development practices while maintaining compatibility with IBM iSeries/AS400 systems. This project draws inspiration from existing solutions like IBM's iAccess Client Solutions and TN5250J.

## License
TN5250R is licensed under the GNU General Public License version 2.0 (GPL-2.0) or later. This is a copyleft license that ensures the software and any derivative works remain open source. Users are free to use, modify, and distribute the software, provided that any distributed modifications are also licensed under the GPL.

## Open Source Inspiration and Code Reuse
TN5250R will leverage existing open source solutions as inspiration and for potential code porting where the licenses are compatible with the GPL-2.0-or-later license of this project. When incorporating code from other projects, proper attribution and license compliance will be maintained. Compatible licenses include:

- GPL-2.0-or-later (GNU General Public License v2.0 or later)
- GPL-3.0-or-later (GNU General Public License v3.0 or later)
- LGPL-2.1-or-later (GNU Lesser General Public License)
- MIT License
- BSD-2-Clause and BSD-3-Clause
- Apache License 2.0
- Mozilla Public License 2.0

The project will specifically look for existing implementations that can be adapted or ported to Rust, prioritizing those that offer:
- Proven 5250 protocol implementations
- Cross-platform compatibility
- Security best practices
- Robust terminal emulation
- Community support and maintenance

When code is ported from other languages to Rust, particular attention will be paid to:
- Maintaining security properties
- Leveraging Rust's memory safety features
- Improving performance where possible
- Following Rust idioms and conventions

## Project Structure
Currently, the project directory is minimal and may be in early development stages. The structure likely includes:
- `src/` - Rust source files (to be added)
- `Cargo.toml` - Rust package manifest (to be added)
- `README.md` - Project documentation (to be added)
- `assets/` - Application icons and other resources (to be added)
- `dist/` - Distribution and packaging configuration (to be added)

## Building and Running (Expected Structure)
Since this is a Rust project with a desktop GUI, the following commands would typically be used:

### Prerequisites
- Install Rust and Cargo: https://www.rust-lang.org/tools/install
- Additional dependencies may be required for GUI functionality depending on the chosen framework

### Building the Project
```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Build the desktop application
cargo build --release
```

For GUI development, common Rust frameworks include:
- `tao`/`iced` - Declarative UI with native look and feel
- `egui` - Immediate mode GUI with a Rust-native approach
- `dioxus` - Component-based UI framework similar to React
- `gtk-rs` - Rust bindings for GTK
- `winit` - Cross-platform window creation and management

### Running the Project
```bash
# Run the desktop application in debug mode
cargo run

# Package for distribution (if using tauri, iced, or similar)
cargo run --features dist
```

### Testing
```bash
# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Development Conventions
As a cross-platform Rust desktop project, this project would adhere to standard Rust conventions:
- Code formatting follows `rustfmt` standards
- All public APIs should be documented with `rustdoc`
- Follow the Rust naming conventions
- Use idiomatic Rust patterns and practices
- Memory safety and performance are key priorities
- Cross-platform compatibility considerations
- GUI accessibility standards
- Proper handling of different DPI settings and screen resolutions

## Inspiration and Reference Solutions

### IBM iAccess Client Solutions
IBM's iAccess Client Solutions provides a comprehensive set of client software that enables users to connect to IBM i (AS/400) systems. Key features include:

- **5250 Emulator**: Terminal emulation for AS/400 applications
- **File Transfer**: S/36, S/38, and AS/400 file transfer capabilities
- **Print Services**: Local and remote printing from AS/400 systems
- **Database Connectivity**: Access to DB2 for i databases
- **Secure Communications**: SSL/TLS encryption support
- **Session Management**: Multiple session handling
- **Keyboard Mapping**: Customizable keyboard layouts
- **Multiple Language Support**: Internationalization features

### TN5250J
TN5250J is an open-source Java-based 5250 terminal emulator that provides robust functionality for connecting to IBM i systems:

- **Cross-platform**: Written in Java, can run on Windows, macOS, and Linux
- **5250 Protocol Support**: Full implementation of the IBM 5250 protocol
- **Connection Management**: Multiple session support with saved profiles
- **Character Sets**: Support for various EBCDIC character sets
- **Print Emulation**: Local print capabilities
- **Macro Support**: Scripting and automation features
- **Customizable Interface**: Configurable appearance and behavior
- **Keypad Support**: Full function key and keypad mapping
- **Session Recording**: Option to record and replay sessions

## Expected Features
The TN5250R desktop application would likely include features such as:
- Terminal emulation for IBM 5250 protocols
- Connection to AS/400 systems with configurable settings
- Secure communications (TLS/SSL support)
- Keyboard mapping for AS/400 function keys
- Session management with saved connections
- Screen buffer handling
- Data stream parsing
- Character set conversion
- Support for various AS/400 display configurations
- Cross-platform GUI with native look and feel
- Connection history and favorites
- Print emulation
- File transfer capabilities
- Customizable appearance (colors, fonts, etc.)
- Macro/scripting support
- Multiple session management
- High DPI display support
- Accessibility features
- Session recording and playback

## Dependencies
Rust project dependencies would be managed in `Cargo.toml` and likely include:
- Networking libraries for connection handling (e.g., `tokio`, `async-std`)
- Terminal/user interface libraries for display
- Cryptographic libraries for secure connections
- Configuration parsing libraries (e.g., `serde`, `config`)
- Cross-platform GUI framework
- Cross-platform packaging tools (e.g., `tauri`, `cargo-bundle`, `cargo-deb`)

## Packaging and Distribution
For cross-platform desktop distribution, the project may use:
- `tauri` - Lightweight framework for creating desktop apps with web technologies
- `cargo-bundle` - Creates platform-specific application bundles
- `cargo-deb` - Creates Debian packages
- `wix` - Creates Windows installer packages
- macOS application bundles for distribution on macOS

## Usage Examples (Expected)
```bash
# Launch the desktop application
cargo run

# The application will provide a GUI for:
# - Creating and managing connections
# - Configuring terminal settings
# - Saving connection profiles
# - Managing sessions
```

## Project Status
This project appears to be in early development stages with the directory structure yet to be fully established. The focus on being a cross-platform desktop application will guide the technical decisions regarding GUI framework and packaging solutions.

## Remaining Items to be Figured Out

### Technical Architecture
- **GUI Framework Selection**: Choose the most appropriate Rust GUI framework for the cross-platform desktop application (tao/iced, egui, dioxus, etc.)
- **Network Protocol Implementation**: Detailed implementation of the 5250 protocol in Rust, possibly porting from existing open-source implementations
- **Threading Model**: Determining how to handle network I/O, UI updates, and background tasks safely in Rust
- **Memory Management**: Optimizing memory usage for terminal emulation, particularly for handling screen buffers

### Development Environment
- **Project Setup**: Creation of `Cargo.toml`, initial directory structure, and basic project scaffolding
- **Build System**: Configuration of build process for cross-platform compilation
- **Testing Strategy**: Development of comprehensive testing approach, including unit tests, integration tests, and protocol validation
- **CI/CD Pipeline**: Setting up continuous integration and deployment for multiple platforms

### Feature Implementation
- **5250 Protocol Details**: Complete implementation of all required 5250 data stream commands and responses
- **Character Set Support**: Handling various EBCDIC character sets used by different AS/400 systems
- **Keypad and Function Key Mapping**: Proper mapping of modern keyboards to IBM 5250 function keys
- **Print Emulation**: Implementation of local and remote printing capabilities
- **File Transfer**: Adding support for file transfer protocols compatible with AS/400
- **Security Implementation**: TLS/SSL support for secure connections
- **Session Management**: Saving and loading connection profiles and preferences

### User Interface
- **UI Design**: Creating mockups and design specifications for the desktop application
- **Accessibility**: Ensuring the application meets accessibility standards
- **Localization**: Planning for multilingual support
- **Customization Options**: Determining what appearance and behavior options to provide users

### Legal and Compliance
- **License Verification**: Ensuring all code from other projects has compatible licenses before incorporation
- **Attribution**: Properly attributing any code or concepts borrowed from other projects
- **Patent Considerations**: Researching any potential patent issues related to terminal emulation

### Distribution
- **Linux Distribution**: Available as AppImage (primary) with possible Flatpak support for different desktop environments
- **Windows Distribution**: Single executable (.exe) file for easy installation and portability
- **macOS Distribution**: Single application bundle (.app) or executable for simplified installation
- **Installation Process**: Planning the user experience for installation and updates
- **Dependencies Management**: Deciding how to handle system dependencies for different platforms