//! Dialog UI components for TN5250R
//!
//! This module contains various dialog windows like debug panel, settings, and advanced settings.

use eframe::egui;
use crate::app_state::TN5250RApp;
use crate::config;
use crate::lib3270::display::ScreenSize;
use crate::network::ProtocolMode;

impl TN5250RApp {
    /// Show debug information panel for troubleshooting
    pub fn show_debug_panel_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("🐛 Debug Information")
            .collapsible(true)
            .resizable(true)
            .default_size([700.0, 500.0])
            .show(ctx, |ui| {
                ui.heading("Terminal Emulator Debug Panel");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.collapsing("Connection State", |ui| {
                        ui.label(format!("Connected: {}", self.connected));
                        ui.label(format!("Connecting: {}", self.connecting));
                        ui.label(format!("Host: {}:{}", self.host, self.port));
                        ui.label(format!("Username: {}", if self.username.is_empty() { "<none>" } else { &self.username }));
                        ui.label(format!("Password: {}", if self.password.is_empty() { "<none>" } else { "****" }));
                        if let Some(time) = self.connection_time {
                            ui.label(format!("Connection duration: {:.2}s", time.elapsed().as_secs_f32()));
                        }
                    });

                    ui.separator();

                    ui.collapsing("Terminal Content", |ui| {
                        ui.label(format!("Content length: {} chars", self.terminal_content.len()));
                        ui.label(format!("Content lines: {}", self.terminal_content.lines().count()));
                        ui.separator();
                        ui.label("First 500 chars:");
                        ui.code(self.terminal_content.chars().take(500).collect::<String>());
                        ui.separator();
                        ui.label("Last 200 chars:");
                        let skip = self.terminal_content.len().saturating_sub(200);
                        ui.code(self.terminal_content.chars().skip(skip).collect::<String>());
                    });

                    ui.separator();

                    ui.collapsing("Field Information", |ui| {
                        ui.label(format!("Number of fields: {}", self.fields_info.len()));
                        for (i, field) in self.fields_info.iter().enumerate() {
                            ui.group(|ui| {
                                ui.label(format!("Field {}:", i + 1));
                                ui.label(format!("  Label: {}", field.label));
                                ui.label(format!("  Content: '{}'", field.content));
                                ui.label(format!("  Active: {}", field.is_active));
                                ui.label(format!("  Highlighted: {}", field.highlighted));
                                if let Some(error) = &field.error_state {
                                    ui.colored_label(egui::Color32::RED, format!("  Error: {}", error.get_user_message()));
                                }
                            });
                        }
                    });

                    ui.separator();

                    ui.collapsing("Raw Data Dump", |ui| {
                        ui.label(format!("Last packet size: {} bytes", self.last_data_size));
                        if !self.raw_buffer_dump.is_empty() {
                            ui.separator();
                            ui.label("Hex dump of last received data:");
                            ui.code(&self.raw_buffer_dump);
                        } else {
                            ui.label("No data captured yet");
                        }
                    });

                    ui.separator();

                    ui.collapsing("Controller State", |ui| {
                        if let Ok(content) = self.controller.get_terminal_content() {
                            ui.label(format!("Controller content length: {} chars", content.len()));
                            ui.separator();
                            ui.label("Raw controller content (first 1000 bytes as hex):");
                            let hex: String = content.bytes().take(1000)
                                .map(|b| format!("{b:02x} "))
                                .collect();
                            ui.code(hex);
                        }
                    });

                    ui.separator();

                    if ui.button("Close Debug Panel").clicked() {
                        self.show_debug_panel = false;
                    }
                });
            });
    }

    /// Show the advanced settings dialog
    pub fn show_advanced_settings_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Advanced Connection Settings")
            .collapsible(false)
            .resizable(true)
            .default_size([400.0, 300.0])
            .show(ctx, |ui| {
                ui.heading("Advanced TLS/SSL Settings");
                ui.separator();

                egui::Grid::new("advanced_settings_grid")
                    .num_columns(2)
                    .spacing([40.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Use TLS (SSL):");
                        let mut ssl_enabled = {
                            if let Ok(cfg) = self.config.try_lock() {
                                cfg.get_boolean_property_or("connection.ssl", self.port == 992)
                            } else {
                                self.port == 992  // Default: TLS on port 992
                            }
                        };
                        let checkbox = ui.checkbox(&mut ssl_enabled, "Enable TLS encryption");
                        if checkbox.changed() {
                            if let Ok(mut cfg) = self.config.try_lock() {
                                cfg.set_property("connection.ssl", ssl_enabled);
                            }
                            config::save_shared_config_async(&self.config);
                        }
                        ui.end_row();

                        ui.label("TLS Options:");
                        let mut insecure = {
                            if let Ok(cfg) = self.config.try_lock() {
                                cfg.get_boolean_property_or("connection.tls.insecure", false)
                            } else {
                                false  // Default: secure mode
                            }
                        };
                        if ui.checkbox(&mut insecure, "Accept invalid certificates (insecure)").changed() {
                            if let Ok(mut cfg) = self.config.try_lock() {
                                cfg.set_property("connection.tls.insecure", insecure);
                            }
                            config::save_shared_config_async(&self.config);
                        }
                        ui.end_row();

                        ui.label("CA bundle path:");
                        let mut ca_path = {
                            if let Ok(cfg) = self.config.try_lock() {
                                cfg.get_string_property_or("connection.tls.caBundlePath", "")
                            } else {
                                String::new()  // Default: empty path
                            }
                        };
                        if ui.text_edit_singleline(&mut ca_path).lost_focus() {
                            if let Ok(mut cfg) = self.config.try_lock() {
                                cfg.set_property("connection.tls.caBundlePath", ca_path.as_str());
                            }
                            config::save_shared_config_async(&self.config);
                        }
                        ui.end_row();
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        self.show_advanced_settings = false;
                    }
                });

                ui.small("Note: These settings will be saved automatically when changed.");
            });
    }

    pub fn show_settings_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Terminal Settings")
            .collapsible(false)
            .resizable(true)
            .default_size([450.0, 350.0])
            .show(ctx, |ui| {
                ui.heading("Terminal Configuration");
                ui.separator();

                egui::Grid::new("terminal_settings_grid")
                    .num_columns(2)
                    .spacing([40.0, 12.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Protocol Mode Selection
                        ui.label("Protocol Mode:");
                        ui.vertical(|ui| {
                            let mut changed = false;

                            if ui.radio_value(&mut self.selected_protocol_mode, ProtocolMode::TN5250, "TN5250 (IBM AS/400)")
                                .on_hover_text("Standard IBM AS/400 terminal protocol").changed() {
                                changed = true;
                            }

                            if ui.radio_value(&mut self.selected_protocol_mode, ProtocolMode::TN3270, "TN3270 (IBM Mainframe)")
                                .on_hover_text("IBM mainframe terminal protocol").changed() {
                                changed = true;
                            }

                            if ui.radio_value(&mut self.selected_protocol_mode, ProtocolMode::AutoDetect, "Auto-Detect")
                                .on_hover_text("Automatically detect protocol based on server response").changed() {
                                changed = true;
                            }

                            if changed {
                                // Save protocol mode to config
                                if let Ok(mut cfg) = self.config.try_lock() {
                                    let mode_str = match self.selected_protocol_mode {
                                        ProtocolMode::TN5250 => "TN5250",
                                        ProtocolMode::TN3270 => "TN3270",
                                        ProtocolMode::AutoDetect => "AutoDetect",
                                        _ => "TN5250", // Default fallback
                                    };
                                    cfg.set_property("terminal.protocolMode", mode_str);
                                }
                                config::save_shared_config_async(&self.config);
                            }
                        });
                        ui.end_row();

                        // Screen Size Selection
                        ui.label("Screen Size:");
                        ui.vertical(|ui| {
                            let mut changed = false;

                            if ui.radio_value(&mut self.selected_screen_size, ScreenSize::Model2, "Model 2 (24×80)")
                                .on_hover_text("Standard 24 rows × 80 columns (1920 characters)").changed() {
                                changed = true;
                            }

                            if ui.radio_value(&mut self.selected_screen_size, ScreenSize::Model3, "Model 3 (32×80)")
                                .on_hover_text("Extended 32 rows × 80 columns (2560 characters)").changed() {
                                changed = true;
                            }

                            if ui.radio_value(&mut self.selected_screen_size, ScreenSize::Model4, "Model 4 (43×80)")
                                .on_hover_text("Large 43 rows × 80 columns (3440 characters)").changed() {
                                changed = true;
                            }

                            if ui.radio_value(&mut self.selected_screen_size, ScreenSize::Model5, "Model 5 (27×132)")
                                .on_hover_text("Wide 27 rows × 132 columns (3564 characters)").changed() {
                                changed = true;
                            }

                            if changed {
                                // Save screen size to config
                                if let Ok(mut cfg) = self.config.try_lock() {
                                    let size_str = match self.selected_screen_size {
                                        ScreenSize::Model2 => "Model2",
                                        ScreenSize::Model3 => "Model3",
                                        ScreenSize::Model4 => "Model4",
                                        ScreenSize::Model5 => "Model5",
                                    };
                                    cfg.set_property("terminal.screenSize", size_str);

                                    // Also save the dimensions for easy access
                                    cfg.set_property("terminal.rows", self.selected_screen_size.rows() as i64);
                                    cfg.set_property("terminal.cols", self.selected_screen_size.cols() as i64);
                                }
                                config::save_shared_config_async(&self.config);
                            }
                        });
                        ui.end_row();

                        // Display current selection info
                        ui.label("Current Configuration:");
                        ui.vertical(|ui| {
                            let protocol_name = match self.selected_protocol_mode {
                                ProtocolMode::TN5250 => "TN5250 (IBM AS/400)",
                                ProtocolMode::TN3270 => "TN3270 (IBM Mainframe)",
                                ProtocolMode::AutoDetect => "Auto-Detect",
                                ProtocolMode::NVT => "NVT (Plain Text)",
                            };
                            ui.label(format!("Protocol: {protocol_name}"));

                            let dimensions = format!("{}×{} ({} chars)",
                                self.selected_screen_size.rows(),
                                self.selected_screen_size.cols(),
                                self.selected_screen_size.buffer_size()
                            );
                            ui.label(format!("Screen: {dimensions}"));

                            ui.colored_label(egui::Color32::from_rgb(150, 150, 150),
                                "Note: Changes take effect on next connection");
                        });
                        ui.end_row();
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        self.show_settings_dialog = false;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Reset to Defaults").clicked() {
                            self.selected_screen_size = ScreenSize::Model2;
                            self.selected_protocol_mode = ProtocolMode::TN5250;

                            // Save defaults to config
                            if let Ok(mut cfg) = self.config.try_lock() {
                                cfg.set_property("terminal.screenSize", "Model2");
                                cfg.set_property("terminal.protocolMode", "TN5250");
                                cfg.set_property("terminal.rows", 24i64);
                                cfg.set_property("terminal.cols", 80i64);
                            }
                            config::save_shared_config_async(&self.config);
                        }
                    });
                });
            });
    }
}