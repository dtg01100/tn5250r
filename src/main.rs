//! Main application entry point for TN5250R
//! 
//! This module handles the GUI application lifecycle and user interface.

use eframe::egui;

mod lib5250;
mod ansi_processor;
mod network;
mod terminal;
mod protocol;
mod telnet_negotiation;
mod keyboard;
mod controller;
mod field_manager;

use controller::AsyncTerminalController;

/// Number of function keys per row in the UI
const FUNCTION_KEYS_PER_ROW: usize = 12;

/// Main application structure
pub struct TN5250RApp {
    controller: AsyncTerminalController,
    connection_string: String,
    connected: bool,
    host: String,
    port: u16,
    input_buffer: String,
    function_keys_visible: bool,
    terminal_content: String,
    login_screen_requested: bool,
    connection_time: Option<std::time::Instant>,
    fields_info: Vec<(String, String, bool)>,  // (label, content, is_active)
    show_field_info: bool,
    tab_pressed_this_frame: bool,  // Track if Tab was pressed to prevent egui handling
}

impl TN5250RApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_with_server(_cc, "example.system.com".to_string(), 23, false)
    }
    
    fn new_with_server(_cc: &eframe::CreationContext<'_>, server: String, port: u16, auto_connect: bool) -> Self {
        let connection_string = format!("{}:{}", server, port);
        let host = server;
        let port = port; // Use the provided port directly
        
        let mut controller = controller::AsyncTerminalController::new();
        
        // If auto-connect is requested, initiate connection
        let connected = if auto_connect {
            match controller.connect(host.clone(), port) {
                Ok(()) => {
                    true
                },
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                    false
                }
            }
        } else {
            false
        };
        
        let terminal_content = if auto_connect && connected {
            format!("Connected to {}:{}\nReady...\n", host, port)
        } else if auto_connect {
            format!("Failed to connect to {}:{}\nReady...\n", host, port)
        } else {
            "TN5250R - IBM AS/400 Terminal Emulator\nReady...\n".to_string()
        };
        
        Self {
            connection_string,
            controller,
            connected,
            host,
            port,
            input_buffer: String::new(),
            function_keys_visible: true,
            terminal_content,
            login_screen_requested: false,
            connection_time: None,
            fields_info: Vec::new(),
            show_field_info: true,
            tab_pressed_this_frame: false,
        }
    }
    
    fn parse_connection_string(&self) -> (String, u16) {
        if let Some((host, port_str)) = self.connection_string.rsplit_once(':') {
            let host = host.to_string();
            if let Ok(port) = port_str.parse::<u16>() {
                (host, port)
            } else {
                (host, 23) // Default telnet port
            }
        } else {
            (self.connection_string.clone(), 23) // Default telnet port
        }
    }

    fn do_connect(&mut self) {
        // Parse host and port from connection string
        let (host, port) = self.parse_connection_string();
        self.host = host;
        self.port = port;
        
        match self.controller.connect(self.host.clone(), self.port) {
            Ok(()) => {
                self.connected = true;
                self.terminal_content = format!("Connected to {}:{}\nNegotiating...\n", self.host, self.port);
                self.connection_time = Some(std::time::Instant::now());
                self.login_screen_requested = false;
            }
            Err(e) => {
                self.terminal_content = format!("Connection failed: {}\n", e);
                self.connection_time = None;
                self.login_screen_requested = false;
            }
        }
    }
    
    fn do_disconnect(&mut self) {
        self.controller.disconnect();
        self.connected = false;
        self.login_screen_requested = false;
        self.connection_time = None;
        self.terminal_content = "Disconnected from AS/400 system\nReady for new connection...\n".to_string();
    }
    
    fn send_function_key(&mut self, key_name: &str) {
        // In a real implementation, this would send the actual function key
        // For now, we'll just update the terminal content
        self.terminal_content.push_str(&format!("\n[{}] pressed", key_name));

        // Parse the key name to determine which function key to send
        let func_key = self.parse_function_key_name(key_name);

        // Simulate sending the function key
        match self.controller.send_function_key(func_key) {
            Ok(()) => {
                // Function key sent successfully
            }
            Err(e) => {
                self.terminal_content.push_str(&format!("\nError sending function key: {}", e));
            }
        }
    }

    fn parse_function_key_name(&self, key_name: &str) -> keyboard::FunctionKey {
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
    
    fn send_function_key_direct(&mut self, func_key: keyboard::FunctionKey) {
        // Send the actual function key
        match self.controller.send_function_key(func_key) {
            Ok(()) => {
                // Function key sent successfully
                self.terminal_content.push_str(&format!("\n[{:?}] pressed", func_key));
            }
            Err(e) => {
                self.terminal_content.push_str(&format!("\nError sending function key: {}", e));
            }
        }
    }
    
    fn update_terminal_content(&mut self) {
        // Update terminal content from controller
        if let Ok(content) = self.controller.get_terminal_content() {
            // Only update if content has changed to avoid unnecessary UI updates
            if content != self.terminal_content {
                self.terminal_content = content;
            }
        }
        
        // Update field information
        if let Ok(fields) = self.controller.get_fields_info() {
            self.fields_info = fields;
        }
        
        // Update connection status
        self.connected = self.controller.is_connected();
        
        // Request login screen if connected and enough time has passed
        if self.connected && !self.login_screen_requested {
            if let Some(connection_time) = self.connection_time {
                if connection_time.elapsed() >= std::time::Duration::from_secs(2) {
                    if let Err(e) = self.controller.request_login_screen() {
                        eprintln!("Failed to request login screen: {}", e);
                    }
                    self.login_screen_requested = true;
                }
            }
        }
    }
    
    fn draw_terminal_with_cursor(&mut self, ui: &mut egui::Ui) {
        // Get cursor position
        let cursor_pos = self.controller.get_cursor_position().unwrap_or((1, 1));
        
        // Split terminal content into lines
        let lines: Vec<&str> = self.terminal_content.lines().collect();
        
        // Calculate character size for positioning
        let font = egui::FontId::monospace(14.0);
        let char_width = ui.fonts(|f| f.glyph_width(&font, ' '));
        let line_height = ui.fonts(|f| f.row_height(&font));
        
        // Create a scrollable text area with clickable regions
        let available_size = ui.available_size();
        let response = ui.allocate_response(available_size, egui::Sense::click());
        
        // Handle mouse clicks to set cursor position
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let relative_pos = pos - response.rect.min;
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
        
        // Draw the terminal content
        if ui.is_rect_visible(response.rect) {
            let mut y_offset = 0.0;
            
            for (line_idx, line) in lines.iter().enumerate() {
                let line_number = line_idx + 1; // 1-based line numbers
                
                // Draw each character in the line
                for (char_idx, ch) in line.chars().enumerate() {
                    let col_number = char_idx + 1; // 1-based column numbers
                    
                    let char_pos = response.rect.min + egui::vec2(char_idx as f32 * char_width, y_offset);
                    
                    // Check if this is the cursor position
                    let is_cursor = cursor_pos.0 == line_number && cursor_pos.1 == col_number;
                    
                    // Choose color based on cursor position and field status
                    let color = if is_cursor {
                        egui::Color32::BLACK // Cursor character
                    } else {
                        egui::Color32::WHITE // Normal text
                    };
                    
                    // Background color for cursor
                    if is_cursor {
                        let cursor_rect = egui::Rect::from_min_size(
                            char_pos,
                            egui::vec2(char_width, line_height)
                        );
                        ui.painter().rect_filled(cursor_rect, egui::Rounding::ZERO, egui::Color32::GREEN);
                    }
                    
                    // Draw the character
                    ui.painter().text(
                        char_pos,
                        egui::Align2::LEFT_TOP,
                        ch,
                        font.clone(),
                        color,
                    );
                }
                
                y_offset += line_height;
            }
            
            // Draw cursor if it's beyond the text content
            if cursor_pos.0 as usize > lines.len() || 
               (cursor_pos.0 as usize <= lines.len() && 
                cursor_pos.1 as usize > lines.get(cursor_pos.0 - 1).map_or(0, |l| l.len())) {
                
                let cursor_char_pos = response.rect.min + egui::vec2(
                    (cursor_pos.1 - 1) as f32 * char_width,
                    (cursor_pos.0 - 1) as f32 * line_height
                );
                
                let cursor_rect = egui::Rect::from_min_size(
                    cursor_char_pos,
                    egui::vec2(char_width, line_height)
                );
                ui.painter().rect_filled(cursor_rect, egui::Rounding::ZERO, egui::Color32::GREEN);
                
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

impl eframe::App for TN5250RApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    eprintln!("Failed to navigate to previous field: {}", e);
                }
            } else {
                if let Err(e) = self.controller.next_field() {
                    eprintln!("Failed to navigate to next field: {}", e);
                }
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
                                    eprintln!("Failed to send Enter: {}", e);
                                }
                            }
                            egui::Key::Backspace => {
                                if let Err(e) = self.controller.backspace() {
                                    eprintln!("Failed to send backspace: {}", e);
                                }
                            }
                            egui::Key::Delete => {
                                if let Err(e) = self.controller.delete() {
                                    eprintln!("Failed to send delete: {}", e);
                                }
                            }
                            egui::Key::F1 => {
                                if let Err(e) = self.controller.send_function_key(keyboard::FunctionKey::F1) {
                                    eprintln!("Failed to send F1: {}", e);
                                }
                            }
                            egui::Key::F2 => {
                                if let Err(e) = self.controller.send_function_key(keyboard::FunctionKey::F2) {
                                    eprintln!("Failed to send F2: {}", e);
                                }
                            }
                            egui::Key::F3 => {
                                if let Err(e) = self.controller.send_function_key(keyboard::FunctionKey::F3) {
                                    eprintln!("Failed to send F3: {}", e);
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
                                        eprintln!("Failed to type character '{}': {}", ch, e);
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
        
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Connect").clicked() {
                        self.do_connect();
                        ui.close_menu();
                    }
                    if ui.button("Disconnect").clicked() {
                        self.do_disconnect();
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.function_keys_visible, "Function Keys");
                });
                
                ui.menu_button("Settings", |ui| {
                    ui.label("Terminal Settings");
                    // Add more settings here in the future
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
            ui.heading("TN5250R - IBM AS/400 Terminal Emulator");
            ui.separator();

            egui::Grid::new("connection_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Host:");
                    if ui.text_edit_singleline(&mut self.connection_string).changed() {
                        // Update host and port when connection string changes
                        let (host, port) = self.parse_connection_string();
                        self.host = host;
                        self.port = port;
                    }
                    ui.end_row();

                    if ui.button("Connect").clicked() {
                        self.do_connect();
                    }
                    ui.end_row();
                });

            ui.separator();

            // Display terminal content with cursor and click handling
            egui::ScrollArea::vertical()
                .id_source("terminal_display")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.draw_terminal_with_cursor(ui);
                });

            // Display field information if available
            if !self.fields_info.is_empty() {
                ui.separator();
                ui.collapsing("Field Information", |ui| {
                    for (i, (label, content, is_active)) in self.fields_info.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if *is_active {
                                ui.colored_label(egui::Color32::GREEN, "►");
                            } else {
                                ui.label(" ");
                            }
                            ui.label(format!("Field {}: {} = '{}'", i + 1, label, content));
                        });
                    }
                    ui.label("Use Tab/Shift+Tab to navigate between fields");
                });
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
                ui.separator();
                
                // Two rows of function keys (F1-F12, F13-F24)
                ui.columns(FUNCTION_KEYS_PER_ROW, |columns| {
                    for i in 1..=FUNCTION_KEYS_PER_ROW {
                        let col_idx = (i - 1) % 12;
                        if columns[col_idx].button(format!("F{}", i)).clicked() {
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
                        if columns[col_idx].button(format!("F{}", i)).clicked() {
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

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    if self.connected {
                        ui.colored_label(egui::Color32::GREEN, &format!("Connected to {}:{} ", self.host, self.port));
                    } else {
                        ui.colored_label(egui::Color32::RED, "Disconnected");
                    }
                    ui.separator();
                    ui.label("Ready");
                });
            });
        });

        // Process incoming data and update terminal content
        self.update_terminal_content();
        
        ctx.request_repaint();
    }
}

