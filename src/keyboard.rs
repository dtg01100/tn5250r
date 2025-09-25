//! Keyboard mapping for AS/400 function keys
//! 
//! This module handles mapping of PC keyboard keys to AS/400 function keys.

use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FunctionKey {
    F1,
    F2, 
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13, // Field Exit
    F14, // Field Mark
    F15, // System Request
    F16, // Print
    F17, // Roll Down
    F18, // Roll Up
    F19, // Print Immediate
    F20, // Roll Left
    F21, // Roll Right
    F22, // Help
    F23, // Attn
    F24, // Attn
    Enter, // Enter key
}

impl FunctionKey {
    // Convert to the byte representation used in 5250 protocol
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            FunctionKey::F1 => vec![0x31, 0xF1],  // F1 key
            FunctionKey::F2 => vec![0x32, 0xF1],  // F2 key
            FunctionKey::F3 => vec![0x33, 0xF1],  // F3 key
            FunctionKey::F4 => vec![0x34, 0xF1],  // F4 key
            FunctionKey::F5 => vec![0x35, 0xF1],  // F5 key
            FunctionKey::F6 => vec![0x36, 0xF1],  // F6 key
            FunctionKey::F7 => vec![0x37, 0xF1],  // F7 key
            FunctionKey::F8 => vec![0x38, 0xF1],  // F8 key
            FunctionKey::F9 => vec![0x39, 0xF1],  // F9 key
            FunctionKey::F10 => vec![0x3A, 0xF1], // F10 key
            FunctionKey::F11 => vec![0x3B, 0xF1], // F11 key
            FunctionKey::F12 => vec![0x3C, 0xF1], // F12 key
            FunctionKey::F13 => vec![0x3D, 0xF1], // F13 key (Duplicate)
            FunctionKey::F14 => vec![0x3E, 0xF1], // F14 key (Field Exit)
            FunctionKey::F15 => vec![0x3F, 0xF1], // F15 key (Field Mark)
            FunctionKey::F16 => vec![0x40, 0xF1], // F16 key (System Request)
            FunctionKey::F17 => vec![0x41, 0xF1], // F17 key (Print)
            FunctionKey::F18 => vec![0x42, 0xF1], // F18 key (Roll Down)
            FunctionKey::F19 => vec![0x43, 0xF1], // F19 key (Roll Up)
            FunctionKey::F20 => vec![0x44, 0xF1], // F20 key (Print Immediate)
            FunctionKey::F21 => vec![0x45, 0xF1], // F21 key (Roll Left)
            FunctionKey::F22 => vec![0x46, 0xF1], // F22 key (Roll Right)
            FunctionKey::F23 => vec![0x47, 0xF1], // F23 key (Help)
            FunctionKey::F24 => vec![0x48, 0xF1], // F24 key (Attn)
            FunctionKey::Enter => vec![0x0D], // Enter key (carriage return)
        }
    }
}

// Map virtual keys to function keys
pub fn map_virtual_key_to_function_key(key: egui::Key) -> Option<FunctionKey> {
    match key {
        egui::Key::F1 => Some(FunctionKey::F1),
        egui::Key::F2 => Some(FunctionKey::F2),
        egui::Key::F3 => Some(FunctionKey::F3),
        egui::Key::F4 => Some(FunctionKey::F4),
        egui::Key::F5 => Some(FunctionKey::F5),
        egui::Key::F6 => Some(FunctionKey::F6),
        egui::Key::F7 => Some(FunctionKey::F7),
        egui::Key::F8 => Some(FunctionKey::F8),
        egui::Key::F9 => Some(FunctionKey::F9),
        egui::Key::F10 => Some(FunctionKey::F10),
        egui::Key::F11 => Some(FunctionKey::F11),
        egui::Key::F12 => Some(FunctionKey::F12),
        egui::Key::F13 => Some(FunctionKey::F13),
        egui::Key::F14 => Some(FunctionKey::F14),
        egui::Key::F15 => Some(FunctionKey::F15),
        egui::Key::F16 => Some(FunctionKey::F16),
        egui::Key::F17 => Some(FunctionKey::F17),
        egui::Key::F18 => Some(FunctionKey::F18),
        egui::Key::F19 => Some(FunctionKey::F19),
        egui::Key::F20 => Some(FunctionKey::F20),
        egui::Key::F21 => Some(FunctionKey::F21),
        egui::Key::F22 => Some(FunctionKey::F22),
        egui::Key::F23 => Some(FunctionKey::F23),
        egui::Key::F24 => Some(FunctionKey::F24),
        _ => None,
    }
}

// Check if a key event should be handled as a special AS/400 key
pub fn is_special_as400_key(key_event: &egui::Event) -> bool {
    match key_event {
        egui::Event::Key { key, pressed, .. } => {
            *pressed && map_virtual_key_to_function_key(*key).is_some()
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_key_to_bytes() {
        let f1_bytes = FunctionKey::F1.to_bytes();
        assert_eq!(f1_bytes, vec![0x31, 0xF1]);
        
        let f12_bytes = FunctionKey::F12.to_bytes();
        assert_eq!(f12_bytes, vec![0x3C, 0xF1]);
    }

    #[test]
    fn test_map_virtual_key() {
        assert_eq!(map_virtual_key_to_function_key(egui::Key::F1), Some(FunctionKey::F1));
        assert_eq!(map_virtual_key_to_function_key(egui::Key::A), None);
    }
}