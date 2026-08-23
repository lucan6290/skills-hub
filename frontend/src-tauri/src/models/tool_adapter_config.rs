use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAdapterConfig {
    pub tool_key: String,
    pub display_name: String,
    pub skills_dir: String,
    pub detect_dir: String,
    pub project_skills_dir: Option<String>,
    pub supports_symlink: bool,
    pub supports_junction: bool,
    pub force_copy: bool,
    pub supports_project_scope: Option<bool>,
    pub is_custom: bool,
    pub enabled: bool,
    pub sort_order: f64,
    pub updated_at: i64,
}

impl Default for ToolAdapterConfig {
    fn default() -> Self {
        Self {
            tool_key: String::new(),
            display_name: String::new(),
            skills_dir: String::new(),
            detect_dir: String::new(),
            project_skills_dir: None,
            supports_symlink: true,
            supports_junction: true,
            force_copy: false,
            supports_project_scope: None,
            is_custom: false,
            enabled: true,
            sort_order: 0.0,
            updated_at: 0,
        }
    }
}
