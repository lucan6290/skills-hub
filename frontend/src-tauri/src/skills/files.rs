//! Skill file listing, reading and writing — mirrors `backend/core/skills/files.py`.

use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::utils::{IGNORE_NAMES, MAX_FILE_SIZE};

/// A file entry within a skill directory.
#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    /// Relative path using POSIX separators.
    pub path: String,
    /// File size in bytes.
    pub size: u64,
}

/// List files in a skill directory, sorted with SKILL.md first.
///
/// Mirrors Python `list_files(community_path)`.
pub fn list_files(community_path: impl AsRef<Path>) -> Result<Vec<FileEntry>, String> {
    let base = community_path.as_ref();
    if !base.is_dir() {
        return Err(format!("not a directory: {}", base.display()));
    }

    let mut entries = Vec::new();
    collect_files(base, base, &mut entries)?;

    // Sort: SKILL.md first, then alphabetical by path
    entries.sort_by(|a, b| {
        let a_priority = if a.path == "SKILL.md" { 0 } else { 1 };
        let b_priority = if b.path == "SKILL.md" { 0 } else { 1 };
        a_priority
            .cmp(&b_priority)
            .then_with(|| a.path.cmp(&b.path))
    });

    Ok(entries)
}

fn collect_files(base: &Path, current: &Path, entries: &mut Vec<FileEntry>) -> Result<(), String> {
    let dir_entries = std::fs::read_dir(current)
        .map_err(|e| format!("failed to read dir {}: {}", current.display(), e))?;

    let mut sorted: Vec<_> = dir_entries.filter_map(|e| e.ok()).collect();
    sorted.sort_by_key(|e| e.file_name());

    for entry in sorted {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if IGNORE_NAMES.contains(&name_str.as_ref()) {
            continue;
        }

        let path = entry.path();

        if path.is_dir() && !path.is_symlink() {
            collect_files(base, &path, entries)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(base)
                .map_err(|e| format!("strip prefix failed: {}", e))?;
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            entries.push(FileEntry {
                path: rel_str,
                size,
            });
        }
    }

    Ok(())
}

/// Read a file from a skill directory with path traversal protection and size limit.
///
/// Mirrors Python `read_file(community_path, relative_path)`.
pub fn read_file(community_path: impl AsRef<Path>, relative_path: &str) -> Result<String, String> {
    let base = canonicalize_safe(community_path.as_ref())?;
    let target = canonicalize_safe_join(&base, relative_path)?;

    // Path traversal protection
    check_within(&target, &base, relative_path)?;

    if !target.is_file() {
        return Err(format!("file not found: {}", relative_path));
    }

    let metadata = std::fs::metadata(&target).map_err(|e| format!("cannot stat file: {}", e))?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(format!("file too large (>1MB): {}", relative_path));
    }

    let content = std::fs::read(&target).map_err(|e| format!("cannot open file: {}", e))?;

    if content.len() as u64 > MAX_FILE_SIZE {
        return Err(format!("file too large (>1MB): {}", relative_path));
    }

    String::from_utf8(content)
        .map_err(|_| format!("file is not valid UTF-8: {}", relative_path))
        .or_else(|_| {
            // Fallback: replace invalid chars (matching Python errors="replace")
            let bytes = std::fs::read(&target).unwrap_or_default();
            Ok(String::from_utf8_lossy(&bytes).to_string())
        })
}

/// Write content to a file in a skill directory with path traversal protection.
///
/// Only allows writing to existing files. Mirrors Python `write_file`.
pub fn write_file(
    community_path: impl AsRef<Path>,
    relative_path: &str,
    content: &str,
) -> Result<(), String> {
    let base = canonicalize_safe(community_path.as_ref())?;
    let target = canonicalize_safe_join(&base, relative_path)?;

    // Path traversal protection
    check_within(&target, &base, relative_path)?;

    if !target.is_file() {
        return Err(format!("file not found: {}", relative_path));
    }

    let data = content.as_bytes();
    if data.len() as u64 > MAX_FILE_SIZE {
        return Err(format!("content too large (>1MB): {}", relative_path));
    }

    std::fs::write(&target, data).map_err(|e| format!("cannot write file: {}", e))
}

