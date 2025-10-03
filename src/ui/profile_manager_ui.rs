//! Profile management UI components for TN5250R
//!
//! This module contains the UI for managing session profiles including
//! listing, creating, editing, and deleting profiles.

use eframe::egui;
use crate::app_state::TN5250RApp;
use crate::session_profile::SessionProfile;
use crate::profile_manager::ProfileManager;
use crate::lib3270::display::ScreenSize;
use crate::network::ProtocolMode;

impl TN5250RApp {
    /// Show the profile management sidebar
    pub fn show_profile_manager(&mut self, ctx: &egui::Context) {
        // Profile manager is now shown as a side panel in app.rs
        // This function is kept for compatibility but now just ensures the sidebar is shown
    }

    /// Main profile manager UI
    pub fn profile_manager_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.heading("Session Profiles");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").on_hover_text("Close sidebar").clicked() {
                    self.show_profile_manager = false;
                }
            });
        });
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("➕ New Profile").clicked() {
                self.show_create_profile_dialog = true;
                self.editing_profile = Some(SessionProfile::new(
                    "New Profile".to_string(),
                    "example.system.com".to_string(),
                    23,
                ));
            }

            if ui.button("🔄 Refresh").clicked() {
                // Refresh the profile list
                if let Ok(pm) = ProfileManager::new() {
                    self.profile_manager = pm;
                }
            }
        });

        ui.separator();

        // Profile list
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Available Profiles");
            ui.separator();

            let profile_names = self.profile_manager.get_profile_names();

            if profile_names.is_empty() {
                ui.label("No profiles found. Create your first profile above.");
            } else {
                for profile_name in &profile_names {
                    if let Some(profile) = self.profile_manager.get_profile_by_name(profile_name) {
                        self.profile_list_item(ui, profile.clone());
                    }
                }
            }
        });

        // Show create/edit dialog if needed
        if self.show_create_profile_dialog {
            self.show_create_profile_dialog(ctx);
        }
    }

    /// Individual profile list item
    fn profile_list_item(&mut self, ui: &mut egui::Ui, profile: SessionProfile) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(&profile.name);
                    ui.label(format!("Host: {}:{}", profile.host, profile.port));
                    ui.label(format!("Protocol: {:?}", profile.protocol));
                    if let Some(username) = &profile.username {
                        ui.label(format!("User: {}", username));
                    }
                    ui.label(format!("Screen: {}x{}", profile.screen_size.cols(), profile.screen_size.rows()));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑 Delete").clicked() {
                        if let Err(e) = self.profile_manager.delete_profile(&profile.id) {
                            eprintln!("Failed to delete profile: {}", e);
                        }
                    }

                    if ui.button("✏ Edit").clicked() {
                        self.show_create_profile_dialog = true;
                        self.editing_profile = Some(profile.clone());
                    }

                    if ui.button("🔗 Connect").clicked() {
                        // Create a new session from this profile
                        self.create_session_from_profile(profile);
                        self.show_profile_manager = false;
                    }
                });
            });
        });
        ui.separator();
    }

    /// Show the create/edit profile dialog
    fn show_create_profile_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_create_profile_dialog;
        let title = if self.editing_profile.is_some() {
            "Edit Profile"
        } else {
            "Create Profile"
        };

        egui::Window::new(title)
            .open(&mut open)
            .default_width(400.0)
            .show(ctx, |ui| {
                self.create_profile_dialog_ui(ui);
            });

        self.show_create_profile_dialog = open;
    }

    /// Create profile dialog UI
    fn create_profile_dialog_ui(&mut self, ui: &mut egui::Ui) {
        // Take the editing profile temporarily to avoid borrow issues
        let editing_profile = self.editing_profile.take();

        if let Some(mut profile) = editing_profile {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut profile.name);
            });

            ui.horizontal(|ui| {
                ui.label("Description:");
                ui.text_edit_singleline(&mut profile.description);
            });

            ui.horizontal(|ui| {
                ui.label("Host:");
                ui.text_edit_singleline(&mut profile.host);
            });

            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.add(egui::DragValue::new(&mut profile.port).range(1..=65535));
            });

            ui.horizontal(|ui| {
                ui.label("Protocol:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", profile.protocol))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut profile.protocol, ProtocolMode::AutoDetect, "Auto Detect");
                        ui.selectable_value(&mut profile.protocol, ProtocolMode::TN5250, "TN5250");
                        ui.selectable_value(&mut profile.protocol, ProtocolMode::TN3270, "TN3270");
                        ui.selectable_value(&mut profile.protocol, ProtocolMode::NVT, "NVT");
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Screen Size:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", profile.screen_size))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut profile.screen_size, ScreenSize::Model2, "Model 2 (24x80)");
                        ui.selectable_value(&mut profile.screen_size, ScreenSize::Model3, "Model 3 (32x80)");
                        ui.selectable_value(&mut profile.screen_size, ScreenSize::Model4, "Model 4 (43x80)");
                        ui.selectable_value(&mut profile.screen_size, ScreenSize::Model5, "Model 5 (27x132)");
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Username:");
                let mut username = profile.username.clone().unwrap_or_default();
                if ui.text_edit_singleline(&mut username).changed() {
                    profile.username = if username.is_empty() { None } else { Some(username) };
                }
            });

            ui.horizontal(|ui| {
                ui.label("Password:");
                let mut password = profile.password.clone().unwrap_or_default();
                if ui.add(egui::TextEdit::singleline(&mut password).password(true)).changed() {
                    profile.password = if password.is_empty() { None } else { Some(password) };
                }
            });

            ui.separator();

            let mut save_clicked = false;
            let mut cancel_clicked = false;

            ui.horizontal(|ui| {
                if ui.button("💾 Save").clicked() {
                    save_clicked = true;
                }

                if ui.button("❌ Cancel").clicked() {
                    cancel_clicked = true;
                }
            });

            // Handle save/cancel after UI
            if save_clicked {
                // Generate ID from name if this is a new profile
                if profile.id.is_empty() {
                    profile.id = profile.name.clone();
                }

                match self.profile_manager.create_profile(profile.clone()) {
                    Ok(_) => {
                        self.show_create_profile_dialog = false;
                        // Don't restore editing_profile since we saved
                    }
                    Err(e) => {
                        ui.colored_label(egui::Color32::RED, format!("Error saving profile: {}", e));
                        self.editing_profile = Some(profile); // Restore on error
                    }
                }
            } else if cancel_clicked {
                self.show_create_profile_dialog = false;
                // Don't restore editing_profile since we canceled
            } else {
                // No action taken, restore the profile
                self.editing_profile = Some(profile);
            }
        }
    }
}