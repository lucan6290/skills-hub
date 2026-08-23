use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub frontmatter_extra: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub category: Option<String>,
    pub homepage: Option<String>,
    pub skill_file_count: Option<i64>,
    pub skill_dir_size: Option<i64>,
    pub source_type: String,
    pub source_ref: Option<String>,
    pub source_subpath: Option<String>,
    pub source_revision: Option<String>,
    pub source_url: Option<String>,
    pub community_path: String,
    pub content_hash: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_sync_at: Option<i64>,
    pub last_seen_at: i64,
    pub status: String,
    pub sort_order: f64,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: None,
            frontmatter_extra: None,
            version: None,
            author: None,
            license: None,
            category: None,
            homepage: None,
            skill_file_count: None,
            skill_dir_size: None,
            source_type: "community".to_string(),
            source_ref: None,
            source_subpath: None,
            source_revision: None,
            source_url: None,
            community_path: String::new(),
            content_hash: None,
            created_at: 0,
            updated_at: 0,
            last_sync_at: None,
            last_seen_at: 0,
            status: "active".to_string(),
            sort_order: 0.0,
        }
    }
}
