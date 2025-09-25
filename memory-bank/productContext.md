# TN5250R Product Context

## Why This Project Exists
TN5250R provides terminal emulation for connecting to IBM AS/400 systems, enabling users to interact with legacy business applications. The original implementation had protocol handling tightly coupled with the application logic, making it difficult to maintain and extend.

## Problems Solved
- **Maintainability**: Separating protocol logic into a dedicated lib5250 port makes the codebase more modular and easier to maintain
- **Safety**: Rust's memory safety guarantees prevent common C vulnerabilities present in the original lib5250
- **Performance**: Rust's zero-cost abstractions and efficient memory management improve performance
- **Future-proofing**: Modern Rust ecosystem provides better tooling, testing, and community support

## How It Should Work
1. **Connection**: User initiates connection to AS/400 system via telnet (port 23/992)
2. **Negotiation**: Telnet options are negotiated (Binary, EOR, SGA) to establish 5250 protocol
3. **Protocol Exchange**: 5250 commands flow bidirectionally, updating terminal screen
4. **Field Interaction**: User can navigate and edit input fields on the terminal
5. **Function Keys**: Special AS/400 function keys (F1-F24) send appropriate protocol commands

## User Experience Goals
- **Seamless Migration**: Existing TN5250R users should see no functional differences
- **Reliability**: Robust error handling prevents connection drops and data corruption
- **Performance**: Fast screen updates and responsive input handling
- **Compatibility**: Works with all standard AS/400 systems and configurations

## Key User Journeys
1. **System Connection**: Launch TN5250R → Enter host/port → Successful connection → Terminal screen displayed
2. **Screen Navigation**: Use arrow keys to move cursor → Tab between input fields → Enter data
3. **Function Key Usage**: Press F1-F12 for standard functions → F13-F24 for extended functions
4. **Session Management**: Handle connection errors gracefully → Reconnect automatically if possible

## Success Metrics
- Zero connection failures under normal conditions
- Sub-second response times for screen updates
- 100% protocol compatibility with existing AS/400 systems
- Clean, maintainable codebase for future enhancements