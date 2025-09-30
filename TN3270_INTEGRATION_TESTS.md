# TN3270 Integration Tests Documentation

## Overview

This document provides comprehensive documentation for the TN3270 integration tests, including test scenarios, expected outcomes, usage examples, known limitations, and troubleshooting guidance.

## Test Files

### 1. `tests/tn3270_integration.rs`
Comprehensive integration test suite validating TN3270 protocol implementation and UI component integration.

### 2. `src/bin/tn3270_test.rs`
Demonstration binary showcasing TN3270 functionality with example usage patterns.

## Test Coverage

### Protocol Initialization Tests

#### `test_protocol_processor_initialization()`
**Purpose**: Verify TN3270 protocol processor initializes correctly.

**Expected Behavior**:
- Processor creates successfully
- Protocol name is "TN3270"
- Initial state is connected

**Usage**:
```rust
let processor = ProtocolProcessor3270::new();
assert_eq!(processor.protocol_name(), "TN3270");
assert!(processor.is_connected());
```

#### `test_14bit_addressing_mode()`
**Purpose**: Validate 14-bit addressing for larger screen sizes.

**Expected Behavior**:
- Processor accepts 14-bit addressing mode
- Works with Model 4 (43x80) and Model 5 (27x132) screens
- Correctly processes data streams with extended addressing

### Protocol Mode Switching Tests

#### `test_protocol_mode_switching()`
**Purpose**: Ensure seamless switching between TN5250 and TN3270 protocols.

**Expected Behavior**:
- ProtocolType converts correctly to/from strings
- Protocol mode conversion works bidirectionally
- Invalid protocols are rejected with clear error messages

**Example**:
```rust
let tn3270 = ProtocolType::TN3270;
assert_eq!(tn3270.to_str(), "tn3270");
assert_eq!(tn3270.to_protocol_mode(), ProtocolMode::TN3270);
```

### Configuration Tests

#### `test_configuration_loading()`
**Purpose**: Validate configuration loading and protocol settings.

**Expected Behavior**:
- Configuration accepts valid protocol modes
- Terminal types are validated
- Protocol/terminal combinations are checked

**Valid Protocol Modes**:
- `auto` - Auto-detection (default)
- `tn5250` - Force TN5250 protocol
- `tn3270` - Force TN3270 protocol

**Valid 3270 Terminal Types**:
- IBM-3278-2 (24x80)
- IBM-3279-2 (24x80 with color)
- IBM-3279-3 (32x80 with color)
- IBM-3278-3 (32x80)
- IBM-3278-4 (43x80)
- IBM-3278-5 (27x132)

#### `test_configuration_validation_errors()`
**Purpose**: Verify configuration validation catches invalid combinations.

**Expected Behavior**:
- TN3270 protocol with 5250 terminal type fails validation
- TN5250 protocol with 3270 terminal type fails validation
- Auto mode accepts any terminal type
- Clear error messages explain validation failures

### Protocol Detection Tests

#### `test_protocol_detection_3270()`
**Purpose**: Verify auto-detection correctly identifies TN3270 data streams.

**Expected Behavior**:
- 3270 command codes are recognized
- Data stream parsing succeeds
- Display state updates correctly

**Example 3270 Data Stream**:
```rust
let data = vec![
    CMD_WRITE,           // 3270 Write command
    WCC_RESTORE,         // Write Control Character
    ORDER_SF,            // Start Field order
    ATTR_PROTECTED,      // Field attribute
    0xC1, 0xC2, 0xC3,   // ABC in EBCDIC
];
```

### Display Buffer Tests

#### `test_display_buffer_operations()`
**Purpose**: Validate display buffer operations for all screen sizes.

**Screen Sizes Tested**:
| Model | Rows | Cols | Buffer Size |
|-------|------|------|-------------|
| Model 2 | 24 | 80 | 1,920 |
| Model 3 | 32 | 80 | 2,560 |
| Model 4 | 43 | 80 | 3,440 |
| Model 5 | 27 | 132 | 3,564 |

**Expected Behavior**:
- Each screen size reports correct dimensions
- Buffer size matches rows × columns
- Coordinate conversion works correctly

### Command Processing Tests

#### `test_write_command()`
**Purpose**: Verify Write command processing.

