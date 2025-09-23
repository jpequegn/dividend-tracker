use std::env;
use std::path::PathBuf;

/// Set up test environment with a temporary data directory
pub fn setup_test_env() -> PathBuf {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = temp_dir.into_path();
    env::set_var("DIVIDEND_TRACKER_DATA_DIR", &path);
    path
}

/// Clean up test environment
pub fn cleanup_test_env(path: PathBuf) {
    if path.exists() {
        std::fs::remove_dir_all(path).ok();
    }
}