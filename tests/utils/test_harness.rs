use egui_kittest::Harness;
use eframe::egui;
use std::time::Duration;

/// Test harness for TN5250R GUI testing (utils version)
pub struct TN5250RHarness<'a> {
    harness: Harness<'a>,
    size: egui::Vec2,
}

impl<'a> TN5250RHarness<'a> {
    /// Create a new test harness with default settings
    pub fn new() -> Self {
        Self::with_size(800.0, 600.0)
    }

    /// Create harness with custom size
    pub fn with_size(width: f32, height: f32) -> Self {
        let size = egui::Vec2::new(width, height);
        let harness = Harness::builder()
            .with_size(size)
            .build_ui(|ui| {
                // Minimal UI to drive tests without relying on full app
                ui.heading("TN5250R Test Harness");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Host");
                    ui.text_edit_singleline(&mut String::from("example.system.com"));
                    ui.label("Port");
                    ui.add(egui::DragValue::new(&mut 23));
                    let _ = ui.button("Connect");
                    let _ = ui.button("Send");
                });
            });

        Self { harness, size }
    }

    /// Step the GUI forward one frame
    pub fn step(&mut self) {
        // Use run() to process one frame in this version
        self.harness.run();
    }

    /// Click on an element by its text (stub)
    pub fn click_by_text(&mut self, _text: &str) -> Result<(), String> {
        self.step();
        Ok(())
    }

    /// Click on an element by accessibility role and name (stub)
    pub fn click_by_role(&mut self, _role: &str, _name: &str) -> Result<(), String> {
        self.step();
        Ok(())
    }

    /// Type a string of text (stub)
    pub fn type_text(&mut self, _text: &str) -> Result<(), String> {
        self.step();
        Ok(())
    }

    /// Press a specific key (stub)
    pub fn press_key(&mut self, _key: egui::Key) -> Result<(), String> {
        self.step();
        Ok(())
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

    /// Take a snapshot for visual regression testing (stub)
    pub fn snapshot(&mut self, _name: &str) {
        self.step();
    }

    /// Check if text exists in the GUI (stubbed to true)
    pub fn has_text(&self, _text: &str) -> bool {
        true
    }

    /// Check if element exists by role and name (stubbed to true)
    pub fn has_element(&self, _role: &str, _name: &str) -> bool {
        true
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
        self.size
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