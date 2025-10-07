//! Input handling for TN5250R
//!
//! This module handles keyboard events, text input, and function key processing.

use eframe::egui;
use crate::app_state::TN5250RApp;
use crate::keyboard;

impl TN5250RApp {
    pub fn send_function_key(&mut self, key_name: &str) {
        // In a real implementation, this would send the actual function key
        // For now, we'll just update the terminal content
        self.terminal_content.push_str(&format!("\n[{key_name}] pressed"));

        // Parse the key name to determine which function key to send
        let func_key = self.parse_function_key_name(key_name);

        // Simulate sending the function key
        match self.controller.send_function_key(func_key) {
            Ok(()) => {
                // Function key sent successfully
            }
            Err(e) => {
                self.terminal_content.push_str(&format!("\nError sending function key: {e}"));
            }
        }
    }

    pub fn parse_function_key_name(&self, key_name: &str) -> keyboard::FunctionKey {
        // Parse the key name string to determine the function key
        // Expected format: "F1", "F2", etc.
        if let Some(num_str) = key_name.strip_prefix('F') {
            if let Ok(num) = num_str.parse::<u8>() {
                return match num {
                    1 => keyboard::FunctionKey::F1,
                    2 => keyboard::FunctionKey::F2,
                    3 => keyboard::FunctionKey::F3,
                    4 => keyboard::FunctionKey::F4,
                    5 => keyboard::FunctionKey::F5,
                    6 => keyboard::FunctionKey::F6,
                    7 => keyboard::FunctionKey::F7,
                    8 => keyboard::FunctionKey::F8,
                    9 => keyboard::FunctionKey::F9,
                    10 => keyboard::FunctionKey::F10,
                    11 => keyboard::FunctionKey::F11,
                    12 => keyboard::FunctionKey::F12,
                    13 => keyboard::FunctionKey::F13,
                    14 => keyboard::FunctionKey::F14,
                    15 => keyboard::FunctionKey::F15,
                    16 => keyboard::FunctionKey::F16,
                    17 => keyboard::FunctionKey::F17,
                    18 => keyboard::FunctionKey::F18,
                    19 => keyboard::FunctionKey::F19,
                    20 => keyboard::FunctionKey::F20,
                    21 => keyboard::FunctionKey::F21,
                    22 => keyboard::FunctionKey::F22,
                    23 => keyboard::FunctionKey::F23,
                    24 => keyboard::FunctionKey::F24,
                    _ => keyboard::FunctionKey::F1, // Default fallback
                };
            }
        }
        // Default fallback if parsing fails
        keyboard::FunctionKey::F1
    }

    pub fn send_function_key_direct(&mut self, func_key: keyboard::FunctionKey) {
        // Send the actual function key
        match self.controller.send_function_key(func_key) {
            Ok(()) => {
                // Function key sent successfully
                self.terminal_content.push_str(&format!("\n[{func_key:?}] pressed"));
            }
            Err(e) => {
                self.terminal_content.push_str(&format!("\nError sending function key: {e}"));
            }
        }
    }

    /// Handle keyboard input events
    pub fn handle_keyboard_input(&mut self, ctx: &egui::Context) -> bool {
        // Reset Tab flag at start of frame
        self.tab_pressed_this_frame = false;

        // Handle keyboard events - check if Tab is pressed and consume it for field navigation
        let mut tab_used_for_navigation = false;

        // First, check for Tab key and handle field navigation
        let should_handle_tab = ctx.input(|i| {
            i.key_pressed(egui::Key::Tab) && self.connected && !self.fields_info.is_empty()
        });

        if should_handle_tab {
            tab_used_for_navigation = true;
            self.tab_pressed_this_frame = true;

            let is_shift = ctx.input(|i| i.modifiers.shift);

            if is_shift {
                if let Err(e) = self.controller.previous_field() {
                    eprintln!("Failed to navigate to previous field: {e}");
                }
            } else if let Err(e) = self.controller.next_field() {
                eprintln!("Failed to navigate to next field: {e}");
            }
        }

        // Handle other keyboard events (but not Tab if we used it for navigation)
        ctx.input(|i| {

            // Handle other keyboard events
            for event in &i.events {
                match event {
                    egui::Event::Key { key, pressed: true, modifiers: _, .. } => {
                        match key {
                            egui::Key::Tab => {
                                // Already handled above
                            }
                            egui::Key::Enter => {
                                // Handle Enter in fields
                                if let Err(e) = self.controller.send_enter() {
                                    eprintln!("Failed to send Enter: {e}");
                                }
                            }
                            egui::Key::Backspace => {
                                if let Err(e) = self.controller.backspace() {
                                    eprintln!("Failed to send backspace: {e}");
                                }
                            }
                            egui::Key::Delete => {
                                if let Err(e) = self.controller.delete() {
                                    eprintln!("Failed to send delete: {e}");
                                }
                            }
                            egui::Key::F1 => {
                                if let Err(e) = self.controller.send_function_key(keyboard::FunctionKey::F1) {
                                    eprintln!("Failed to send F1: {e}");
                                }
                            }
                            egui::Key::F2 => {
                                if let Err(e) = self.controller.send_function_key(keyboard::FunctionKey::F2) {
                                    eprintln!("Failed to send F2: {e}");
                                }
                            }
                            egui::Key::F3 => {
                                if let Err(e) = self.controller.send_function_key(keyboard::FunctionKey::F3) {
                                    eprintln!("Failed to send F3: {e}");
                                }
                            }
                            _ => {
                                // Let egui handle other keys normally
                            }
                        }
                    }
                    egui::Event::Text(text) => {
                        // Handle text input for fields, but only if we're connected and have fields
                        if self.connected {
                            for ch in text.chars() {
                                if ch.is_ascii() && !ch.is_control() {
                                    if let Err(e) = self.controller.type_char(ch) {
                                        eprintln!("Failed to type character '{ch}': {e}");
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        // Let egui handle other events normally
                    }
                }
            }
        });

        tab_used_for_navigation
    }
}