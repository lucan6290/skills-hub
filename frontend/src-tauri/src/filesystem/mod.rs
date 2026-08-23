//! Platform-independent filesystem operations.
//!
//! All functions here delegate to std or the platform module,
//! keeping business logic free of OS-specific calls.

use std::path::Path;

use crate::platform;
use crate::utils::IGNORE_NAMES;

/// Check if a path exists (follows symlinks).
pub fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

/// Check if a path is a regular file (not a directory, not a symlink to dir).
pub fn is_file(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_file()
}

/// Check if a path is a directory.
pub fn is_dir(path: impl AsRef<Path>) -> bool {
    path.as_ref().is_dir()
}

/// Read entire file as UTF-8 string.
pub fn read_file(path: impl AsRef<Path>) -> Result<String, String> {
    let path = path.as_ref();
    std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path.display(), e))
}

/// Write content to a file, creating parent directories as needed.
pub fn write_file(path: impl AsRef<Path>, content: &[u8]) -> Result<(), String> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create parent dirs for {}: {}", path.display(), e))?;
    }
    std::fs::write(path, content).map_err(|e| format!("failed to write {}: {}", path.display(), e))
}

/// List files in a directory (non-recursive), returning sorted entries.
pub fn list_files(dir: impl AsRef<Path>) -> Result<Vec<std::path::PathBuf>, String> {
    let dir = dir.as_ref();
    if !dir.is_dir() {
        return Err(format!("not a directory: {}", dir.display()));
    }
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("failed to read dir {}: {}", dir.display(), e))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    entries.sort();
    Ok(entries)
}

/// Recursively copy a directory, skipping `.git` and symlinks.
pub fn copy_directory(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<(), String> {
    let source = source.as_ref();
    let target = target.as_ref();

    if !source.is_dir() {
        return Err(format!("source is not a directory: {}", source.display()));
    }

    std::fs::create_dir_all(target)
        .map_err(|e| format!("failed to create target dir {}: {}", target.display(), e))?;

    copy_dir_recursive_inner(source, target)
}

fn copy_dir_recursive_inner(source: &Path, target: &Path) -> Result<(), String> {
    let entries = std::fs::read_dir(source)
        .map_err(|e| format!("failed to read dir {}: {}", source.display(), e))?;

    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip ignored names
        if IGNORE_NAMES.contains(&name_str.as_ref()) {
            continue;
        }

        let src_path = entry.path();
        let dst_path = target.join(&name);

        // Skip symlinks to avoid infinite recursion
        if src_path.is_symlink() {
            continue;
        }

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path)
                .map_err(|e| format!("failed to create dir {}: {}", dst_path.display(), e))?;
            copy_dir_recursive_inner(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| {
                format!(
                    "failed to copy {} to {}: {}",
                    src_path.display(),
                    dst_path.display(),
                    e
                )
            })?;
        }
    }

    Ok(())
}

/// Create a symbolic link (directory).
pub fn create_symlink(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<(), String> {
    let source = source.as_ref();
    let target = target.as_ref();

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
            .map_err(|e| format!("failed to create symlink: {}", e))
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(source, target)
            .map_err(|e| format!("failed to create symlink: {}", e))
    }
}

/// Create a junction (Windows only, delegates to platform module).
pub fn create_junction(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<(), String> {
    platform::create_junction(source.as_ref(), target.as_ref())
}

/// Remove a file, directory, symlink, or junction.
///
/// For links/junctions, only removes the reparse point itself — never follows into target.
pub fn remove_link_or_directory(path: impl AsRef<Path>) -> Result<(), String> {
    let path = path.as_ref();
    let is_link = platform::is_link_or_junction(path);

    if !is_link && !path.exists() {
        return Ok(());
    }

    if is_link {
        // Try unlink first (works for file symlinks)
        match std::fs::remove_file(path) {
            Ok(_) => return Ok(()),
            Err(e) => {
                // Directory-type link/junction: use rmdir to only remove reparse point
                if e.kind() == std::io::ErrorKind::IsADirectory
                    || e.kind() == std::io::ErrorKind::PermissionDenied
                {
                    return std::fs::remove_dir(path).map_err(|e2| {
                        format!("failed to remove link/junction {}: {}", path.display(), e2)
                    });
                }
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Ok(());
                }
                return Err(format!("failed to remove link {}: {}", path.display(), e));
            }
        }
    }

    if path.is_dir() {
        std::fs::remove_dir_all(path)
            .map_err(|e| format!("failed to remove directory {}: {}", path.display(), e))
    } else {
        std::fs::remove_file(path)
            .map_err(|e| format!("failed to remove file {}: {}", path.display(), e))
    }
}

/// Open a folder in the system file explorer.
pub fn open_folder(path: impl AsRef<Path>) -> Result<(), String> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(format!("path does not exist: {}", path.display()));
    }

    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("failed to open folder: {}", e))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn copy_directory_copies_content_and_skips_git() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let target = dir.path().join("target");

        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("a.txt"), "alpha").unwrap();
        fs::create_dir(source.join("sub")).unwrap();
        fs::write(source.join("sub").join("b.txt"), "beta").unwrap();
        fs::create_dir(source.join(".git")).unwrap();
        fs::write(source.join(".git").join("config"), "git").unwrap();

        copy_directory(&source, &target).unwrap();

        assert_eq!(fs::read_to_string(target.join("a.txt")).unwrap(), "alpha");
        assert_eq!(
            fs::read_to_string(target.join("sub").join("b.txt")).unwrap(),
            "beta"
        );
        assert!(!target.join(".git").exists());
    }

    #[test]
    fn remove_link_or_directory_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("file.txt");
        fs::write(&file, "hello").unwrap();

        remove_link_or_directory(&file).unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn remove_link_or_directory_removes_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("empty");
        fs::create_dir(&sub).unwrap();

        remove_link_or_directory(&sub).unwrap();
        assert!(!sub.exists());
    }

    #[test]
    fn remove_link_or_directory_removes_dir_with_contents() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("with-content");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("nested.txt"), "nested").unwrap();

        remove_link_or_directory(&sub).unwrap();
        assert!(!sub.exists());
    }

    #[test]
    fn remove_link_or_directory_noop_for_nonexistent() {
        let result = remove_link_or_directory(Path::new("/nonexistent/path/12345"));
        assert!(result.is_ok());
    }

    #[test]
    fn write_file_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let deep = dir.path().join("a").join("b").join("c.txt");
        write_file(&deep, b"content").unwrap();
        assert_eq!(fs::read_to_string(&deep).unwrap(), "content");
    }

    #[test]
    fn exists_is_file_is_dir() {
        let dir = tempfile::tempdir().unwrap();
        assert!(exists(dir.path()));
        assert!(is_dir(dir.path()));
        assert!(!is_file(dir.path()));

        let file = dir.path().join("f.txt");
        fs::write(&file, "x").unwrap();
        assert!(exists(&file));
        assert!(is_file(&file));
        assert!(!is_dir(&file));
    }
}
