//! Rust Module - 1 functions, 0 structs
//! Core functionality: Backend operations and data processing
//! External crates: 0 dependencies

fn main() {
    
    println!("cargo:rerun-if-env-changed=TARGET");
    
    tauri_build::build()
}