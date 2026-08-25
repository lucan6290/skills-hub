//! Community Repo path management.

use std::path::PathBuf;

use crate::db::Database;

pub const DEFAULT_COMMUNITY_REPO_NAME: &str = "skills";

/// Resolve the Community Repo path.
/// Priority: DB setting > ~/.skillshub/skills (default).
pub fn resolve_community_repo_path(db: &Database) -> PathBuf {
    use crate::repositories::SettingsRepository;
    let repo = SettingsRepository::new(db);
    if let Ok(Some(stored)) = repo.get("community_repo_path") {
        let p = PathBuf::from(&stored);
        if p.is_absolute() {
            return p;
        }
    }

    base_dir().join(DEFAULT_COMMUNITY_REPO_NAME)
}

/// Resolve the custom skill repo path.
/// Priority: DB setting > ~/.skillshub/custom-skills (default).
pub fn resolve_custom_repo_path(db: &Database) -> PathBuf {
    use crate::repositories::SettingsRepository;
    let repo = SettingsRepository::new(db);
    if let Ok(Some(stored)) = repo.get("custom_repo_path") {
        let p = PathBuf::from(&stored);
        if p.is_absolute() {
            return p;
        }
    }

    base_dir().join("custom-skills")
}

/// Base data directory: ~/.skillshub
fn base_dir() -> PathBuf {
    home_dir().join(".skillshub")
}

/// Ensure the community repo directory exists.
pub fn ensure_community_repo(path: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| format!("failed to create community repo dir: {}", e))
}

fn home_dir() -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(
            std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_else(|_| "C:\\Users\\Default".to_string()),
        )
    }
    #[cfg(not(windows))]
    {
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/root".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_community_repo_default() {
        let db = Database::new_in_memory().unwrap();
        let path = resolve_community_repo_path(&db);
        assert!(path.to_string_lossy().contains(DEFAULT_COMMUNITY_REPO_NAME));
    }

    #[test]
    fn test_resolve_custom_repo_default() {
        let db = Database::new_in_memory().unwrap();
        let path = resolve_custom_repo_path(&db);
        assert!(path.to_string_lossy().contains("custom-skills"));
    }

    #[test]
    fn test_resolve_community_repo_from_db() {
        let db = Database::new_in_memory().unwrap();
        // Use a platform-appropriate absolute path
        let custom_path = if cfg!(windows) {
            "C:\\custom\\community\\path"
        } else {
            "/custom/community/path"
        };
        {
            use crate::repositories::SettingsRepository;
            let repo = SettingsRepository::new(&db);
            repo.set("community_repo_path", custom_path).unwrap();
        }
        let path = resolve_community_repo_path(&db);
        assert_eq!(path, PathBuf::from(custom_path));
    }

    #[test]
    fn test_ensure_community_repo() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path().join("test_repo");
        ensure_community_repo(&repo_path).unwrap();
        assert!(repo_path.is_dir());
    }
}
