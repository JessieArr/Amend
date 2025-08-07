use eframe::egui;
use std::fs;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use egui::IconData;
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

fn load_icon() -> IconData {
    // Return a minimal transparent icon immediately
    // We'll load the real icon asynchronously later if needed
    IconData {
        rgba: vec![0, 0, 0, 0], // Transparent 1x1 pixel
        width: 1,
        height: 1,
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_icon(load_icon()),
        ..Default::default()
    };
    
    // Check for command line arguments
    #[cfg(windows)]
    {
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            let file_path = &args[1];
            let mut app = TextEditorApp::default();
            
            // Store the file path to load after UI is ready
            app.pending_file_to_load = Some(file_path.to_string());
            
            return eframe::run_native(
                "Amend Text Editor",
                options,
                Box::new(|_cc| Box::new(app)),
            );
        }
    }
    
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
    is_loading: bool,
    loading_filename: Option<String>,
    pending_file_content: Arc<Mutex<Option<(String, String)>>>, // (filename, content)
    pending_file_to_load: Option<String>, // File to load after UI is ready
    loading_progress: f32, // Progress indicator (0.0 to 1.0)
    partial_content: Arc<Mutex<String>>, // Content being loaded progressively
    bytes_loaded: Arc<Mutex<usize>>, // Number of bytes loaded so far
    total_bytes: Arc<Mutex<usize>>, // Total file size
}

impl Default for TextEditorApp {
    fn default() -> Self {
        Self {
            text: String::new(),
            filename: None,
            is_modified: false,
            is_loading: false,
            loading_filename: None,
            pending_file_content: Arc::new(Mutex::new(None)),
            pending_file_to_load: None,
            loading_progress: 0.0,
            partial_content: Arc::new(Mutex::new(String::new())),
            bytes_loaded: Arc::new(Mutex::new(0)),
            total_bytes: Arc::new(Mutex::new(0)),
        }
    }
}

