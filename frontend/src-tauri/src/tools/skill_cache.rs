//! Tool skill cache — mirrors `backend/core/tools/skill_cache.py`.
//!
//! Scans tool skills directories, builds cache entries, and manages
//! the tool_skill_cache database table.

use serde::Serialize;

use crate::db::{now_ms, Database};
use crate::repositories::tool_cache::ToolSkillCacheEntry;
use crate::repositories::ToolCacheRepository;
use crate::tools::adapter::{self, scan_tool_dir, supports_project_scope, ToolAdapter};
use crate::utils::path_safety;

/// Response DTO for tool skills queries.
#[derive(Debug, Clone, Serialize)]
pub struct ToolSkillsResponse {
    pub tool_key: String,
    pub tool_name: String,
    pub installed: bool,
    pub skills_dir: Option<String>,
    pub supports_project_scope: bool,
    pub skills: Vec<ToolSkillEntryDto>,
    pub cached: bool,
    pub scanned_at: Option<i64>,
}

/// A single skill entry in the response.
#[derive(Debug, Clone, Serialize)]
pub struct ToolSkillEntryDto {
    pub name: String,
    pub path: String,
    pub is_link: bool,
    pub link_target: Option<String>,
    pub description: Option<String>,
    pub in_community_repo: bool,
}

/// Get mtime (nanoseconds) for a path. Returns None if not accessible.
fn path_mtime_ns(path: Option<&str>) -> Option<i64> {
    let p = path?;
    std::fs::metadata(p)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as i64)
}

