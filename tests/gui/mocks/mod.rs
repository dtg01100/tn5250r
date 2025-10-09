//! Mock implementations for GUI testing infrastructure
//!
//! This module provides mock objects and utilities to support comprehensive
//! GUI testing of the TN5250R application without external dependencies.

// Note: GUI-specific mocks are defined in this module; avoid cross-crate re-exports to reduce coupling.

/// Mock UI component for testing button interactions
pub struct MockButton {
    pub label: String,
    pub clicked: std::cell::RefCell<bool>,
    pub enabled: std::cell::RefCell<bool>,
}

impl MockButton {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            clicked: std::cell::RefCell::new(false),
            enabled: std::cell::RefCell::new(true),
        }
    }

    pub fn click(&self) {
        *self.clicked.borrow_mut() = true;
    }

    pub fn is_clicked(&self) -> bool {
        *self.clicked.borrow()
    }

    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.borrow_mut() = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.borrow()
    }

    pub fn reset(&self) {
        *self.clicked.borrow_mut() = false;
    }
}


/// Mock text input field for testing user input
pub struct MockTextField {
    pub placeholder: String,
    pub value: std::cell::RefCell<String>,
    pub focused: std::cell::RefCell<bool>,
    pub max_length: Option<usize>,
}

impl MockTextField {
    pub fn new(placeholder: &str) -> Self {
        Self {
            placeholder: placeholder.to_string(),
            value: std::cell::RefCell::new(String::new()),
            focused: std::cell::RefCell::new(false),
            max_length: None,
        }
    }

    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn set_value(&self, value: &str) {
        let mut current_value = self.value.borrow_mut();
        if let Some(max_len) = self.max_length {
            *current_value = value.chars().take(max_len).collect();
        } else {
            *current_value = value.to_string();
        }
    }

    pub fn get_value(&self) -> String {
        self.value.borrow().clone()
    }

    pub fn append_text(&self, text: &str) {
        let mut current_value = self.value.borrow_mut();
        if let Some(max_len) = self.max_length {
            let new_value = format!("{}{}", *current_value, text);
            *current_value = new_value.chars().take(max_len).collect();
        } else {
            current_value.push_str(text);
        }
    }

    pub fn clear(&self) {
        self.value.borrow_mut().clear();
    }

    pub fn focus(&self) {
        *self.focused.borrow_mut() = true;
    }

    pub fn unfocus(&self) {
        *self.focused.borrow_mut() = false;
    }

    pub fn is_focused(&self) -> bool {
        *self.focused.borrow()
    }
}


/// Mock dialog for testing modal interactions
pub struct MockDialog {
    pub title: String,
    pub visible: std::cell::RefCell<bool>,
    pub confirmed: std::cell::RefCell<bool>,
    pub cancelled: std::cell::RefCell<bool>,
}

impl MockDialog {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            visible: std::cell::RefCell::new(false),
            confirmed: std::cell::RefCell::new(false),
            cancelled: std::cell::RefCell::new(false),
        }
    }

    pub fn show(&self) {
        *self.visible.borrow_mut() = true;
        *self.confirmed.borrow_mut() = false;
        *self.cancelled.borrow_mut() = false;
    }

    pub fn hide(&self) {
        *self.visible.borrow_mut() = false;
    }

    pub fn is_visible(&self) -> bool {
        *self.visible.borrow()
    }

    pub fn confirm(&self) {
        *self.confirmed.borrow_mut() = true;
        self.hide();
    }

    pub fn cancel(&self) {
        *self.cancelled.borrow_mut() = true;
        self.hide();
    }

    pub fn is_confirmed(&self) -> bool {
        *self.confirmed.borrow()
    }

    pub fn is_cancelled(&self) -> bool {
        *self.cancelled.borrow()
    }

    pub fn reset(&self) {
        *self.visible.borrow_mut() = false;
        *self.confirmed.borrow_mut() = false;
        *self.cancelled.borrow_mut() = false;
    }
}


/// Mock keyboard event simulator
pub struct MockKeyboard {
    pub pressed_keys: std::cell::RefCell<Vec<egui::Key>>,
    pub modifiers: std::cell::RefCell<egui::Modifiers>,
}

impl MockKeyboard {
    pub fn new() -> Self {
        Self {
            pressed_keys: std::cell::RefCell::new(Vec::new()),
            modifiers: std::cell::RefCell::new(egui::Modifiers::default()),
        }
    }

