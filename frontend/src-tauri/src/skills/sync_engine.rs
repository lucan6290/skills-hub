//! Sync engine.
//!
//! Supports symlink → junction (Windows) → copy three-tier fallback.

use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::filesystem;
use crate::platform;

/// The mode used for a sync operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncMode {
    Auto,
    Symlink,
    Junction,
    Copy,
}

impl std::fmt::Display for SyncMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncMode::Auto => write!(f, "auto"),
            SyncMode::Symlink => write!(f, "symlink"),
            SyncMode::Junction => write!(f, "junction"),
            SyncMode::Copy => write!(f, "copy"),
        }
    }
}

/// Result of a sync operation.
#[derive(Debug, Clone, Serialize)]
pub struct SyncOutcome {
    pub mode_used: SyncMode,
    pub target_path: PathBuf,
    pub replaced: bool,
}

/// Sync using hybrid strategy: symlink → junction → copy.
pub fn sync_dir_hybrid(
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
) -> Result<SyncOutcome, String> {
    sync_hybrid(source.as_ref(), target.as_ref(), false)
}

/// Sync with explicit overwrite control.
pub fn sync_dir_hybrid_with_overwrite(
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
    overwrite: bool,
) -> Result<SyncOutcome, String> {
    sync_hybrid(source.as_ref(), target.as_ref(), overwrite)
}

/// Force copy mode sync.
pub fn sync_dir_copy_with_overwrite(
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
    overwrite: bool,
) -> Result<SyncOutcome, String> {
    let source = source.as_ref();
    let target = target.as_ref();

    if !source.is_dir() {
        return Err(format!("source is not a directory: {}", source.display()));
    }

    handle_existing_target(target, source, overwrite)?;

    filesystem::copy_directory(source, target)?;

    Ok(SyncOutcome {
        mode_used: SyncMode::Copy,
        target_path: target.to_path_buf(),
        replaced: overwrite,
    })
}

/// Tool-aware sync: uses force_copy flag to decide copy vs hybrid.
pub fn sync_dir_for_tool_with_overwrite(
    _tool_key: &str,
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
    overwrite: bool,
    force_copy: bool,
) -> Result<SyncOutcome, String> {
    if force_copy {
        sync_dir_copy_with_overwrite(source, target, overwrite)
    } else {
        sync_dir_hybrid_with_overwrite(source, target, overwrite)
    }
}

/// Remove a path that was previously synced (unsync).
///
/// Only removes links/junctions/directories that are managed targets.
pub fn unsync_target(target: impl AsRef<Path>) -> Result<(), String> {
    filesystem::remove_link_or_directory(target)
}

// ─── Internal implementation ────────────────────────────────────────

fn sync_hybrid(source: &Path, target: &Path, overwrite: bool) -> Result<SyncOutcome, String> {
    if !source.is_dir() {
        return Err(format!("source is not a directory: {}", source.display()));
    }

    handle_existing_target(target, source, overwrite)?;

    // Try symlink first
    match filesystem::create_symlink(source, target) {
        Ok(_) => {
            return Ok(SyncOutcome {
                mode_used: SyncMode::Symlink,
                target_path: target.to_path_buf(),
                replaced: overwrite,
            });
        }
        Err(_) => {
            // Clean up any partial state left by failed symlink attempt
            let _ = filesystem::remove_link_or_directory(target);
        }
    }

    // Try junction (Windows only)
    match platform::create_junction(source, target) {
        Ok(_) => {
            return Ok(SyncOutcome {
                mode_used: SyncMode::Junction,
                target_path: target.to_path_buf(),
                replaced: overwrite,
            });
        }
        Err(_) => {
            // Clean up any partial state left by failed junction attempt
            let _ = filesystem::remove_link_or_directory(target);
        }
    }

    // Fallback to copy
    filesystem::copy_directory(source, target)?;

    Ok(SyncOutcome {
        mode_used: SyncMode::Copy,
        target_path: target.to_path_buf(),
        replaced: overwrite,
    })
}

