# TN5250R GUI Testing

This directory contains automated GUI tests for the TN5250R terminal emulator using the egui_kittest framework.

## Overview

The GUI testing infrastructure provides:
- **Component Tests**: Test individual UI elements (buttons, input fields, etc.)
- **Integration Tests**: Test complete user workflows (connection, navigation, etc.)
- **Visual Regression Tests**: Detect UI layout and appearance changes automatically
- **Mock Network Layer**: Deterministic testing without external AS/400 dependencies

## Quick Start

### Running All Tests
```bash
# Using the test runner script (recommended)
./run-gui-tests.sh

# Or directly with cargo
cargo test --test gui
```

### Running Specific Test Categories
```bash
# Component tests only
./run-gui-tests.sh components

# Integration tests only
./run-gui-tests.sh integration

# Visual regression tests only
./run-gui-tests.sh visual
```

### Updating Visual Snapshots
When visual changes are intentional, update the snapshots:
```bash
./run-gui-tests.sh update
```

## Test Structure

```
tests/
├── gui/
│   ├── main.rs                 # Main test entry point
│   ├── components/             # Component-level tests
│   │   ├── main.rs
│   │   └── test_ui_components.rs
│   ├── integration/            # Integration tests
│   │   ├── main.rs
│   │   └── test_connection.rs
│   └── visual/                 # Visual regression tests
│       ├── main.rs
│       └── test_visual_regression.rs
├── mocks/                      # Mock implementations
│   └── mock_network.rs
└── utils/                      # Test utilities
    └── test_harness.rs
```

## Test Harness

The `TN5250RHarness` provides methods for GUI interaction:

```rust
use tn5250r::tests::utils::test_harness::TN5250RHarness;

let mut harness = TN5250RHarness::new();

// Click elements by text
harness.click_by_text("Connect").unwrap();

// Type text
harness.type_text("test.as400.com").unwrap();

// Press keys
harness.press_enter().unwrap();

// Take snapshots for visual regression
harness.snapshot("my_snapshot");

// Wait for conditions
harness.wait_for_text("Connected", Duration::from_secs(5)).unwrap();
```

## Mock Network Layer

The `MockAS400Connection` simulates AS/400 responses for testing:

```rust
use tn5250r::tests::mocks::mock_network::MockScenario;

// Pre-configured scenarios
let (mock, responses) = MockScenario::successful_connection();

// Custom responses
let mut mock = MockAS400Connection::new();
mock.add_response(vec![0xF1, 0x00, 0x00, 0x00, 0x00]); // Signon screen
```

## CI/CD Integration

Tests run automatically on GitHub Actions with Xvfb for headless execution. See `.github/workflows/gui-tests.yml` for the workflow configuration.

### Local Headless Testing

On Linux systems without a display:
```bash
# Install Xvfb
sudo apt-get install xvfb

# Run tests headlessly
xvfb-run -a cargo test --test gui
```

## Writing New Tests

### Component Test Example
```rust
#[test]
fn test_my_button() {
    let mut harness = TN5250RHarness::new();
    harness.step();

    // Verify button exists
    assert!(harness.has_element("button", "My Button"));

    // Test interaction
    harness.click_by_text("My Button").unwrap();

    // Verify result
    assert!(harness.has_text("Button clicked"));
}
```

### Integration Test Example
```rust
#[test]
fn test_user_workflow() {
    let mut harness = TN5250RHarness::new();
    let (mock, _) = MockScenario::successful_connection();

    harness.step();

    // Simulate user actions
    harness.click_by_text("Host").unwrap();
    harness.type_text("as400.example.com").unwrap();
    harness.click_by_text("Connect").unwrap();

    // Verify expected outcome
    harness.wait_for_text("Connected", Duration::from_secs(2)).unwrap();
}
```

### Visual Regression Test Example
```rust
#[test]
fn test_layout_unchanged() {
    let mut harness = TN5250RHarness::new();
    harness.step();

    // Take snapshot - will fail if layout changes
    harness.snapshot("my_layout");
}
```

## Troubleshooting

### Common Issues

1. **Display Server Not Found (Linux)**
   ```bash
   # Install virtual display
   sudo apt-get install xvfb
   xvfb-run -a cargo test --test gui
   ```

2. **Snapshot Test Failures**
   - Check if UI changes are intentional
   - Update snapshots: `./run-gui-tests.sh update`
   - Review visual differences in CI artifacts

3. **Test Timeouts**
   - Increase wait durations in `wait_for_*` calls
   - Check that mock responses are configured correctly
   - Verify GUI state transitions are working

4. **AccessKit Element Not Found**
   - Ensure UI elements have proper accessibility labels
   - Check element selectors in test code
   - Use `harness.step()` to advance GUI state

### Debug Mode

Enable verbose logging:
```bash
RUST_LOG=debug ./run-gui-tests.sh
```

## Best Practices

1. **Use Descriptive Test Names**: `test_connection_successful` vs `test_connect`
2. **Wait for Conditions**: Use `wait_for_*` methods instead of fixed delays
3. **Mock External Dependencies**: Use `MockAS400Connection` for reliable testing
4. **Test Visual Changes**: Add snapshots for new UI features
5. **Keep Tests Fast**: Aim for < 30 seconds per test
6. **Test Edge Cases**: Include error states and boundary conditions

## Dependencies

- `egui_kittest`: Official egui testing framework
- `eframe`: GUI framework with testing support
- `AccessKit`: Cross-platform accessibility framework
- `dify`: Image comparison for visual regression

See `Cargo.toml` for version specifications.