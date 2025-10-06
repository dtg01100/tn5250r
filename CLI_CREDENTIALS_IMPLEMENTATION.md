# CLI Credential Options - Implementation Complete

## Overview

Added command-line options `--user` and `--password` to the main TN5250R application, allowing users to specify AS/400 authentication credentials directly at launch. These credentials automatically populate the GUI fields and are used for auto-connect when combined with `--server`.

## New Command-Line Options

### `--user <username>` or `-u <username>`
Specifies the AS/400 username for RFC 4777 authentication.
- Automatically converts to uppercase (AS/400 convention)
- Populates the Username field in the GUI
- Used immediately if `--server` triggers auto-connect

### `--password <password>` or `--pass <password>`
Specifies the AS/400 password for RFC 4777 authentication.
- Sent as plain text unless TLS is enabled (use `--ssl` for security)
- Populates the Password field in the GUI (displayed as dots)
- Used immediately if `--server` triggers auto-connect

## Usage Examples

### Basic authentication with credentials:
```bash
cargo run --bin tn5250r -- --server as400.example.com --port 23 --user myuser --password myuser
```

### With TLS encryption (recommended for security):
```bash
cargo run --bin tn5250r -- --server secure.as400.com --port 992 --ssl --user admin --password mypass
```

### Launch GUI with credentials pre-filled (no auto-connect):
```bash
cargo run --bin tn5250r -- --user myuser --password mypass
```
In this case, the GUI opens with credentials already filled in, but you must click "Connect" manually after entering the server address.

### View all options:
```bash
cargo run --bin tn5250r -- --help
```

## Implementation Details

### Changes to `src/main.rs`

#### 1. CLI Argument Parsing
Added credential variables and parsing logic:
```rust
let mut cli_username: Option<String> = None;
let mut cli_password: Option<String> = None;

// Parse arguments
"--user" | "-u" => {
    if i + 1 < args.len() {
        cli_username = Some(args[i + 1].clone());
        i += 1;
    } else {
        eprintln!("Error: --user requires a username");
        std::process::exit(1);
    }
}
"--password" | "--pass" => {
    if i + 1 < args.len() {
        cli_password = Some(args[i + 1].clone());
        i += 1;
    } else {
        eprintln!("Error: --password requires a password");
        std::process::exit(1);
    }
}
```

#### 2. Function Signature Update
Modified `new_with_server()` to accept credentials:
```rust
fn new_with_server(
    _cc: &eframe::CreationContext<'_>, 
    server: String, 
    port: u16, 
    auto_connect: bool, 
    cli_ssl_override: Option<bool>,
    cli_username: Option<String>,      // NEW
    cli_password: Option<String>,      // NEW
) -> Self
```

#### 3. Credential Configuration Before Auto-Connect
If credentials are provided via CLI and auto-connect is enabled, they are configured before connection:
```rust
let username = cli_username.unwrap_or_default();
let password = cli_password.unwrap_or_default();

let connected = if auto_connect {
    // Set credentials before connecting
    if !username.is_empty() && !password.is_empty() {
        controller.set_credentials(&username, &password);
        println!("CLI: Configured credentials for user: {}", username);
    }
    
    // ... proceed with connection ...
}
```

#### 4. GUI Field Population
Credentials from CLI are stored in the `TN5250RApp` struct fields:
```rust
Self {
    // ...
    username,  // Use CLI credentials or empty string
    password,  // Use CLI credentials or empty string
    // ...
}
```

These fields are bound to the GUI text inputs, so CLI credentials appear pre-filled in the connection dialog.

#### 5. Updated Help Text
```
Options:
  --server <server> or -s <server>    Set the server to connect to and auto-connect
  --port <port> or -p <port>          Set the port to connect to (default: 23)
  --user <username> or -u <username>  AS/400 username for authentication (RFC 4777)
  --password <password> or --pass     AS/400 password for authentication (RFC 4777)
  --ssl | --no-ssl                    Force TLS on/off for this run (overrides config)
  --insecure                          Accept invalid TLS certs and hostnames (NOT recommended)
  --ca-bundle <path>                  Use a custom CA bundle (PEM or DER) to validate server certs
  --help or -h                        Show this help message

Example:
  tn5250r --server as400.example.com --port 23 --user myuser --password myuser
```

## Authentication Flow with CLI Credentials

When launching with full credentials:
```bash
cargo run --bin tn5250r -- --server as400.example.com --port 23 --user myuser --password myuser
```

The application performs this sequence:

1. **Parse CLI Arguments**: Extract username and password
2. **Initialize Controller**: Create `AsyncTerminalController`
3. **Configure Credentials**: Call `controller.set_credentials("myuser", "myuser")`
4. **Auto-Connect**: Since `--server` was provided:
   - Establish TCP connection to as400.example.com:23
   - Perform telnet option negotiation
   - Send RFC 4777 NEW-ENVIRON with credentials
   - Server authenticates the user
   - Send Query command (0x04 0xF3)
   - Receive and display ANSI/VT100 Sign-On screen