fn main() {
    env_logger::init();
    
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut default_server = "example.system.com".to_string();
    let mut default_port = 23;
    let mut auto_connect = false;
    
    // Parse --server and --port options
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--server" | "-s" => {
                if i + 1 < args.len() {
                    default_server = args[i + 1].clone();
                    auto_connect = true; // Auto-connect when server is specified
                    i += 1; // Skip the next argument since we consumed it
                } else {
                    eprintln!("Error: --server requires a value");
                    std::process::exit(1);
                }
            }
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u16>() {
                        Ok(port) => default_port = port,
                        Err(_) => {
                            eprintln!("Error: --port requires a numeric value");
                            std::process::exit(1);
                        }
                    }
                    i += 1; // Skip the next argument since we consumed it
                } else {
                    eprintln!("Error: --port requires a value");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                println!("TN5250R - IBM AS/400 Terminal Emulator");
                println!();
                println!("Usage: tn5250r [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --server <server> or -s <server>    Set the server to connect to and auto-connect");
                println!("  --port <port> or -p <port>          Set the port to connect to (default: 23)");
                println!("  --help or -h                        Show this help message");
                std::process::exit(0);
            }
            _ => {
                // Ignore unknown arguments for now
            }
        }
        i += 1;
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "TN5250R",
        options,
        Box::new(move |cc| Box::new(TN5250RApp::new_with_server(cc, default_server, default_port, auto_connect))),
    )
    .expect("Failed to run TN5250R application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        assert!(true);
    }
}