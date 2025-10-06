# TN5250R Protocol Validation Test Plan

## Executive Summary

This document outlines the comprehensive testing strategy for validating the 47 identified issues in the TN5250R codebase. Testing will focus on establishing baseline behavior, documenting actual vs expected behavior, and creating reproducible test cases for regression testing.

**CRITICAL**: This is a **validation-only** phase. NO fixes will be applied during this subtask.

## Test Environment

**Target Servers:**
- **Primary**: pub400.com:23 (public AS/400 test system)
- **Secondary**: as400.example.com:23 (local test system if available)
- **Fallback**: Mock telnet server for controlled testing

**Test Tools:**
- Existing: [`src/bin/test_connection.rs`](src/bin/test_connection.rs:1)
- Existing: [`src/bin/protocol_test.rs`](src/bin/protocol_test.rs:1)
- New: Comprehensive protocol validation suite (to be created)
- External: `tcpdump`, `wireshark` for packet capture

## Test Categories

### Category 1: Connection Establishment (Priority: CRITICAL)

**Test 1.1: Basic TCP Connection**
- **Expected**: Connection establishes within 5 seconds
- **Test**: Connect to pub400.com:23 with timeout
- **Metrics**: Connection time, success rate
- **Issue Reference**: None (baseline)

**Test 1.2: TLS Connection (Port 992)**
- **Expected**: TLS handshake completes with valid certificate
- **Test**: Connect to pub400.com:992
- **Metrics**: Handshake time, certificate validation
- **Issue Reference**: Issue 3.1 (TLS Certificate Validation Bypass)

**Test 1.3: Connection Timeout Handling**
- **Expected**: Graceful timeout after 30 seconds
- **Test**: Connect to unreachable host
- **Metrics**: Timeout accuracy, resource cleanup
- **Issue Reference**: Issue 1.6 (Session Timeout Problems)

**Test 1.4: Protocol Detection - NVT vs 5250**
- **Expected**: Correct protocol detected within 256 bytes
- **Test**: Analyze initial server response
- **Metrics**: Detection accuracy, time to detection
- **Issue Reference**: Issue 4.1 (NVT Mode vs 5250 Protocol Confusion)

### Category 2: Telnet Negotiation (Priority: CRITICAL)

**Test 2.1: IAC Command Processing**
- **Expected**: All IAC sequences processed correctly
- **Test**: Send IAC WILL/WONT/DO/DONT sequences
- **Metrics**: Command recognition, response correctness
- **Issue Reference**: Issue 1.2 (Telnet Command Processing State Machine)

**Test 2.2: Option Negotiation Sequence**
- **Expected**: Binary, EOR, SGA negotiated successfully
- **Test**: Complete full negotiation handshake
- **Metrics**: Negotiation rounds, completion time
- **Issue Reference**: Issue 1.10 (Telnet Option Negotiation Logic)

**Test 2.3: Terminal Type Cycling**
- **Expected**: Terminal type sent as "IBM-3179-2"
- **Test**: TERMINAL-TYPE SEND command
- **Metrics**: Type string accuracy, cycling behavior
- **Issue Reference**: Issue 4.3 (Terminal Type Negotiation Issues)

**Test 2.4: Environment Variable Negotiation**
- **Expected**: DEVNAME, CODEPAGE, USER variables sent
- **Test**: NEW-ENVIRON SEND command
- **Metrics**: Variable completeness, format compliance
- **Issue Reference**: Issue 4.4 (Environment Variable Handling)

**Test 2.5: IAC Escaping in Binary Mode**
- **Expected**: IAC (0xFF) doubled in data stream
- **Test**: Send data containing 0xFF bytes
- **Metrics**: Escaping accuracy, data integrity
- **Issue Reference**: BUG_REPORT issue (IAC escaping)

**Test 2.6: Concurrent Negotiation Handling**
- **Expected**: Correct state transitions with overlapping negotiations
- **Test**: Multiple simultaneous option requests
- **Metrics**: State consistency, no deadlocks
- **Issue Reference**: Issue 1.2 (State Machine Errors)

### Category 3: Data Stream Parsing (Priority: HIGH)

**Test 3.1: Packet Structure Validation**
- **Expected**: Proper parsing of [CMD][SEQ][LEN][FLAGS][DATA]
- **Test**: Parse various packet structures
- **Metrics**: Parse success rate, error handling
- **Issue Reference**: Issue 1.3 (Buffer Overflow in Packet Processing)

