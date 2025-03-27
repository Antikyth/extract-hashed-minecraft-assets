use std::path::PathBuf;

// Windows
/// Returns the default location of the `.minecraft` directory.
#[cfg(target_os = "windows")]
pub fn minecraft_dir() -> Option<PathBuf> {
    dirs::data_dir()
        .map(|path| path.join(".minecraft"))
        .filter(|path| path.is_dir())
}

// Mac
/// Returns the default location of the `minecraft` directory.
#[cfg(target_os = "macos")]
fn minecraft_dir() -> Option<PathBuf> {
    dirs::data_dir()
        .map(|path| path.join("minecraft"))
        .filter(|path| path.is_dir())
}

// Linux
/// Returns the default location of the `.minecraft` directory.
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn minecraft_dir() -> Option<PathBuf> {
    dirs::home_dir()
        .map(|path| path.join(".minecraft"))
        .filter(|path| path.is_dir())
}
