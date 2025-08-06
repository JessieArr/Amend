use eframe::egui;
use std::fs;
use std::env;
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

fn main() -> Result<(), eframe::Error> {
    // Register as JSON editor on Windows if running with admin privileges
    #[cfg(windows)]
    {
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let file_path = &args[1];
            if file_path.ends_with(".json") {
                // Open the JSON file directly
                let mut app = TextEditorApp::default();
                if let Ok(contents) = fs::read_to_string(file_path) {
                    app.text = contents;
                    app.filename = Some(file_path.to_string());
                    app.is_modified = false;
                    
                    let options = eframe::NativeOptions {
                        viewport: egui::ViewportBuilder::default()
                            .with_inner_size([800.0, 600.0])
                            .with_min_inner_size([400.0, 300.0]),
                        ..Default::default()
                    };
                    
                    return eframe::run_native(
                        "Amend Text Editor",
                        options,
                        Box::new(|_cc| Box::new(app)),
                    );
                }
            }
        }
    }
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Amend Text Editor",
        options,
        Box::new(|_cc| Box::new(TextEditorApp::default())),
    )
}

struct TextEditorApp {
    text: String,
    filename: Option<String>,
    is_modified: bool,
}

impl Default for TextEditorApp {
    fn default() -> Self {
        Self {
            text: String::new(),
            filename: None,
            is_modified: false,
        }
    }
}

impl eframe::App for TextEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Menu bar
            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("New").clicked() {
                        self.new_file();
                    }
                    if ui.button("Open").clicked() {
                        self.open_file();
                    }
                    if ui.button("Save").clicked() {
                        self.save_file();
                    }
                    if ui.button("Save As").clicked() {
                        self.save_file_as();
                    }
                    
                    #[cfg(windows)]
                    if ui.button("Register as JSON Editor").clicked() {
                        self.register_as_json_editor();
                    }
                    
                    ui.separator();
                    
                    if let Some(filename) = &self.filename {
                        let title = if self.is_modified {
                            format!("*{}", filename)
                        } else {
                            filename.clone()
                        };
                        ui.label(title);
                    } else {
                        ui.label("Untitled");
                    }
                });
            });
            
            // Text editor area
            ui.add_space(25.0);
            
            // Create a scroll area for the text editor
            egui::ScrollArea::both().show(ui, |ui| {
                // Calculate the width needed based on the longest line
                let font_id = egui::TextStyle::Monospace.resolve(&ui.style());
                let mut max_width: f32 = 0.0;
                
                for line in self.text.lines() {
                    let line_width = ui.fonts(|fonts| fonts.layout_no_wrap(line.to_string(), font_id.clone(), egui::Color32::WHITE).rect.width());
                    max_width = max_width.max(line_width);
                }
                
                // Add some padding and ensure minimum width
                let content_width = (max_width + 50.0).max(ui.available_width());
                
                // Create a container that's exactly as wide as needed
                ui.horizontal(|ui| {
                    ui.set_min_width(content_width);
                    let text_edit = egui::TextEdit::multiline(&mut self.text)
                        .desired_width(f32::INFINITY)
                        .desired_rows(30)
                        .font(egui::TextStyle::Monospace);
                    
                    let response = ui.add(text_edit);
                    
                    // Track modifications
                    if response.changed() {
                        self.is_modified = true;
                    }
                });
            });
            
            // Keyboard shortcuts
            if ui.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
                self.save_file();
            }
            if ui.input(|i| i.key_pressed(egui::Key::O) && i.modifiers.ctrl) {
                self.open_file();
            }
            if ui.input(|i| i.key_pressed(egui::Key::N) && i.modifiers.ctrl) {
                self.new_file();
            }
        });
    }
}

impl TextEditorApp {
    fn new_file(&mut self) {
        if self.is_modified {
            // In a real app, you'd ask the user if they want to save
            // For now, we'll just clear without asking
        }
        self.text.clear();
        self.filename = None;
        self.is_modified = false;
    }
    
    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            if let Ok(contents) = fs::read_to_string(&path) {
                self.text = contents;
                self.filename = Some(path.display().to_string());
                self.is_modified = false;
            }
        }
    }
    
    fn save_file(&mut self) {
        if let Some(filename) = &self.filename {
            if let Ok(()) = fs::write(filename, &self.text) {
                self.is_modified = false;
            }
        } else {
            self.save_file_as();
        }
    }
    
    fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt"])
            .add_filter("All files", &["*"])
            .save_file()
        {
            if let Ok(()) = fs::write(&path, &self.text) {
                self.filename = Some(path.display().to_string());
                self.is_modified = false;
            }
        }
    }
    
    #[cfg(windows)]
    fn register_as_json_editor(&self) {
        if let Ok(exe_path) = env::current_exe() {
            let exe_path_str = exe_path.to_string_lossy().to_string();
            
            // Register file association for .json files
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            
            // Create .json file association
            if let Ok((json_key, _)) = hkcu.create_subkey("Software\\Classes\\.json") {
                json_key.set_value("", &"AmendTextEditor.json").unwrap_or_default();
            }
            
            // Create application key
            if let Ok((app_key, _)) = hkcu.create_subkey("Software\\Classes\\AmendTextEditor.json") {
                app_key.set_value("", &"Amend Text Editor").unwrap_or_default();
                
                // Set default icon
                if let Ok((icon_key, _)) = app_key.create_subkey("DefaultIcon") {
                    icon_key.set_value("", &format!("{},0", exe_path_str)).unwrap_or_default();
                }
                
                // Set shell command
                if let Ok((shell_key, _)) = app_key.create_subkey("shell\\open\\command") {
                    shell_key.set_value("", &format!("\"{}\" \"%1\"", exe_path_str)).unwrap_or_default();
                }
            }
        }
    }
} 