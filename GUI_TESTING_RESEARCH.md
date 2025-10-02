# Automated GUI Testing Research for TN5250R

## Overview

This document explores automated testing approaches for the TN5250R terminal emulator GUI, which uses egui/eframe. The goal is to create a robust testing strategy that can verify GUI functionality, prevent regressions, and enable CI/CD integration.

## Current Testing Status

### Existing Tests
- **Unit Tests**: Basic protocol parsing, field management
- **Integration Tests**: Network connection testing (`test_connection.rs`)
- **Manual Testing**: Interactive GUI testing (now working)

### Testing Gaps
- No automated GUI interaction testing
- No visual regression testing
- No end-to-end workflow testing
- No CI/CD GUI validation

## GUI Testing Approaches

### 1. egui_kittest (Official egui Testing Framework)

**Description**: Official testing library for egui applications using AccessKit for accessibility-based testing.

**Key Features**:
- AccessKit integration for semantic GUI element access
- Screenshot-based visual regression testing
- Event simulation (clicks, typing, etc.)
- Headless testing support
- Snapshot testing with diffy

**Pros**:
- Official egui support
- AccessKit provides semantic element access
- Screenshot comparison for visual regression
- Works with existing egui/eframe apps
- Good documentation and examples

**Cons**:
- Requires AccessKit feature in eframe
- Learning curve for AccessKit queries
- Screenshot testing can be flaky with fonts/rendering

**Implementation**:
```rust
// Add to Cargo.toml
[dev-dependencies]
egui_kittest = { version = "0.32", features = ["eframe", "snapshot"] }

// Basic test structure
#[test]
fn test_signon_screen_display() {
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::Vec2::new(800.0, 600.0))
        .build_eframe(|ctx| {
            Box::new(TN5250RApp::new(ctx))
        });

    // Simulate connection
    harness.step();
    harness.click_by_accesskit("Connect");

    // Take snapshot
    harness.snapshot("signon_screen");
}
```

**Current Status**: Our eframe already has AccessKit enabled, so this is ready to use.

---

### 2. egui-screenshot-testing (Screenshot-Based Testing)

**Description**: Lightweight screenshot testing library for egui applications.

**Key Features**:
- Simple screenshot capture and comparison
- PNG-based snapshot storage
- Automatic diff generation
- Minimal dependencies

**Pros**:
- Simple to set up and use
- Good for visual regression testing
- Lightweight (no AccessKit complexity)
- Clear failure visualization

**Cons**:
- No semantic element interaction
- Screenshot testing can be brittle
- Requires manual screenshot updates
- No event simulation

**Implementation**:
```rust
use egui_screenshot_testing::ScreenshotTesting;

#[test]
fn test_gui_layout() {
    let screenshot = ScreenshotTesting::new()
        .with_size(800, 600)
        .capture_app(|ctx| {
            TN5250RApp::new(ctx).ui(ctx);
        });

    screenshot.assert_matches_snapshot("gui_layout");
}
```

---

### 3. terminator-rs (Desktop Automation)

**Description**: Playwright-style SDK for automating desktop GUI applications using accessibility APIs.

**Key Features**:
- Cross-platform desktop automation
- Element finding by accessibility properties
- Event simulation (clicks, typing, etc.)
- Screenshot capture
- Wait conditions and assertions

**Pros**:
- Powerful automation capabilities
- Cross-platform support
- Familiar Playwright-style API
- Good for end-to-end testing

**Cons**:
- Runs actual GUI application (not headless)
- Requires display server in CI
- More complex setup
- Not egui-specific

**Implementation**:
```rust
use terminator::Terminator;

#[test]
fn test_terminal_connection() {
    let mut term = Terminator::new("tn5250r")
        .arg("--server")
        .arg("10.100.200.1")
        .arg("--port")
        .arg("23")
        .spawn();

    // Wait for signon screen
    term.wait_for_text("Sign On");

    // Type credentials
    term.click_text("User");
    term.type_text("dave3");
    term.click_text("Password");
    term.type_text("dave3");
    term.press_enter();

    // Verify connection
    term.wait_for_text("Main menu");
}
```

---

### 4. Custom Integration Testing with Mocks

**Description**: Create integration tests that mock the network layer and test GUI logic directly.

**Key Features**:
- Mock AS/400 server responses
- Test GUI state changes
- Headless testing
- Fast execution
- No external dependencies

**Pros**:
- Fast and reliable
- No GUI rendering required
- Good for logic testing
- Easy to run in CI
- Can test error conditions

**Cons**:
- Doesn't test actual rendering
- Requires mocking infrastructure
- Limited visual validation
- More complex to set up

