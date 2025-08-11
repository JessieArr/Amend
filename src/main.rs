use eframe::egui;
#[cfg(any(windows, target_os = "macos"))]
use std::env;

mod app;
mod ui;
mod file_ops;
mod platform;

use app::TextEditorApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    
    // Set the application icon on Windows
    #[cfg(windows)]
    {
        if let Some(icon_data) = platform::load_application_icon() {
            options.viewport = options.viewport.with_icon(icon_data);
        }
    }
    
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
                Box::new(|_cc| Ok(Box::new(app))),
            );
        }
    }
    
    // Check for command line arguments on macOS
    #[cfg(target_os = "macos")]
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
                Box::new(|_cc| Ok(Box::new(app))),
            );
        }
    }
    
    eframe::run_native(
        "Amend Text Editor",
        options,
        Box::new(|_cc| Ok(Box::new(TextEditorApp::default()))),
    )
} 