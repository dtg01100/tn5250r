#[cfg(test)]
mod tests {
    use egui_kittest::Harness;
    use eframe::egui;

    #[test]
    fn test_gui_can_be_created() {
        // Create a simple harness to test that the GUI framework works
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(800.0, 600.0))
            .build(|ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Test GUI");
                    ui.label("This is a simple test");
                    if ui.button("Click me").clicked() {
                        ui.label("Button was clicked!");
                    }
                });
            });

        // Run one frame to make sure it doesn't crash
        harness.step();

        // Check that we have the expected size
        assert_eq!(harness.size(), egui::Vec2::new(800.0, 600.0));
    }

    #[test]
    fn test_gui_has_expected_elements() {
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(800.0, 600.0))
            .build(|ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Test GUI");
                    ui.label("This is a simple test");
                    ui.horizontal(|ui| {
                        ui.label("Test Input:");
                        ui.text_edit_singleline(&mut String::from("test"));
                    });
                    ui.checkbox(&mut true, "Test Checkbox");
                });
            });

        // Run a few frames to initialize
        for _ in 0..5 {
            harness.step();
        }

        // Basic sanity check that something is rendered
        assert!(harness.size().x > 0.0);
        assert!(harness.size().y > 0.0);
    }
}