**Test 3.2: Structured Field Processing**
- **Expected**: All SF types processed per RFC 2877
- **Test**: Send EraseReset, QueryCommand, SetReplyMode SFs
- **Metrics**: SF recognition, processing correctness
- **Issue Reference**: Issue 1.6 (Structured Field Length Validation)

**Test 3.3: WCC (Write Control Character) Parsing**
- **Expected**: Alarm, keyboard reset, clear operations work
- **Test**: Parse WCC with various bit combinations
- **Metrics**: Flag recognition, action execution
- **Issue Reference**: PROTOCOL_ANALYSIS.md (WCC handling)

**Test 3.4: Field Attribute Parsing**
- **Expected**: Correct interpretation of field attributes (bits 2-5)
- **Test**: Parse protected, numeric, skip, mandatory fields
- **Metrics**: Attribute accuracy, bit mask correctness
- **Issue Reference**: Issue 1.5 (Field Attribute Processing Logic)

**Test 3.5: Buffer Overflow Conditions**
- **Expected**: Rejection of oversized packets
- **Test**: Send packets with invalid length fields
- **Metrics**: Overflow detection, safe rejection
- **Issue Reference**: Issue 1.3, Issue 3.2 (Buffer Overflow Vulnerabilities)

**Test 3.6: Malformed Packet Handling**
- **Expected**: Graceful error handling, no crashes
- **Test**: Send truncated, corrupted packets
- **Metrics**: Error handling, system stability
- **Issue Reference**: Multiple security issues

### Category 4: EBCDIC Conversion (Priority: MEDIUM)

**Test 4.1: Character Set Completeness**
- **Expected**: All EBCDIC chars (0x00-0xFF) have mappings
- **Test**: Convert full EBCDIC range to ASCII
- **Metrics**: Coverage percentage, unmapped chars
- **Issue Reference**: Issue 1.1 (EBCDIC Character Conversion Errors)

**Test 4.2: Special Character Accuracy**
- **Expected**: 0x4B→'.', 0x6B→',', etc. per EBCDIC tables
- **Test**: Verify known character mappings
- **Metrics**: Mapping correctness, error count
- **Issue Reference**: Issue 1.1 (specific character errors)

**Test 4.3: Lowercase Character Range**
- **Expected**: s-z (0xA2-0xA9) correctly mapped
- **Test**: Convert lowercase alphabet
- **Metrics**: Range completeness
- **Issue Reference**: Issue 1.1 (missing s-z mappings)

**Test 4.4: Performance Benchmark**
- **Expected**: <100ns per character conversion
- **Test**: Convert large text blocks
- **Metrics**: Conversion rate, lookup table efficiency
- **Issue Reference**: Issue 2.2 (String Processing Inefficiency)

### Category 5: Session Management (Priority: MEDIUM)

**Test 5.1: Keyboard Lock States**
- **Expected**: Proper lock/unlock on field exit, errors
- **Test**: Trigger various lock conditions
- **Metrics**: State accuracy, unlock timing
- **Issue Reference**: Issue 1.8 (Keyboard State Management)

**Test 5.2: Screen Save/Restore**
- **Expected**: Full screen state preserved
- **Test**: Save, modify, restore screen
- **Metrics**: Data integrity, cursor position
- **Issue Reference**: Issue 1.9 (Save/Restore Functionality Bugs)

**Test 5.3: Partial Save/Restore Bounds**
- **Expected**: Coordinate validation, no buffer overruns
- **Test**: Save/restore with various coordinate ranges
- **Metrics**: Bounds checking, safety
- **Issue Reference**: Issue 1.9 (boundary errors)

**Test 5.4: Timeout Scenarios**
- **Expected**: Connection maintained with keepalive
- **Test**: Idle connection for extended period
- **Metrics**: Connection stability, timeout accuracy
- **Issue Reference**: Session timeout issues

**Test 5.5: Reconnection Logic**
- **Expected**: Clean reconnection after disconnect
- **Test**: Disconnect and reconnect
- **Metrics**: Reconnection time, state cleanup
- **Issue Reference**: Network layer issues

### Category 6: Security Vulnerabilities (Priority: CRITICAL)

