//! Platform-specific implementations.
//!
//! Windows junction/symlink creation and detection live here,
//! keeping Windows API calls out of business logic.

#[cfg(windows)]
pub mod windows;

/// Create a directory junction (Windows only).
///
/// On non-Windows platforms this always returns an error.
pub fn create_junction(source: &std::path::Path, target: &std::path::Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        windows::create_junction(source, target)
    }
    #[cfg(not(windows))]
    {
        let _ = (source, target);
        Err("junctions are only supported on Windows".to_string())
    }
}

/// Check if a path is a Windows junction.
pub fn is_junction(path: &std::path::Path) -> bool {
    #[cfg(windows)]
    {
        windows::is_junction(path)
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        false
    }
}

/// Check if a path is a symlink or junction.
pub fn is_link_or_junction(path: &std::path::Path) -> bool {
    path.is_symlink() || is_junction(path)
}