**Expected Behavior**:
- WCC (Write Control Character) bits are processed
- Keyboard unlock works (WCC_RESTORE)
- Alarm setting works (WCC_ALARM)
- Data is written to buffer

#### `test_erase_write_command()`
**Purpose**: Verify Erase/Write command clears buffer.

**Expected Behavior**:
- Buffer is cleared before writing
- Cursor resets to position 0
- New data is written correctly

#### `test_set_buffer_address()`
**Purpose**: Verify SBA (Set Buffer Address) order.

**Expected Behavior**:
- Cursor moves to specified address
- Subsequent data writes at new position
- 12-bit and 14-bit addressing both work

### Field Management Tests

#### `test_start_field_order()`
**Purpose**: Verify Start Field (SF) order creates fields.

**Expected Behavior**:
- Field is added to field manager
- Field attributes are set correctly
- Protected/unprotected status is tracked

#### `test_field_attributes()`
**Purpose**: Verify field attribute handling.

**Field Attributes Tested**:
- ATTR_PROTECTED - Field is read-only
- ATTR_NUMERIC - Field accepts only numbers
- ATTR_MODIFIED - Field has been changed
- ATTR_INTENSIFIED - Field is highlighted

### Response Generation Tests

#### `test_read_buffer_response()`
**Purpose**: Verify Read Buffer response generation.

**Expected Behavior**:
- Response includes AID (Attention Identifier)
- Cursor address is encoded (2 bytes)
- Entire buffer contents are included

**Response Format**:
```
[AID byte][Cursor High][Cursor Low][Buffer Data...]
```

#### `test_read_modified_response()`
**Purpose**: Verify Read Modified response generation.

**Expected Behavior**:
- Response includes AID and cursor address
- Only modified fields are included
- MDT (Modified Data Tag) bit is checked

### Error Handling Tests

#### `test_error_handling_missing_data()`
**Purpose**: Verify graceful error handling for malformed data.

**Expected Behavior**:
- Missing WCC byte returns error
- Missing attribute byte returns error
- Error messages are descriptive

#### `test_invalid_configuration_handling()`
**Purpose**: Verify configuration validation.

**Expected Behavior**:
- Invalid protocol modes are rejected
- Invalid terminal types are rejected
- Error messages explain the problem

### Integration Tests

#### `test_complete_3270_session()`
**Purpose**: Simulate a complete 3270 session.

**Session Flow**:
1. Erase screen and write header
2. Position cursor and create input fields
3. Verify display state
4. Generate responses

**Expected Behavior**:
- All commands process successfully
- Fields are created correctly
- Keyboard state is managed properly
- Responses are generated correctly

## Running the Tests

### Run All Integration Tests
```bash
cargo test --test tn3270_integration
```

### Run Specific Test
```bash
cargo test --test tn3270_integration test_protocol_initialization
```

### Run with Output
```bash
cargo test --test tn3270_integration -- --nocapture
```

### Run Demonstration Binary
```bash
cargo run --bin tn3270_test
```

## Example Usage Patterns

### Basic Protocol Usage

```rust
use tn5250r::lib3270::{Display3270, ProtocolProcessor3270};

// Create display and processor
let mut display = Display3270::new();
let mut processor = ProtocolProcessor3270::new();

// Process incoming data
let data = vec![/* 3270 data stream */];
processor.process_data(&data, &mut display)?;

// Get display content
let content = display.to_string();
```

### Configuration Setup

```rust
use tn5250r::config::SessionConfig;

// Create configuration
let mut config = SessionConfig::new(
    "config.json".to_string(),
    "my_session".to_string()
);

// Set TN3270 protocol
config.set_protocol_mode("tn3270")?;
config.set_terminal_type("IBM-3278-2")?;

// Validate configuration
config.validate_protocol_terminal_combination()?;
```

### Controller Integration

```rust
use tn5250r::controller::{TerminalController, ProtocolType};

// Create controller
let mut controller = TerminalController::new();

// Connect with specific protocol
controller.connect_with_protocol(
    "mainframe.example.com".to_string(),
    23,
    ProtocolType::TN3270,
    None // TLS override
)?;

// Get protocol mode
let mode = controller.get_protocol_mode();
```

## Known Limitations

### Phase 2 Implementation Status

