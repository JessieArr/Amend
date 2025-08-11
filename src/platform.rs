#[cfg(windows)]
use std::env;
#[cfg(windows)]
use std::path::Path;
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;
use eframe::egui;
use crate::app::TextEditorApp;

#[cfg(windows)]
#[allow(dead_code)]
pub fn load_application_icon() -> Option<egui::IconData> {
    // Try to load the icon from the assets directory
    let icon_path = Path::new("assets/icon.ico");
    if icon_path.exists() {
        // Use the image crate to decode the ICO file
        if let Ok(img) = image::open(icon_path) {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            
            return Some(egui::IconData {
                rgba: rgba.into_raw(),
                width: width as u32,
                height: height as u32,
            });
        }
    }
    None
}

#[cfg(not(windows))]
pub fn load_application_icon() -> Option<egui::IconData> {
    None
}

impl TextEditorApp {
    #[cfg(windows)]
    #[allow(dead_code)]
    pub fn register_as_context_menu_editor(&self) {
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
    
    #[cfg(not(windows))]
    pub fn register_as_context_menu_editor(&self) {
        // No-op on non-Windows platforms
        println!("Context menu registration is only available on Windows");
    }
}
