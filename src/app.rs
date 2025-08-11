use std::sync::{Arc, Mutex};

pub struct TextEditorApp {
    pub text: String,
    pub filename: Option<String>,
    pub is_modified: bool,
    pub is_loading: bool,
    pub loading_filename: Option<String>,
    pub pending_file_content: Arc<Mutex<Option<(String, String)>>>, // (filename, content)
    pub pending_file_to_load: Option<String>, // File to load after UI is ready
    pub loading_progress: f32, // Progress indicator (0.0 to 1.0)
    pub partial_content: Arc<Mutex<String>>, // Content being loaded progressively
    pub bytes_loaded: Arc<Mutex<usize>>, // Number of bytes loaded so far
    pub total_bytes: Arc<Mutex<usize>>, // Total file size
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

impl TextEditorApp {
    pub fn new_file(&mut self) {
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
}
