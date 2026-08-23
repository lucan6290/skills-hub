use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub id: i64,
    pub project_path: String,
    pub last_used_at: i64,
}

impl Default for RecentProject {
    fn default() -> Self {
        Self {
            id: 0,
            project_path: String::new(),
            last_used_at: 0,
        }
    }
}
