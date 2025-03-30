use std::path::PathBuf;

pub trait OptionExt<T> {
    /// Calls a function with a mutable reference to the contained value if [`Some`].
    ///
    /// Returns the original option.
    ///
    /// This is a mutable version of [`Option::inspect`].
    ///
    /// # Examples
    /// ```
    /// let list = vec![1, 2, 3];
    ///
    /// list.get(2).inspect_mut(|element| element += 1);
    ///
    /// assert_eq!(list.get(2), Some(4));
    /// ```
    fn inspect_mut(self, f: impl FnOnce(&mut T)) -> Self;
}

impl<T> OptionExt<T> for Option<T> {
    fn inspect_mut(mut self, f: impl FnOnce(&mut T)) -> Self {
        if let Some(x) = &mut self {
            f(x);
        }

        self
    }
}

// Windows
/// Returns the default location of the `.minecraft` directory.
#[cfg(target_os = "windows")]
pub fn minecraft_dir() -> Option<PathBuf> {
    dirs::data_dir()
        .inspect_mut(|path| path.push(".minecraft"))
        .filter(|path| path.is_dir())
}

// Mac
/// Returns the default location of the `minecraft` directory.
#[cfg(target_os = "macos")]
pub fn minecraft_dir() -> Option<PathBuf> {
    dirs::data_dir()
        .inspect_mut(|path| path.push("minecraft"))
        .filter(|path| path.is_dir())
}

// Linux
/// Returns the default location of the `.minecraft` directory.
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn minecraft_dir() -> Option<PathBuf> {
    dirs::home_dir()
        .inspect_mut(|path| path.push(".minecraft"))
        .filter(|path| path.is_dir())
}

/// Returns the default location of the `.minecraft/assets/` directory.
pub fn hashed_assets_dir() -> Option<PathBuf> {
    minecraft_dir().inspect_mut(|path| path.push("assets"))
}

/// Returns the default location of the `.minecraft/versions/` directory.
pub fn versions_dir() -> Option<PathBuf> {
    minecraft_dir().inspect_mut(|path| path.push("versions"))
}
