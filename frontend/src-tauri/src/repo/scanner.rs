//! Repository scanner.
//!
//! Scans community and custom repos for skills, registers them in the database,
//! and removes entries for skills that no longer exist on disk.

use std::path::{Path, PathBuf};

use crate::db::{now_ms, Database};
use crate::models::Skill;
use crate::utils::content_hash;
use crate::utils::path_safety;

use super::community::{resolve_community_repo_path, resolve_custom_repo_path};

/// Check if a directory is a valid skill (has SKILL.md or .claude/skills with SKILL.md).
pub fn is_skill_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    if path.join("SKILL.md").exists() {
        return true;
    }
    let claude_skills = path.join(".claude").join("skills");
    if claude_skills.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&claude_skills) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                    && entry.path().join("SKILL.md").exists()
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if a directory contains sub skills (any child is a skill dir).
pub fn has_sub_skills(path: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                && is_skill_dir(&entry.path())
            {
                return true;
            }
        }
    }
    false
}

/// Check if a directory is a strict suite (all children are skill dirs and >= 2).
pub fn is_suite_dir(path: &Path) -> bool {
    if !path.is_dir() || path.join("SKILL.md").exists() {
        return false;
    }
    let entries: Vec<_> = match std::fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .collect(),
        Err(_) => return false,
    };
    if entries.len() < 2 {
        return false;
    }
    entries.iter().all(|e| is_skill_dir(&e.path()))
}

