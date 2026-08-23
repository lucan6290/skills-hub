//! Path safety utilities.

use std::path::{Path, PathBuf};

/// Windows reserved device names (case-insensitive).
const WINDOWS_RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Return a single safe directory component derived from a display name.
pub fn safe_dir_name(name: Option<&str>) -> String {
    safe_dir_name_with_fallback(name, "skill")
}

pub fn safe_dir_name_with_fallback(name: Option<&str>, fallback: &str) -> String {
    let raw = name.unwrap_or("").trim();

    // Replace forbidden characters with '-'
    let mut component = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || ch.is_control() {
            component.push('-');
        } else {
            component.push(ch);
        }
    }

    // Collapse whitespace, trim leading/trailing spaces and dots
    let trimmed: String = component.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = trimmed.trim_matches(|c: char| c == ' ' || c == '.');

    let component = if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        fallback.to_string()
    } else {
        trimmed.to_string()
    };

    // Check Windows reserved names
    let stem_upper = component.split('.').next().unwrap_or("").to_uppercase();
    let component = if WINDOWS_RESERVED_NAMES.contains(&stem_upper.as_str()) {
        format!("{}-skill", component)
    } else {
        component
    };

    // Truncate to 120 chars, then trim trailing spaces/dots
    let truncated: String = component.chars().take(120).collect();
    let result = truncated.trim_end_matches(|c: char| c == ' ' || c == '.');

    if result.is_empty() {
        fallback.to_string()
    } else {
        result.to_string()
    }
}

/// Normalize a path to an absolute, case-normalized form.
///
/// On Windows this lowercases the path for case-insensitive comparison.
pub fn norm_path(path: impl AsRef<Path>) -> String {
    let abs = std::path::absolute(path.as_ref()).unwrap_or_else(|_| path.as_ref().to_path_buf());
    normalize_case(&abs)
}

/// Lexical containment check without following the final symlink target.
pub fn is_path_within(path: impl AsRef<Path>, base: impl AsRef<Path>) -> bool {
    let candidate = match std::path::absolute(path.as_ref()) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let root = match std::path::absolute(base.as_ref()) {
        Ok(p) => p,
        Err(_) => return false,
    };

    let candidate_norm = normalize_case(&candidate);
    let root_norm = normalize_case(&root);

    // Check that candidate starts with root + separator (or equals root)
    if candidate_norm == root_norm {
        return true;
    }

    let root_with_sep = if root_norm.ends_with(std::path::MAIN_SEPARATOR) {
        root_norm
    } else {
        format!("{}{}", root_norm, std::path::MAIN_SEPARATOR)
    };

    candidate_norm.starts_with(&root_with_sep)
}

/// Require that `path` is within `base`, returning an error otherwise.
pub fn require_path_within(
    path: impl AsRef<Path>,
    base: impl AsRef<Path>,
    label: &str,
) -> Result<PathBuf, String> {
    let path = path.as_ref();
    let base = base.as_ref();
    if !is_path_within(path, base) {
        return Err(format!(
            "{} escapes base directory: {}",
            label,
            path.display()
        ));
    }
    Ok(path.to_path_buf())
}

/// Join a child name to a base path and verify containment.
pub fn safe_child_path(
    base: impl AsRef<Path>,
    child_name: &str,
    label: &str,
) -> Result<PathBuf, String> {
    let target = base.as_ref().join(child_name);
    require_path_within(&target, base, label)
}

/// Expand `~` and `~/` to the user's home directory.
pub fn expand_home(input: &str) -> String {
    let p = input.trim();
    let home = dirs_or_fallback();

    if p == "~" {
        return home;
    }
    if p.starts_with("~/") {
        return format!("{}{}{}", home, std::path::MAIN_SEPARATOR, &p[2..]);
    }
    if p.starts_with("~\\") {
        return format!("{}{}{}", home, std::path::MAIN_SEPARATOR, &p[2..]);
    }
    p.to_string()
}

