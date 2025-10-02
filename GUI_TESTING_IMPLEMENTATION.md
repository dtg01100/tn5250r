# GUI Testing Implementation for TN5250R

## Setup egui_kittest

First, add the testing dependencies to `Cargo.toml`:

```toml
[dev-dependencies]
egui_kittest = { version = "0.32", features = ["eframe", "snapshot"] }
image = "0.24"  # For image processing
```

## Basic Test Harness

Create `tests/gui/test_harness.rs`:

```rust
use egui_kittest::{Harness, egui};
use tn5250r::TN5250RApp;

/// Test harness for TN5250R GUI testing
pub struct TN5250RHarness {
    harness: Harness<'static>,
}

impl TN5250RHarness {
    pub fn new() -> Self {
        let harness = Harness::builder()
            .with_size(egui::Vec2::new(800.0, 600.0))
            .build_eframe(|ctx| {
                Box::new(TN5250RApp::new(ctx))
            });

        Self { harness }
    }

    pub fn step(&mut self) {
        self.harness.step();
    }

    pub fn click_by_text(&mut self, text: &str) {
        self.harness.click_by_accesskit(&format!("text:'{}'", text));
    }

    pub fn type_text(&mut self, text: &str) {
        for ch in text.chars() {
            self.harness.type_char(ch);
        }
    }

    pub fn press_enter(&mut self) {
        self.harness.press_key(egui::Key::Enter);
    }

    pub fn snapshot(&mut self, name: &str) {
        self.harness.snapshot(name);
    }

    pub fn has_text(&self, text: &str) -> bool {
        self.harness.has_accesskit_node(&format!("text:'{}'", text))
    }

    pub fn get_text_content(&self) -> String {
        // Extract terminal content from GUI state
        // This would need access to the app's internal state
        todo!("Implement text content extraction")
    }
}
```

## Example GUI Tests

Create `tests/gui/integration/test_connection.rs`:

```rust
use super::TN5250RHarness;

#[test]
fn test_connection_flow() {
    let mut harness = TN5250RHarness::new();

    // Initial state - should show connection dialog
    harness.step();
    assert!(harness.has_text("Host"));
    assert!(harness.has_text("Port"));

    // Enter connection details
    harness.click_by_text("Host");
    harness.type_text("10.100.200.1");
    harness.click_by_text("Port");
    harness.type_text("23");

    // Click connect
    harness.click_by_text("Connect");
    harness.step();

    // Should show connecting state
    assert!(harness.has_text("Connecting"));

    // Wait for connection (in real test, would mock this)
    // harness.wait_for_text("Sign On");

    // Take snapshot for visual regression
    harness.snapshot("connecting_state");
}

#[test]
fn test_signon_screen_display() {
    let mut harness = TN5250RHarness::new();

    // Mock a successful connection that shows signon screen
    // This would require mocking the network layer

    harness.step();

    // Verify signon screen elements
    assert!(harness.has_text("Sign On"));
    assert!(harness.has_text("System"));
    assert!(harness.has_text("User"));
    assert!(harness.has_text("Password"));

    // Visual regression test
    harness.snapshot("signon_screen");
}

#[test]
fn test_user_input() {
    let mut harness = TN5250RHarness::new();

    // Assume we're on signon screen
    harness.step();

    // Click in user field and type
    harness.click_by_text("User");
    harness.type_text("testuser");

    // Verify input (would need to check GUI state)
    // assert_eq!(harness.get_user_field_text(), "testuser");

    harness.step();
    harness.snapshot("user_input");
}

#[test]
fn test_error_handling() {
    let mut harness = TN5250RHarness::new();

    // Try to connect to invalid server
    harness.click_by_text("Host");
    harness.type_text("invalid.server");
    harness.click_by_text("Connect");

    harness.step();

    // Should show error
    assert!(harness.has_text("failed"));
    harness.snapshot("connection_error");
}
```

## Mock Network Layer

Create `tests/mocks/mock_network.rs` for testing without real AS/400:

```rust
use std::sync::mpsc;
use tn5250r::network::AS400Connection;

/// Mock AS/400 connection for testing
pub struct MockAS400Connection {
    responses: Vec<Vec<u8>>,
    current_response: usize,
}

impl MockAS400Connection {
    pub fn new() -> Self {
        Self {
            responses: Vec::new(),
            current_response: 0,
        }
    }

    pub fn add_response(&mut self, data: Vec<u8>) {
        self.responses.push(data);
    }

    pub fn with_signon_response() -> Self {
        let mut mock = Self::new();
        // Mock ANSI signon screen response
        let signon_data = b"\x1b[1;1HSign On\x1b[7;10HUser\x1b[9;10HPassword";
        mock.add_response(signon_data.to_vec());
        mock
    }
}

impl AS400Connection for MockAS400Connection {
    fn connect(&mut self, _host: &str, _port: u16) -> Result<(), String> {
        Ok(())
    }

    fn send(&mut self, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, String> {
        if self.current_response < self.responses.len() {
            let response = self.responses[self.current_response].clone();
            self.current_response += 1;
            Ok(response)
        } else {
            Ok(Vec::new()) // No more data
        }
    }

    fn is_connected(&self) -> bool {
        true
    }

    fn disconnect(&mut self) -> Result<(), String> {
        Ok(())
    }
}
```

