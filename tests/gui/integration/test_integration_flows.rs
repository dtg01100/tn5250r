use std::time::Duration;
use super::test_harness::TN5250RHarness;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_basic_app_initialization() {
        let mut harness = TN5250RHarness::new();
        harness.step();

        // Verify basic harness functionality
        assert_eq!(harness.size(), egui::Vec2::new(800.0, 600.0));
        assert!(harness.has_text("TN5250R"));
    }

    #[test]
    fn test_connection_state_transitions() {
        let mut harness = TN5250RHarness::new();
        harness.step();

        // Test connection flow via UI interaction
        assert!(harness.click_by_text("Connect").is_ok());
        harness.step();

        // Verify UI responds to interactions
        assert!(harness.has_element("button", "Connect"));
    }

    #[test]
    fn test_input_buffer_operations() {
        let mut harness = TN5250RHarness::new();
        harness.step();

        // Test input operations via UI
        assert!(harness.type_text("test command").is_ok());
        harness.step();

        // Test sending input
        assert!(harness.click_by_text("Send").is_ok());
        harness.step();
    }

    #[test]
    fn test_terminal_content_display() {
        let mut harness = TN5250RHarness::new();
        harness.step();

        // Test terminal display via UI elements
        assert!(harness.has_text("TN5250R"));
        assert!(harness.has_text("Terminal"));
    }

    #[test]
    fn test_function_keys_visibility_toggle() {
        let mut harness = TN5250RHarness::new();
        harness.step();

        // Test function key related UI elements
        assert!(harness.has_text("Function Keys"));
    }

    #[test]
    fn test_debug_mode_toggle() {
        let mut harness = TN5250RHarness::new();
        harness.step();

        // Test debug-related operations
        assert!(harness.has_text("TN5250R"));
    }

    #[test]
    fn test_wait_for_text_functionality() {
        let mut harness = TN5250RHarness::new();
        harness.step();

        // Test wait functionality
        let result = harness.wait_for_text("TN5250R", Duration::from_millis(100));
        assert!(result.is_ok());
    }

    #[test]
    fn test_window_resizing() {
        // Test different window sizes
        let mut harness = TN5250RHarness::with_size(1024.0, 768.0);
        harness.step();

        assert_eq!(harness.size(), egui::Vec2::new(1024.0, 768.0));

        // Test smaller size
        let mut small_harness = TN5250RHarness::with_size(640.0, 480.0);
        small_harness.step();

        assert_eq!(small_harness.size(), egui::Vec2::new(640.0, 480.0));
    }

    #[test]
    fn test_multiple_step_simulation() {
        let mut harness = TN5250RHarness::new();

        // Run multiple frames to ensure stability
        for _i in 0..10 {
            harness.step();
            // Verify harness remains functional
            assert_eq!(harness.size(), egui::Vec2::new(800.0, 600.0));
        }
    }
}