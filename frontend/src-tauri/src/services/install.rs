//! Skill installation service — mirrors `backend/core/skills/installer.py` and
//! `backend/core/skills/install_service.py`.

use std::path::Path;

use serde::Serialize;

use crate::db::{now_ms, Database};
use crate::models::Skill;
use crate::repo::scanner::{is_suite_dir, normalize_source_type};
use crate::utils::content_hash;
use crate::utils::path_safety;

/// Parsed SKILL.md frontmatter.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SkillFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub category: Option<String>,
    pub homepage: Option<String>,
    pub frontmatter_extra: Option<String>,
}

/// Result of a skill installation.
#[derive(Debug, Clone, Serialize)]
pub struct InstallResult {
    pub skill_id: String,
    pub name: String,
    pub community_path: String,
    pub content_hash: Option<String>,
    pub description: Option<String>,
    pub source_subpath: Option<String>,
    pub frontmatter: Option<SkillFrontmatter>,
    pub skill_file_count: Option<i64>,
    pub skill_dir_size: Option<i64>,
}

/// A candidate skill found during local directory scanning.
#[derive(Debug, Clone, Serialize)]
pub struct LocalSkillCandidate {
    pub name: String,
    pub description: Option<String>,
    pub subpath: String,
    pub valid: bool,
    pub reason: Option<String>,
}

/// Known frontmatter keys matching Python KNOWN_FRONTMATTER_KEYS.
const KNOWN_FRONTMATTER_KEYS: &[&str] = &[
    "name",
    "description",
    "version",
    "author",
    "license",
    "category",
    "homepage",
];

/// Parse SKILL.md frontmatter from a skill directory.
pub fn parse_skill_md(path: &Path) -> SkillFrontmatter {
    let md_path = path.join("SKILL.md");
    if !md_path.exists() {
        return SkillFrontmatter::default();
    }

    let content = match std::fs::read_to_string(&md_path) {
        Ok(c) => c,
        Err(_) => return SkillFrontmatter::default(),
    };

    extract_frontmatter(&content)
}

fn extract_frontmatter(content: &str) -> SkillFrontmatter {
    let mut result = SkillFrontmatter::default();
    let mut all_fields: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    let re = match regex::Regex::new(r"(?s)^---\s*\r?\n(.*?)\r?\n---\s*\r?\n") {
        Ok(r) => r,
        Err(_) => return result,
    };

    if let Some(captures) = re.captures(content) {
        if let Some(fm_text) = captures.get(1) {
            for line in fm_text.as_str().lines() {
                let line = line.trim();
                if let Some((key, val)) = line.split_once(':') {
                    let key = key.trim();
                    let val = val.trim().trim_matches(|c: char| c == '"' || c == '\'');
                    if !key.is_empty() && !val.is_empty() {
                        all_fields.insert(key.to_string(), val.to_string());
                    }
                }
            }
        }
    }

    // Extract known fields
    for key in KNOWN_FRONTMATTER_KEYS {
        if let Some(val) = all_fields.remove(*key) {
            match *key {
                "name" => result.name = Some(val),
                "description" => result.description = Some(val),
                "version" => result.version = Some(val),
                "author" => result.author = Some(val),
                "license" => result.license = Some(val),
                "category" => result.category = Some(val),
                "homepage" => result.homepage = Some(val),
                _ => {}
            }
        }
    }

    // Remaining unknown fields → frontmatter_extra (JSON)
    if !all_fields.is_empty() {
        result.frontmatter_extra = serde_json::to_string(&all_fields).ok();
    }

    // Fallback: extract name from first # heading
    if result.name.is_none() {
        for line in content.lines() {
            let line = line.trim();
            if let Some(name) = line.strip_prefix("# ") {
                result.name = Some(name.trim().to_string());
                break;
            }
        }
    }

    result
}