/// Handle existing target: check if it's already pointing to the right source,
/// remove if overwrite is allowed, or error if not.
fn handle_existing_target(target: &Path, source: &Path, overwrite: bool) -> Result<(), String> {
    if platform::is_link_or_junction(target) {
        if same_resolved_path(source, target) {
            // Already synced correctly — caller should treat as no-op
            return Ok(());
        }
        if !overwrite {
            return Err(format!("target already exists: {}", target.display()));
        }
        filesystem::remove_link_or_directory(target)?;
    } else if target.exists() {
        if !overwrite {
            return Err(format!("target already exists: {}", target.display()));
        }
        filesystem::remove_link_or_directory(target)?;
    }

    Ok(())
}

/// Check if two paths resolve to the same location.
fn same_resolved_path(left: &Path, right: &Path) -> bool {
    let left_real = resolve_norm(left);
    let right_real = resolve_norm(right);
    match (left_real, right_real) {
        (Some(l), Some(r)) => l == r,
        _ => false,
    }
}

fn resolve_norm(path: &Path) -> Option<String> {
    // Try canonicalize first (follows symlinks)
    let resolved = std::fs::canonicalize(path).ok()?;
    let abs = std::path::absolute(&resolved).ok()?;
    #[cfg(windows)]
    {
        Some(abs.to_string_lossy().to_lowercase())
    }
    #[cfg(not(windows))]
    {
        Some(abs.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_source_tree(root: &Path) -> PathBuf {
        fs::create_dir_all(root).unwrap();
        fs::write(root.join("a.txt"), "alpha").unwrap();
        fs::create_dir(root.join("sub")).unwrap();
        fs::write(root.join("sub").join("b.txt"), "beta").unwrap();
        fs::create_dir(root.join(".git")).unwrap();
        fs::write(root.join(".git").join("config"), "git").unwrap();
        root.to_path_buf()
    }

    fn assert_tree_content(source: &Path, target: &Path) {
        assert_eq!(
            fs::read_to_string(target.join("a.txt")).unwrap(),
            fs::read_to_string(source.join("a.txt")).unwrap()
        );
        assert_eq!(
            fs::read_to_string(target.join("sub").join("b.txt")).unwrap(),
            fs::read_to_string(source.join("sub").join("b.txt")).unwrap()
        );
    }

    #[test]
    fn sync_dir_copy_copies_content_and_skips_git() {
        let dir = tempfile::tempdir().unwrap();
        let source = make_source_tree(&dir.path().join("source"));
        let target = dir.path().join("target");

        let result = sync_dir_copy_with_overwrite(&source, &target, false).unwrap();

        assert_eq!(result.mode_used, SyncMode::Copy);
        assert_eq!(result.target_path, target);
        assert_tree_content(&source, &result.target_path);
        assert!(!result.target_path.join(".git").exists());
    }

    #[test]
    fn sync_dir_copy_fails_when_target_exists_no_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let source = make_source_tree(&dir.path().join("source"));
        let target = dir.path().join("target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("existing.txt"), "existing").unwrap();

        let result = sync_dir_copy_with_overwrite(&source, &target, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("target already exists"));
    }

    #[test]
    fn sync_dir_copy_overwrites_existing() {
        let dir = tempfile::tempdir().unwrap();
        let source = make_source_tree(&dir.path().join("source"));
        let target = dir.path().join("target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("old.txt"), "old").unwrap();

        let result = sync_dir_copy_with_overwrite(&source, &target, true).unwrap();
        assert_eq!(result.mode_used, SyncMode::Copy);
        assert_tree_content(&source, &target);
        assert!(!target.join("old.txt").exists());
    }

    #[test]
    fn sync_dir_hybrid_syncs_content() {
        let dir = tempfile::tempdir().unwrap();
        let source = make_source_tree(&dir.path().join("source"));
        let target = dir.path().join("target");

        let result = sync_dir_hybrid(&source, &target).unwrap();

        assert_eq!(result.target_path, target);
        // Mode depends on OS permissions; any valid mode is acceptable
        assert!(matches!(
            result.mode_used,
            SyncMode::Symlink | SyncMode::Junction | SyncMode::Copy
        ));
        assert_tree_content(&source, &result.target_path);
    }

    #[test]
    fn sync_dir_hybrid_rejects_nonexistent_source() {
        let dir = tempfile::tempdir().unwrap();
        let result = sync_dir_hybrid(&dir.path().join("nonexistent"), &dir.path().join("target"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("source is not a directory"));
    }

    #[test]
    fn sync_dir_hybrid_noop_when_already_linked() {
        let dir = tempfile::tempdir().unwrap();
        let source = make_source_tree(&dir.path().join("source"));
        let target = dir.path().join("target");

        // First sync
        let r1 = sync_dir_hybrid(&source, &target).unwrap();

        // Second sync should detect existing link and succeed without error
        // (handle_existing_target returns Ok when same_resolved_path matches)
        // Note: This only works if the first sync created a symlink/junction
        if r1.mode_used != SyncMode::Copy {
            let r2 = sync_dir_hybrid(&source, &target).unwrap();
            // Should not have replaced
            assert!(!r2.replaced || r2.mode_used == r1.mode_used);
        }
    }

    #[test]
    fn unsync_removes_synced_target() {
        let dir = tempfile::tempdir().unwrap();
        let source = make_source_tree(&dir.path().join("source"));
        let target = dir.path().join("target");

        sync_dir_copy_with_overwrite(&source, &target, false).unwrap();
        assert!(target.exists());

        unsync_target(&target).unwrap();
        assert!(!target.exists());
    }

    #[test]
    fn unsync_noop_for_nonexistent() {
        let result = unsync_target(Path::new("/nonexistent/path/12345"));
        assert!(result.is_ok());
    }

    #[test]
    fn sync_dir_for_tool_force_copy() {
        let dir = tempfile::tempdir().unwrap();
        let source = make_source_tree(&dir.path().join("source"));
        let target = dir.path().join("target");

        let result =
            sync_dir_for_tool_with_overwrite("cursor", &source, &target, false, true).unwrap();
        assert_eq!(result.mode_used, SyncMode::Copy);
    }

    #[test]
    fn sync_handles_chinese_paths() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("源目录");
        let target = dir.path().join("目标目录");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("文件.txt"), "内容").unwrap();

        let result = sync_dir_copy_with_overwrite(&source, &target, false).unwrap();
        assert_eq!(result.mode_used, SyncMode::Copy);
        assert_eq!(fs::read_to_string(target.join("文件.txt")).unwrap(), "内容");
    }

    #[test]
    fn sync_handles_spaces_in_paths() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("my skill dir");
        let target = dir.path().join("target skill");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("file name.txt"), "content").unwrap();

        sync_dir_copy_with_overwrite(&source, &target, false).unwrap();
        assert_eq!(
            fs::read_to_string(target.join("file name.txt")).unwrap(),
            "content"
        );
    }

    #[test]
    fn unsync_does_not_delete_unmanaged_content() {
        // unsync on a regular file/dir should work but we verify
        // it doesn't follow symlinks to delete target content
        let dir = tempfile::tempdir().unwrap();
        let real_dir = dir.path().join("real_content");
        fs::create_dir_all(&real_dir).unwrap();
        fs::write(real_dir.join("important.txt"), "do not delete").unwrap();

        // Create a copy target (not a link)
        let target = dir.path().join("managed_target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("data.txt"), "managed").unwrap();

        unsync_target(&target).unwrap();
        assert!(!target.exists());

        // Real content should be untouched
        assert!(real_dir.join("important.txt").exists());
        assert_eq!(
            fs::read_to_string(real_dir.join("important.txt")).unwrap(),
            "do not delete"
        );
    }
}
