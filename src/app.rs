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

                ui.menu_button("Profiles", |ui| {
                    if ui.button("Manage Profiles").clicked() {
                        self.show_profile_manager = !self.show_profile_manager;
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

            // Show session tabs if we have multiple sessions
            if self.sessions.len() > 1 {
                let session_ids: Vec<String> = self.sessions.keys().cloned().collect();
                ui.horizontal(|ui| {
                    for session_id in &session_ids {
                        if let Some(session) = self.sessions.get(session_id) {
                            let is_active = Some(session_id.clone()) == self.active_session_id;
                            let tab_name = if session_id == "legacy" {
                                "Main Session".to_string()
                            } else {
                                session.profile.name.clone()
                            };

                            if ui.selectable_label(is_active, &tab_name).clicked() {
                                self.switch_to_session(session_id.clone());
                            }
                        }
                    }

                    // Add close buttons for sessions
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for session_id in &session_ids {
                            if session_id != "legacy" {  // Don't allow closing the legacy session
                                if ui.button("✕").on_hover_text("Close session").clicked() {
                                    self.close_session(session_id);
                                    break; // Avoid modifying while iterating
                                }
                            }
                        }
                    });
                });
                ui.separator();
            }

            // Show session-specific content
            if let Some(active_session_id) = self.active_session_id.clone() {
                self.show_session_content(ui, &active_session_id);
            } else {
                // Fallback to legacy single-session mode
                self.show_legacy_session_content(ui);
            }
        });

        // Show profile sidebar if requested
        if self.show_profile_manager {
            egui::SidePanel::left("profiles_panel")
                .default_width(350.0)
                .show(ctx, |ui| {
                    self.profile_manager_ui(ui, ctx);
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {

            // Show error message prominently if present
            if let Some(ref error) = self.error_message {
                ui.colored_label(egui::Color32::RED, format!("⚠ Error: {}", error));
                ui.separator();
            }

            // Show session tabs if we have multiple sessions
            if self.sessions.len() > 1 {
                let session_ids: Vec<String> = self.sessions.keys().cloned().collect();
                ui.horizontal(|ui| {
                    for session_id in &session_ids {
                        if let Some(session) = self.sessions.get(session_id) {
                            let is_active = Some(session_id.clone()) == self.active_session_id;
                            let tab_name = if session_id == "legacy" {
                                "Main Session".to_string()
                            } else {
                                session.profile.name.clone()
                            };

                            if ui.selectable_label(is_active, &tab_name).clicked() {
                                self.switch_to_session(session_id.clone());
                            }
                        }
                    }

                    // Add close buttons for sessions
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for session_id in &session_ids {
                            if session_id != "legacy" {  // Don't allow closing the legacy session
                                if ui.button("✕").on_hover_text("Close session").clicked() {
                                    self.close_session(session_id);
                                    break; // Avoid modifying while iterating
                                }
                            }
                        }
                    });
                });
                ui.separator();
            }

            // Show session-specific content
            if let Some(active_session_id) = self.active_session_id.clone() {
                self.show_session_content(ui, &active_session_id);
            } else {
                // Fallback to legacy single-session mode
                self.show_legacy_session_content(ui);
            }
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

        // Show profile manager if requested
        if self.show_profile_manager {
            self.show_profile_manager(ctx);
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