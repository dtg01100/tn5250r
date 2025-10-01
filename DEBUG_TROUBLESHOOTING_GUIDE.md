# Debug Mode and Troubleshooting Guide

## Overview

Added comprehensive debug and troubleshooting capabilities to TN5250R to help diagnose display issues, connection problems, and data processing errors.

## New Debug Features

### 1. Debug Mode (`--debug` or `-d`)

Enables verbose logging and activates the debug panel in the GUI.

```bash
cargo run --bin tn5250r -- --debug --server 10.100.200.1 --port 23 --user dave3 --password dave3
```

**What it does:**
- Enables verbose console logging for all operations
- Automatically shows the Debug Panel button (🐛) in the GUI
- Captures raw data dumps for inspection
- Displays detailed connection state information

### 2. Verbose Mode (`--verbose` or `-v`)

Enables detailed connection logging without full debug mode.

```bash
cargo run --bin tn5250r -- --verbose --server 10.100.200.1 --port 23
```

**What it does:**
- Detailed telnet negotiation logs
- Protocol state machine transitions
- Authentication flow details
- Less intrusive than full debug mode

## Debug Panel

When debug mode is enabled, click the **🐛 Debug** button in the top-right of the GUI to open the Debug Panel.

### Debug Panel Sections

#### 1. **Connection State**
- Connection status (connected/connecting/disconnected)
- Host and port information
- Configured credentials (password masked)
- Connection duration

