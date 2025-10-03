//! Terminal display rendering for TN5250R
//!
//! This module handles the visual rendering of the terminal content, including cursor positioning
//! and field highlighting.

use eframe::egui;
use crate::app_state::TN5250RApp;

impl TN5250RApp {
    pub fn draw_terminal_with_cursor(&mut self, ui: &mut egui::Ui) {
        // Get cursor position
        let cursor_pos = self.controller.get_cursor_position().unwrap_or((1, 1));

        // Split terminal content into lines
        let lines: Vec<&str> = self.terminal_content.lines().collect();

        // Calculate character size for positioning
        let font = egui::FontId::monospace(14.0);
        let char_width = ui.fonts(|f| f.glyph_width(&font, ' '));
        let line_height = ui.fonts(|f| f.row_height(&font));

        // Create a layout area for the terminal content
        let available_size = ui.available_size();
        let (rect, _response) = ui.allocate_exact_size(available_size, egui::Sense::hover());

        // Draw the terminal content
        if ui.is_rect_visible(rect) {
            let mut y_offset = 0.0;

            for (line_idx, line) in lines.iter().enumerate() {
                let line_number = line_idx + 1; // 1-based line numbers

                // Draw each character in the line
                for (char_idx, ch) in line.chars().enumerate() {
                    let col_number = char_idx + 1; // 1-based column numbers

                    let char_pos = rect.min + egui::vec2(char_idx as f32 * char_width, y_offset);

                    // Check if this is the cursor position
                    let is_cursor = cursor_pos.0 == line_number && cursor_pos.1 == col_number;

                    // Determine background color based on field status
                    let mut bg_color = egui::Color32::TRANSPARENT;
                    let mut text_color = egui::Color32::WHITE;

                    // Check if position is in an error field
                    for field in &self.fields_info {
                        if line_number == field.start_row &&
                            col_number >= field.start_col &&
                            col_number < field.start_col + field.length {
                            if field.error_state.is_some() {
                                bg_color = egui::Color32::RED; // Red background for error fields
                                text_color = egui::Color32::WHITE;
                            } else if field.highlighted {
                                bg_color = egui::Color32::YELLOW; // Yellow background for highlighted fields
                                text_color = egui::Color32::BLACK;
                            } else if field.is_active {
                                bg_color = egui::Color32::BLUE; // Blue background for active field
                                text_color = egui::Color32::WHITE;
                            }
                            break;
                        }
                    }

                    // Override for cursor
                    if is_cursor {
                        bg_color = egui::Color32::GREEN;
                        text_color = egui::Color32::BLACK;
                    }

                    // Draw background if needed
                    if bg_color != egui::Color32::TRANSPARENT {
                        let char_rect = egui::Rect::from_min_size(
                            char_pos,
                            egui::vec2(char_width, line_height)
                        );
                        ui.painter().rect_filled(char_rect, egui::CornerRadius::ZERO, bg_color);
                    }

                    // Draw the character
                    ui.painter().text(
                        char_pos,
                        egui::Align2::LEFT_TOP,
                        ch,
                        font.clone(),
                        text_color,
                    );
                }

                y_offset += line_height;
            }

            // Draw cursor if it's beyond the text content
            if cursor_pos.0 as usize > lines.len() ||
                (cursor_pos.0 as usize <= lines.len() &&
                 cursor_pos.1 as usize > lines.get(cursor_pos.0 - 1).map_or(0, |l| l.len())) {

                let cursor_char_pos = rect.min + egui::vec2(
                    (cursor_pos.1 - 1) as f32 * char_width,
                    (cursor_pos.0 - 1) as f32 * line_height
                );

                let cursor_rect = egui::Rect::from_min_size(
                    cursor_char_pos,
                    egui::vec2(char_width, line_height)
                );
                ui.painter().rect_filled(cursor_rect, egui::CornerRadius::ZERO, egui::Color32::GREEN);

                // Draw a space character at cursor
                ui.painter().text(
                    cursor_char_pos,
                    egui::Align2::LEFT_TOP,
                    ' ',
                    font,
                    egui::Color32::BLACK,
                );
            }
        }
    }
}