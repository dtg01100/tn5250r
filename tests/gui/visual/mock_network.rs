use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::integration::mock_network::Connection;

/// Mock AS/400 connection for visual regression testing
/// Provides deterministic responses for testing without external dependencies
pub struct MockAS400Connection {
    responses: Arc<Mutex<VecDeque<Vec<u8>>>>,
    sent_data: Arc<Mutex<Vec<Vec<u8>>>>,
    connected: Arc<Mutex<bool>>,
}

impl MockAS400Connection {
    /// Create a new mock connection with predefined responses
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::new())),
            sent_data: Arc::new(Mutex::new(Vec::new())),
            connected: Arc::new(Mutex::new(false)),
        }
    }

    /// Add a response that will be returned on the next read
    pub fn add_response(&self, data: Vec<u8>) {
        self.responses.lock().unwrap().push_back(data);
    }

    /// Add multiple responses in sequence
    pub fn add_responses(&self, responses: Vec<Vec<u8>>) {
        let mut queue = self.responses.lock().unwrap();
        for response in responses {
            queue.push_back(response);
        }
    }

    /// Get all data that was sent to the mock
    pub fn sent_data(&self) -> Vec<Vec<u8>> {
        self.sent_data.lock().unwrap().clone()
    }

    /// Clear sent data history
    pub fn clear_sent_data(&self) {
        self.sent_data.lock().unwrap().clear();
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    /// Set connection state
    pub fn set_connected(&self, connected: bool) {
        *self.connected.lock().unwrap() = connected;
    }

    /// Create a mock with typical TN5250 signon screen response
    pub fn with_signon_screen() -> Self {
        let mock = Self::new();

        // TN5250 signon screen data (simplified)
        // This would be the actual 5250 protocol data for a signon screen
        let signon_data = vec![
            // Write to Display command (0xF1)
            0xF1,
            // Screen data would follow...
            // For testing, we'll use minimal data
            0x00, 0x00, 0x00, 0x00,
        ];

        mock.add_response(signon_data);
        mock
    }

    /// Create a mock that simulates connection failure
    pub fn with_connection_failure() -> Self {
        let mock = Self::new();
        mock.set_connected(false);
        mock
    }

    /// Create a mock with menu screen response
    pub fn with_menu_screen() -> Self {
        let mock = Self::new();

        let menu_data = vec![
            // Write to Display command
            0xF1,
            // Menu screen data...
            0x00, 0x00, 0x00, 0x00,
        ];

        mock.add_response(menu_data);
        mock
    }
}

impl Connection for MockAS400Connection {
    fn connect(&mut self, _host: &str, _port: u16) -> Result<(), String> {
        self.set_connected(true);
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), String> {
        self.set_connected(false);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.is_connected()
    }

    fn send(&mut self, data: &[u8]) -> Result<(), String> {
        if !self.is_connected() {
            return Err("Not connected".to_string());
        }
        self.sent_data.lock().unwrap().push(data.to_vec());
        Ok(())
    }

    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, String> {
        if !self.is_connected() {
            return Err("Not connected".to_string());
        }

        let mut responses = self.responses.lock().unwrap();
        if let Some(response) = responses.pop_front() {
            let len = response.len().min(buffer.len());
            buffer[..len].copy_from_slice(&response[..len]);
            Ok(len)
        } else {
            // No more responses - simulate waiting
            Err("No data available".to_string())
        }
    }

    fn set_timeout(&mut self, _timeout_ms: u64) {
        // Mock doesn't need timeout
    }
}

/// Helper for creating test scenarios
pub struct MockScenario {
    mock: MockAS400Connection,
}

impl MockScenario {
    /// Create a successful connection scenario
    pub fn successful_connection() -> (MockAS400Connection, Vec<Vec<u8>>) {
        let mock = MockAS400Connection::with_signon_screen();
        let expected_responses = vec![
            vec![0xF1, 0x00, 0x00, 0x00, 0x00], // Signon screen
        ];
        (mock, expected_responses)
    }

    /// Create a connection with menu navigation
    pub fn menu_navigation() -> (MockAS400Connection, Vec<Vec<u8>>) {
        let mock = MockAS400Connection::with_menu_screen();

        // Simulate user typing menu option and pressing enter
        let responses = vec![
            vec![0xF1, 0x00, 0x00, 0x00, 0x00], // Initial menu
            vec![0xF1, 0x00, 0x00, 0x00, 0x00], // Response after selection
        ];

        (mock, responses)
    }

    /// Create a scenario with connection failure
    pub fn connection_failure() -> MockAS400Connection {
        MockAS400Connection::with_connection_failure()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_connection() {
        let mut mock = MockAS400Connection::new();
        assert!(!mock.is_connected());

        mock.set_connected(true);
        assert!(mock.is_connected());

        mock.set_connected(false);
        assert!(!mock.is_connected());
    }

    #[test]
    fn test_mock_send_receive() {
        let mut mock = MockAS400Connection::new();
        mock.set_connected(true);

        let test_data = vec![0x01, 0x02, 0x03];
        mock.add_response(test_data.clone());

        // Send data
        mock.send(&[0xF1]).unwrap();
        assert_eq!(mock.sent_data(), vec![vec![0xF1]]);

        // Receive data
        let mut buffer = vec![0; 10];
        let received = mock.receive(&mut buffer).unwrap();
        assert_eq!(received, 3);
        assert_eq!(&buffer[..3], &test_data[..]);
    }

    #[test]
    fn test_signon_scenario() {
        let (mock, _) = MockScenario::successful_connection();
        assert!(mock.responses.lock().unwrap().len() > 0);
    }
}