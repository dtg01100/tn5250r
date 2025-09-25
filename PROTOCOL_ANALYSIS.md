# 5250 Protocol Analysis: Open Source Implementations

## Overview
Analysis of how mature open source 5250 terminal emulators handle protocol negotiation and handshake sequences, comparing tn5250j (Java) and hlandau/tn5250 (C) implementations with our TN5250R (Rust) implementation.

## Key Findings

### 1. Telnet Negotiation Constants

Both implementations use identical telnet protocol constants:

```c
#define IAC  255    // 0xFF - Interpret As Command
#define WILL 251    // 0xFB - WILL option
#define WONT 252    // 0xFC - WONT option  
#define DO   253    // 0xFD - DO option
#define DONT 254    // 0xFE - DONT option
#define SB   250    // 0xFA - Subnegotiation Begin
#define SE   240    // 0xF0 - Subnegotiation End
#define EOR  239    // 0xEF - End of Record

// Critical options for 5250 protocol
#define TRANSMIT_BINARY  0    // Binary transmission mode
#define END_OF_RECORD   25    // EOR option (0x19)
#define TERMINAL_TYPE   24    // Terminal type option (0x18)
#define NEW_ENVIRON     39    // New environment option (0x27)
#define TIMING_MARK      6    // Timing mark option
```

### 2. Essential Negotiation Sequence

All successful implementations follow this strict negotiation pattern:

#### Phase 1: Initial Connection
1. **Client connects** to server (typically port 23 or 992 for SSL)
2. **Server sends initial DO requests**:
   ```
   IAC DO NEW_ENVIRON (0xFF 0xFD 0x27)
   IAC DO TERMINAL_TYPE (0xFF 0xFD 0x18)
   IAC DO END_OF_RECORD (0xFF 0xFD 0x19)
   IAC DO TRANSMIT_BINARY (0xFF 0xFD 0x00)
   ```

#### Phase 2: Client Response
3. **Client responds with WILL confirmations**:
   ```
   IAC WILL NEW_ENVIRON (0xFF 0xFB 0x27)
   IAC WILL TERMINAL_TYPE (0xFF 0xFB 0x18)
   IAC WILL END_OF_RECORD (0xFF 0xFB 0x19)
   IAC WILL TRANSMIT_BINARY (0xFF 0xFB 0x00)
   ```

#### Phase 3: Subnegotiation
4. **Server requests terminal type**:
   ```
   IAC SB TERMINAL_TYPE SEND IAC SE
   ```
5. **Client sends terminal type**:
   ```
   IAC SB TERMINAL_TYPE IS "IBM-3179-2" IAC SE  (24x80)
   IAC SB TERMINAL_TYPE IS "IBM-3477-FC" IAC SE (27x132)
   ```
6. **Server may request environment variables**:
   ```
   IAC SB NEW_ENVIRON SEND IAC SE
   ```

### 3. Critical Implementation Details

#### A. Terminal Type Identification
Both implementations support multiple terminal types:

```java
// tn5250j approach
if (!support132)
    baosp.write("IBM-3179-2".getBytes());  // 24x80 terminal
else
    baosp.write("IBM-3477-FC".getBytes()); // 27x132 terminal
```

```c
// hlandau/tn5250 approach
if (!support132)
    termtype = "IBM-3179-2";
else
    termtype = "IBM-3477-FC";
```

#### B. Device Name Negotiation
**tn5250j** implements sophisticated device name negotiation:

```java
private void negNewEnvironment() throws IOException {
    baosp.write(IAC);
    baosp.write(SB);
    baosp.write(NEW_ENVIRONMENT);
    baosp.write(IS);
    
    if (kbdTypesCodePage != null) {
        baosp.write(USERVAR);
        baosp.write("KBDTYPE".getBytes());
        baosp.write(VALUE);
        baosp.write(kbdTypesCodePage.kbdType.getBytes());
        
        baosp.write(USERVAR);
        baosp.write("CODEPAGE".getBytes());
        baosp.write(VALUE);
        baosp.write(kbdTypesCodePage.codepage.getBytes());
    }
}
```

#### C. State Machine Implementation
Both use similar state machines for parsing telnet protocol:

```java
// tn5250j states
switch (bk.getOpCode()) {
    case 0: // No operation
        break;
    case 1: // Invite Operation
        parseIncoming();
        pendingUnlock = true;
        cursorOn = true;
        setInvited();
        break;
    case 2: // Output Operation
        // Handle output
        break;
}
```

```c
// hlandau/tn5250 states  
#define TN5250_STREAM_STATE_NO_DATA     0
#define TN5250_STREAM_STATE_DATA        1
#define TN5250_STREAM_STATE_HAVE_IAC    2
#define TN5250_STREAM_STATE_HAVE_VERB   3
#define TN5250_STREAM_STATE_HAVE_SB     4
#define TN5250_STREAM_STATE_HAVE_SB_IAC 5
```

### 4. Error Handling Patterns

#### Connection Timeout Handling
```c
// hlandau/tn5250 timeout handling
fd_set fdr;
struct timeval tv;
tv.tv_sec = 5;  // 5 second timeout
tv.tv_usec = 0;
TN_SELECT(This->sockfd + 1, &fdr, NULL, NULL, &tv);
if (!FD_ISSET(This->sockfd, &fdr))
    return -1;  // Timeout
```

#### IAC Byte Escaping
Critical for proper 5250 data transmission:

