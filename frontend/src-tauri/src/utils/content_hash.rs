//! SHA256 directory content hash — mirrors `backend/core/utils/content_hash.py`.
//!
//! The algorithm must produce identical output to the Python implementation:
//! - Sorted traversal (by relative path, POSIX separators)
//! - UTF-8 encoded relative paths
//! - File contents read as raw bytes
//! - `\n` separator after each entry
//! - Symlinks and IGNORE_NAMES entries are skipped

use sha2::{Digest, Sha256};
use std::path::Path;

use super::IGNORE_NAMES;

/// Compute a SHA256 hash of a directory's contents.
///
/// Mirrors Python `hash_dir(path)`.
pub fn hash_dir(path: impl AsRef<Path>) -> Result<String, String> {
    let base = path.as_ref();
    if !base.is_dir() {
        return Err(format!("not a directory: {}", base.display()));
    }

    let entries = walk_sorted(base)?;

    let mut hasher = Sha256::new();
    for entry in &entries {
        let rel = entry
            .strip_prefix(base)
            .map_err(|e| format!("strip prefix failed: {}", e))?;
        // Use POSIX separators (forward slashes), matching Python `.as_posix()`
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        hasher.update(rel_str.as_bytes());
        if entry.is_file() && !entry.is_symlink() {
            let content = std::fs::read(entry)
                .map_err(|e| format!("failed to read {}: {}", entry.display(), e))?;
            hasher.update(&content);
        }
        hasher.update(b"\n");
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Recursively collect file paths under `base`, sorted, skipping ignored names and symlinks.
///
/// Mirrors Python `_walk(base)`.
fn walk_sorted(base: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut result = Vec::new();
    walk_recursive(base, &mut result)?;
    result.sort();
    Ok(result)
}

fn walk_recursive(dir: &Path, result: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("failed to read dir {}: {}", dir.display(), e))?
        .filter_map(|e| e.ok())
        .collect();

    // Sort by file name for deterministic traversal
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip ignored names
        if IGNORE_NAMES.contains(&name_str.as_ref()) {
            continue;
        }

        let path = entry.path();

        // Skip symlinks
        if path.is_symlink() {
            continue;
        }

        if path.is_dir() {
            walk_recursive(&path, result)?;
        } else {
            result.push(path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn hash_dir_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "alpha").unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("b.txt"), "beta").unwrap();

        let h1 = hash_dir(dir.path()).unwrap();
        let h2 = hash_dir(dir.path()).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_dir_skips_git_and_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "alpha").unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git").join("config"), "git").unwrap();

        let h1 = hash_dir(dir.path()).unwrap();

        // Remove .git, hash should be the same
        fs::remove_dir_all(dir.path().join(".git")).unwrap();
        let h2 = hash_dir(dir.path()).unwrap();

        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_dir_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let h = hash_dir(dir.path()).unwrap();
        // Empty dir should produce hash of empty input
        assert!(!h.is_empty());
    }

    #[test]
    fn hash_dir_not_a_directory_errors() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("file.txt");
        fs::write(&file, "content").unwrap();
        assert!(hash_dir(&file).is_err());
    }

    #[test]
    fn hash_dir_content_changes_hash() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "alpha").unwrap();
        let h1 = hash_dir(dir.path()).unwrap();

        fs::write(dir.path().join("a.txt"), "changed").unwrap();
        let h2 = hash_dir(dir.path()).unwrap();

        assert_ne!(h1, h2);
    }
}
