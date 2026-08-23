use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub sort_order: f64,
}

impl Default for Tag {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            sort_order: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagWithCount {
    pub id: i64,
    pub name: String,
    pub skill_count: i64,
    pub updated_at: i64,
    pub sort_order: f64,
}

impl Default for TagWithCount {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            skill_count: 0,
            updated_at: 0,
            sort_order: 0.0,
        }
    }
}
