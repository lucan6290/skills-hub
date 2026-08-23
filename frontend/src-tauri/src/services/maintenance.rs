//! Maintenance service — mirrors `backend/core/skills/maintenance.py`.
//!
//! Provides sync health scanning, repair operations, and cache cleanup.

use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

use crate::db::{now_ms, Database};
use crate::models::Skill;
use crate::repo::community::{resolve_community_repo_path, resolve_custom_repo_path};
use crate::repo::scanner::is_skill_dir;
use crate::utils::content_hash;
use crate::utils::path_safety;

/// A single issue found during health scan.
#[derive(Debug, Clone, Serialize)]
pub struct HealthIssue {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub repair_action: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Summary of issues by severity.
#[derive(Debug, Clone, Serialize)]
pub struct IssueSummary {
    pub error: usize,
    pub warning: usize,
    pub info: usize,
    pub repairable: usize,
}

/// Full sync health report.
#[derive(Debug, Clone, Serialize)]
pub struct SyncHealthReport {
    pub community_repo: String,
    pub skills_checked: usize,
    pub issues: Vec<HealthIssue>,
    pub summary: IssueSummary,
    pub generated_at: i64,
}

/// Result of a repair operation.
#[derive(Debug, Clone, Serialize)]
pub struct RepairReport {
    pub dry_run: bool,
    pub operations: Vec<RepairOperation>,
    pub operation_count: usize,
    pub before: SyncHealthReport,
    pub after: Option<SyncHealthReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepairOperation {
    pub action: String,
    #[serde(flatten)]
    pub details: HashMap<String, serde_json::Value>,
}

fn issue(
    code: &str,
    severity: &str,
    message: &str,
    repair: Option<&str>,
) -> (HealthIssue, HashMap<String, serde_json::Value>) {
    (
        HealthIssue {
            code: code.to_string(),
            severity: severity.to_string(),
            message: message.to_string(),
            repair_action: repair.map(|r| r.to_string()),
            extra: HashMap::new(),
        },
        HashMap::new(),
    )
}

fn add_extra(issue: &mut HealthIssue, key: &str, value: serde_json::Value) {
    issue.extra.insert(key.to_string(), value);
}

fn summarize_issues(issues: &[HealthIssue]) -> IssueSummary {
    let mut summary = IssueSummary {
        error: 0,
        warning: 0,
        info: 0,
        repairable: 0,
    };
    for issue in issues {
        match issue.severity.as_str() {
            "error" => summary.error += 1,
            "warning" => summary.warning += 1,
            _ => summary.info += 1,
        }
        if issue.repair_action.is_some() {
            summary.repairable += 1;
        }
    }
    summary
}

/// Scan sync health without mutating state.
pub fn scan_sync_health(db: &Database) -> SyncHealthReport {
    use crate::repositories::{SkillTargetsRepository, SkillsRepository};

    let community_base = resolve_community_repo_path(db);
    let _custom_base = resolve_custom_repo_path(db);
    let skills_repo = SkillsRepository::new(db);
    let targets_repo = SkillTargetsRepository::new(db);

    let skills = skills_repo.list("manual").unwrap_or_default();
    let mut issues: Vec<HealthIssue> = Vec::new();
    let mut target_paths: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut community_paths: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut custom_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

    for skill in &skills {
        let source_type = crate::repo::scanner::normalize_source_type(&skill.source_type);
        let source_label = if source_type == "custom" {
            "自制源目录"
        } else {
            "中央仓库"
        };

        // Check source path validity
        let source_path = resolve_skill_source_path(skill, db);
        let source_path = match source_path {
            Ok(p) => p,
            Err(_) => {
                let (mut iss, _) = issue(
                    "source_path_outside_repo",
                    "error",
                    &format!(
                        "{}路径超出允许范围 / source path escapes repo",
                        source_label
                    ),
                    None,
                );
                add_extra(&mut iss, "skill_id", serde_json::json!(skill.id));
                add_extra(&mut iss, "skill_name", serde_json::json!(skill.name));
                issues.push(iss);
                continue;
            }
        };

        if source_type == "community" {
            community_paths.insert(path_safety::norm_path(&source_path));
        } else {
            custom_paths.insert(path_safety::norm_path(&source_path));
        }

        // Check source directory exists
        if !source_path.is_dir() {
            let missing_code = if source_type == "custom" {
                "missing_source_dir"
            } else {
                "missing_community_dir"
            };
            let (mut iss, _) = issue(
                missing_code,
                "error",
                &format!(
                    "{}中的 Skill 目录丢失 / source skill directory is missing",
                    source_label
                ),
                Some("mark_skill_missing"),
            );
            add_extra(&mut iss, "skill_id", serde_json::json!(skill.id));
            add_extra(&mut iss, "skill_name", serde_json::json!(skill.name));
            add_extra(
                &mut iss,
                "community_path",
                serde_json::json!(source_path.to_string_lossy()),
            );
            issues.push(iss);
            continue;
        }

        // Check content hash drift
        if let Ok(source_hash) = content_hash::hash_dir(&source_path) {
            if let Some(ref stored_hash) = skill.content_hash {
                if stored_hash != &source_hash {
                    let (mut iss, _) = issue(
                        "source_hash_drift",
                        "info",
                        &format!(
                            "{}内容与数据库哈希不一致 / source content hash differs from DB",
                            source_label
                        ),
                        Some("update_content_hash"),
                    );
                    add_extra(&mut iss, "skill_id", serde_json::json!(skill.id));
                    add_extra(&mut iss, "stored_hash", serde_json::json!(stored_hash));
                    add_extra(&mut iss, "actual_hash", serde_json::json!(source_hash));
                    issues.push(iss);
                }
            }
        }

        // Check targets
        let targets = targets_repo.list_by_skill(&skill.id).unwrap_or_default();
        for target in &targets {
            target_paths.insert(path_safety::norm_path(&target.target_path));

            let target_path = Path::new(&target.target_path);
            let exists = target_path.exists()
                || target_path.is_symlink()
                || crate::platform::is_junction(target_path);

            if !exists {
                let (mut iss, _) = issue(
                    "missing_target_path",
                    "warning",
                    "数据库记录存在，但工具目录中的目标已丢失 / target path is missing",
                    Some("resync_target"),
                );
                add_extra(&mut iss, "skill_id", serde_json::json!(skill.id));
                add_extra(&mut iss, "tool", serde_json::json!(target.tool));
                add_extra(
                    &mut iss,
                    "target_path",
                    serde_json::json!(target.target_path),
                );
                issues.push(iss);
            }
        }
    }

    // Check for orphan dirs in community repo
    if community_base.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&community_base) {
            let mut items: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            items.sort_by_key(|e| e.file_name());
            for entry in items {
                let item = entry.path();
                if !item.is_dir() {
                    continue;
                }
                if entry.file_name() == ".snapshots" {
                    continue;
                }
                if !is_skill_dir(&item) {
                    continue;
                }
                let norm = path_safety::norm_path(&item);
                if !community_paths.contains(&norm) {
                    let (mut iss, _) = issue(
                        "community_orphan_dir",
                        "info",
                        "Community Repo 中存在未登记的 Skill / community skill is not registered",
                        Some("register_community_skill"),
                    );
                    add_extra(
                        &mut iss,
                        "community_path",
                        serde_json::json!(item.to_string_lossy()),
                    );
                    add_extra(
                        &mut iss,
                        "skill_name",
                        serde_json::json!(entry.file_name().to_string_lossy()),
                    );
                    issues.push(iss);
                }
            }
        }
    }