/// Compute file count and total size of a skill directory.
/// Excludes .git, .DS_Store, Thumbs.db, .gitignore.
pub fn compute_skill_file_stats(dir_path: &Path) -> (i64, i64) {
    let mut file_count: i64 = 0;
    let mut total_size: i64 = 0;

    fn walk(dir: &Path, count: &mut i64, size: &mut i64) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();
                if crate::utils::IGNORE_NAMES.contains(&name.as_str()) {
                    continue;
                }
                let path = entry.path();
                if path.is_symlink() {
                    continue;
                }
                if path.is_dir() {
                    walk(&path, count, size);
                } else if path.is_file() {
                    if let Ok(meta) = path.metadata() {
                        *size += meta.len() as i64;
                    }
                    *count += 1;
                }
            }
        }
    }

    walk(dir_path, &mut file_count, &mut total_size);
    (file_count, total_size)
}

/// Scan a directory for skill candidates.
pub fn list_local_skills(base_path: &Path) -> Result<Vec<LocalSkillCandidate>, String> {
    if !base_path.is_dir() {
        return Err(format!("not a directory: {}", base_path.display()));
    }

    let mut candidates = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(base_path)
        .map_err(|e| format!("failed to read dir: {}", e))?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }

        let item = entry.path();
        let fm = parse_skill_md(&item);
        let name = fm.name.clone();
        let desc = fm.description.clone();

        let (final_name, final_desc, valid, reason) = if name.is_some() {
            (name.unwrap(), desc, true, None)
        } else if is_suite_dir(&item) {
            (
                item.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                None,
                true,
                None,
            )
        } else {
            let claude_skills = item.join(".claude").join("skills");
            if claude_skills.is_dir() {
                let sub_items: Vec<_> = std::fs::read_dir(&claude_skills)
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                            .collect()
                    })
                    .unwrap_or_default();
                if let Some(first) = sub_items.first() {
                    let fm2 = parse_skill_md(&first.path());
                    if fm2.name.is_some() {
                        (fm2.name.unwrap(), fm2.description, true, None)
                    } else {
                        (
                            item.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default(),
                            None,
                            false,
                            Some("missing_skill_md".to_string()),
                        )
                    }
                } else {
                    (
                        item.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        None,
                        false,
                        Some("missing_skill_md".to_string()),
                    )
                }
            } else {
                (
                    item.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    None,
                    false,
                    Some("missing_skill_md".to_string()),
                )
            }
        };

        candidates.push(LocalSkillCandidate {
            name: final_name,
            description: final_desc,
            subpath: entry.file_name().to_string_lossy().to_string(),
            valid,
            reason,
        });
    }

    Ok(candidates)
}

/// Install a skill from a local directory.
pub fn install_local_skill(
    db: &Database,
    source_path: &Path,
    name: Option<&str>,
    community_repo: Option<&Path>,
    source_type: &str,
) -> Result<InstallResult, String> {
    if !source_path.is_dir() {
        return Err(format!(
            "source is not a directory: {}",
            source_path.display()
        ));
    }

    let fm = parse_skill_md(source_path);
    let skill_name = name
        .map(|n| n.to_string())
        .or_else(|| fm.name.clone())
        .unwrap_or_else(|| {
            source_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "skill".to_string())
        });
    let skill_id = uuid::Uuid::new_v4().to_string();

    let (file_count, dir_size) = compute_skill_file_stats(source_path);

    if normalize_source_type(source_type) == "custom" {
        let content = content_hash::hash_dir(source_path).ok();
        return Ok(InstallResult {
            skill_id,
            name: skill_name,
            community_path: source_path.to_string_lossy().to_string(),
            content_hash: content,
            description: fm.description.clone(),
            source_subpath: None,
            frontmatter: Some(fm),
            skill_file_count: Some(file_count),
            skill_dir_size: Some(dir_size),
        });
    }

    // Community install: copy to community repo
    let community_base = match community_repo {
        Some(p) => p.to_path_buf(),
        None => crate::repo::community::resolve_community_repo_path(db),
    };
    std::fs::create_dir_all(&community_base)
        .map_err(|e| format!("failed to create community repo: {}", e))?;

    let dir_name = path_safety::safe_dir_name(Some(&skill_name));
    let mut target_dir = path_safety::safe_child_path(&community_base, &dir_name, "skill name")?;

    if target_dir.exists() {
        let alt_name = format!("{}-{}", dir_name, &skill_id[..8]);
        target_dir = path_safety::safe_child_path(&community_base, &alt_name, "skill name")?;
    }

    crate::filesystem::copy_directory(source_path, &target_dir)?;

    let content = content_hash::hash_dir(&target_dir).ok();
    let (target_file_count, target_dir_size) = compute_skill_file_stats(&target_dir);

    Ok(InstallResult {
        skill_id,
        name: skill_name,
        community_path: target_dir.to_string_lossy().to_string(),
        content_hash: content,
        description: fm.description.clone(),
        source_subpath: None,
        frontmatter: Some(fm),
        skill_file_count: Some(target_file_count),
        skill_dir_size: Some(target_dir_size),
    })
}

