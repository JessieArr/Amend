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
fn main() {} 