// ─── Internal helpers ────────────────────────────────────────────────

fn canonicalize_safe(path: &Path) -> Result<PathBuf, String> {
    // Use absolute path (doesn't require existence for the base)
    std::path::absolute(path)
        .map_err(|e| format!("failed to resolve path {}: {}", path.display(), e))
}

fn canonicalize_safe_join(base: &Path, relative: &str) -> Result<PathBuf, String> {
    let joined = base.join(relative);
    std::path::absolute(&joined)
        .map_err(|e| format!("failed to resolve path {}: {}", joined.display(), e))
}

fn check_within(target: &Path, base: &Path, label: &str) -> Result<(), String> {
    use crate::utils::path_safety::is_path_within;
    if !is_path_within(target, base) {
        return Err(format!("path traversal not allowed: {}", label));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_skill(dir: &Path) {
        fs::write(dir.join("SKILL.md"), "# Test Skill").unwrap();
        fs::write(dir.join("config.json"), "{}").unwrap();
        fs::create_dir(dir.join("sub")).unwrap();
        fs::write(dir.join("sub").join("helper.md"), "# Helper").unwrap();
        fs::create_dir(dir.join(".git")).unwrap();
        fs::write(dir.join(".git").join("HEAD"), "ref").unwrap();
    }

    #[test]
    fn list_files_returns_sorted_with_skill_md_first() {
        let dir = tempfile::tempdir().unwrap();
        create_test_skill(dir.path());

        let entries = list_files(dir.path()).unwrap();
        assert!(!entries.is_empty());
        assert_eq!(entries[0].path, "SKILL.md");

        // Should contain sub/helper.md but not .git/HEAD
        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        assert!(paths.contains(&"config.json"));
        assert!(paths.contains(&"sub/helper.md"));
        assert!(!paths.iter().any(|p| p.contains(".git")));
    }

    #[test]
    fn list_files_not_a_directory_errors() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("notadir");
        fs::write(&file, "x").unwrap();
        assert!(list_files(&file).is_err());
    }

    #[test]
    fn read_file_returns_content() {
        let dir = tempfile::tempdir().unwrap();
        create_test_skill(dir.path());

        let content = read_file(dir.path(), "SKILL.md").unwrap();
        assert_eq!(content, "# Test Skill");
    }

    #[test]
    fn read_file_rejects_traversal() {
        let dir = tempfile::tempdir().unwrap();
        create_test_skill(dir.path());

        let result = read_file(dir.path(), "../../../etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("path traversal"));
    }

    #[test]
    fn read_file_rejects_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        create_test_skill(dir.path());

        let result = read_file(dir.path(), "nonexistent.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("file not found"));
    }

    #[test]
    fn read_file_rejects_oversized() {
        let dir = tempfile::tempdir().unwrap();
        let big_content = "x".repeat((MAX_FILE_SIZE + 1) as usize);
        fs::write(dir.path().join("big.txt"), &big_content).unwrap();

        let result = read_file(dir.path(), "big.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too large"));
    }

    #[test]
    fn write_file_updates_existing() {
        let dir = tempfile::tempdir().unwrap();
        create_test_skill(dir.path());

        write_file(dir.path(), "SKILL.md", "# Updated").unwrap();
        let content = fs::read_to_string(dir.path().join("SKILL.md")).unwrap();
        assert_eq!(content, "# Updated");
    }

    #[test]
    fn write_file_rejects_new_file() {
        let dir = tempfile::tempdir().unwrap();
        create_test_skill(dir.path());

        let result = write_file(dir.path(), "new_file.txt", "content");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("file not found"));
    }

    #[test]
    fn write_file_rejects_traversal() {
        let dir = tempfile::tempdir().unwrap();
        create_test_skill(dir.path());

        let result = write_file(dir.path(), "../../../tmp/hack.txt", "evil");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("path traversal"));
    }

    #[test]
    fn list_files_handles_chinese_and_spaces() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("中文文件.md"), "# 测试").unwrap();
        fs::write(dir.path().join("file with spaces.txt"), "content").unwrap();

        let entries = list_files(dir.path()).unwrap();
        assert_eq!(entries.len(), 2);
    }
}
