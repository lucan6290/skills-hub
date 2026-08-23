//! Onboarding service.
//!
//! Scans installed tools for existing skills, groups them by name,
//! detects conflicts, and generates an onboarding plan.

use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

use crate::db::Database;
use crate::tools::adapter::{
    effective_tool_adapters, is_tool_installed, resolve_default_path, scan_tool_dir, ToolAdapter,
};
use crate::utils::content_hash;

/// A single variant of a skill found in a tool.
#[derive(Debug, Clone, Serialize)]
pub struct OnboardingVariant {
    pub tool: String,
    pub name: String,
    pub path: String,
    pub fingerprint: Option<String>,
    pub is_link: bool,
    pub link_target: Option<String>,
}

/// A group of same-named skills across tools.
#[derive(Debug, Clone, Serialize)]
pub struct OnboardingGroup {
    pub name: String,
    pub variants: Vec<OnboardingVariant>,
    pub has_conflict: bool,
}

/// The complete onboarding plan.
#[derive(Debug, Clone, Serialize)]
pub struct OnboardingPlan {
    pub total_tools_scanned: usize,
    pub total_skills_found: usize,
    pub groups: Vec<OnboardingGroup>,
}

/// Normalize a path for comparison (lowercase, forward slashes, strip Windows prefixes).
fn normalize_path_for_compare(p: &str) -> String {
    let mut s = p.replace('\\', "/").to_lowercase();
    for prefix in &["//?/", "//./"] {
        if s.starts_with(prefix) {
            s = s[prefix.len()..].to_string();
        }
    }
    s.trim_end_matches('/').to_string()
}

/// Check if path starts with a prefix (directory boundary aware).
fn path_starts_with(path: &str, prefix: &str) -> bool {
    let np = normalize_path_for_compare(path);
    let nprefix = normalize_path_for_compare(prefix);
    np == nprefix || np.starts_with(&format!("{}/", nprefix))
}

/// Build the onboarding plan by scanning all installed tools.
pub fn build_onboarding_plan(
    db: &Database,
    community_repo_path: Option<&str>,
    managed_target_paths: Option<&std::collections::HashSet<String>>,
    custom_repo_path: Option<&str>,
) -> OnboardingPlan {
    let adapters = effective_tool_adapters(db);
    let mut all_variants: Vec<OnboardingVariant> = Vec::new();
    let mut scanned = 0;

    for adapter in &adapters {
        if !is_tool_installed(adapter) {
            continue;
        }
        scanned += 1;

        let skills_dir = resolve_default_path(adapter);

        // Try cache first, fall back to FS scan
        let detected = detect_from_cache(db, adapter, &skills_dir, custom_repo_path)
            .unwrap_or_else(|| scan_tool_dir(adapter, &skills_dir));

        for skill in detected {
            // Filter community repo paths
            if let Some(cr) = community_repo_path {
                if path_starts_with(&skill.path, cr) {
                    continue;
                }
            }

            // Filter managed target paths
            if let Some(targets) = managed_target_paths {
                if targets.contains(&skill.path) {
                    continue;
                }
            }

            // Filter links pointing to managed repos
            if skill.is_link {
                if let Some(ref lt) = skill.link_target {
                    if let Some(cr) = community_repo_path {
                        if path_starts_with(lt, cr) {
                            continue;
                        }
                    }
                    if let Some(cu) = custom_repo_path {
                        if path_starts_with(lt, cu) {
                            continue;
                        }
                    }
                }
            }

            // Compute fingerprint
            let fingerprint = if Path::new(&skill.path).is_dir() {
                content_hash::hash_dir(&skill.path).ok()
            } else {
                None
            };

            all_variants.push(OnboardingVariant {
                tool: skill.tool.clone(),
                name: skill.name.clone(),
                path: skill.path,
                fingerprint,
                is_link: skill.is_link,
                link_target: skill.link_target,
            });
        }
    }

    // Group by name
    let mut groups_map: HashMap<String, Vec<OnboardingVariant>> = HashMap::new();
    for v in all_variants {
        groups_map.entry(v.name.clone()).or_default().push(v);
    }

    let mut groups: Vec<OnboardingGroup> = groups_map
        .into_iter()
        .map(|(name, variants)| {
            let fingerprints: Vec<_> = variants
                .iter()
                .filter_map(|v| v.fingerprint.as_ref())
                .collect();
            let unique_count = {
                let mut set = std::collections::HashSet::new();
                for fp in &fingerprints {
                    set.insert(fp.as_str());
                }
                set.len()
            };
            let has_conflict = unique_count > 1 && fingerprints.len() > 1;

            OnboardingGroup {
                name,
                variants,
                has_conflict,
            }
        })
        .collect();

    groups.sort_by(|a, b| a.name.cmp(&b.name));

    OnboardingPlan {
        total_tools_scanned: scanned,
        total_skills_found: groups.len(),
        groups,
    }
}