**Implementation**:
```rust
#[cfg(test)]
mod gui_integration_tests {
    use super::*;
    use mockito::mock;

    #[test]
    fn test_connection_flow() {
        // Mock AS/400 server
        let mock = mock("GET", "/")
            .with_body(b"\x1b[1;1HSign On\x1b[7;10HUser")
            .create();

        // Create GUI with mock controller
        let mut app = TN5250RApp::new_with_controller(mock_controller);

        // Simulate user actions
        app.connect("mock-server", 23);

        // Verify GUI state
        assert!(app.is_connected());
        assert!(app.terminal_content.contains("Sign On"));
    }
}
```

---

### 5. Headless eframe Testing

**Description**: Use eframe's headless mode with custom test harness.

**Key Features**:
- Direct egui context manipulation
- No window creation
- Fast testing
- Memory-based rendering

**Pros**:
- Very fast execution
- No display requirements
- Direct API access
- Good for unit testing GUI logic

**Cons**:
- Limited visual testing
- Requires custom harness code
- No real event loop
- Manual state management

**Implementation**:
```rust
#[test]
fn test_gui_state_changes() {
    let mut ctx = egui::Context::default();
    let mut app = TN5250RApp::new(&ctx);

    // Simulate connection
    app.do_connect();

    // Run GUI update
    app.update(&ctx);

    // Check state
    assert!(app.connected);
    assert!(app.terminal_content.contains("Connecting"));
}
```

---

## Recommended Testing Strategy

### Primary Approach: egui_kittest + Integration Tests

**Why this combination?**
- egui_kittest provides official egui testing with visual regression
- Integration tests cover business logic and error conditions
- Balanced coverage of GUI and functionality
- CI/CD friendly

### Implementation Plan

#### Phase 1: Basic Setup (1-2 days)
1. Add egui_kittest dependency
2. Create basic test harness
3. Set up snapshot testing infrastructure

#### Phase 2: Core GUI Tests (2-3 days)
1. Connection flow testing
2. Screen display validation
3. Basic input testing
4. Error state testing

#### Phase 3: Visual Regression (1-2 days)
1. Screenshot baseline creation
2. Visual diff testing
3. Cross-platform validation

#### Phase 4: CI/CD Integration (1 day)
1. GitHub Actions setup
2. Test result reporting
3. Snapshot artifact storage

### Test Categories

#### 1. Unit Tests (Existing)
- Protocol parsing
- Field management
- Keyboard mapping
- EBCDIC conversion

#### 2. GUI Component Tests
- Button interactions
- Text input handling
- Screen rendering
- Cursor positioning

#### 3. Integration Tests
- Full connection flow
- Authentication sequence
- Screen transitions
- Error handling

#### 4. Visual Regression Tests
- Layout consistency
- Font rendering
- Color schemes
- Responsive behavior

#### 5. End-to-End Tests
- Complete user workflows
- Multi-screen navigation
- Session management

### Example Test Suite Structure

```
tests/
├── gui/
│   ├── components/
│   │   ├── test_buttons.rs
│   │   ├── test_input.rs
│   │   └── test_display.rs
│   ├── integration/
│   │   ├── test_connection.rs
│   │   ├── test_authentication.rs
│   │   └── test_navigation.rs
│   ├── visual/
│   │   ├── test_layout.rs
│   │   └── test_themes.rs
│   └── snapshots/
│       ├── signon_screen.png
│       ├── main_menu.png
│       └── error_dialog.png
├── mocks/
│   ├── mock_server.rs
│   └── mock_responses.rs
└── utils/
    ├── test_harness.rs
    └── screenshot_utils.rs
```

### CI/CD Integration

#### GitHub Actions Example
```yaml
name: GUI Tests
on: [push, pull_request]

jobs:
  gui-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install dependencies
        run: sudo apt-get update && sudo apt-get install -y libgtk-3-dev
      - name: Run GUI tests
        run: cargo test --test gui
      - name: Upload snapshots
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: test-snapshots
          path: tests/gui/snapshots/
```

### Challenges and Solutions

#### Challenge 1: Headless Testing
**Problem**: GUI tests need display server in CI
**Solution**: Use Xvfb or similar virtual display

#### Challenge 2: Timing Issues
**Problem**: Async network operations cause flaky tests
**Solution**: Mock network layer for deterministic testing

#### Challenge 3: Screenshot Consistency
**Problem**: Screenshots differ across platforms/fonts
**Solution**: Use consistent fonts, OS-specific thresholds

#### Challenge 4: Test Maintenance
**Problem**: GUI changes break many tests
**Solution**: Modular test design, update snapshots atomically

### Success Metrics

- **Coverage**: 80%+ GUI interaction coverage
- **Reliability**: <5% flaky test rate
- **Speed**: <30 seconds for full GUI test suite
- **Maintenance**: <1 hour to update tests per major GUI change

### Next Steps

1. **Immediate**: Add egui_kittest dependency and create basic harness
2. **Week 1**: Implement connection flow tests
3. **Week 2**: Add visual regression testing
4. **Week 3**: Create comprehensive test suite
5. **Week 4**: CI/CD integration and documentation

This testing strategy will provide robust validation of the TN5250R GUI, prevent regressions, and enable confident development of new features.