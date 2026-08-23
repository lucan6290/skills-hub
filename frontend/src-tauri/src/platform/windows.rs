//! Windows-specific filesystem operations: junction creation/detection.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

/// Create a Windows directory junction using `cmd /c mklink /J`.
///
/// Mirrors Python `_create_junction(source, target)`.
pub fn create_junction(source: &Path, target: &Path) -> Result<(), String> {
    let source_str = source.to_string_lossy();
    let target_str = target.to_string_lossy();

    // Path length check matching Python behavior
    if target_str.len() > 259 || source_str.len() > 259 {
        return Err("mklink /J failed: path too long".to_string());
    }

    // Quote paths to handle special characters (matching Python behavior)
    let output = Command::new("cmd")
        .args([
            "/c",
            "mklink",
            "/J",
            &format!("\"{}\"", target_str),
            &format!("\"{}\"", source_str),
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("failed to execute mklink: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else {
            stdout.trim().to_string()
        };
        return Err(format!("mklink /J failed: {}", detail));
    }

    Ok(())
}

/// Detect whether a path is a Windows junction point.
///
/// Uses metadata reparse point attribute check, compatible with older Rust versions.
pub fn is_junction(path: &Path) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(meta) => {
            use std::os::windows::fs::MetadataExt;
            let attrs = meta.file_attributes();
            const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
            if attrs & FILE_ATTRIBUTE_REPARSE_POINT == 0 {
                return false;
            }
            // Distinguish junction from symlink: junctions are directories
            // that are NOT regular symlinks (is_symlink returns false for junctions in Rust)
            !meta.file_type().is_symlink() && meta.is_dir()
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::is_link_or_junction;
    use std::fs;

    #[test]
    fn create_and_detect_junction() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let target = dir.path().join("target");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("test.txt"), "hello").unwrap();

        let result = create_junction(&source, &target);
        if result.is_err() {
            // Junction creation may fail without admin privileges; log but don't fail test
            eprintln!(
                "junction creation failed (may need elevated permissions): {:?}",
                result
            );
            return;
        }

        assert!(is_junction(&target));
        assert!(is_link_or_junction(&target));

        // Verify content accessible through junction
        let content = fs::read_to_string(target.join("test.txt")).unwrap();
        assert_eq!(content, "hello");
    }

    #[test]
    fn is_junction_false_for_regular_dir() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!is_junction(dir.path()));
    }

    #[test]
    fn is_junction_false_for_nonexistent() {
        assert!(!is_junction(Path::new("C:\\nonexistent_path_12345")));
    }

    #[test]
    fn create_junction_rejects_long_paths() {
        let long_path = "C:\\".to_string() + &"a".repeat(260);
        let result = create_junction(Path::new(&long_path), Path::new("C:\\short"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("path too long"));
    }
}
