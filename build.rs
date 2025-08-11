#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    
    // Only set icon if the file exists
    if std::path::Path::new("assets/icon.ico").exists() {
        res.set_icon("assets/icon.ico");
    }
    
    res.compile().unwrap();
    
    // Set the subsystem to Windows GUI to hide console window
    println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
}

#[cfg(not(windows))]
fn main() {
    // Only create app bundle on release builds for macOS
    #[cfg(target_os = "macos")]
    {
        // Check if this is a release build
        if std::env::var("PROFILE").unwrap_or_default() == "release" {
            // Just print instructions for now - we'll create a separate script
            println!("cargo:warning=For macOS app bundle, run: ./create_bundle.sh");
        }
    }
}

#[cfg(target_os = "macos")]
fn create_macos_bundle() {
    use std::fs;
    use std::path::Path;
    
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir).ancestors().nth(4).unwrap(); // Go up to target directory
    let profile = std::env::var("PROFILE").unwrap();
    let exe_name = "amend-editor";
    
    // Paths for the bundle
    let bundle_name = "Amend.app";
    let bundle_path = target_dir.join(&profile).join(bundle_name);
    let contents_path = bundle_path.join("Contents");
    let macos_path = contents_path.join("MacOS");
    let resources_path = contents_path.join("Resources");
    
    // Create directory structure
    if let Err(e) = fs::create_dir_all(&macos_path) {
        eprintln!("Failed to create MacOS directory: {}", e);
        return;
    }
    if let Err(e) = fs::create_dir_all(&resources_path) {
        eprintln!("Failed to create Resources directory: {}", e);
        return;
    }
    
    // Copy executable
    let exe_source = target_dir.join(profile).join(exe_name);
    let exe_dest = macos_path.join(exe_name);
    
    if let Err(e) = fs::copy(&exe_source, &exe_dest) {
        eprintln!("Failed to copy executable: {}", e);
        return;
    }
    
    // Copy Info.plist
    let plist_source = Path::new("Info.plist");
    let plist_dest = contents_path.join("Info.plist");
    
    if let Err(e) = fs::copy(plist_source, plist_dest) {
        eprintln!("Failed to copy Info.plist: {}", e);
        return;
    }
    
    // Copy icon if it exists (convert PNG to ICNS format)
    let icon_source = Path::new("assets/icon.png");
    if icon_source.exists() {
        let icon_dest = resources_path.join("icon.icns");
        
        // For now, just copy the PNG and we'll convert it later
        // In a production setup, you'd want to convert PNG to ICNS
        if let Err(e) = fs::copy(icon_source, icon_dest) {
            eprintln!("Failed to copy icon: {}", e);
        }
    }
    
    println!("cargo:warning=macOS app bundle created at: {}", bundle_path.display());
    println!("cargo:rerun-if-changed=Info.plist");
    println!("cargo:rerun-if-changed=assets/icon.png");
} 