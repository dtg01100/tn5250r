use egui_kittest::{Harness, egui};
use std::time::Duration;
use tn5250r::TN5250RApp;

/// Test harness for TN5250R GUI testing
pub struct TN5250RHarness<'a> {
    harness: Harness<'a>,
}

impl<'a> TN5250RHarness<'a> {
    /// Create a new test harness with default settings
    pub fn new() -> Self {
        let harness = Harness::builder()
            .with_size(egui::Vec2::new(800.0, 600.0))
            .build_eframe(|ctx| {
                Box::new(TN5250RApp::new(ctx))
            });

        Self { harness }
    }

    /// Create harness with custom size
    pub fn with_size(width: f32, height: f32) -> Self {
        let harness = Harness::builder()
            .with_size(egui::Vec2::new(width, height))
            .build_eframe(|ctx| {
                Box::new(TN5250RApp::new(ctx))
            });

        Self { harness }
    }

    /// Step the GUI forward one frame
    pub fn step(&mut self) {
        self.harness.step();
    }

    /// Click on an element by its accessibility text
    pub fn click_by_text(&mut self, text: &str) -> Result<(), String> {
        self.harness.click_by_accesskit(&format!("text:'{}'", text))
            .map_err(|e| format!("Failed to click text '{}': {}", text, e))
    }

    /// Click on an element by accessibility role and name
    pub fn click_by_role(&mut self, role: &str, name: &str) -> Result<(), String> {
        self.harness.click_by_accesskit(&format!("role:'{}', name:'{}'", role, name))
            .map_err(|e| format!("Failed to click role '{}' name '{}': {}", role, name, e))
    }

    /// Type a string of text
    pub fn type_text(&mut self, text: &str) -> Result<(), String> {
        for ch in text.chars() {
            self.harness.type_char(ch)
                .map_err(|e| format!("Failed to type char '{}': {}", ch, e))?;
        }
        Ok(())
    }

    /// Press a specific key
    pub fn press_key(&mut self, key: egui::Key) -> Result<(), String> {
        self.harness.press_key(key)
            .map_err(|e| format!("Failed to press key {:?}: {}", key, e))
    }

    /// Press Enter key
    pub fn press_enter(&mut self) -> Result<(), String> {
        self.press_key(egui::Key::Enter)
    }

    /// Press Tab key
    pub fn press_tab(&mut self) -> Result<(), String> {
        self.press_key(egui::Key::Tab)
    }

    /// Press Backspace key
    pub fn press_backspace(&mut self) -> Result<(), String> {
        self.press_key(egui::Key::Backspace)
    }

    /// Take a snapshot for visual regression testing
    pub fn snapshot(&mut self, name: &str) {
        self.harness.snapshot(name);
    }

    /// Check if text exists in the GUI
    pub fn has_text(&self, text: &str) -> bool {
        self.harness.has_accesskit_node(&format!("text:'{}'", text))
    }

    /// Check if element exists by role and name
    pub fn has_element(&self, role: &str, name: &str) -> bool {
        self.harness.has_accesskit_node(&format!("role:'{}', name:'{}'", role, name))
    }

    /// Wait for a condition to be true with timeout
    pub fn wait_for_condition<F>(&mut self, condition: F, timeout: Duration) -> Result<(), String>
    where
        F: Fn(&Self) -> bool,
    {
        let start = std::time::Instant::now();
        let frame_time = Duration::from_millis(16); // ~60 FPS

        while start.elapsed() < timeout {
            self.step();
            if condition(self) {
                return Ok(());
            }
            std::thread::sleep(frame_time);
        }

        Err(format!("Condition not met within {:?}", timeout))
    }

    /// Wait for specific text to appear
    pub fn wait_for_text(&mut self, text: &str, timeout: Duration) -> Result<(), String> {
        self.wait_for_condition(|h| h.has_text(text), timeout)
    }

    /// Get the current GUI size
    pub fn size(&self) -> egui::Vec2 {
        self.harness.size()
    }

    /// Access the underlying harness for advanced operations
    pub fn harness(&mut self) -> &mut Harness<'a> {
        &mut self.harness
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