/// Parse simple frontmatter from SKILL.md.
fn parse_frontmatter(skill_path: &Path) -> SkillFrontmatter {
    let md_path = skill_path.join("SKILL.md");
    let content = match std::fs::read_to_string(&md_path) {
        Ok(c) => c,
        Err(_) => return SkillFrontmatter::default(),
    };

    let mut fm = SkillFrontmatter::default();
    let re = match regex::Regex::new(r"^---\s*\n(.*?)\n---\s*\n") {
        Ok(r) => r,
        Err(_) => return fm,
    };

    if let Some(captures) = re.captures(&content) {
        if let Some(fm_text) = captures.get(1) {
            for line in fm_text.as_str().lines() {
                let line = line.trim();
                if let Some((key, val)) = line.split_once(':') {
                    let key = key.trim();
                    let val = val.trim().trim_matches(|c: char| c == '"' || c == '\'');
                    if !key.is_empty() && !val.is_empty() {
                        match key {
                            "name" => fm.name = Some(val.to_string()),
                            "description" => fm.description = Some(val.to_string()),
                            "version" => fm.version = Some(val.to_string()),
                            "author" => fm.author = Some(val.to_string()),
                            "license" => fm.license = Some(val.to_string()),
                            "category" => fm.category = Some(val.to_string()),
                            "homepage" => fm.homepage = Some(val.to_string()),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // Fallback: extract name from first # heading
    if fm.name.is_none() {
        for line in content.lines() {
            let line = line.trim();
            if let Some(name) = line.strip_prefix("# ") {
                fm.name = Some(name.trim().to_string());
                break;
            }
        }
    }

    fm
}

#[derive(Debug, Default)]
struct SkillFrontmatter {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    author: Option<String>,
    license: Option<String>,
    category: Option<String>,
    homepage: Option<String>,
}

/// Normalize source type string.
pub fn normalize_source_type(source_type: &str) -> String {
    let s = source_type.trim().to_lowercase();
    if s == "custom" || s == "self-made" || s == "self_made" || s == "自制" {
        "custom".to_string()
    } else {
        "community".to_string()
    }
}

/// Scan and register skills from the community repo.
pub fn scan_and_register_community_repo(
    db: &Database,
    community_repo_path: Option<&Path>,
) -> Result<usize, String> {
    let base_path = match community_repo_path {
        Some(p) => p.to_path_buf(),
        None => resolve_community_repo_path(db),
    };
    scan_and_register_repo(db, &base_path, "community")
}

/// Scan and register skills from the custom repo.
pub fn scan_and_register_custom_repo(
    db: &Database,
    custom_repo_path: Option<&Path>,
) -> Result<usize, String> {
    let base_path = match custom_repo_path {
        Some(p) => p.to_path_buf(),
        None => resolve_custom_repo_path(db),
    };
    scan_and_register_repo(db, &base_path, "custom")
}

fn scan_and_register_repo(
    db: &Database,
    base_path: &Path,
    source_type: &str,
) -> Result<usize, String> {
    if !base_path.is_dir() {
        return Ok(0);
    }

    use crate::repositories::SkillsRepository;
    let skills_repo = SkillsRepository::new(db);

    // Build existing map by normalized community_path
    let existing_skills = skills_repo.list("manual").map_err(|e| e.to_string())?;
    let mut existing_by_path: std::collections::HashMap<String, Skill> = existing_skills
        .into_iter()
        .filter(|s| !s.community_path.is_empty())
        .map(|s| (path_safety::norm_path(&s.community_path), s))
        .collect();

    let mut registered = 0;

    // Suite root handling for custom repos
    if source_type == "custom" && is_suite_dir(base_path) {
        if upsert_scanned_skill(db, base_path, source_type, &mut existing_by_path)? {
            registered += 1;
        }
        return Ok(registered);
    }

    let entries =
        std::fs::read_dir(base_path).map_err(|e| format!("failed to read repo dir: {}", e))?;
    let mut items: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    items.sort();

    for item in items {
        if !item.is_dir() {
            continue;
        }
        if item.file_name().map(|n| n == ".snapshots").unwrap_or(false) {
            continue;
        }

        let is_valid =
            is_skill_dir(&item) || (source_type == "custom" && has_sub_skills(&item));

        if !is_valid {
            continue;
        }

        if upsert_scanned_skill(db, &item, source_type, &mut existing_by_path)? {
            registered += 1;
        }
    }

    Ok(registered)
}

fn upsert_scanned_skill(
    db: &Database,
    item: &Path,
    source_type: &str,
    existing_by_path: &mut std::collections::HashMap<String, Skill>,
) -> Result<bool, String> {
    use crate::repositories::SkillsRepository;
    let skills_repo = SkillsRepository::new(db);
    let str_path = item.to_string_lossy().to_string();
    let fm = parse_frontmatter(item);
    let name = fm.name.unwrap_or_else(|| {
        item.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    let content_hash = content_hash::hash_dir(item).ok();
    let now = now_ms();

    let norm = path_safety::norm_path(&str_path);
    if let Some(existing) = existing_by_path.get(&norm) {
        // Update existing record
        let mut updated = existing.clone();
        updated.name = name;
        updated.description = fm.description;
        updated.version = fm.version;
        updated.author = fm.author;
        updated.license = fm.license;
        updated.category = fm.category;
        updated.homepage = fm.homepage;
        updated.source_type = source_type.to_string();
        updated.source_ref = Some(str_path.clone());
        updated.community_path = str_path;
        updated.content_hash = content_hash;
        updated.updated_at = now;
        updated.last_seen_at = now;
        updated.status = "active".to_string();
        skills_repo
            .upsert(&mut updated)
            .map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        // Create new record
        let mut new_skill = Skill {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: fm.description,
            version: fm.version,
            author: fm.author,
            license: fm.license,
            category: fm.category,
            homepage: fm.homepage,
            source_type: normalize_source_type(source_type),
            source_ref: Some(str_path.clone()),
            community_path: str_path.clone(),
            content_hash,
            created_at: now,
            updated_at: now,
            last_seen_at: now,
            status: "active".to_string(),
            ..Default::default()
        };
        skills_repo
            .upsert(&mut new_skill)
            .map_err(|e| e.to_string())?;
        existing_by_path.insert(norm, new_skill);
        Ok(true)
    }
}

/// Remove DB entries for skills that no longer exist on disk in the community repo.
pub fn remove_missing_community_repo_skills(db: &Database) -> Result<usize, String> {
    let base = resolve_community_repo_path(db);
    remove_missing_repo_skills(db, &base, "community")
}

/// Remove DB entries for skills that no longer exist on disk in the custom repo.
pub fn remove_missing_custom_repo_skills(db: &Database) -> Result<usize, String> {
    let base = resolve_custom_repo_path(db);
    remove_missing_repo_skills(db, &base, "custom")
}

fn remove_missing_repo_skills(
    db: &Database,
    repo_base: &Path,
    source_type: &str,
) -> Result<usize, String> {
    use crate::repositories::SkillsRepository;
    let skills_repo = SkillsRepository::new(db);
    let skills = skills_repo.list("manual").map_err(|e| e.to_string())?;
    let mut removed = 0;

    for skill in &skills {
        let normalized = normalize_source_type(&skill.source_type);
        if normalized != source_type {
            continue;
        }
        if skill.community_path.is_empty() {
            continue;
        }
        let path = PathBuf::from(&skill.community_path);
        if !path_safety::is_path_within(&path, repo_base) {
            continue;
        }
        if path.exists() {
            continue;
        }
        skills_repo.delete(&skill.id).map_err(|e| e.to_string())?;
        removed += 1;
    }

    Ok(removed)
}

/// Sync community repo registry: reconcile + remove missing + scan.
pub fn sync_community_repo_registry(
    db: &Database,
    community_repo_path: Option<&Path>,
) -> Result<SyncResult, String> {
    let removed = remove_missing_community_repo_skills(db)?;
    let registered = scan_and_register_community_repo(db, community_repo_path)?;
    Ok(SyncResult {
        removed,
        registered,
        normalized: 0,
    })
}

/// Sync custom repo registry: reconcile + remove missing + scan.
pub fn sync_custom_repo_registry(
    db: &Database,
    custom_repo_path: Option<&Path>,
) -> Result<SyncResult, String> {
    let removed = remove_missing_custom_repo_skills(db)?;
    let registered = scan_and_register_custom_repo(db, custom_repo_path)?;
    Ok(SyncResult {
        removed,
        registered,
        normalized: 0,
    })
}

/// Sync all repo registries.
pub fn sync_all_repo_registries(db: &Database) -> Result<SyncResult, String> {
    let removed =
        remove_missing_community_repo_skills(db)? + remove_missing_custom_repo_skills(db)?;
    let registered =
        scan_and_register_community_repo(db, None)? + scan_and_register_custom_repo(db, None)?;
    Ok(SyncResult {
        removed,
        registered,
        normalized: 0,
    })
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncResult {
    pub removed: usize,
    pub registered: usize,
    pub normalized: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_skill_dir_with_skill_md() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("SKILL.md"), "# Test").unwrap();
        assert!(is_skill_dir(dir.path()));
    }

    #[test]
    fn test_is_skill_dir_without_skill_md() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("README.md"), "# Test").unwrap();
        assert!(!is_skill_dir(dir.path()));
    }

    #[test]
    fn test_is_skill_dir_with_claude_skills() {
        let dir = tempfile::tempdir().unwrap();
        let claude_skills = dir.path().join(".claude").join("skills").join("my-skill");
        std::fs::create_dir_all(&claude_skills).unwrap();
        std::fs::write(claude_skills.join("SKILL.md"), "# My Skill").unwrap();
        assert!(is_skill_dir(dir.path()));
    }

    #[test]
    fn test_is_suite_dir() {
        let dir = tempfile::tempdir().unwrap();
        let s1 = dir.path().join("skill1");
        let s2 = dir.path().join("skill2");
        std::fs::create_dir_all(&s1).unwrap();
        std::fs::write(s1.join("SKILL.md"), "# Skill 1").unwrap();
        std::fs::create_dir_all(&s2).unwrap();
        std::fs::write(s2.join("SKILL.md"), "# Skill 2").unwrap();
        assert!(is_suite_dir(dir.path()));
    }

    #[test]
    fn test_is_suite_dir_single_child() {
        let dir = tempfile::tempdir().unwrap();
        let s1 = dir.path().join("skill1");
        std::fs::create_dir_all(&s1).unwrap();
        std::fs::write(s1.join("SKILL.md"), "# Skill 1").unwrap();
        assert!(!is_suite_dir(dir.path()));
    }

    #[test]
    fn test_normalize_source_type() {
        assert_eq!(normalize_source_type("community"), "community");
        assert_eq!(normalize_source_type("custom"), "custom");
        assert_eq!(normalize_source_type("Custom"), "custom");
        assert_eq!(normalize_source_type("self-made"), "custom");
        assert_eq!(normalize_source_type("unknown"), "community");
    }

    #[test]
    fn test_scan_empty_repo() {
        let db = Database::new_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let result = scan_and_register_repo(&db, dir.path(), "community").unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_scan_and_register_skills() {
        let db = Database::new_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();

        // Create two skills
        let s1 = dir.path().join("skill-alpha");
        std::fs::create_dir_all(&s1).unwrap();
        std::fs::write(
            s1.join("SKILL.md"),
            "---\nname: Alpha\ndescription: Alpha skill\n---\n# Alpha\n",
        )
        .unwrap();

        let s2 = dir.path().join("skill-beta");
        std::fs::create_dir_all(&s2).unwrap();
        std::fs::write(
            s2.join("SKILL.md"),
            "---\nname: Beta\ndescription: Beta skill\n---\n# Beta\n",
        )
        .unwrap();

        let registered = scan_and_register_repo(&db, dir.path(), "community").unwrap();
        assert_eq!(registered, 2);

        // Re-scan should update, not add
        let registered2 = scan_and_register_repo(&db, dir.path(), "community").unwrap();
        assert_eq!(registered2, 0);
    }

    #[test]
    fn test_remove_missing_skills() {
        let db = Database::new_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();

        let s1 = dir.path().join("skill-to-delete");
        std::fs::create_dir_all(&s1).unwrap();
        std::fs::write(
            s1.join("SKILL.md"),
            "---\nname: To Delete\n---\n# To Delete\n",
        )
        .unwrap();

        scan_and_register_repo(&db, dir.path(), "community").unwrap();

        // Delete the skill from disk
        std::fs::remove_dir_all(&s1).unwrap();

        let removed = remove_missing_repo_skills(&db, dir.path(), "community").unwrap();
        assert_eq!(removed, 1);
    }
}
