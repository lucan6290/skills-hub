use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUsage {
    pub id: i64,
    pub skill_id: String,
    pub tool: String,
    pub sync_count: i64,
    pub last_synced_at: Option<i64>,
    pub last_viewed_at: Option<i64>,
    pub view_count: i64,
}

impl Default for SkillUsage {
    fn default() -> Self {
        Self {
            id: 0,
            skill_id: String::new(),
            tool: String::new(),
            sync_count: 0,
            last_synced_at: None,
            last_viewed_at: None,
            view_count: 0,
        }
    }
}