/// Install a selected skill from a base path + subpath.
pub fn install_local_skill_from_selection(
    db: &Database,
    base_path: &Path,
    subpath: &str,
    name: Option<&str>,
    community_repo: Option<&Path>,
    source_type: &str,
) -> Result<InstallResult, String> {
    let source =
        path_safety::require_path_within(&base_path.join(subpath), base_path, "skill selection")?;
    if !source.is_dir() {
        return Err(format!("skill not found: {}", source.display()));
    }

    let fm = parse_skill_md(&source);
    let skill_name = name
        .map(|n| n.to_string())
        .or_else(|| fm.name.clone())
        .unwrap_or_else(|| {
            Path::new(subpath)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "skill".to_string())
        });

    install_local_skill(db, &source, Some(&skill_name), community_repo, source_type)
}

/// Build a Skill record from an install result.
pub fn build_skill_record(result: &InstallResult, source_path: &str, source_type: &str) -> Skill {
    let now = now_ms();
    let fm = result.frontmatter.as_ref();
    Skill {
        id: result.skill_id.clone(),
        name: result.name.clone(),
        description: result.description.clone(),
        frontmatter_extra: fm.and_then(|f| f.frontmatter_extra.clone()),
        version: fm.and_then(|f| f.version.clone()),
        author: fm.and_then(|f| f.author.clone()),
        license: fm.and_then(|f| f.license.clone()),
        category: fm.and_then(|f| f.category.clone()),
        homepage: fm.and_then(|f| f.homepage.clone()),
        skill_file_count: result.skill_file_count,
        skill_dir_size: result.skill_dir_size,
        source_type: normalize_source_type(source_type),
        source_ref: Some(source_path.to_string()),
        source_subpath: result.source_subpath.clone(),
        community_path: result.community_path.clone(),
        content_hash: result.content_hash.clone(),
        created_at: now,
        updated_at: now,
        last_seen_at: now,
        status: "active".to_string(),
        ..Default::default()
    }
}

/// Upsert a skill from an install result into the database.
pub fn upsert_skill_from_install(
    db: &Database,
    result: &InstallResult,
    source_path: &str,
    source_type: &str,
) -> Result<(), String> {
    use crate::repositories::SkillsRepository;
    let repo = SkillsRepository::new(db);
    let mut record = build_skill_record(result, source_path, source_type);
    repo.upsert(&mut record).map_err(|e| e.to_string())
}

