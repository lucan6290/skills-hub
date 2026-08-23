pub mod content_hash;
pub mod path_safety;

/// Shared ignore names.
pub const IGNORE_NAMES: &[&str] = &[".git", ".DS_Store", "Thumbs.db", ".gitignore"];

/// Maximum file size for skill file read/write (1 MB).
pub const MAX_FILE_SIZE: u64 = 1 * 1024 * 1024;
