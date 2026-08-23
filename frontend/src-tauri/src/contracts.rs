use serde::{Deserialize, Serialize};

use crate::models::{Skill, SkillTarget, SkillUsage, Tag};

// ── Managed Skill DTO ───────────────────────────────────

/// 受管技能 DTO：在前端 `ManagedSkill` 类型基础上补充 tags / targets / usage / is_suite。
/// 通过 `#[serde(flatten)]` 平铺 `Skill` 的全部字段。
#[derive(Debug, Clone, Serialize)]
pub struct ManagedSkillDto {
    #[serde(flatten)]
    pub skill: Skill,
    pub tags: Vec<Tag>,
    pub targets: Vec<SkillTarget>,
    pub usage: Vec<SkillUsage>,
    pub is_suite: bool,
}

// ── Health ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
}

// ── Reorder ─────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct ReorderItem {
    pub id: String,
    pub sort_order: f64,
}

// ── Tool Status ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ToolStatusInfo {
    pub key: String,
    pub label: String,
    pub installed: bool,
    pub skills_dir: String,
    pub supports_project_scope: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolStatusDto {
    pub tools: Vec<ToolStatusInfo>,
    pub installed: Vec<String>,
    pub newly_installed: Vec<String>,
}

// ── Database ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct DbTableInfo {
    pub table_name: String,
    pub display_name: String,
    pub row_count: i64,
    pub size_bytes: i64,
    pub size_human: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DbOverview {
    pub db_path: String,
    pub file_size: i64,
    pub file_size_human: String,
    pub last_modified: i64,
    pub sqlite_version: String,
    pub page_size: i64,
    pub page_count: i64,
    pub freelist_count: i64,
    pub free_size: i64,
    pub free_size_human: String,
    pub fragmentation_pct: f64,
    pub tables: Vec<DbTableInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DbColumnInfo {
    pub cid: i32,
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
    pub notnull: bool,
    pub default: Option<String>,
    pub pk: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DbTableData {
    pub table: String,
    pub display_name: String,
    pub columns: Vec<DbColumnInfo>,
    pub rows: Vec<serde_json::Value>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DbMaintenanceResult {
    pub ok: bool,
    pub action: String,
    pub message: String,
    pub integrity_result: Option<String>,
}

// ── Pick Folder ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct PickFolderResult {
    pub path: Option<String>,
}

// ── Generic OK responses ────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct OkResponse {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OkPathResponse {
    pub ok: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OkRemovedResponse {
    pub ok: bool,
    pub removed: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OkNameResponse {
    pub ok: bool,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetRepoPathResponse {
    pub new_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetCustomRepoPathResponse {
    pub ok: bool,
    pub path: String,
    pub empty: Option<bool>,
}
