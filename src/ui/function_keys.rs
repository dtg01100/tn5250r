//! Function key UI components for TN5250R
//!
//! This module handles the rendering and interaction with function keys.

use eframe::egui;
use crate::app_state::TN5250RApp;
use crate::keyboard;
use crate::constants::FUNCTION_KEYS_PER_ROW;

/// Render function keys for a specific session
pub fn render_function_keys_for_session(ui: &mut egui::Ui, session: &mut crate::session::Session) {
    ui.separator();

    // Two rows of function keys (F1-F12, F13-F24)
    ui.columns(FUNCTION_KEYS_PER_ROW, |columns| {
        for i in 1..=FUNCTION_KEYS_PER_ROW {
            let col_idx = (i - 1) % 12;
            if columns[col_idx].button(format!("F{i}")).clicked() {
                // Convert number to function key and handle it
                use crate::keyboard::FunctionKey::*;
                let func_key = match i {
                    1 => F1, 2 => F2, 3 => F3, 4 => F4, 5 => F5, 6 => F6,
                    7 => F7, 8 => F8, 9 => F9, 10 => F10, 11 => F11, 12 => F12,
                    _ => F1, // Should not happen
                };
                // Send function key to session controller
                if let Err(e) = session.controller.send_function_key(func_key) {
                    session.terminal_content.push_str(&format!("\nError sending function key: {e}"));
                } else {
                    session.terminal_content.push_str(&format!("\n[{func_key:?}] pressed"));
                }
            }
        }
    });

    ui.columns(FUNCTION_KEYS_PER_ROW, |columns| {
        for i in (FUNCTION_KEYS_PER_ROW + 1)..=(FUNCTION_KEYS_PER_ROW * 2) {
            let col_idx = (i - 13) % 12;
            if columns[col_idx].button(format!("F{i}")).clicked() {
                // Convert number to function key and handle it
                use crate::keyboard::FunctionKey::*;
                let func_key = match i {
                    13 => F13, 14 => F14, 15 => F15, 16 => F16, 17 => F17, 18 => F18,
                    19 => F19, 20 => F20, 21 => F21, 22 => F22, 23 => F23, 24 => F24,
                    _ => F1, // Should not happen
                };
                // Send function key to session controller
                if let Err(e) = session.controller.send_function_key(func_key) {
                    session.terminal_content.push_str(&format!("\nError sending function key: {e}"));
                } else {
                    session.terminal_content.push_str(&format!("\n[{func_key:?}] pressed"));
                }
            }
        }
    });
}

impl TN5250RApp {
    /// Render function keys in the UI
    pub fn render_function_keys(&mut self, ui: &mut egui::Ui) {
        ui.separator();

        // Two rows of function keys (F1-F12, F13-F24)
        ui.columns(FUNCTION_KEYS_PER_ROW, |columns| {
            for i in 1..=FUNCTION_KEYS_PER_ROW {
                let col_idx = (i - 1) % 12;
                if columns[col_idx].button(format!("F{i}")).clicked() {
                    // Convert number to function key and handle it
                    use keyboard::FunctionKey::*;
                    let func_key = match i {
                        1 => F1, 2 => F2, 3 => F3, 4 => F4, 5 => F5, 6 => F6,
                        7 => F7, 8 => F8, 9 => F9, 10 => F10, 11 => F11, 12 => F12,
                        _ => F1, // Should not happen
                    };
                    self.send_function_key_direct(func_key);
                }
            }
        });

        ui.columns(FUNCTION_KEYS_PER_ROW, |columns| {
            for i in (FUNCTION_KEYS_PER_ROW + 1)..=(FUNCTION_KEYS_PER_ROW * 2) {
                let col_idx = (i - 13) % 12;
                if columns[col_idx].button(format!("F{i}")).clicked() {
                    // Convert number to function key and handle it
                    use keyboard::FunctionKey::*;
                    let func_key = match i {
                        13 => F13, 14 => F14, 15 => F15, 16 => F16, 17 => F17, 18 => F18,
                        19 => F19, 20 => F20, 21 => F21, 22 => F22, 23 => F23, 24 => F24,
                        _ => F1, // Should not happen
                    };
                    self.send_function_key_direct(func_key);
                }
            }
        });
    }
}