impl eframe::App for TextEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Start loading pending file after a few frames to ensure UI is ready
        static mut FRAME_COUNT: u32 = 0;
        unsafe {
            FRAME_COUNT += 1;
            if FRAME_COUNT >= 3 { // Wait 3 frames before starting file load
                if let Some(file_path) = self.pending_file_to_load.take() {
                    self.start_loading_file(file_path);
                }
            }
        }
        
        // Check if file loading is complete
        if self.is_loading {
            // Update text with partial content during loading
            if let Ok(partial) = self.partial_content.lock() {
                if !partial.is_empty() {
                    self.text = partial.clone();
                }
            }
            
            if let Ok(mut pending) = self.pending_file_content.lock() {
                if let Some((filename, content)) = pending.take() {
                    self.text = content;
                    self.filename = Some(filename);
                    self.is_modified = false;
                    self.is_loading = false;
                    self.loading_filename = None;
                    self.loading_progress = 0.0;
                    
                    // Clear partial content
                    if let Ok(mut partial) = self.partial_content.lock() {
                        *partial = String::new();
                    }
                }
            }
        }
        
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
                    if ui.button("Register as Context Menu Editor").clicked() {
                        self.register_as_context_menu_editor();
                    }
                    
                    ui.separator();
                    
                    if self.is_loading {
                        if let Some(filename) = &self.loading_filename {
                            ui.label(format!("Loading: {}", filename));
                        } else {
                            ui.label("Loading...");
                        }
                    } else if let Some(filename) = &self.filename {
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
            
            // Text editor area with reduced height to make room for status
            ui.add_space(25.0);
            
            // Calculate available height for text editor (leave room for status bar)
            let available_height = ui.available_height() - if self.is_loading { 50.0 } else { 0.0 };
            
            // Always show the text editor, but disable during loading
            egui::ScrollArea::both()
                .max_height(available_height)
                .show(ui, |ui| {
                    let text_edit = egui::TextEdit::multiline(&mut self.text)
                        .desired_width(f32::INFINITY)
                        .desired_rows(25) // Reduced from 30 to make room for status
                        .font(egui::TextStyle::Monospace)
                        .interactive(!self.is_loading); // Disable during loading
                    
                    let response = ui.add(text_edit);
                    
                    // Track modifications (only when not loading)
                    if response.changed() && !self.is_loading {
                        self.is_modified = true;
                    }
                });
            
            // Show loading status at the bottom if loading
            if self.is_loading {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if let Some(filename) = &self.loading_filename {
                        ui.label(format!("Loading: {}", filename));
                    }
                    
                    // Show bytes loaded
                    if let Ok(bytes_loaded) = self.bytes_loaded.lock() {
                        if let Ok(total_bytes) = self.total_bytes.lock() {
                            if *total_bytes > 0 {
                                let percentage = (*bytes_loaded as f32 / *total_bytes as f32 * 100.0) as u32;
                                ui.label(format!("({} / {} bytes, {}%)", 
                                    bytes_loaded, total_bytes, percentage));
                            } else {
                                ui.label(format!("({} bytes loaded)", bytes_loaded));
                            }
                        }
                    }
                });
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
    fn start_loading_file(&mut self, file_path: String) {
        self.is_loading = true;
        self.loading_filename = Some(file_path.clone());
        
        // Clear partial content and reset counters
        if let Ok(mut partial) = self.partial_content.lock() {
            *partial = String::new();
        }
        if let Ok(mut bytes_loaded) = self.bytes_loaded.lock() {
            *bytes_loaded = 0;
        }
        if let Ok(mut total_bytes) = self.total_bytes.lock() {
            *total_bytes = 0;
        }
        
        // Spawn a thread to load the file asynchronously
        let file_path_clone = file_path.clone();
        let pending_content = Arc::clone(&self.pending_file_content);
        let partial_content = Arc::clone(&self.partial_content);
        let bytes_loaded = Arc::clone(&self.bytes_loaded);
        let total_bytes = Arc::clone(&self.total_bytes);
        
        // Use a completely non-blocking approach
        thread::spawn(move || {
            // Try to read the file size first
            let file_size = std::fs::metadata(&file_path_clone)
                .map(|m| m.len())
                .unwrap_or(0);
            
            // Set total bytes
            if let Ok(mut total) = total_bytes.lock() {
                *total = file_size as usize;
            }
            
            // For very large files, we might want to show a warning
            if file_size > 100 * 1024 * 1024 { // 100MB
                // For very large files, just read a preview
                let mut contents = String::new();
                if let Ok(mut file) = std::fs::File::open(&file_path_clone) {
                    use std::io::Read;
                    let mut buffer = [0; 512]; // Very small chunks
                    let mut total_read = 0;
                    let max_read = 10 * 1024 * 1024; // 10MB max
                    
                    loop {
                        match file.read(&mut buffer) {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                if let Ok(chunk) = std::str::from_utf8(&buffer[..n]) {
                                    contents.push_str(chunk);
                                    total_read += n;
                                    
                                    // Update partial content only for the first 2KB to fill the screen
                                    if total_read <= 2048 && total_read % 512 < 256 { // First 2KB, every ~256 bytes
                                        if let Ok(mut partial) = partial_content.lock() {
                                            *partial = contents.clone();
                                        }
                                    }
                                    
                                    // Update bytes loaded every few chunks
                                    if total_read % 4096 < 512 { // Every ~4KB
                                        if let Ok(mut loaded) = bytes_loaded.lock() {
                                            *loaded = total_read;
                                        }
                                    }
                                    
                                    if total_read >= max_read {
                                        contents.push_str("\n\n... (file truncated - too large to display completely)");
                                        // Set final bytes loaded
                                        if let Ok(mut loaded) = bytes_loaded.lock() {
                                            *loaded = total_read;
                                        }
                                        break;
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                        
                        // Yield very frequently
                        std::thread::yield_now();
                        std::thread::sleep(std::time::Duration::from_millis(2));
                    }
                }
                
                // Store the result in the shared state
                if let Ok(mut pending) = pending_content.lock() {
                    *pending = Some((file_path_clone, contents));
                }
            } else {
                // For smaller files, read normally but with frequent yields
                let mut contents = String::new();
                if let Ok(mut file) = std::fs::File::open(&file_path_clone) {
                    use std::io::Read;
                    let mut buffer = [0; 512]; // Small chunks
                    let mut total_read = 0;
                    
                    loop {
                        match file.read(&mut buffer) {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                if let Ok(chunk) = std::str::from_utf8(&buffer[..n]) {
                                    contents.push_str(chunk);
                                    total_read += n;
                                    
                                    // Update partial content only for the first 2KB to fill the screen
                                    if total_read <= 2048 && total_read % 512 < 256 { // First 2KB, every ~256 bytes
                                        if let Ok(mut partial) = partial_content.lock() {
                                            *partial = contents.clone();
                                        }
                                    }
                                    
                                    // Update bytes loaded every few chunks
                                    if total_read % 2048 < 512 { // Every ~2KB
                                        if let Ok(mut loaded) = bytes_loaded.lock() {
                                            *loaded = total_read;
                                        }
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                        
                        // Yield very frequently
                        std::thread::yield_now();
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
                
                // Store the result in the shared state
                if let Ok(mut pending) = pending_content.lock() {
                    *pending = Some((file_path_clone, contents));
                }
            }
        });
    }
    
    fn new_file(&mut self) {
        if self.is_modified {
            // In a real app, you'd ask the user if they want to save
            // For now, we'll just clear without asking
        }
        self.text.clear();
        self.filename = None;
        self.is_modified = false;
        self.is_loading = false;
        self.loading_filename = None;
        self.pending_file_to_load = None;
        self.loading_progress = 0.0;
        
        // Clear partial content and counters
        if let Ok(mut partial) = self.partial_content.lock() {
            *partial = String::new();
        }
        if let Ok(mut bytes_loaded) = self.bytes_loaded.lock() {
            *bytes_loaded = 0;
        }
        if let Ok(mut total_bytes) = self.total_bytes.lock() {
            *total_bytes = 0;
        }
    }
    
    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            let file_path = path.display().to_string();
            self.start_loading_file(file_path);
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
    fn register_as_context_menu_editor(&self) {
        if let Ok(exe_path) = env::current_exe() {
            let exe_path_str = exe_path.to_string_lossy().to_string();
            
            // Register as a context menu option for all files
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            
            // Add to the * file type (all files)
            if let Ok((all_files_key, _)) = hkcu.create_subkey("Software\\Classes\\*\\shell\\AmendTextEditor") {
                all_files_key.set_value("", &"Edit with Amend").unwrap_or_default();
                
                // Set the icon for the context menu item
                all_files_key.set_value("Icon", &format!("{},0", exe_path_str)).unwrap_or_default();
                
                // Set the command
                if let Ok((command_key, _)) = all_files_key.create_subkey("command") {
                    command_key.set_value("", &format!("\"{}\" \"%1\"", exe_path_str)).unwrap_or_default();
                    
                    // Show success message (you could add a status field to the app struct for this)
                    println!("Successfully registered 'Edit with Amend' in context menu");
                    println!("Icon will be displayed next to the menu item");
                }
            }
        }
    }
    
} 