    // Check for duplicate content hashes
    let mut hash_groups: HashMap<String, Vec<&Skill>> = HashMap::new();
    for skill in &skills {
        if let Some(ref h) = skill.content_hash {
            hash_groups.entry(h.clone()).or_default().push(skill);
        }
    }
    for (hash, group) in &hash_groups {
        if group.len() > 1 {
            let (mut iss, _) = issue(
                "duplicate_content_hash",
                "info",
                "多个 Skill 内容完全相同 / duplicate skill content",
                None,
            );
            add_extra(
                &mut iss,
                "skill_ids",
                serde_json::json!(group.iter().map(|s| &s.id).collect::<Vec<_>>()),
            );
            add_extra(&mut iss, "content_hash", serde_json::json!(hash));
            issues.push(iss);
        }
    }

    let summary = summarize_issues(&issues);

    SyncHealthReport {
        community_repo: community_base.to_string_lossy().to_string(),
        skills_checked: skills.len(),
        issues,
        summary,
        generated_at: now_ms(),
    }
}

/// Run conservative repairs. Destructive deletes limited to DB records or managed targets.
pub fn repair_sync_health(db: &Database, dry_run: bool) -> RepairReport {
    use crate::repositories::SkillsRepository;

    let report = scan_sync_health(db);
    let mut operations: Vec<RepairOperation> = Vec::new();

    for issue_item in &report.issues {
        let action = match issue_item.repair_action.as_deref() {
            Some(a) => a,
            None => continue,
        };

        match action {
            "mark_skill_missing" => {
                let skill_id = issue_item
                    .extra
                    .get("skill_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                operations.push(RepairOperation {
                    action: action.to_string(),
                    details: [("skill_id".to_string(), serde_json::json!(skill_id))]
                        .into_iter()
                        .collect(),
                });
                if !dry_run && !skill_id.is_empty() {
                    let repo = SkillsRepository::new(db);
                    if let Ok(Some(mut skill)) = repo.get_by_id(skill_id) {
                        skill.status = "missing".to_string();
                        skill.updated_at = now_ms();
                        let _ = repo.upsert(&mut skill);
                    }
                }
            }
            "update_content_hash" => {
                let skill_id = issue_item
                    .extra
                    .get("skill_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let actual_hash = issue_item
                    .extra
                    .get("actual_hash")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                operations.push(RepairOperation {
                    action: action.to_string(),
                    details: [("skill_id".to_string(), serde_json::json!(skill_id))]
                        .into_iter()
                        .collect(),
                });
                if !dry_run && !skill_id.is_empty() {
                    let repo = SkillsRepository::new(db);
                    if let Ok(Some(mut skill)) = repo.get_by_id(skill_id) {
                        skill.content_hash = Some(actual_hash.to_string());
                        skill.updated_at = now_ms();
                        let _ = repo.upsert(&mut skill);
                    }
                }
            }
            _ => {
                operations.push(RepairOperation {
                    action: action.to_string(),
                    details: issue_item.extra.clone(),
                });
            }
        }
    }

    let after = if !dry_run {
        Some(scan_sync_health(db))
    } else {
        None
    };

    RepairReport {
        dry_run,
        operation_count: operations.len(),
        operations,
        before: report,
        after,
    }
}