/// Check for duplicate by content hash. Returns existing skill ID if found.
pub fn dedupe_install_result(
    db: &Database,
    result: &InstallResult,
    source_type: &str,
) -> Option<String> {
    use crate::repositories::SkillsRepository;
    let content_hash = result.content_hash.as_ref()?;
    let repo = SkillsRepository::new(db);
    let normalized = normalize_source_type(source_type);

    if let Ok(Some(existing)) = repo.get_by_content_hash(content_hash) {
        if normalize_source_type(&existing.source_type) == normalized {
            // Clean up duplicate community path if different
            if normalized != "custom"
                && !result.community_path.is_empty()
                && result.community_path != existing.community_path
            {
                let _ = crate::filesystem::remove_link_or_directory(&result.community_path);
            }
            return Some(existing.id);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_md_with_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("SKILL.md"),
            "---\nname: Test Skill\ndescription: A test\nversion: 1.0\nauthor: Me\n---\n# Test\n",
        )
        .unwrap();

        let fm = parse_skill_md(dir.path());
        assert_eq!(fm.name.as_deref(), Some("Test Skill"));
        assert_eq!(fm.description.as_deref(), Some("A test"));
        assert_eq!(fm.version.as_deref(), Some("1.0"));
        assert_eq!(fm.author.as_deref(), Some("Me"));
    }

    #[test]
    fn test_parse_skill_md_fallback_heading() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("SKILL.md"), "# My Heading Skill\nContent\n").unwrap();

        let fm = parse_skill_md(dir.path());
        assert_eq!(fm.name.as_deref(), Some("My Heading Skill"));
    }

    #[test]
    fn test_parse_skill_md_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let fm = parse_skill_md(dir.path());
        assert!(fm.name.is_none());
    }

    #[test]
    fn test_compute_skill_file_stats() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "alpha").unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("sub").join("b.txt"), "beta").unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join(".git").join("config"), "git").unwrap();

        let (count, size) = compute_skill_file_stats(dir.path());
        assert_eq!(count, 2);
        assert_eq!(size, 9); // "alpha" + "beta"
    }

    #[test]
    fn test_list_local_skills() {
        let dir = tempfile::tempdir().unwrap();
        let s1 = dir.path().join("skill-a");
        std::fs::create_dir_all(&s1).unwrap();
        std::fs::write(s1.join("SKILL.md"), "---\nname: Skill A\n---\n# A\n").unwrap();

        let s2 = dir.path().join("not-skill");
        std::fs::create_dir_all(&s2).unwrap();
        std::fs::write(s2.join("README.md"), "hello").unwrap();

        let candidates = list_local_skills(dir.path()).unwrap();
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().any(|c| c.valid && c.name == "Skill A"));
        assert!(candidates.iter().any(|c| !c.valid));
    }

    #[test]
    fn test_install_local_skill_community() {
        let db = Database::new_in_memory().unwrap();
        let source = tempfile::tempdir().unwrap();
        std::fs::write(
            source.path().join("SKILL.md"),
            "---\nname: Install Test\n---\n# Install Test\n",
        )
        .unwrap();
        std::fs::write(source.path().join("code.py"), "print('hello')").unwrap();

        let community = tempfile::tempdir().unwrap();
        let result = install_local_skill(
            &db,
            source.path(),
            None,
            Some(community.path()),
            "community",
        )
        .unwrap();

        assert_eq!(result.name, "Install Test");
        assert!(result.content_hash.is_some());
        assert!(Path::new(&result.community_path).exists());
    }

    #[test]
    fn test_install_local_skill_custom() {
        let db = Database::new_in_memory().unwrap();
        let source = tempfile::tempdir().unwrap();
        std::fs::write(
            source.path().join("SKILL.md"),
            "---\nname: Custom Skill\n---\n# Custom\n",
        )
        .unwrap();

        let result = install_local_skill(&db, source.path(), None, None, "custom").unwrap();
        assert_eq!(result.name, "Custom Skill");
        // Custom skills keep original path
        assert_eq!(
            result.community_path,
            source.path().to_string_lossy().to_string()
        );
    }

    #[test]
    fn test_install_from_selection() {
        let db = Database::new_in_memory().unwrap();
        let base = tempfile::tempdir().unwrap();
        let skill = base.path().join("my-skill");
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(
            skill.join("SKILL.md"),
            "---\nname: Selected\n---\n# Selected\n",
        )
        .unwrap();

        let community = tempfile::tempdir().unwrap();
        let result = install_local_skill_from_selection(
            &db,
            base.path(),
            "my-skill",
            None,
            Some(community.path()),
            "community",
        )
        .unwrap();
        assert_eq!(result.name, "Selected");
    }
}