/// Try to get detected skills from the tool_skill_cache.
fn detect_from_cache(
    db: &Database,
    adapter: &ToolAdapter,
    skills_dir: &str,
    custom_repo_path: Option<&str>,
) -> Option<Vec<crate::tools::adapter::DetectedSkill>> {
    use crate::repositories::ToolCacheRepository;
    let repo = ToolCacheRepository::new(db);
    let tool_key = &adapter.tool_key;

    let state = repo.get_scan_state(tool_key).ok().flatten()?;
    if !state.installed {
        return None;
    }

    // Check mtime match
    let actual_mtime = dir_mtime_ns(skills_dir);
    if actual_mtime.is_none() || state.dir_mtime_ns.is_none() {
        return None;
    }
    if actual_mtime != state.dir_mtime_ns {
        return None;
    }

    let cache_entries = repo.list_skill_cache(tool_key).ok()?;
    if cache_entries.is_empty() {
        return None;
    }

    let result: Vec<_> = cache_entries
        .into_iter()
        .filter(|e| !e.in_community_repo)
        .filter(|e| {
            if let Some(cr) = custom_repo_path {
                if e.is_link {
                    if let Some(ref lt) = e.link_target {
                        let clean_lt = clean_windows_prefix(lt);
                        if path_starts_with(&clean_lt, cr) {
                            return false;
                        }
                    }
                }
            }
            true
        })
        .map(|e| crate::tools::adapter::DetectedSkill {
            tool: adapter.tool_key.clone(),
            name: e.name,
            path: e.path,
            is_link: e.is_link,
            link_target: if e.is_link {
                e.link_target.map(|lt| clean_windows_prefix(&lt))
            } else {
                None
            },
        })
        .collect();

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn dir_mtime_ns(path: &str) -> Option<i64> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos() as i64)
}

fn clean_windows_prefix(s: &str) -> String {
    let mut result = s.to_string();
    for prefix in &["\\\\?\\", "\\??\\"] {
        if result.starts_with(prefix) {
            result = result[prefix.len()..].to_string();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_for_compare() {
        assert_eq!(
            normalize_path_for_compare("C:\\Users\\Test"),
            "c:/users/test"
        );
        assert_eq!(
            normalize_path_for_compare("//?/C:/Users/Test"),
            "c:/users/test"
        );
    }

    #[test]
    fn test_path_starts_with() {
        assert!(path_starts_with(
            "/home/user/.skillshub/skill1",
            "/home/user/.skillshub"
        ));
        assert!(!path_starts_with(
            "/home/user/.skillshub-evil/skill1",
            "/home/user/.skillshub"
        ));
    }

    #[test]
    fn test_build_onboarding_plan_structure() {
        let db = Database::new_in_memory().unwrap();
        let plan = build_onboarding_plan(&db, None, None, None);
        // Verify structural correctness regardless of environment
        assert_eq!(plan.groups.len(), plan.total_skills_found);
        for group in &plan.groups {
            assert!(!group.name.is_empty());
            assert!(!group.variants.is_empty());
            for variant in &group.variants {
                assert!(!variant.tool.is_empty());
                assert!(!variant.path.is_empty());
            }
        }
    }
}
