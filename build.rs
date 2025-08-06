#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    // Set the subsystem to Windows GUI to hide the console window
    println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
    
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("manifest.xml");
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {} 