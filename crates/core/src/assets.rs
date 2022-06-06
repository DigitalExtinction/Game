use std::{
    env::{self, current_exe},
    path::{Path, PathBuf},
};

/// Converts a relative asset path to an absolute path.
///
/// If the game is executed with Cargo, the path is interpreted as relative to
/// assets/ directory in the directory with Cargo manifest file.
///
/// Otherwise, it is interpreted as relative to assets/ directory in the
/// directory with the binary.
pub fn asset_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let mut new_path = match env::var("CARGO_MANIFEST_DIR") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            current_exe().expect("Failed to retrieve current executable path during map loading")
        }
    };
    new_path.push("assets");
    new_path.push(path);
    new_path
}