**Implemented**:
- ✅ Core 3270 command codes
- ✅ Basic and extended field attributes
- ✅ Buffer addressing (12-bit and 14-bit)
- ✅ Screen buffer management
- ✅ Multiple screen sizes
- ✅ Configuration integration

**Not Yet Implemented** (Phase 3):
- ⏳ TN3270E telnet negotiation
- ⏳ Device type negotiation
- ⏳ Session establishment
- ⏳ Error recovery

**Not Yet Implemented** (Phase 4):
- ⏳ Structured fields
- ⏳ Color and highlighting
- ⏳ Graphics support
- ⏳ Printer support

### Current Limitations

1. **Read Modified**: Currently returns empty modified data. Full implementation pending.

2. **Character Attributes**: Set Attribute (SA) order is parsed but not fully applied to individual characters.

3. **Modify Field**: MF order is parsed but field modification logic is incomplete.

4. **Program Tab**: PT order advances cursor but doesn't properly tab to next unprotected field.

5. **Structured Fields**: WSF command is recognized but structured field processing is not implemented.

## Troubleshooting Guide

### Test Failures

#### "Protocol processor not connected"
**Cause**: Processor initialization failed.
**Solution**: Check that processor is created with `new()` before use.

#### "Invalid protocol type"
**Cause**: Attempting to use unsupported protocol string.
**Solution**: Use valid protocol strings: "auto", "tn5250", "tn3270", "nvt".

#### "Protocol/terminal mismatch"
**Cause**: Invalid combination of protocol and terminal type.
**Solution**: 
- Use 3270 terminal types (IBM-3278-x, IBM-3279-x) with TN3270
- Use 5250 terminal types (IBM-3179-2, etc.) with TN5250
- Or use "auto" protocol mode

#### "Buffer address out of range"
**Cause**: Attempting to access position beyond buffer size.
**Solution**: Ensure addresses are within screen buffer size (rows × cols).

### Common Issues

#### Display Not Updating
**Check**:
1. Data stream is valid 3270 format
2. Commands are processed without errors
3. Display buffer is not locked

#### Fields Not Detected
**Check**:
1. Start Field (SF) orders are present
2. Field attributes are valid
3. Field manager is initialized

#### Configuration Not Loading
**Check**:
1. Configuration file path is correct
2. JSON format is valid
3. Property names match expected keys

## Performance Considerations

### Buffer Operations
- Buffer operations are O(1) for direct access
- Screen clearing is O(n) where n = buffer size
- Field detection is O(n) where n = number of fields

### Memory Usage
- Model 2 (24x80): ~2KB per display
- Model 3 (32x80): ~2.5KB per display
- Model 4 (43x80): ~3.5KB per display
- Model 5 (27x132): ~3.6KB per display

### Optimization Tips
1. Reuse display buffers when possible
2. Batch multiple commands in single data stream
3. Use appropriate screen size for application
4. Enable 14-bit addressing only for large screens

## Backward Compatibility

### TN5250 Compatibility
All tests verify that TN5250 functionality remains intact:
- TN5250 protocol mode still works
- 5250 terminal types are supported
- Protocol switching doesn't break existing code

### Migration Path
To migrate from TN5250 to TN3270:
1. Update configuration: `config.set_protocol_mode("tn3270")`
2. Update terminal type: `config.set_terminal_type("IBM-3278-2")`
3. Validate configuration: `config.validate_protocol_terminal_combination()`
4. Test with demonstration binary: `cargo run --bin tn3270_test`

## Future Enhancements

### Phase 3 (Session Management)
- Complete telnet negotiation for TN3270E
- Implement device type negotiation
- Add session establishment protocols
- Enhance error recovery mechanisms

### Phase 4 (Advanced Features)
- Implement structured field processing
- Add color and highlighting support
- Support graphics orders
- Add printer support

## Contributing

When adding new tests:
1. Follow existing test naming conventions
2. Include descriptive comments
3. Test both success and failure cases
4. Update this documentation
5. Verify backward compatibility

## References

- RFC 1205: 5250 Telnet Interface
- RFC 2355: TN3270 Enhancements
- IBM 3270 Data Stream Programmer's Reference
- tn5250j SessionConfig architecture

## Support

For issues or questions:
1. Check this documentation
2. Review test examples in `tn3270_test.rs`
3. Examine integration tests in `tn3270_integration.rs`
4. Consult protocol implementation in `src/lib3270/`