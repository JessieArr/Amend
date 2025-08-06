use eframe::egui;
use std::fs;

fn main() -> Result<(), eframe::Error> {
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
            ui.add_space(5.0);
            
            let text_edit = egui::TextEdit::multiline(&mut self.text)
                .desired_width(f32::INFINITY)
                .desired_rows(30)
                .font(egui::TextStyle::Monospace);
            
            let response = ui.add(text_edit);
            
            // Track modifications
            if response.changed() {
                self.is_modified = true;
            }
            
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
} 