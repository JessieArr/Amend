use eframe::egui;
use crate::app::TextEditorApp;

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