/// Resolve the source path for a skill record.
fn resolve_skill_source_path(skill: &Skill, db: &Database) -> Result<std::path::PathBuf, String> {
    let community_path = &skill.community_path;
    if community_path.is_empty() {
        return Err("empty community path".to_string());
    }

    let path = std::path::PathBuf::from(community_path);
    let source_type = crate::repo::scanner::normalize_source_type(&skill.source_type);

    let base = if source_type == "custom" {
        resolve_custom_repo_path(db)
    } else {
        resolve_community_repo_path(db)
    };

    // Custom source type allows paths outside the repo
    if source_type == "custom" {
        return Ok(path);
    }

    if path_safety::is_path_within(&path, &base) {
        Ok(path)
    } else {
        Err(format!("source path escapes repo: {}", community_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_sync_health_empty_db() {
        let db = Database::new_in_memory().unwrap();
        let report = scan_sync_health(&db);
        // No manually installed skills in a fresh DB
        assert_eq!(report.skills_checked, 0);
        // There should be no error-level issues for an empty DB
        assert!(
            report.issues.iter().all(|i| i.severity != "error"),
            "empty DB should not produce error-level issues"
        );
    }

    #[test]
    fn test_summarize_issues() {
        let issues = vec![
            HealthIssue {
                code: "a".to_string(),
                severity: "error".to_string(),
                message: "err".to_string(),
                repair_action: Some("fix".to_string()),
                extra: HashMap::new(),
            },
            HealthIssue {
                code: "b".to_string(),
                severity: "warning".to_string(),
                message: "warn".to_string(),
                repair_action: None,
                extra: HashMap::new(),
            },
            HealthIssue {
                code: "c".to_string(),
                severity: "info".to_string(),
                message: "info".to_string(),
                repair_action: Some("register".to_string()),
                extra: HashMap::new(),
            },
        ];
        let summary = summarize_issues(&issues);
        assert_eq!(summary.error, 1);
        assert_eq!(summary.warning, 1);
        assert_eq!(summary.info, 1);
        assert_eq!(summary.repairable, 2);
    }

    #[test]
    fn test_repair_sync_health_dry_run() {
        let db = Database::new_in_memory().unwrap();
        let report = repair_sync_health(&db, true);
        assert!(report.dry_run);
        assert!(report.after.is_none());
    }
}
