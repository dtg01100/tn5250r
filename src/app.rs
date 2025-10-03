//! Main application loop for TN5250R
//!
//! This module contains the eframe::App implementation and the main UI update loop.

use eframe::egui;
use crate::app_state::TN5250RApp;
use crate::config;



impl eframe::App for TN5250RApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let tab_used_for_navigation = self.handle_keyboard_input(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Connect").clicked() {
                        self.do_connect();
                        ui.close();
                    }
                    if ui.button("Disconnect").clicked() {
                        self.do_disconnect();
                        ui.close();
                    }
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.function_keys_visible, "Function Keys");
                    ui.checkbox(&mut self.show_monitoring_dashboard, "Monitoring Dashboard");
                });

                ui.menu_button("Settings", |ui| {
                    if ui.button("Terminal Settings").clicked() {
                        self.show_settings_dialog = true;
                        ui.close();
                    }
                    if ui.button("Advanced Connection Settings").clicked() {
                        self.show_advanced_settings = true;
                        ui.close();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // If we used Tab for field navigation, prevent egui widget focus
            if tab_used_for_navigation {
                ui.memory_mut(|mem| {
                    // Clear focus entirely to prevent widgets from getting Tab focus
                    mem.surrender_focus(egui::Id::NULL);
                });
            }

            // Show error message prominently if present
            if let Some(ref error) = self.error_message {
                ui.colored_label(egui::Color32::RED, format!("⚠ Error: {}", error));
                ui.separator();
            }

            ui.heading("TN5250R - IBM AS/400 Terminal Emulator");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Host:");
                if ui.text_edit_singleline(&mut self.connection_string).changed() {
                    // Update host and port when connection string changes
                    let (host, port) = self.parse_connection_string();
                    self.host = host;
                    self.port = port;
                    // Sync to configuration; do NOT auto-toggle TLS, keep user's persisted choice
                    if let Ok(mut cfg) = self.config.try_lock() {
                        cfg.set_property("connection.host", self.host.as_str());
                        cfg.set_property("connection.port", self.port as i64);
                    }
                    // Persist change (async to avoid blocking GUI)
                    config::save_shared_config_async(&self.config);
                }

                if ui.button("Connect").clicked() {
                    self.do_connect();
                }

                if self.connecting {
                    if ui.button("Cancel").clicked() {
                        self.controller.cancel_connect();
                        self.connecting = false;
                        self.connection_time = None;
                        self.terminal_content.push_str("\nConnection canceled by user.\n");
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.debug_mode {
                        if ui.button("🐛 Debug").on_hover_text("Show debug information panel").clicked() {
                            self.show_debug_panel = !self.show_debug_panel;
                        }
                    }
                    if ui.button("⚙ Advanced").on_hover_text("Advanced connection settings").clicked() {
                        self.show_advanced_settings = true;
                    }
                });
            });

            // Username and Password fields for AS/400 authentication (RFC 4777)
            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.text_edit_singleline(&mut self.username);

                ui.label("Password:");
                ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
            });

            ui.separator();

            // Display terminal content with cursor and click handling
            let scroll_area_response = egui::ScrollArea::vertical()
                .id_salt("terminal_display")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.draw_terminal_with_cursor(ui);
                });

            // Handle mouse clicks on the scroll area content
            let content_rect = scroll_area_response.inner_rect;
            let response = ui.interact(content_rect, egui::Id::new("terminal_area"), egui::Sense::click());

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    // Calculate position relative to the content area
                    let relative_pos = pos - content_rect.min;

                    // Get font metrics for coordinate calculation
                    let font = egui::FontId::monospace(14.0);
                    let char_width = ui.fonts(|f| f.glyph_width(&font, ' '));
                    let line_height = ui.fonts(|f| f.row_height(&font));

                    let col = (relative_pos.x / char_width).floor() as usize + 1; // Convert to 1-based
                    let row = (relative_pos.y / line_height).floor() as usize + 1; // Convert to 1-based

                    // Clamp to valid terminal bounds
                    let row = row.max(1).min(24);
                    let col = col.max(1).min(80);

                    if let Err(e) = self.controller.click_at_position(row, col) {
                        eprintln!("Failed to click at position ({}, {}): {}", row, col, e);
                    }
                }
            }

            // Display field information if available
            if !self.fields_info.is_empty() {
                ui.separator();
                ui.collapsing("Field Information", |ui| {
                    for (i, field) in self.fields_info.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if field.is_active {
                                ui.colored_label(egui::Color32::GREEN, "►");
                            } else {
                                ui.label(" ");
                            }
                            ui.label(format!("Field {}: {}", i + 1, field.label));
                            ui.label(format!("Content: '{}'", field.content));

                            // Show error if present
                            if let Some(error) = &field.error_state {
                                ui.colored_label(egui::Color32::RED, format!("Error: {}", error.get_user_message()));
                            }

                            // Show highlight status
                            if field.highlighted {
                                ui.colored_label(egui::Color32::YELLOW, "Highlighted");
                            }
                        });
                    }
                    ui.label("Use Tab/Shift+Tab to navigate between fields");
                });
            }

            // Display monitoring dashboard if enabled
            if self.show_monitoring_dashboard {
                ui.separator();
                self.show_monitoring_dashboard_ui(ui);
            }

            ui.separator();

            // Input area for commands
            ui.horizontal(|ui| {
                ui.label("Input:");
                if ui.text_edit_singleline(&mut self.input_buffer).lost_focus() &&
                    ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    // Process the input when Enter is pressed
                    if !self.input_buffer.is_empty() {
                        // Echo the input to terminal
                        self.terminal_content.push_str(&format!("\n> {}", self.input_buffer));

                        // Send to controller
                        if let Err(e) = self.controller.send_input(self.input_buffer.as_bytes()) {
                            self.terminal_content.push_str(&format!("\nError: {}", e));
                        }

                        self.terminal_content.push_str("\nResponse would go here...\n");
                        self.input_buffer.clear();
                    }
                }

                if ui.button("Send").clicked() && !self.input_buffer.is_empty() {
                    // Process the input when Send button is clicked
                    self.terminal_content.push_str(&format!("\n> {}", self.input_buffer));

                    // Send to controller
                    if let Err(e) = self.controller.send_input(self.input_buffer.as_bytes()) {
                        self.terminal_content.push_str(&format!("\nError: {}", e));
                    }

                    self.terminal_content.push_str("\nResponse would go here...\n");
                    self.input_buffer.clear();
                }
            });

            // Display function keys if enabled
            if self.function_keys_visible {
                self.render_function_keys(ui);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    if self.connecting {
                        ui.colored_label(egui::Color32::YELLOW, &format!("Connecting to {}:{} ... ", self.host, self.port));
                    } else if self.connected {
                        ui.colored_label(egui::Color32::GREEN, &format!("Connected to {}:{} ", self.host, self.port));
                    } else {
                        ui.colored_label(egui::Color32::RED, "Disconnected");
                    }
                    ui.separator();

                    // Show input buffer status for feedback
                    if let Ok(pending_size) = self.controller.get_pending_input_size() {
                        if pending_size > 0 {
                            ui.colored_label(egui::Color32::BLUE, &format!("Input buffered ({} bytes)", pending_size));
                            ui.separator();
                        }
                    }

                    ui.label("Ready");
                });
            });
        });

        // Process incoming data and update terminal content
        let content_changed = self.update_terminal_content();

        // Check if new data arrived for event-driven repaints
        let data_arrived = self.controller.check_data_arrival().unwrap_or(false);

        // Show debug panel if requested
        if self.show_debug_panel {
            self.show_debug_panel_dialog(ctx);
        }

        // Show advanced settings dialog if requested
        if self.show_advanced_settings {
            self.show_advanced_settings_dialog(ctx);
        }

        // Show terminal settings dialog if requested
        if self.show_settings_dialog {
            self.show_settings_dialog(ctx);
        }

        // Smart repaint logic to prevent CPU waste:
        // - Disconnected: No repaints (only on user interaction)
        // - Connecting: Check every 100ms for connection completion
        // - Connected with recent data: Check every 50ms for smooth updates
        // - Connected but idle: Check every 500ms for status/errors
        if self.connecting {
            // Check every 100ms while connecting
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else if self.connected {
            if content_changed || data_arrived {
                // Content just changed or data arrived, check again soon for more data
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            } else {
                // No recent changes, check every 500ms for connection status/errors
                // This reduces CPU usage dramatically when idle
                ctx.request_repaint_after(std::time::Duration::from_millis(500));
            }
        }
        // When disconnected, egui only repaints on user interaction (0% CPU)
    }
}