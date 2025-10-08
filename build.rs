use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let protocol_c = format!("{}/pointer-constraints-unstable-v1-protocol.c", out_dir);
    
    // Generate the protocol implementation from XML
    let output = Command::new("wayland-scanner")
        .args(&[
            "private-code",
            "/usr/share/wayland-protocols/unstable/pointer-constraints/pointer-constraints-unstable-v1.xml",
            &protocol_c
        ])
        .output()
        .expect("Failed to execute wayland-scanner");
    
    if !output.status.success() {
        panic!("wayland-scanner failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Compile both the generated protocol implementation and our wrapper
    cc::Build::new()
        .file("c_src/pointer_constraints_wrapper.c")
        .file(&protocol_c)
        .include("c_src")
        .include("/usr/include")
        .compile("pointer_constraints_wrapper");
    
    // Link against wayland-client library
    println!("cargo:rustc-link-lib=wayland-client");
    
    // Tell cargo to look for wayland libraries in standard locations
    println!("cargo:rustc-link-search=native=/usr/lib64");
    println!("cargo:rustc-link-search=native=/usr/lib");
}
