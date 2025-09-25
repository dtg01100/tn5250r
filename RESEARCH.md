# Research on Existing 5250 Terminal Emulators

## TN5250 Project (C Implementation)

### Project Details:
- **URL**: https://sourceforge.net/projects/tn5250/
- **License**: GNU General Public License version 2.0 (GPLv2), GNU Library or Lesser General Public License version 2.0 (LGPLv2)
- **Programming Language**: C
- **Status**: Mature project active since 2001

### Key Features:
- 5250 TELNET CLIENT
- Emulates a 5250 terminal or printer over telnet
- Connects to IBM Power Systems, iSeries and AS/400 computers running IBM i, i5/OS and OS/400
- Runs on Unix or Windows systems
- Curses/Ncurses and Win32 user interface
- Supports BSD, Linux, and Windows operating systems
- Terminal emulator, telnet, and communications software

### Potential for Code Reuse:
- The C implementation is licensed under GPLv2, which is compatible with our GPL-2.0-or-later license
- Could serve as a reference for 5250 protocol implementation
- The project has been maintained for over 20 years and provides a proven implementation
- Source code could be ported to Rust, maintaining the same functionality

### Architecture Notes:
- Uses Curses/Ncurses for terminal interface on Unix systems
- Uses Win32 API for Windows interface
- Network communication handled through telnet protocols
- Handles 5250 data streams for communication with AS/400 systems

## TN5250J Project (Java Implementation)

### Project Details:
- **URL**: https://sourceforge.net/projects/tn5250j/
- **License**: GNU General Public License version 2.0 (GPLv2)
- **Programming Language**: Java
- **Status**: Cross-platform Java implementation

### Key Features:
- Pure Java implementation, providing cross-platform compatibility
- Full 5250 terminal emulation capabilities
- Support for various EBCDIC character sets
- Session management with saved profiles
- Print emulation capabilities
- Macro/scripting support
- Multiple session management
- Customizable keyboard mapping

### Potential for Code Reuse:
- Java implementation licensed under GPLv2, compatible with our license
- Could serve as reference for cross-platform GUI implementation
- Session management and configuration handling could be adapted
- Protocol implementation could be ported to Rust
- UI patterns and user experience concepts could be applied

## Protocol Specifications

### RFCs for 5250 Telnet:
- RFC 1205: TN3270 Current Practices
- RFC 2877: Definition of a Character Repertoire for the Telnet 5250 Protocol
- RFC 4777: Problem Analysis for the Telnet 5250 Protocol

These RFCs define the 5250 Telnet protocols which are foundational to open source implementations and should be referenced during development.

## Key Implementation Considerations

### 5250 Protocol Elements:
- Data stream format and parsing
- Keyboard response handling
- Screen buffer management
- Field attribute handling
- Character set conversion (EBCDIC/ASCII)
- Connection management and session handling

### Security:
- SSL/TLS support for secure connections
- Certificate handling
- Encryption of sensitive data

### User Experience:
- Terminal appearance customization
- Keyboard mapping configuration
- Session history and favorites
- Connection profile management

## Recommendations for TN5250R

1. Use the original tn5250 C project as the primary reference for protocol implementation
2. Use TN5250J as reference for cross-platform GUI and user experience patterns
3. Study the RFCs to ensure protocol compliance
4. Port key algorithms and data structures from C/Java implementations to Rust
5. Maintain compatibility with existing AS/400 systems and configurations