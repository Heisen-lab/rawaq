use std::fs;
use std::path::Path;

fn main() {
    // Ensure ../dist exists when running cargo test/clippy before a full frontend bundle build
    let dist_dir = Path::new("../dist");
    if !dist_dir.exists() {
        let _ = fs::create_dir_all(dist_dir);
    }
    tauri_build::build()
}