/// Get the latest mtime of a skill directory and its SKILL.md.
fn skill_mtime_ns(skill_path: &str) -> Option<i64> {
    let dir_time = path_mtime_ns(Some(skill_path));
    let md_path = std::path::Path::new(skill_path).join("SKILL.md");
    let md_time = path_mtime_ns(Some(&md_path.to_string_lossy()));
    match (dir_time, md_time) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Check if a skill path is within a managed repo (community or custom).
fn check_in_managed_repo(db: &Database, skill_path: &str) -> bool {
    use crate::repo::community::{resolve_community_repo_path, resolve_custom_repo_path};
    let community = resolve_community_repo_path(db);
    if path_safety::is_path_within(skill_path, &community) {
        return true;
    }
    let custom = resolve_custom_repo_path(db);
    path_safety::is_path_within(skill_path, &custom)
}

/// Build skill entries by scanning a tool's skills directory.
pub fn build_skill_entries(
    db: &Database,
    skills_dir: &str,
    tool_key: &str,
) -> Vec<ToolSkillEntryDto> {
    let adapters = adapter::effective_tool_adapters(db);
    let adapter = match adapter::adapter_by_key(&adapters, tool_key) {
        Some(a) => a.clone(),
        None => return Vec::new(),
    };

    let detected = scan_tool_dir(&adapter, skills_dir);

    // Get managed source paths from skills table
    use crate::repositories::SkillsRepository;
    let skills_repo = SkillsRepository::new(db);
    let managed_paths: Vec<String> = skills_repo
        .list("manual")
        .unwrap_or_default()
        .into_iter()
        .filter_map(|s| {
            if s.community_path.is_empty() {
                None
            } else {
                Some(s.community_path.replace('\\', "/"))
            }
        })
        .collect();

    detected
        .into_iter()
        .map(|skill| {
            // Parse description from SKILL.md
            let description = parse_skill_description(&skill.path);

            let normalized = skill.path.replace('\\', "/");
            let in_community = managed_paths.iter().any(|cp| {
                cp == &normalized
                    || normalized.starts_with(&format!("{}/", cp.trim_end_matches('/')))
            }) || check_in_managed_repo(db, &skill.path);

            // For symlink skills, also check link target
            let in_community = if !in_community && skill.is_link {
                if let Some(ref lt) = skill.link_target {
                    check_in_managed_repo(db, lt)
                } else {
                    false
                }
            } else {
                in_community
            };

            ToolSkillEntryDto {
                name: skill.name,
                path: skill.path,
                is_link: skill.is_link,
                link_target: skill.link_target,
                description,
                in_community_repo: in_community,
            }
        })
        .collect()
}

/// Parse the description from a skill's SKILL.md frontmatter.
fn parse_skill_description(skill_path: &str) -> Option<String> {
    let md_path = std::path::Path::new(skill_path).join("SKILL.md");
    let content = std::fs::read_to_string(&md_path).ok()?;
    // Simple frontmatter extraction
    if let Some(fm) = extract_simple_frontmatter(&content) {
        fm.get("description").cloned()
    } else {
        None
    }
}

/// Extract simple YAML-like frontmatter from markdown content.
fn extract_simple_frontmatter(content: &str) -> Option<std::collections::HashMap<String, String>> {
    let re = regex::Regex::new(r"(?s)^---\s*\r?\n(.*?)\r?\n---\s*\r?\n").ok()?;
    let captures = re.captures(content)?;
    let fm_text = captures.get(1)?.as_str();

    let mut map = std::collections::HashMap::new();
    for line in fm_text.lines() {
        let line = line.trim();
        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim();
            let val = val.trim().trim_matches(|c: char| c == '"' || c == '\'');
            if !key.is_empty() && !val.is_empty() {
                map.insert(key.to_string(), val.to_string());
            }
        }
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

/// Convert skill entries to cache payload with mtime.
fn cache_payload_from_entries(entries: &[ToolSkillEntryDto]) -> Vec<ToolSkillCacheEntry> {
    entries
        .iter()
        .map(|e| ToolSkillCacheEntry {
            path: e.path.clone(),
            name: e.name.clone(),
            is_link: e.is_link,
            link_target: e.link_target.clone(),
            description: e.description.clone(),
            in_community_repo: e.in_community_repo,
            skill_mtime_ns: skill_mtime_ns(&e.path),
        })
        .collect()
}

/// Refresh tool cache: rescan and write to database.
pub fn refresh_tool_cache(
    db: &Database,
    adapter: &ToolAdapter,
    installed: bool,
    skills_dir: Option<&str>,
) -> Result<ToolSkillsResponse, String> {
    let tool_key = &adapter.tool_key;
    let scanned_at = now_ms();
    let dir_mtime = path_mtime_ns(skills_dir);

    let entries = if installed && skills_dir.is_some() {
        build_skill_entries(db, skills_dir.unwrap(), tool_key)
    } else {
        Vec::new()
    };

    let cache_entries = cache_payload_from_entries(&entries);

    let repo = ToolCacheRepository::new(db);
    repo.replace_skill_cache(
        tool_key,
        &adapter.display_name,
        installed,
        skills_dir,
        supports_project_scope(adapter),
        dir_mtime,
        scanned_at,
        &cache_entries,
    )
    .map_err(|e| format!("failed to update cache: {}", e))?;

    Ok(ToolSkillsResponse {
        tool_key: tool_key.clone(),
        tool_name: adapter.display_name.clone(),
        installed,
        skills_dir: skills_dir.map(|s| s.to_string()),
        supports_project_scope: supports_project_scope(adapter),
        skills: entries,
        cached: false,
        scanned_at: Some(scanned_at),
    })
}

/// Build a cached tool response from the database cache.
pub fn cached_tool_response(db: &Database, adapter: &ToolAdapter) -> ToolSkillsResponse {
    let repo = ToolCacheRepository::new(db);
    let tool_key = &adapter.tool_key;

    let state = repo.get_scan_state(tool_key).ok().flatten();

    let skills = if state.is_some() {
        repo.list_skill_cache(tool_key)
            .unwrap_or_default()
            .into_iter()
            .map(|c| ToolSkillEntryDto {
                name: c.name,
                path: c.path,
                is_link: c.is_link,
                link_target: c.link_target,
                description: c.description,
                in_community_repo: c.in_community_repo,
            })
            .collect()
    } else {
        Vec::new()
    };

    ToolSkillsResponse {
        tool_key: tool_key.clone(),
        tool_name: state
            .as_ref()
            .map(|s| s.tool_name.clone())
            .unwrap_or_else(|| adapter.display_name.clone()),
        installed: state.as_ref().map(|s| s.installed).unwrap_or(false),
        skills_dir: state.as_ref().and_then(|s| s.skills_dir.clone()),
        supports_project_scope: state
            .as_ref()
            .map(|s| s.supports_project_scope)
            .unwrap_or_else(|| supports_project_scope(adapter)),
        skills,
        cached: true,
        scanned_at: state.as_ref().map(|s| s.scanned_at),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_frontmatter() {
        let content =
            "---\nname: Test Skill\ndescription: A test skill\nversion: 1.0\n---\n# Test\n";
        let fm = extract_simple_frontmatter(content).unwrap();
        assert_eq!(fm.get("name").unwrap(), "Test Skill");
        assert_eq!(fm.get("description").unwrap(), "A test skill");
        assert_eq!(fm.get("version").unwrap(), "1.0");
    }

    #[test]
    fn test_extract_frontmatter_no_frontmatter() {
        let content = "# Just a heading\nSome content";
        assert!(extract_simple_frontmatter(content).is_none());
    }

    #[test]
    fn test_skill_mtime_ns_nonexistent() {
        assert!(skill_mtime_ns("/nonexistent/path/12345").is_none());
    }

    #[test]
    fn test_build_skill_entries_unknown_tool() {
        let db = Database::new_in_memory().unwrap();
        let result = build_skill_entries(&db, "/some/path", "nonexistent_tool");
        assert!(result.is_empty());
    }

    #[test]
    fn test_refresh_tool_cache_not_installed() {
        let db = Database::new_in_memory().unwrap();
        let adapter = ToolAdapter {
            tool_key: "test_tool".to_string(),
            display_name: "Test Tool".to_string(),
            relative_skills_dir: ".test/skills".to_string(),
            relative_detect_dir: ".test".to_string(),
            supports_symlink: true,
            supports_junction: true,
            force_copy: false,
            supports_project_scope_override: None,
            project_relative_skills_dir: None,
            is_custom: false,
        };

        let response = refresh_tool_cache(&db, &adapter, false, None).unwrap();
        assert!(!response.installed);
        assert!(response.skills.is_empty());
        assert!(!response.cached);
    }

    #[test]
    fn test_cached_tool_response_no_state() {
        let db = Database::new_in_memory().unwrap();
        let adapter = ToolAdapter {
            tool_key: "no_cache_tool".to_string(),
            display_name: "No Cache".to_string(),
            relative_skills_dir: ".nocache/skills".to_string(),
            relative_detect_dir: ".nocache".to_string(),
            supports_symlink: true,
            supports_junction: true,
            force_copy: false,
            supports_project_scope_override: None,
            project_relative_skills_dir: None,
            is_custom: false,
        };

        let response = cached_tool_response(&db, &adapter);
        assert!(response.cached);
        assert!(!response.installed);
        assert!(response.skills.is_empty());
    }
}
