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
