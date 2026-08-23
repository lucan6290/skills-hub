use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopePreference {
    pub skill_id: String,
    pub scope: String,
    pub project_paths: String,
    pub updated_at: i64,
}

impl Default for ScopePreference {
    fn default() -> Self {
        Self {
            skill_id: String::new(),
            scope: "global".to_string(),
            project_paths: "[]".to_string(),
            updated_at: 0,
        }
    }
}
