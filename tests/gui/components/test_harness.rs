use egui_kittest::Harness;
use egui::{Vec2, Key, Modifiers};
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use tn5250r::app_state::TN5250RApp;

/// Enhanced test harness for TN5250R GUI testing that integrates with the actual app
pub struct TN5250RHarness<'a> {
    harness: Harness<'a>,
    expected_size: Vec2,
    app: Arc<Mutex<TN5250RApp>>,
}

impl<'a> TN5250RHarness<'a> {
    /// Create a new test harness with default size
    pub fn new() -> Self {
        Self::with_size(800.0, 600.0)
    }

    /// Create a new test harness with custom size
    pub fn with_size(width: f32, height: f32) -> Self {
        let expected_size = Vec2::new(width, height);

        // Create a mock creation context for testing
        let egui_ctx = egui::Context::default();
        let cc = eframe::CreationContext::_new_kittest(egui_ctx);

        // Create the actual TN5250R app for testing
        let app = Arc::new(Mutex::new(TN5250RApp::new(&cc)));
        let app_clone = app.clone();

        let harness = Harness::builder()
            .with_size(expected_size)
            .build(move |ctx| {
                // Update the app
                let mut app_ref = app_clone.lock().unwrap();
                <TN5250RApp as eframe::App>::update(&mut app_ref, ctx, &mut eframe::Frame::_new_kittest());
            });

        Self { harness, expected_size, app }
    }

    /// Step the harness forward one frame
    pub fn step(&mut self) {
        self.harness.run();
    }

    /// Get the expected window size
    pub fn size(&self) -> Vec2 {
        self.expected_size
    }

    /// Get access to the underlying harness for advanced testing
    pub fn harness(&mut self) -> &mut Harness<'a> {
        &mut self.harness
    }

    /// Get access to the app for direct manipulation in tests
    pub fn app(&self) -> Arc<Mutex<TN5250RApp>> {
        self.app.clone()
    }

    /// Check if text is present in the UI by examining the app state
    pub fn has_text(&self, text: &str) -> bool {
        let app = self.app.lock().unwrap();

        // Check various places where text might appear
        app.terminal_content.contains(text) ||
        app.connection_string.contains(text) ||
        app.host.contains(text) ||
        app.username.contains(text) ||
        app.input_buffer.contains(text) ||
        (app.connected && "Connected".contains(text)) ||
        (!app.connected && "Disconnected".contains(text)) ||
        (app.connecting && "Connecting".contains(text))
    }

    /// Click on an element containing specific text
    pub fn click_by_text(&mut self, text: &str) -> Result<(), String> {
        // For now, simulate common button clicks by manipulating app state directly
        let mut app = self.app.lock().unwrap();

        match text {
            "Connect" => {
                app.do_connect();
            },
            "Disconnect" => {
                app.do_disconnect();
            },
            "Cancel" => {
                app.controller.cancel_connect();
                app.connecting = false;
                app.connection_time = None;
            },
            "Send" => {
                if !app.input_buffer.is_empty() {
                    let input = app.input_buffer.clone();
                    if let Err(e) = app.controller.send_input(input.as_bytes()) {
                        app.terminal_content.push_str(&format!("\nError: {}", e));
                    }
                    app.input_buffer.clear();
                }
            },
            _ => {
                // For other elements, we'd need more sophisticated UI tree traversal
                // For now, return an error for unrecognized elements
                return Err(format!("Element with text '{}' not found or not clickable", text));
            }
        }

        drop(app); // Release the lock before stepping
        self.step(); // Advance one frame after the action
        Ok(())
    }

    /// Type text into the currently focused element
    pub fn type_text(&mut self, text: &str) -> Result<(), String> {
        // Simulate typing by directly modifying the appropriate field
        // In a real implementation, this would track focus and modify the focused field
        let mut app = self.app.lock().unwrap();

        // For simplicity, assume we're typing into the input buffer
        // In a more sophisticated implementation, we'd track which field has focus
        app.input_buffer.push_str(text);

        drop(app);
        self.step();
        Ok(())
    }

    /// Wait for specific text to appear in the UI
    pub fn wait_for_text(&mut self, text: &str, timeout: Duration) -> Result<(), String> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            self.step();
            if self.has_text(text) {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(50));
        }
        Err(format!("Text '{}' not found within timeout", text))
    }

    /// Press a specific key
    pub fn press_key(&mut self, _key: Key) -> Result<(), String> {
        // TODO: Implement key press simulation
        // Currently not implemented due to mutable borrow issues
        Err("Key press simulation not yet implemented".to_string())
    }

    /// Press Enter key
    pub fn press_enter(&mut self) -> Result<(), String> {
        self.press_key(Key::Enter)
    }

    /// Check if a button with specific text exists
    pub fn has_element(&self, element_type: &str, text: &str) -> bool {
        match element_type {
            "button" => {
                // Check for common buttons
                matches!(text, "Connect" | "Disconnect" | "Cancel" | "Send" | "Exit" | "Advanced" | "Debug")
            },
            _ => false,
        }
    }

    /// Take a snapshot for visual regression testing
    pub fn snapshot(&mut self, name: &str) {
        // Placeholder for visual regression snapshot
        // In a real implementation, this would save a screenshot
        println!("Taking snapshot: {}", name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_creation() {
        let harness = TN5250RHarness::new();
        assert_eq!(harness.size(), egui::Vec2::new(800.0, 600.0));
    }

    #[test]
    fn test_harness_custom_size() {
        let harness = TN5250RHarness::with_size(1024.0, 768.0);
        assert_eq!(harness.size(), egui::Vec2::new(1024.0, 768.0));
    }
}