    pub fn press_key(&self, key: egui::Key) {
        self.pressed_keys.borrow_mut().push(key);
    }

    pub fn press_enter(&self) {
        self.press_key(egui::Key::Enter);
    }

    pub fn press_tab(&self) {
        self.press_key(egui::Key::Tab);
    }

    pub fn press_escape(&self) {
        self.press_key(egui::Key::Escape);
    }

    pub fn press_function_key(&self, f_key: u8) {
        match f_key {
            1 => self.press_key(egui::Key::F1),
            2 => self.press_key(egui::Key::F2),
            3 => self.press_key(egui::Key::F3),
            4 => self.press_key(egui::Key::F4),
            5 => self.press_key(egui::Key::F5),
            6 => self.press_key(egui::Key::F6),
            7 => self.press_key(egui::Key::F7),
            8 => self.press_key(egui::Key::F8),
            9 => self.press_key(egui::Key::F9),
            10 => self.press_key(egui::Key::F10),
            11 => self.press_key(egui::Key::F11),
            12 => self.press_key(egui::Key::F12),
            _ => {} // Invalid function key
        }
    }

    pub fn set_modifier(&self, modifier: egui::Modifiers) {
        *self.modifiers.borrow_mut() = modifier;
    }

    pub fn get_pressed_keys(&self) -> Vec<egui::Key> {
        self.pressed_keys.borrow().clone()
    }

    pub fn clear_pressed_keys(&self) {
        self.pressed_keys.borrow_mut().clear();
    }

    pub fn get_modifiers(&self) -> egui::Modifiers {
        *self.modifiers.borrow()
    }
}

/// Mock mouse event simulator
pub struct MockMouse {
    pub position: std::cell::RefCell<egui::Pos2>,
    pub clicked: std::cell::RefCell<bool>,
    pub button: std::cell::RefCell<egui::PointerButton>,
}

impl MockMouse {
    pub fn new() -> Self {
        Self {
            position: std::cell::RefCell::new(egui::Pos2::ZERO),
            clicked: std::cell::RefCell::new(false),
            button: std::cell::RefCell::new(egui::PointerButton::Primary),
        }
    }

    pub fn move_to(&self, position: egui::Pos2) {
        *self.position.borrow_mut() = position;
    }

    pub fn click(&self) {
        *self.clicked.borrow_mut() = true;
    }

    pub fn click_at(&self, position: egui::Pos2) {
        self.move_to(position);
        self.click();
    }

    pub fn set_button(&self, button: egui::PointerButton) {
        *self.button.borrow_mut() = button;
    }

    pub fn get_position(&self) -> egui::Pos2 {
        *self.position.borrow()
    }

    pub fn is_clicked(&self) -> bool {
        *self.clicked.borrow()
    }

    pub fn get_button(&self) -> egui::PointerButton {
        *self.button.borrow()
    }

    pub fn reset_click(&self) {
        *self.clicked.borrow_mut() = false;
    }
}

/// Test scenario builder for common GUI testing patterns
pub struct MockScenarioBuilder {
    pub buttons: Vec<MockButton>,
    pub text_fields: Vec<MockTextField>,
    pub dialogs: Vec<MockDialog>,
    pub keyboard: MockKeyboard,
    pub mouse: MockMouse,
}

impl MockScenarioBuilder {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            text_fields: Vec::new(),
            dialogs: Vec::new(),
            keyboard: MockKeyboard::new(),
            mouse: MockMouse::new(),
        }
    }

    pub fn add_button(mut self, label: &str) -> Self {
        self.buttons.push(MockButton::new(label));
        self
    }

    pub fn add_text_field(mut self, placeholder: &str) -> Self {
        self.text_fields.push(MockTextField::new(placeholder));
        self
    }

    pub fn add_dialog(mut self, title: &str) -> Self {
        self.dialogs.push(MockDialog::new(title));
        self
    }

    pub fn build(self) -> MockScenario {
        MockScenario {
            buttons: self.buttons,
            text_fields: self.text_fields,
            dialogs: self.dialogs,
            keyboard: self.keyboard,
            mouse: self.mouse,
        }
    }
}

