use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    println!("Testing eframe/egui initialization...");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    
    println!("Starting eframe with GUI test...");
    
    eframe::run_native(
        "TN5250R GUI Test",
        options,
        Box::new(|_cc| {
            println!("eframe initialized successfully!");
            Ok(Box::<TestApp>::default())
        }),
    )
}

#[derive(Default)]
struct TestApp {
    counter: i32,
}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("TN5250R GUI Test");
            ui.label(format!("Counter: {}", self.counter));
            if ui.button("Increment").clicked() {
                self.counter += 1;
            }
            if self.counter > 0 && ui.button("Test passed - Close").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}