// ─── Internal helpers ────────────────────────────────────────────────

fn normalize_case(path: &Path) -> String {
    #[cfg(windows)]
    {
        path.to_string_lossy().to_lowercase()
    }
    #[cfg(not(windows))]
    {
        path.to_string_lossy().to_string()
    }
}

fn dirs_or_fallback() -> String {
    // Use environment variable as primary source (avoids extra crate dependency)
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| "C:\\Users\\Default".to_string())
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME").unwrap_or_else(|_| "/root".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_dir_name_returns_single_component() {
        let name = safe_dir_name(Some("../bad/name"));
        assert!(!name.contains('/'));
        assert!(!name.contains('\\'));
        assert_ne!(name, "");
        assert_ne!(name, ".");
        assert_ne!(name, "..");
    }

    #[test]
    fn safe_dir_name_none_uses_fallback() {
        assert_eq!(safe_dir_name(None), "skill");
    }

    #[test]
    fn safe_dir_name_empty_uses_fallback() {
        assert_eq!(safe_dir_name(Some("")), "skill");
    }

    #[test]
    fn safe_dir_name_rewrites_windows_reserved_names() {
        for reserved in &["CON", "PRN", "AUX", "NUL"] {
            let result = safe_dir_name(Some(reserved));
            assert_ne!(result, *reserved);
            let stem = result.split('.').next().unwrap_or("").to_uppercase();
            assert_ne!(&stem, reserved);
        }
    }

    #[test]
    fn safe_dir_name_truncates_long_names() {
        let long_name = "a".repeat(200);
        let result = safe_dir_name(Some(&long_name));
        assert!(result.len() <= 120);
    }

    #[test]
    fn require_path_within_rejects_parent_escape() {
        let base = std::env::temp_dir().join("skills_hub_test_base");
        let _ = std::fs::create_dir_all(&base);
        let escape = base.join("..").join("outside");
        assert!(require_path_within(&escape, &base, "test").is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn require_path_within_accepts_child() {
        let base = std::env::temp_dir().join("skills_hub_test_base2");
        let _ = std::fs::create_dir_all(&base);
        let child = base.join("child");
        assert!(require_path_within(&child, &base, "test").is_ok());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn safe_child_path_joins() {
        let base = std::env::temp_dir().join("skills_hub_test_base3");
        let _ = std::fs::create_dir_all(&base);
        let child = safe_child_path(&base, "child", "test").unwrap();
        assert_eq!(child, base.join("child"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn safe_child_path_rejects_parent_escape() {
        let base = std::env::temp_dir().join("skills_hub_test_base4");
        let _ = std::fs::create_dir_all(&base);
        assert!(safe_child_path(&base, "..", "test").is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn expand_home_expands_tilde() {
        let home = dirs_or_fallback();
        assert_eq!(expand_home("~"), home);
    }

    #[test]
    fn expand_home_keeps_absolute_path() {
        let abs = std::env::temp_dir().to_string_lossy().to_string();
        assert_eq!(expand_home(&abs), abs);
    }

    #[test]
    fn expand_home_expands_tilde_slash() {
        let home = dirs_or_fallback();
        let result = expand_home("~/Documents");
        assert!(result.starts_with(&home));
        assert!(result.contains("Documents"));
    }

    #[test]
    fn is_path_within_same_path() {
        let base = std::env::temp_dir().join("skills_hub_within_test");
        let _ = std::fs::create_dir_all(&base);
        assert!(is_path_within(&base, &base));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn is_path_within_child() {
        let base = std::env::temp_dir().join("skills_hub_within_test2");
        let _ = std::fs::create_dir_all(&base);
        let child = base.join("sub").join("deep");
        assert!(is_path_within(&child, &base));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn is_path_within_rejects_sibling() {
        let base = std::env::temp_dir().join("skills_hub_within_test3");
        let sibling = std::env::temp_dir().join("skills_hub_within_test3_sibling");
        assert!(!is_path_within(&sibling, &base));
    }
}
