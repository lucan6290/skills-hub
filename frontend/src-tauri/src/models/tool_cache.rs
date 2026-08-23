use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolScanState {
    pub tool_key: String,
    pub tool_name: String,
    pub installed: bool,
    pub skills_dir: Option<String>,
    pub supports_project_scope: bool,
    pub dir_mtime_ns: Option<i64>,
    pub scanned_at: i64,
    pub first_seen_at: Option<i64>,
}

impl Default for ToolScanState {
    fn default() -> Self {
        Self {
            tool_key: String::new(),
            tool_name: String::new(),
            installed: false,
            skills_dir: None,
            supports_project_scope: true,
            dir_mtime_ns: None,
            scanned_at: 0,
            first_seen_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSkillCache {
    pub tool_key: String,
    pub name: String,
    pub path: String,
    pub is_link: bool,
    pub link_target: Option<String>,
    pub description: Option<String>,
    pub in_community_repo: bool,
    pub skill_mtime_ns: Option<i64>,
    pub scanned_at: i64,
}

impl Default for ToolSkillCache {
    fn default() -> Self {
        Self {
            tool_key: String::new(),
            name: String::new(),
            path: String::new(),
            is_link: false,
            link_target: None,
            description: None,
            in_community_repo: false,
            skill_mtime_ns: None,
            scanned_at: 0,
        }
    }
}
