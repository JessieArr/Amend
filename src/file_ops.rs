use std::fs;
use std::sync::Arc;
use std::thread;
use crate::app::TextEditorApp;

impl TextEditorApp {
    pub fn start_loading_file(&mut self, file_path: String) {
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
    
    pub fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            let file_path = path.display().to_string();
            self.start_loading_file(file_path);
        }
    }
    
    pub fn save_file(&mut self) {
        if let Some(filename) = &self.filename {
            if let Ok(()) = fs::write(filename, &self.text) {
                self.is_modified = false;
            }
        } else {
            self.save_file_as();
        }
    }
    
    pub fn save_file_as(&mut self) {
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