/// Complete mock scenario for GUI testing
pub struct MockScenario {
    pub buttons: Vec<MockButton>,
    pub text_fields: Vec<MockTextField>,
    pub dialogs: Vec<MockDialog>,
    pub keyboard: MockKeyboard,
    pub mouse: MockMouse,
}

impl MockScenario {
    /// Create a login form scenario
    pub fn login_form() -> Self {
        MockScenarioBuilder::new()
            .add_text_field("Username")
            .add_text_field("Password")
            .add_button("Login")
            .add_button("Cancel")
            .build()
    }

    /// Create a connection dialog scenario
    pub fn connection_dialog() -> Self {
        MockScenarioBuilder::new()
            .add_text_field("Host")
            .add_text_field("Port")
            .add_text_field("Username")
            .add_text_field("Password")
            .add_button("Connect")
            .add_button("Advanced")
            .add_button("Cancel")
            .build()
    }

    /// Create a menu navigation scenario
    pub fn menu_navigation() -> Self {
        MockScenarioBuilder::new()
            .add_text_field("Selection")
            .add_button("Enter")
            .build()
    }

    /// Reset all mock components to their initial state
    pub fn reset(&self) {
        for button in &self.buttons {
            button.reset();
        }
        for text_field in &self.text_fields {
            text_field.clear();
            text_field.unfocus();
        }
        for dialog in &self.dialogs {
            dialog.reset();
        }
        self.keyboard.clear_pressed_keys();
        self.mouse.reset_click();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_button() {
        let button = MockButton::new("Test Button");
        assert!(!button.is_clicked());
        assert!(button.is_enabled());

        button.click();
        assert!(button.is_clicked());

        button.set_enabled(false);
        assert!(!button.is_enabled());

        button.reset();
        assert!(!button.is_clicked());
    }

    #[test]
    fn test_mock_text_field() {
        let field = MockTextField::new("Enter text").with_max_length(10);

        assert_eq!(field.get_value(), "");
        assert!(!field.is_focused());

        field.set_value("Hello World");
        assert_eq!(field.get_value(), "Hello Worl"); // Truncated to 10 chars

        field.focus();
        assert!(field.is_focused());

        field.clear();
        assert_eq!(field.get_value(), "");
    }

    #[test]
    fn test_mock_dialog() {
        let dialog = MockDialog::new("Test Dialog");
        assert!(!dialog.is_visible());

        dialog.show();
        assert!(dialog.is_visible());

        dialog.confirm();
        assert!(!dialog.is_visible());
        assert!(dialog.is_confirmed());
        assert!(!dialog.is_cancelled());
    }

    #[test]
    fn test_mock_keyboard() {
        let keyboard = MockKeyboard::new();
        assert!(keyboard.get_pressed_keys().is_empty());

        keyboard.press_enter();
        keyboard.press_tab();

        let keys = keyboard.get_pressed_keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&egui::Key::Enter));
        assert!(keys.contains(&egui::Key::Tab));

        keyboard.clear_pressed_keys();
        assert!(keyboard.get_pressed_keys().is_empty());
    }

    #[test]
    fn test_mock_mouse() {
        let mouse = MockMouse::new();
        assert!(!mouse.is_clicked());

        let pos = egui::pos2(100.0, 200.0);
        mouse.click_at(pos);

        assert!(mouse.is_clicked());
        assert_eq!(mouse.get_position(), pos);

        mouse.reset_click();
        assert!(!mouse.is_clicked());
    }

    #[test]
    fn test_mock_scenario_builder() {
        let scenario = MockScenarioBuilder::new()
            .add_button("OK")
            .add_text_field("Name")
            .add_dialog("Confirm")
            .build();

        assert_eq!(scenario.buttons.len(), 1);
        assert_eq!(scenario.text_fields.len(), 1);
        assert_eq!(scenario.dialogs.len(), 1);
    }

    #[test]
    fn test_login_form_scenario() {
        let scenario = MockScenario::login_form();
        assert_eq!(scenario.buttons.len(), 2); // Login and Cancel
        assert_eq!(scenario.text_fields.len(), 2); // Username and Password
    }

    #[test]
    fn test_connection_dialog_scenario() {
        let scenario = MockScenario::connection_dialog();
        assert_eq!(scenario.buttons.len(), 3); // Connect, Advanced, Cancel
        assert_eq!(scenario.text_fields.len(), 4); // Host, Port, Username, Password
    }
}
