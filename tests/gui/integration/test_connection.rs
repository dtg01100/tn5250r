use std::time::Duration;
use crate::test_harness::TN5250RHarness;
use super::mock_network::{MockAS400Connection, MockScenario};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_dialog_display() {
        let mut harness = TN5250RHarness::new();

        // Step to initialize the GUI
        harness.step();

        // Check that connection dialog elements are present
        assert!(harness.has_text("Host"));
        assert!(harness.has_text("Port"));
        assert!(harness.has_element("button", "Connect"));
    }

    #[test]
    fn test_successful_connection_flow() {
        let mut harness = TN5250RHarness::new();
        let (mock_connection, _) = MockScenario::successful_connection();

        // Step to initialize GUI
        harness.step();

        // Enter connection details
        harness.click_by_text("Host").unwrap();
        harness.type_text("test.as400.com").unwrap();

        harness.click_by_text("Port").unwrap();
        harness.type_text("23").unwrap();

        // Click connect
        harness.click_by_text("Connect").unwrap();

        // Wait for connection to establish and signon screen to appear
        harness.wait_for_text("Sign On", Duration::from_secs(2)).unwrap();

        // Take snapshot for visual regression testing
        harness.snapshot("signon_screen");
    }

    #[test]
    fn test_connection_failure_handling() {
        let mut harness = TN5250RHarness::new();
        let mock_connection = MockScenario::connection_failure();

        // Step to initialize GUI
        harness.step();

        // Enter invalid connection details
        harness.click_by_text("Host").unwrap();
        harness.type_text("invalid.host").unwrap();

        harness.click_by_text("Port").unwrap();
        harness.type_text("9999").unwrap();

        // Click connect
        harness.click_by_text("Connect").unwrap();

        // Wait for error message
        harness.wait_for_text("Connection failed", Duration::from_secs(2)).unwrap();

        // Take snapshot of error state
        harness.snapshot("connection_error");
    }

    #[test]
    fn test_menu_navigation() {
        let mut harness = TN5250RHarness::new();
        let (mock_connection, _) = MockScenario::menu_navigation();

        // Step to initialize GUI
        harness.step();

        // Connect and navigate to menu
        harness.click_by_text("Host").unwrap();
        harness.type_text("test.as400.com").unwrap();

        harness.click_by_text("Port").unwrap();
        harness.type_text("23").unwrap();

        harness.click_by_text("Connect").unwrap();

        // Wait for menu to appear
        harness.wait_for_text("MAIN MENU", Duration::from_secs(2)).unwrap();

        // Navigate menu (simulate typing option and pressing enter)
        harness.type_text("1").unwrap();
        harness.press_enter().unwrap();

        // Wait for response
        harness.wait_for_text("Option 1 selected", Duration::from_secs(1)).unwrap();

        // Take snapshot
        harness.snapshot("menu_navigation");
    }

    #[test]
    fn test_function_key_usage() {
        let mut harness = TN5250RHarness::new();
        let (mock_connection, _) = MockScenario::successful_connection();

        // Step to initialize GUI
        harness.step();

        // Connect to system
        harness.click_by_text("Host").unwrap();
        harness.type_text("test.as400.com").unwrap();

        harness.click_by_text("Port").unwrap();
        harness.type_text("23").unwrap();

        harness.click_by_text("Connect").unwrap();

        // Wait for signon screen
        harness.wait_for_text("Sign On", Duration::from_secs(2)).unwrap();

        // Test F1 (Help) key
        harness.press_key(egui::Key::F1).unwrap();

        // Wait for help to appear
        harness.wait_for_text("Help", Duration::from_secs(1)).unwrap();

        // Take snapshot
        harness.snapshot("function_key_f1");
    }

    #[test]
    fn test_user_input_and_submission() {
        let mut harness = TN5250RHarness::new();
        let (mock_connection, _) = MockScenario::successful_connection();

        // Step to initialize GUI
        harness.step();

        // Connect to system
        harness.click_by_text("Host").unwrap();
        harness.type_text("test.as400.com").unwrap();

        harness.click_by_text("Port").unwrap();
        harness.type_text("23").unwrap();

        harness.click_by_text("Connect").unwrap();

        // Wait for signon screen with input fields
        harness.wait_for_text("User", Duration::from_secs(2)).unwrap();

        // Enter user credentials
        harness.click_by_text("User").unwrap();
        harness.type_text("TESTUSER").unwrap();

        harness.click_by_text("Password").unwrap();
        harness.type_text("testpass").unwrap();

        // Submit the form
        harness.press_enter().unwrap();

        // Wait for successful signon
        harness.wait_for_text("Welcome", Duration::from_secs(2)).unwrap();

        // Take snapshot
        harness.snapshot("successful_signon");
    }

    #[test]
    fn test_window_resizing() {
        // Test with different window sizes
        let mut harness = TN5250RHarness::with_size(1024.0, 768.0);
        harness.step();

        // Check that UI adapts to larger size
        assert!(harness.has_text("TN5250R"));

        // Test smaller size
        let mut small_harness = TN5250RHarness::with_size(640.0, 480.0);
        small_harness.step();

        // UI should still be functional at smaller size
        assert!(small_harness.has_text("Host"));
    }

    #[test]
    fn test_disconnect_functionality() {
        let mut harness = TN5250RHarness::new();
        let (mock_connection, _) = MockScenario::successful_connection();

        // Step to initialize GUI
        harness.step();

        // Connect first
        harness.click_by_text("Host").unwrap();
        harness.type_text("test.as400.com").unwrap();

        harness.click_by_text("Port").unwrap();
        harness.type_text("23").unwrap();

        harness.click_by_text("Connect").unwrap();

        // Wait for connection
        harness.wait_for_text("Sign On", Duration::from_secs(2)).unwrap();

        // Now disconnect
        harness.click_by_text("Disconnect").unwrap();

        // Wait for disconnection
        harness.wait_for_text("Host", Duration::from_secs(1)).unwrap();

        // Connection dialog should be visible again
        assert!(harness.has_text("Port"));
        assert!(harness.has_element("button", "Connect"));
    }
}