**Test 6.1: Buffer Overflow Attack Vectors**
- **Expected**: All oversized inputs rejected safely
- **Test**: Malformed packets with invalid lengths
- **Metrics**: Rejection rate, crash prevention
- **Issue Reference**: Issue 3.2 (Buffer Overflow Vulnerabilities)

**Test 6.2: Information Disclosure via Errors**
- **Expected**: Generic error messages only
- **Test**: Trigger various error conditions
- **Metrics**: Information leakage, error message content
- **Issue Reference**: Issue 3.3 (Information Disclosure Through Errors)

**Test 6.3: Input Sanitization**
- **Expected**: Dangerous characters filtered/escaped
- **Test**: Input with control chars, escape sequences
- **Metrics**: Sanitization effectiveness
- **Issue Reference**: Issue 3.5 (Missing Input Sanitization)

**Test 6.4: Certificate Bundle Parsing**
- **Expected**: Only valid certificates accepted
- **Test**: Load malformed certificate bundles
- **Metrics**: Validation strictness, error handling
- **Issue Reference**: Issue 3.4 (Weak Certificate Bundle Parsing)

**Test 6.5: Random Number Predictability**
- **Expected**: Non-predictable sequence numbers
- **Test**: Generate sequence number distribution
- **Metrics**: Randomness quality, predictability
- **Issue Reference**: Issue 3.6 (Insecure Random Number Generation)

## Test Execution Strategy

### Phase 1: Baseline Establishment (Days 1-2)
1. Run existing test programs against pub400.com
2. Capture packet traces with tcpdump
3. Document current behavior (no judgment on correctness)
4. Establish performance baselines

### Phase 2: Critical Issue Validation (Days 3-4)
1. Test buffer overflow conditions
2. Test telnet state machine edge cases
3. Test protocol detection
4. Test connection timeouts

### Phase 3: High Priority Validation (Days 5-6)
1. Test data stream parsing
2. Test structured field handling
3. Test field attribute processing
4. Test EBCDIC conversion

### Phase 4: Medium Priority Validation (Days 7-8)
1. Test session management
2. Test keyboard states
3. Test save/restore functionality
4. Test performance metrics

### Phase 5: Documentation and Reporting (Days 9-10)
1. Compile all test results
2. Create reproduction steps for confirmed issues
3. Build regression test suite
4. Prioritize issues by severity

## Test Output Format

Each test will produce structured output:

```
TEST: [Test ID] [Test Name]
EXPECTED: [Expected behavior per RFC/spec]
ACTUAL: [Observed behavior]
STATUS: [PASS | FAIL | PARTIAL]
SEVERITY: [CRITICAL | HIGH | MEDIUM | LOW]
REPRODUCTION:
  1. [Step 1]
  2. [Step 2]
  ...
LOGS:
  [Relevant log output]
PACKET_TRACE:
  [tcpdump/wireshark capture if applicable]
ISSUE_REFERENCE: [Link to specific issue in BUG_REPORT]
```

## Success Criteria

1. **Coverage**: All 47 identified issues have corresponding test cases
2. **Reproducibility**: Each confirmed issue has clear reproduction steps
3. **Documentation**: Complete test results with expected vs actual behavior
4. **Baseline**: Performance metrics established for future comparison
5. **Regression Suite**: Automated tests ready for post-fix validation

## Deliverables

1. **Test Programs**:
   - Comprehensive protocol validation suite
   - Focused test programs for each category
   - Mock server for controlled testing

2. **Documentation**:
   - Test execution results
   - Confirmed issue list with severity
   - Reproduction steps for each issue
   - Packet captures for complex scenarios

3. **Regression Tests**:
   - Automated test suite
   - Expected output baselines
   - Performance benchmarks

## Risk Mitigation

- **Risk**: Tests may crash the application
  - **Mitigation**: Run in isolated environment, capture core dumps
  
- **Risk**: pub400.com may be unavailable
  - **Mitigation**: Have fallback test servers, mock server capability
  
- **Risk**: Tests may take longer than estimated
  - **Mitigation**: Prioritize CRITICAL issues first, time-box each phase

## Notes

- This is **validation only** - no fixes during this phase
- Focus on reproducible test cases
- Document everything, even "expected" failures
- Capture packet traces for complex scenarios
- Build regression test suite for future validation