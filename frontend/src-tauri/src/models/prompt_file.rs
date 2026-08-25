use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PromptFile {
    pub id: String,
    pub tool: String,
    pub scope: String,
    pub file_name: String,
    pub file_path: String,
    pub content_hash: Option<String>,
    pub exists_on_disk: bool,
    pub last_scanned_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}