5. **GUI Display**: Username/password fields show the provided credentials

## Security Considerations

### Plain Text Passwords
⚠️ **Warning**: By default, passwords are sent in plain text per RFC 4777 (IBMRSEED empty).

**Mitigation**: Use TLS encryption:
```bash
cargo run --bin tn5250r -- --server as400.example.com --port 992 --ssl --user admin --password secret
```

### Command History Exposure
⚠️ **Warning**: Passwords in command-line arguments are visible in:
- Shell history (`~/.bash_history`)
- Process lists (`ps aux`)
- System logs

**Mitigations**:
1. **Disable history for sensitive commands**:
   ```bash
   HISTCONTROL=ignorespace
    cargo run --bin tn5250r -- --server as400.example.com --user myuser --password myuser
   # Note the leading space to prevent history recording
   ```

2. **Use environment variables**:
   ```bash
   export AS400_USER=myuser
   export AS400_PASS=myuser
   # Then modify code to read from env vars (future enhancement)
   ```

3. **GUI-only for production**: For production use, prefer entering credentials in the GUI where they're not exposed in process lists or history.

4. **Future enhancement**: Implement password prompt for interactive CLI usage:
   ```bash
   cargo run --bin tn5250r -- --server as400.example.com --user myuser --ask-password
   # Would prompt: "Password: " (hidden input)
   ```

## Testing

### Verify help text:
```bash
cargo run --bin tn5250r -- --help
# Should show --user and --password options with example
```

### Test credential auto-fill without connection:
```bash
cargo run --bin tn5250r -- --user testuser --password testpass
# GUI opens with username "testuser" and password "*******" visible
# No automatic connection occurs
```

### Test full auto-connect with credentials:
```bash
cargo run --bin tn5250r -- --server as400.example.com --port 23 --user myuser --password myuser
# Should connect immediately and display AS/400 Sign-On screen
```

### Test credential validation:
```bash
# Test missing username value
cargo run --bin tn5250r -- --user
# Error: --user requires a username

# Test missing password value  
cargo run --bin tn5250r -- --password
# Error: --password requires a password
```

## Build Status

✅ **Compilation**: Success (431 warnings, 0 errors)
```bash
cargo build --bin tn5250r
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.39s
```

✅ **Help Output**: Verified
```bash
cargo run --bin tn5250r -- --help
# Options displayed correctly with credential options
```

## Integration with Existing Features

The CLI credential options work seamlessly with:

- **GUI Fields**: CLI credentials auto-populate GUI text inputs
- **Manual Connection**: User can override CLI credentials in GUI before clicking Connect
- **Session Configuration**: Credentials don't persist to config files (security by design)
- **Auto-Connect**: Credentials used immediately when `--server` is specified
- **TLS/SSL**: Can combine `--user`/`--password` with `--ssl` for encrypted transmission
- **Multiple Sessions**: Each launch can use different credentials

## Files Modified

- `src/main.rs`:
  - Added `cli_username` and `cli_password` variable declarations
  - Added `--user`/`-u` and `--password`/`--pass` argument parsing
  - Updated `new_with_server()` signature to accept credential parameters
  - Added credential configuration before auto-connect
  - Updated help text with credential options and example
  - Modified app initialization to use CLI credentials or defaults

## Future Enhancements

Potential improvements for credential handling:

1. **Interactive Password Prompt**:
   ```bash
   cargo run --bin tn5250r -- --server host --user admin --ask-password
   # Prompts for password without echoing to terminal
   ```

2. **Environment Variable Support**:
   ```bash
   export TN5250R_USER=admin
   export TN5250R_PASSWORD=secret
   cargo run --bin tn5250r -- --server host --env-credentials
   ```

3. **Credential Files**:
   ```bash
   # ~/.tn5250r/credentials (encrypted)
   cargo run --bin tn5250r -- --server host --profile production
   ```

4. **Keyring Integration**:
   - Store credentials in system keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
   - Retrieve securely without CLI exposure

5. **Password Encryption**:
   - Implement DES/SHA encryption per RFC 4777
   - Add `--encryption-method des|sha` option

## Conclusion

CLI credential options provide a convenient way to launch TN5250R with authentication credentials for scripting, testing, and quick connections. The implementation maintains security by:
- Not persisting passwords to configuration files
- Supporting TLS encryption for transmission
- Allowing GUI override of CLI credentials
- Providing clear documentation of security considerations

For production use with sensitive credentials, users should prefer GUI input or implement one of the future enhancements for secure credential storage.
