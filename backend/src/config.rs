use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Resolved data directory path.
/// Priority: IRONPAD_DATA_DIR env var > auto-detect (production vs development).
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Initialize the data directory path. Call once at startup.
///
/// Resolution order:
/// 1. `IRONPAD_DATA_DIR` environment variable (if set)
/// 2. `./data` if `static/index.html` exists (production mode)
/// 3. `../data` (development mode, binary runs from backend/)
pub fn init_data_dir() {
    let path = if let Ok(custom) = std::env::var("IRONPAD_DATA_DIR") {
        tracing::info!("Using custom data directory from IRONPAD_DATA_DIR");
        PathBuf::from(custom)
    } else if Path::new("static/index.html").exists() {
        // Production mode: data/ is next to the binary
        PathBuf::from("data")
    } else {
        // Development mode: binary runs from backend/, data/ is one level up
        PathBuf::from("../data")
    };

    // Create the data directory if it doesn't exist
    if !path.exists() {
        if let Err(e) = std::fs::create_dir_all(&path) {
            tracing::error!("Failed to create data directory {}: {}", path.display(), e);
        }
    }

    tracing::info!("Data directory: {}", path.display());
    DATA_DIR
        .set(path)
        .expect("Data directory already initialized");
}

/// Get the resolved data directory path.
pub fn data_dir() -> &'static Path {
    DATA_DIR
        .get()
        .expect("Data directory not initialized. Call config::init_data_dir() first.")
}