#### 2. **Terminal Content**
- Total character count
- Number of lines
- First 500 characters (to see what's being received)
- Last 200 characters (to see most recent data)

#### 3. **Field Information**
- Number of detected fields
- Each field's properties:
  - Label
  - Current content
  - Active status
  - Highlight status
  - Error states (if any)

#### 4. **Raw Data Dump**
- Last packet size in bytes
- Hexadecimal dump of last received data
- Useful for protocol analysis

#### 5. **Controller State**
- Direct content from controller (bypasses GUI processing)
- Raw hex dump of controller buffer (first 1000 bytes)
- Useful for comparing what controller sees vs. what GUI displays

## Common Issues and Diagnostic Approaches

### Issue: Mangled Text in Display

**Symptoms:**
- Characters appear garbled or incorrect
- Box-drawing characters display as random symbols
- Screen layout is broken

**Diagnostic Steps:**

1. **Launch with debug mode:**
   ```bash
   cargo run --bin tn5250r -- --debug -s 10.100.200.1 -p 23 -u dave3 --pass dave3
   ```

2. **Open Debug Panel** (click 🐛 button)

3. **Check "Terminal Content" section:**
   - Are the first 500 characters readable ASCII?
   - Do you see ANSI escape sequences (e.g., `ESC[2J`, `ESC[H`)?
   - Are there unexpected control characters?

4. **Check "Raw Data Dump":**
   - Look for the hex pattern: `1B 5B` (ESC [) indicates ANSI mode
   - Look for the hex pattern: `FF FB/FC/FD/FE` indicates telnet negotiation
   - Look for pure text vs. binary data

5. **Check Console Output:**
   ```
   Controller: Detected ANSI/VT100 data - switching to ANSI mode
   DEBUG: Retrieved terminal content (660 chars)
   ```

**Possible Causes:**

- **ANSI not detected**: Data is ANSI/VT100 but wasn't recognized
  - **Fix**: Check ANSI detection logic in controller.rs (looks for ESC [)
  
- **Character encoding issues**: EBCDIC not properly converted to ASCII
  - **Fix**: Check translation tables in protocol_state.rs

- **Control characters not handled**: Special chars causing display issues
  - **Fix**: Check ANSI processor in ansi_processor.rs

- **Buffer corruption**: Data getting mangled during transmission
  - **Fix**: Check network layer in network.rs

### Issue: Nothing Displays After Connection

**Symptoms:**
- Connection succeeds
- Blank screen or "Negotiating..." message stays
- No terminal content appears

**Diagnostic Steps:**

1. **Launch with verbose mode:**
   ```bash
   cargo run --bin tn5250r -- --verbose -s 10.100.200.1 -p 23 -u dave3 --pass dave3
   ```

2. **Check console for:**
   ```
   CLI: Configured credentials for user: DAVE3
   Network: Credentials configured for telnet negotiation
   ```

3. **Open Debug Panel** and check:
   - **Connection State**: Should show "Connected: true"
   - **Terminal Content**: Should have character count > 0
   - **Raw Data Dump**: Should show data was received

4. **Check controller state:**
   - If "Controller content length" is > 0 but "Terminal content" is 0
   - Indicates GUI update problem
   - If both are 0, indicates no data received from server

**Possible Causes:**

- **Query command not sent**: Server waiting for initialization
  - **Fix**: Check that `send_initial_5250_data()` is called after auth
  
- **Authentication failed**: Server rejected credentials
  - **Fix**: Verify credentials with system administrator

- **Telnet negotiation incomplete**: Still negotiating options
  - **Fix**: Check telnet negotiation state machine

- **Data received but not processed**: Controller has data but GUI doesn't
  - **Fix**: Check `update_terminal_content()` method

### Issue: Connection Hangs or Times Out

**Symptoms:**
- "Connecting..." message never completes
- Application appears frozen
- No response from server

**Diagnostic Steps:**

1. **Use test_connection for raw output:**
   ```bash
   cargo run --bin test_connection -- --host 10.100.200.1 --port 23 --user dave3 --password dave3
   ```
   This bypasses GUI and shows raw protocol flow.

2. **Check firewall/network:**
   ```bash
   telnet 10.100.200.1 23
   ```
   Should connect and show telnet prompt.

3. **Try with debug mode:**
   ```bash
   cargo run --bin tn5250r -- --debug -s 10.100.200.1 -p 23
   ```
   Watch console for where connection stops.

**Possible Causes:**

- **Port blocked**: Firewall preventing connection
  - **Fix**: Check firewall rules, try different network

- **TLS mismatch**: Server expects TLS but `--ssl` not specified
  - **Fix**: Add `--ssl` flag

- **Server down**: AS/400 system offline or subsystem inactive
  - **Fix**: Verify server is running

### Issue: Fields Not Detected

**Symptoms:**
- Terminal content displays but field navigation doesn't work
- Tab key doesn't move cursor
- "Field Information" section empty

**Diagnostic Steps:**

1. **Open Debug Panel** and check "Field Information" section
   - If "Number of fields: 0", fields weren't detected

2. **Check console for:**
   ```
   DEBUG: Processed data in ANSI mode
   ```
   If in ANSI mode, field detection may work differently.

3. **Check "Terminal Content" for field markers:**
   - Look for underscores `_______` indicating input fields
   - Check for form layout structure

**Possible Causes:**

- **ANSI mode vs. 5250 mode**: Field detection different for each
  - **Fix**: Check field detection logic in field_manager.rs

- **Non-standard screen format**: Server sending unusual layout
  - **Fix**: May need custom field detection rules

- **Data stream incomplete**: Screen not fully received
  - **Fix**: Wait for complete screen transfer

## Verbose Console Logging

When running with `--debug` or `--verbose`, you'll see detailed logging:

### Connection Flow Logs:
```
CLI: Configured credentials for user: DAVE3
Controller: Configuring authentication for user: DAVE3
Network: Credentials configured for telnet negotiation
Controller: Detected ANSI/VT100 data - switching to ANSI mode
DEBUG: Retrieved terminal content (660 chars): '[screen content]'
DEBUG: Terminal content changed, updating GUI
```

### Data Processing Logs:
```
DEBUG: Received 660 bytes of data
DEBUG: First 50 bytes: [hex dump]
DEBUG: Processing data through ANSI processor
DEBUG: Processed data in ANSI mode
```

### Field Detection Logs:
```
Field Manager: Detected 5 fields on screen
Field 0: "User" at position (10, 5)
Field 1: "Password" at position (10, 7)
```

## Command-Line Options Summary

```bash
# Full debug with credentials
cargo run --bin tn5250r -- --debug -s HOST -p PORT -u USER --pass PASS

# Verbose logging only
cargo run --bin tn5250r -- --verbose -s HOST -p PORT

# Debug without auto-connect (to troubleshoot GUI)
cargo run --bin tn5250r -- --debug

# Combine with other options
cargo run --bin tn5250r -- --debug --ssl --insecure -s HOST -u USER --pass PASS
```

## Collecting Debug Information for Bug Reports

When reporting issues, include:

1. **Command used:**
   ```bash
   cargo run --bin tn5250r -- --debug -s 10.100.200.1 -p 23 -u dave3 --pass dave3
   ```

2. **Console output:**
   Redirect to file:
   ```bash
   cargo run --bin tn5250r -- --debug -s HOST -p PORT -u USER --pass PASS 2>&1 | tee debug.log
   ```

3. **Debug Panel screenshots:**
   - Connection State section
   - Terminal Content section (first/last chars)
   - Raw Data Dump section

4. **Expected vs. Actual:**
   - What you expected to see
   - What actually appeared

5. **Environment info:**
   ```bash
   rustc --version
   uname -a
   echo $DISPLAY
   ```

## Advanced Troubleshooting

### Capture Raw Network Traffic

Use tcpdump or Wireshark to capture the actual network packets:

```bash
# Capture on port 23
sudo tcpdump -i any -w tn5250r_capture.pcap port 23

# Then run your connection
cargo run --bin tn5250r -- -s HOST -p 23 -u USER --pass PASS
```

Open `tn5250r_capture.pcap` in Wireshark to see:
- Telnet option negotiation sequence
- Credential transmission (NEW-ENVIRON)
- Query command (0x04 0xF3)
- Server responses (5250 or ANSI data)

### Compare with test_connection

The `test_connection` binary is simpler and can help isolate issues:

```bash
# Test connection works
cargo run --bin test_connection -- --host HOST --port PORT --user USER --password PASS

# Main program has issues
cargo run --bin tn5250r -- -s HOST -p PORT -u USER --pass PASS
```

If `test_connection` works but `tn5250r` doesn't:
- Issue is in GUI layer or controller integration
- Check `update_terminal_content()` method
- Check ANSI processor vs. terminal screen rendering

If both fail:
- Issue is in network/protocol layer
- Check telnet negotiation
- Check authentication flow
- Check server compatibility

### Check ANSI Processing

The Debug Panel shows both processed and raw content:

1. **Terminal Content** = After ANSI processing (what GUI should display)
2. **Controller State** = Before ANSI processing (what was received)

Compare these to see if ANSI processor is working correctly.

### Inspect Field Manager

If fields aren't detected, check the field manager logic:

```bash
# Run with debug and watch console for field detection logs
cargo run --bin tn5250r -- --debug -s HOST -p PORT -u USER --pass PASS 2>&1 | grep -i field
```

## Known Issues

### 1. Mangled Text in GUI
- **Status**: Under investigation
- **Workaround**: Use debug panel to inspect raw content
- **Likely cause**: Character encoding or ANSI escape sequence handling

### 2. GUI Freezes on Connect
- **Status**: Rare occurrence
- **Workaround**: Use `--verbose` to see where it hangs
- **Likely cause**: Blocking operation in GUI thread

### 3. Field Tab Navigation Not Working
- **Status**: Known limitation
- **Workaround**: Click on fields directly
- **Cause**: Field detection for ANSI mode incomplete

## Future Improvements

Planned debug features:

1. **Packet logger**: Save all received packets to file
2. **Protocol analyzer**: Decode 5250/3270/ANSI in debug panel
3. **Performance metrics**: Track frame times, data rates
4. **Replay mode**: Load saved packets for offline debugging
5. **Screen differ**: Compare expected vs. actual screen output

## Build Status

✅ **Compiled successfully** (433 warnings, 0 errors)
```bash
cargo build --bin tn5250r
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.59s
```

## Testing Debug Mode

```bash
# Test debug mode activates
cargo run --bin tn5250r -- --debug
# Should print: "DEBUG MODE ENABLED: Verbose logging and data dumps active"
# GUI should show 🐛 Debug button

# Test verbose mode
cargo run --bin tn5250r -- --verbose
# Should print: "VERBOSE MODE: Detailed connection logs active"

# Test debug panel
cargo run --bin tn5250r -- --debug -s 10.100.200.1 -p 23 -u dave3 --pass dave3
# Click 🐛 button, should open debug panel with all sections
```

## Conclusion

Debug mode provides comprehensive visibility into the terminal emulator's operation, making it much easier to diagnose and fix issues. Use the Debug Panel to inspect connection state, terminal content, field information, and raw data dumps. Combine with console logging for complete troubleshooting coverage.
