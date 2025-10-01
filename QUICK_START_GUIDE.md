# TN5250R Quick Start Guide

## Installation & Build

```bash
cd /workspaces/tn5250r
cargo build --release
```

## Usage Methods

### 1. GUI with Manual Connection (Default)

Launch the GUI and enter connection details manually:

```bash
cargo run --bin tn5250r
```

Then in the GUI:
- **Host**: Enter server address with port (e.g., `10.100.200.1:23`)
- **Username**: Enter your AS/400 username
- **Password**: Enter your password (displayed as dots)
- Click **Connect**

---

### 2. CLI Auto-Connect with Credentials

Connect immediately without GUI interaction:

```bash
cargo run --bin tn5250r -- \
  --server 10.100.200.1 \
  --port 23 \
  --user dave3 \
  --password dave3
```

**Short form:**
```bash
cargo run --bin tn5250r -- -s 10.100.200.1 -p 23 -u dave3 --pass dave3
```

---

### 3. Secure Connection with TLS

For encrypted connections (recommended):

```bash
cargo run --bin tn5250r -- \
  --server secure.as400.com \
  --port 992 \
  --ssl \
  --user admin \
  --password secret
```

---

### 4. GUI with Pre-Filled Credentials

Launch GUI with credentials already filled (no auto-connect):

```bash
cargo run --bin tn5250r -- --user myuser --password mypass
```

Then enter the server address and click Connect manually.

---

### 5. Test Connection (Standalone)

Use the test harness for debugging:

```bash
cargo run --bin test_connection -- \
  --host 10.100.200.1 \
  --port 23 \
  --user dave3 \
  --password dave3
```

This displays raw terminal output for troubleshooting.

---

## Common Scenarios

### Connecting to pub400.com (Free AS/400 Server)

```bash
cargo run --bin tn5250r -- \
  --server 10.100.200.1 \
  --port 23 \
  --user dave3 \
  --password dave3
```

### Connecting to IBM i with TLS

```bash
cargo run --bin tn5250r -- \
  --server ibmi.example.com \
  --port 992 \
  --ssl \
  --user QSECOFR \
  --password password
```

### Development/Testing with Insecure Certs

```bash
cargo run --bin tn5250r -- \
  --server dev.as400.local \
  --port 992 \
  --ssl \
  --insecure \
  --user developer \
  --password devpass
```

---

## Command-Line Options Reference

```
Usage: tn5250r [OPTIONS]

Connection Options:
  --server <server> or -s <server>    Set the server to connect to and auto-connect
  --port <port> or -p <port>          Set the port to connect to (default: 23)

Authentication Options (RFC 4777):
  --user <username> or -u <username>  AS/400 username for authentication
  --password <password> or --pass     AS/400 password for authentication

Security Options:
  --ssl | --no-ssl                    Force TLS on/off for this run (overrides config)
  --insecure                          Accept invalid TLS certs and hostnames (NOT recommended)
  --ca-bundle <path>                  Use a custom CA bundle (PEM or DER) to validate server certs

Help:
  --help or -h                        Show this help message
```

---

## Expected Behavior

### Successful Connection

When connection succeeds, you'll see:

```
TN5250R - IBM AS/400 Terminal Emulator
Connected to 10.100.200.1:23
Controller: Credentials configured for user: DAVE3
Network: Credentials configured for telnet negotiation
Controller: Detected ANSI/VT100 data - switching to ANSI mode

[AS/400 Sign-On Screen displayed]
System: S215D18V
Subsystem: QINTER
Display: QPADEV001L

User: _______
Password: _______
Program/procedure: _______
Menu: _______
Current library: _______
```

### Connection Failure

If connection fails:
```
Connection failed to start: <error details>
```

Common causes:
- Server unreachable (check network/firewall)
- Invalid port number
- Wrong credentials
- TLS negotiation failure

---

## Security Best Practices

### 1. Use TLS for Production

Always use `--ssl` when connecting over untrusted networks:
```bash
cargo run --bin tn5250r -- -s host -p 992 --ssl -u user --pass secret
```

### 2. Avoid Password in Command History

Use a leading space to prevent history recording (bash):
```bash
 cargo run --bin tn5250r -- -s host -u user --pass secret
#^ Note the space
```

Or use GUI input for sensitive credentials.

### 3. Clear Terminal After Use

Passwords may remain in scrollback buffer:
```bash
clear && history -c  # Clear screen and history
```

### 4. Use Dedicated Service Accounts

Don't use personal or high-privilege accounts (like QSECOFR) for automated connections.

---

## Troubleshooting

### GUI Doesn't Launch

**Symptom**: No window appears, or immediate exit

**Solution**: Ensure display environment is set (dev container):
```bash
export DISPLAY=:0
cargo run --bin tn5250r
```

### Connection Hangs

**Symptom**: "Connecting..." but never completes

**Possible causes**:
1. Firewall blocking port
2. Server requires TLS but `--ssl` not specified
3. Telnet negotiation failure

**Debug**:
```bash
# Try test_connection for verbose output
cargo run --bin test_connection -- --host HOST --port PORT --user USER --password PASS
```

### Authentication Fails

**Symptom**: Connection establishes but no sign-on screen appears

**Check**:
1. Username/password are correct
2. User profile is enabled on AS/400
3. User has authority to sign on
4. QINTER subsystem is active

### Invalid Credentials Message

**Symptom**: AS/400 shows "Not authorized to system"

**Solution**: Verify credentials with system administrator

---

## Advanced Usage

### Custom CA Bundle for Self-Signed Certs

```bash
cargo run --bin tn5250r -- \
  --server as400.internal \
  --port 992 \
  --ssl \
  --ca-bundle /path/to/ca-cert.pem \
  --user admin \
  --password secret
```

### Multiple Sessions

Launch multiple instances with different profiles:
```bash
# Terminal 1
cargo run --bin tn5250r -- -s prod.as400.com -u produser --pass prod123

# Terminal 2  
cargo run --bin tn5250r -- -s dev.as400.com -u devuser --pass dev456
```

---

## Next Steps

After connecting successfully:

1. **Navigate the AS/400 Screen**:
   - Type in input fields
   - Use Tab to move between fields
   - Press function keys (F1-F24) using GUI buttons

2. **Explore GUI Features**:
   - Click "⚙ Advanced" for connection settings
   - Toggle "Show Field Info" to see field metadata
   - Use function key panel for quick access

3. **Test Different Protocols**:
   - Try TN3270 mode for different systems
   - Experiment with auto-detection

---

## Documentation

For more detailed information, see:

- `CREDENTIAL_INTEGRATION_COMPLETE.md` - Full authentication implementation details
- `CLI_CREDENTIALS_IMPLEMENTATION.md` - CLI options and security considerations
- `CONNECTION_SUCCESS.md` - Technical authentication flow
- `IMPLEMENTATION_ROADMAP.md` - Planned features and enhancements

---

## Support & Contribution

**Report Issues**: Check console output for error messages and logs

**Test Server**: Use pub400.com (10.100.200.1:23) for testing - free AS/400 access with demo credentials

**Development**: See project README for contribution guidelines

---

**Last Updated**: October 1, 2025
**Version**: Development (post-credential integration)