```c
// Escape IAC bytes in data stream
for (n = 0; n < length; n++) {
    c = data[n];
    tn5250_buffer_append_byte(&out, c);
    if (c == IAC)
        tn5250_buffer_append_byte(&out, IAC);  // Double IAC
}
```

### 5. 5250 Protocol Layer

After telnet negotiation, both implementations handle 5250-specific protocol:

#### A. Record Structure
```c
// 5250 record header format
typedef struct {
    uint16_t length;     // Record length
    uint16_t record_type; // Record type
    uint8_t flags;       // Flags
    uint8_t opcode;      // Operation code
} Record5250Header;
```

#### B. Common Operation Codes
```java
// tn5250j operation codes
static final int INVITE_OPERATION = 1;
static final int OUTPUT_OPERATION = 2;
static final int PUT_GET_OPERATION = 3;
static final int SAVE_SCREEN_OPERATION = 4;
```

### 6. Key Differences from Our Implementation

#### A. Our Current Approach vs Industry Standard

**Our TN5250R implementation:**
```rust
// Our telnet constants (correct)
const IAC: u8 = 0xFF;
const WILL: u8 = 0xFB;
const WONT: u8 = 0xFC;
const DO: u8 = 0xFD;
const DONT: u8 = 0xFE;

// Our negotiation (needs enhancement)
fn negotiate(&mut self, option: TelnetOption) -> Result<(), String> {
    match option {
        TelnetOption::Binary => {
            self.send_will(TelnetOption::Binary)?;
        }
        TelnetOption::EndOfRecord => {
            self.send_will(TelnetOption::EndOfRecord)?;
        }
        TelnetOption::SuppressGoAhead => {
            self.send_will(TelnetOption::SuppressGoAhead)?;
        }
    }
    Ok(())
}
```

#### B. Missing Features in Our Implementation

1. **Terminal Type Negotiation**: We don't send terminal type information
2. **NEW_ENVIRON Support**: Missing environment variable negotiation  
3. **Device Name Negotiation**: No device name handling
4. **Comprehensive State Machine**: Simpler state handling than mature implementations
5. **Proper IAC Escaping**: May not handle IAC byte escaping in data streams

### 7. Recommendations for TN5250R Enhancement

#### A. Immediate Improvements

1. **Add Terminal Type Negotiation**:
```rust
fn send_terminal_type(&mut self) -> Result<(), String> {
    let term_type = if self.support_132 {
        b"IBM-3477-FC"  // 27x132
    } else {
        b"IBM-3179-2"   // 24x80
    };
    
    self.send_subnegotiation(TelnetOption::TerminalType, &[
        TELNET_IS,
        term_type,
    ])?;
    Ok(())
}
```

2. **Enhanced State Machine**:
```rust
#[derive(Debug, Clone, PartialEq)]
enum TelnetState {
    Data,
    HaveIAC,
    HaveVerb(u8),
    HaveSB,
    HaveSBIAC,
}
```

3. **NEW_ENVIRON Support**:
```rust
fn negotiate_environment(&mut self) -> Result<(), String> {
    let env_data = [
        TELNET_IS,
        // Add KBDTYPE, CODEPAGE, etc.
    ];
    self.send_subnegotiation(TelnetOption::NewEnviron, &env_data)
}
```

#### B. Protocol Compatibility

Our implementation should match the negotiation sequence exactly:

```rust
pub fn initiate_5250_negotiation(&mut self) -> Result<(), String> {
    // Phase 1: Send WILL responses to DO requests
    self.send_will(TelnetOption::NewEnviron)?;
    self.send_will(TelnetOption::TerminalType)?;
    self.send_will(TelnetOption::EndOfRecord)?;
    self.send_will(TelnetOption::Binary)?;
    
    // Phase 2: Wait for subnegotiation requests
    // Handle TERMINAL_TYPE SEND
    // Handle NEW_ENVIRON SEND
    
    Ok(())
}
```

### 8. Testing Against Reference Implementations

#### A. Wireshark Analysis
Both reference implementations can be analyzed with Wireshark to capture exact negotiation sequences:

```bash
# Capture tn5250j negotiation
wireshark -i lo -f "port 23" &
java -jar tn5250j.jar your-system.com

# Capture hlandau/tn5250 negotiation  
tn5250 your-system.com
```

#### B. Protocol Compliance Testing
Our implementation should produce identical byte sequences to these mature implementations during the negotiation phase.

## Conclusion

Both tn5250j and hlandau/tn5250 follow RFC 2877 meticulously, implementing comprehensive telnet option negotiation before transitioning to 5250 protocol. Our TN5250R implementation has the core foundation correct but needs enhancement in:

1. **Terminal type negotiation** - Critical for proper 5250 session establishment
2. **Environment variable handling** - Required for device name assignment
3. **Comprehensive state machine** - Better parsing of telnet protocol sequences
4. **IAC escaping** - Proper handling of 0xFF bytes in data streams

The successful connection to 66.189.134.90:2323 indicates our basic protocol implementation works, but adding these enhancements would improve compatibility with a wider range of AS/400 systems and provide better protocol compliance.

## References

- **tn5250j**: https://github.com/tn5250j/tn5250j
- **hlandau/tn5250**: https://github.com/hlandau/tn5250
- **RFC 2877**: 5250 Telnet Enhancements
- **RFC 4777**: IBM's iSeries Telnet Enhancements