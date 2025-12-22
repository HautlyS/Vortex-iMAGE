fn main() {
    // Re-run if target changes
    println!("cargo:rerun-if-env-changed=TARGET");
    
    tauri_build::build()
}
