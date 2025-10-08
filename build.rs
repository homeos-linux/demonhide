fn main() {
    // No C code compilation needed anymore - everything is pure Rust!
    println!("cargo:rerun-if-changed=src/");
}
