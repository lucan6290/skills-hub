use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTarget {
    pub id: String,
    pub skill_id: String,
    pub tool: String,
    pub scope: String,
    pub project_path: Option<String>,
    pub target_path: String,
    pub mode: String,
    pub status: String,
    pub last_error: Option<String>,
    pub synced_at: Option<i64>,
    pub target_content_hash: Option<String>,
    pub target_updated_at: Option<i64>,
    pub suite_skill_id: Option<String>,
}

impl Default for SkillTarget {
    fn default() -> Self {
        Self {
            id: String::new(),
            skill_id: String::new(),
            tool: String::new(),
            scope: "global".to_string(),
            project_path: None,
            target_path: String::new(),
            mode: String::new(),
            status: String::new(),
            last_error: None,
            synced_at: None,
            target_content_hash: None,
            target_updated_at: None,
            suite_skill_id: None,
        }
    }
}
