# TN5250R

TN5250R is a cross-platform desktop terminal emulator written in Rust, designed to connect to IBM AS/400 (now known as IBM i) systems. It emulates IBM 5250-series terminals and provides a modern, safe, and efficient implementation of the protocol and terminal emulation functionality.

## Features

- Cross-platform desktop application (Windows, macOS, Linux)
- IBM 5250 terminal emulation
- Secure connections to AS/400 systems (TLS on port 992 by default)
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

### Secure (TLS) connections

TN5250R supports SSL/TLS for encrypted sessions:

- TLS is automatically enabled when connecting to port 992 (TN5250 over SSL standard port)
- TLS is disabled by default on port 23 (plain telnet)
- You can override this behavior programmatically by toggling TLS on the connection before connecting

Example (programmatic override):

```rust
use tn5250r::network::AS400Connection;

let mut conn = AS400Connection::new("your-as400.company.com".to_string(), 23);
conn.set_tls(true); // force TLS on non-SSL port
conn.connect()?;
```

You can also override TLS from the command line for the current run:

```bash
# Force TLS on
cargo run -- --server your-as400.company.com --port 992 --ssl

# Force TLS off
cargo run -- --server your-as400.company.com --port 23 --no-ssl
```

#### Advanced TLS options

You can further control certificate validation when using TLS:

- --insecure: Accept invalid TLS certificates and hostnames. This is NOT recommended for production but can help with testing or self-signed endpoints.
- --ca-bundle <path>: Provide a custom CA bundle (PEM or DER) to validate the server certificate.

Examples:

```bash
# Allow invalid/self-signed certificates (not recommended)
cargo run -- --server your-as400.company.com --port 992 --ssl --insecure

# Use a custom CA bundle (PEM or DER)
cargo run -- --server your-as400.company.com --port 992 --ssl --ca-bundle ./certs/ibmi-ca.pem
```

Notes:
- The CA bundle path can point to a single certificate file or a bundle containing multiple certs. PEM files with one or more certificates are supported; DER-encoded single certificates are also supported.
- When both a custom CA bundle is provided and --insecure is set, the insecure option takes precedence (certificate errors will be ignored).

### Configuration persistence

TN5250R persists session settings (host, port, TLS) to a JSON file and reloads them on startup. The UI automatically saves changes to host/port and the TLS checkbox.

Config file location preference order:
- TN5250R_CONFIG env var (absolute path)
- Linux: $XDG_CONFIG_HOME/tn5250r/session.json or ~/.config/tn5250r/session.json
- macOS: ~/Library/Application Support/tn5250r/session.json
- Windows: %APPDATA%/tn5250r/session.json
- Fallback: ./session.json

Persisted keys relevant to TLS:

- connection.ssl: boolean (true to enable TLS; defaults true on port 992, false otherwise)
- connection.tls.insecure: boolean (default: false)
- connection.tls.caBundlePath: string path to custom CA bundle (default: empty)

These can also be modified in the UI under Connection → TLS Options.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the GNU General Public License v2.0 or later (GPL-2.0-or-later) - see the [LICENSE](LICENSE) file for details.