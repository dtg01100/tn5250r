# TN5250R Active Context

#### Telnet Module (`src/lib5250/telnet.rs`)
- Status: Core options implemented; subnegotiation indexing bug fixed and covered by tests. Advanced options available for future work.

### Primary Objective
Deepening the lib5250 Rust port implementation with real protocol, field, and telnet logic to achieve full feature parity with the original C library.

### Immediate Tasks
- Protocol: Continue aligning with canonical lib5250 codes; add structured field coverage as needed
- Field Detection: Incrementally expand attributes and navigation behavior
- Telnet: Keep Binary/EOR/SGA stable; stage NEW-ENVIRON and Terminal-Type when needed
- Testing: Maintain 100% passing; add new tests per feature

## Recent Changes

### Display Module (`src/lib5250/display.rs`) - COMPLETED
- **Complete lib5250 Port**: Full implementation of display.c functions (clear_unit, addch, set_cursor, erase_region, roll)
- **Session Integration**: Session successfully updated to use Display instead of direct TerminalScreen
- **Architecture Compliance**: Proper lib5250 session → display → terminal layering established
- **Compilation Success**: All display operations compile and integrate correctly
- **EBCDIC Handling**: Complete character translation following lib5250 patterns

### Session Module (`src/lib5250/session.rs`) - COMPLETED WITH STRUCTURED FIELDS
- **Complete Command Processing**: All 5250 protocol commands ported from session.c
- **Display Integration**: Session now properly calls Display methods instead of direct screen access
- **Structured Field Handling**: Session-level structured field processing for QueryCommand (0x84) → SetReplyMode (0x85) and SF_5250_QUERY (0x70)
- **Architecture Compliance**: Full session-level 5250 command processing following canonical lib5250 patterns
- **Protocol Refactoring**: Removed structured field handling from ProtocolProcessor, properly delegated to Session layer

### Protocol Module (`src/lib5250/protocol.rs`) - REFACTORED FOR PROPER SEPARATION
- **Packet-Level Processing**: ProtocolProcessor focuses on low-level packet handling and command routing
- **Structured Field Delegation**: WriteStructuredField commands now properly delegated to Session layer
- **Canonical Architecture**: Protocol layer handles packets, Session layer handles 5250 command semantics
- **Clean Separation**: Removed session-level logic from protocol layer following lib5250 architecture patterns

### Field Module (`src/lib5250/field.rs`)
- Created Field struct with position and attribute tracking
- Implemented basic underscore-based field detection
- Added detect_fields_from_screen() function
- Integrated with field_manager.rs

### Telnet Module (`src/lib5250/telnet.rs`)
- TelnetNegotiator with Binary, EOR, SGA; NEW-ENVIRON parsing available; subnegotiation parsing bug fixed

### Integration Points
- Updated protocol_state.rs to delegate parsing to lib5250::protocol
- Modified field_manager.rs to use lib5250::field for detection
- All existing TN5250R tests still pass (25/25)

### UI and Connection Flow (Non-blocking Connect) - COMPLETED
- Added `AsyncTerminalController::connect_async(host, port)` to perform blocking TCP connect and telnet negotiation on a background thread, returning immediately to the UI.
- Added connection progress and error tracking (`is_connecting`, `take_last_connect_error`).
- Updated GUI (`main.rs`) to call `connect_async` from `do_connect()` and show a connecting status; avoids blocking egui event loop.
- UI now transitions from "Connecting..." to "Negotiating..." upon connection and surfaces async errors if they occur.
- All tests remain green after the change.

### Final refinements to async connect and interactivity - COMPLETED
- Revised `connect_async` to construct and connect `AS400Connection` outside the controller lock and only acquire the lock to update controller state. This avoids holding the mutex during blocking I/O and eliminates first-connect contention.
- Fixed borrow-check issue by formatting the "Connected" message prior to taking the lock.
- Routed click activation through `TerminalController::activate_field_at_position`, which also updates the lib5250 `Display` cursor, ensuring UI cursor stays in sync with field activation.
- Re-ran full test suite: all tests passed across bins and integration tests.

### Connect cancel + timeout polish - COMPLETED
- Implemented user-initiated Cancel while "Connecting…" via `AsyncTerminalController::cancel_connect()` and a UI Cancel button.
- Added `AS400Connection::connect_with_timeout(timeout)` and updated async connect path to use explicit timeouts; improved UI error messaging to distinguish canceled vs timeout vs generic failure.
- Full `cargo test --all --no-fail-fast` run after these changes: all tests green.

## Active Decisions and Considerations

### Architecture Decisions
- **Delegation over Replacement**: Keeping TN5250R's existing structure and delegating to lib5250 modules rather than wholesale replacement
- **Struct-Based Design**: Using structs for parsers/negotiators to maintain state, following Rust best practices
- **Incremental Implementation**: Adding real logic gradually with tests to ensure stability

### Technical Considerations
- **EBCDIC Translation**: Need complete EBCDIC table for all character mappings
- **Structured Fields**: Complex 5250 commands require careful parsing of variable-length data
- **Thread Safety**: Ensuring lib5250 components work correctly in multi-threaded TN5250R environment
- **Error Handling**: Balancing robustness with performance in protocol parsing

## Next Steps

### Short Term (Next Session)
1. Solidify structured field parsing pathways in lib5250::protocol (start with Query/QueryReply and basic SF orders)
2. Expand field navigation and validation semantics
3. Add focused tests for new protocol branches, including timeout/cancel UI states where feasible
4. Keep telnet tests covering subnegotiation and EOR corner cases

### Medium Term
1. **Structured Field Parsing**: Handle complex 5250 structured fields
2. **Performance Optimization**: Profile and optimize hot paths
3. **Integration Testing**: Full end-to-end tests with real AS/400 connections
4. **Documentation Updates**: Update lib5250_PORTING.md with implementation details

### Long Term
1. **Feature Parity**: Ensure 100% compatibility with original lib5250
2. **Code Review**: Clean up and optimize implementation
3. **Migration Completion**: Remove old protocol code once lib5250 is proven

## Known Issues and Blockers

### Current Status
- ✅ lib5250 integration in use; legacy protocol module removed from build
- ✅ Tests passing, including telnet subnegotiation
- ✅ UI connect flow is now non-blocking; Wayland/GUI may not start in headless CI which is expected
- ⚠️ Structured fields and broader command surface pending as needed

### Potential Risks
- **Protocol Complexity**: 5250 structured fields are complex; parsing errors could cause connection issues
- **EBCDIC Edge Cases**: Some character mappings may need special handling
- **Performance Impact**: New Rust implementation must match or exceed C performance
- **Threading Issues**: Async integration needs careful testing

## Testing Strategy

### Current Testing
- Unit tests for each lib5250 module
- Integration tests ensuring TN5250R still works
- Manual testing with local telnet servers

### Planned Testing
- Comprehensive protocol parsing tests with real 5250 data
- Field detection accuracy tests
- Telnet negotiation compliance tests
- Performance benchmarks comparing old vs new implementation

## Communication Notes

### User Preferences
- Prefers incremental development with frequent testing
- Values documentation and clear progress tracking
- Interested in technical details and architectural decisions
- Open to Rust best practices and modern patterns

### Work Patterns
- Sessions focused on deepening one module at a time
- Tests run after each significant change
- Documentation updated to reflect current state
- Memory bank maintained for continuity