## Visual Regression Testing

Create `tests/gui/visual/test_screenshots.rs`:

```rust
use super::TN5250RHarness;

#[test]
fn test_initial_screen_layout() {
    let mut harness = TN5250RHarness::new();
    harness.step();

    // Take baseline snapshot
    harness.snapshot("initial_screen");
}

#[test]
fn test_connected_state_layout() {
    let mut harness = TN5250RHarness::new();

    // Simulate connection
    harness.click_by_text("Host");
    harness.type_text("10.100.200.1");
    harness.click_by_text("Port");
    harness.type_text("23");
    harness.click_by_text("Connect");

    harness.step();

    // Verify layout after connection
    harness.snapshot("connected_layout");
}

#[test]
fn test_signon_screen_visual() {
    let mut harness = TN5250RHarness::new();

    // Mock signon screen display
    // (Would need to inject mock data)

    harness.step();
    harness.snapshot("signon_screen_visual");
}

#[test]
fn test_error_dialog_visual() {
    let mut harness = TN5250RHarness::new();

    // Trigger error condition
    harness.click_by_text("Host");
    harness.type_text("invalid.host");
    harness.click_by_text("Connect");

    harness.step();

    // Verify error dialog appearance
    harness.snapshot("error_dialog");
}
```

## CI/CD Integration

Create `.github/workflows/gui-tests.yml`:

```yaml
name: GUI Tests
on: [push, pull_request]

jobs:
  gui-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev xvfb

      - name: Run GUI tests
        run: xvfb-run -a cargo test --test gui -- --nocapture

      - name: Upload test snapshots
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: gui-test-snapshots
          path: |
            tests/gui/snapshots/
            target/debug/deps/snapshots/

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: gui-test-results
          path: target/debug/deps/test_results/
```

## Running Tests Locally

```bash
# Run all GUI tests
cargo test --test gui

# Run specific test
cargo test --test gui test_connection_flow

# Update snapshots (after intentional changes)
UPDATE_SNAPSHOTS=1 cargo test --test gui

# Run with verbose output
cargo test --test gui -- --nocapture
```

## Test Organization

```
tests/
├── gui/
│   ├── mod.rs                    # Common test utilities
│   ├── test_harness.rs          # TN5250RHarness implementation
│   ├── components/
│   │   ├── test_buttons.rs      # Button interaction tests
│   │   ├── test_input.rs        # Text input tests
│   │   └── test_display.rs      # Display/rendering tests
│   ├── integration/
│   │   ├── test_connection.rs   # Connection flow tests
│   │   ├── test_authentication.rs # Login/auth tests
│   │   └── test_navigation.rs   # Screen navigation tests
│   └── visual/
│       ├── test_layout.rs       # Layout regression tests
│       └── test_themes.rs       # Theme/color tests
├── mocks/
│   ├── mock_network.rs          # Mock AS/400 connection
│   └── mock_responses.rs        # Predefined response data
└── utils/
    ├── screenshot_utils.rs      # Screenshot helpers
    └── test_data.rs            # Test data generators
```

## Handling Async Operations

For testing async network operations:

```rust
#[test]
fn test_async_connection() {
    let mut harness = TN5250RHarness::new();

    // Start connection
    harness.click_by_text("Connect");

    // Wait for async operation to complete
    harness.wait_for_condition(|h| h.has_text("Sign On"), Duration::from_secs(5));

    // Verify result
    assert!(harness.has_text("System"));
}
```

## Best Practices

1. **Mock External Dependencies**: Use mock network layer for reliable tests
2. **Snapshot Management**: Keep snapshots in version control for regression detection
3. **Cross-Platform Testing**: Test on multiple OS combinations
4. **Flaky Test Prevention**: Use proper wait conditions, avoid timing assumptions
5. **Test Data Management**: Use realistic test data that matches production
6. **CI Optimization**: Run GUI tests in parallel, cache dependencies

## Troubleshooting

### Common Issues

**AccessKit queries failing**:
- Verify AccessKit feature is enabled in eframe
- Check element accessibility labels
- Use browser dev tools to inspect accessibility tree

**Screenshot differences**:
- Update snapshots after intentional changes
- Use OS-specific thresholds for pixel differences
- Ensure consistent fonts and rendering

**Timing issues**:
- Use proper wait conditions instead of sleep()
- Mock async operations for deterministic testing
- Increase timeouts for CI environments

**CI failures**:
- Ensure display server is available (Xvfb)
- Check system dependencies
- Verify test isolation (no shared state)

This implementation provides comprehensive GUI testing coverage for TN5250R, enabling reliable automated testing of the terminal emulator interface.