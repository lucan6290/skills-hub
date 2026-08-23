use tauri::State;

use crate::error::{AppError, AppResult};
use crate::models::{Tag, TagWithCount};
use crate::repositories::TagsRepository;
use crate::state::AppState;

#[tauri::command]
pub async fn get_tags(
    state: State<'_, AppState>,
    source_type: Option<String>,
    sort: Option<String>,
) -> AppResult<Vec<TagWithCount>> {
    let sort = sort.unwrap_or_else(|| "name".to_string());
    let repo = TagsRepository::new(&state.db);
    repo.list_with_counts(source_type.as_deref(), &sort)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command]
pub async fn create_tag(state: State<'_, AppState>, name: String) -> AppResult<()> {
    let repo = TagsRepository::new(&state.db);
    repo.create(&name)
        .map_err(|e| AppError::InvalidInput(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn rename_tag(
    state: State<'_, AppState>,
    tag_id: i64,
    name: String,
) -> AppResult<serde_json::Value> {
    let repo = TagsRepository::new(&state.db);
    let tag = repo
        .rename(tag_id, &name)
        .map_err(|e| AppError::InvalidInput(e.to_string()))?;
    Ok(serde_json::json!({ "id": tag.id, "name": tag.name }))
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, AppState>, tag_id: i64) -> AppResult<()> {
    let repo = TagsRepository::new(&state.db);
    repo.delete(tag_id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command]
pub async fn get_skill_tags(state: State<'_, AppState>, skill_id: String) -> AppResult<Vec<Tag>> {
    let repo = TagsRepository::new(&state.db);
    repo.get_skill_tags(&skill_id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command]
pub async fn set_skill_tags(
    state: State<'_, AppState>,
    skill_id: String,
    tag_ids: Vec<i64>,
) -> AppResult<()> {
    let repo = TagsRepository::new(&state.db);
    repo.set_skill_tags(&skill_id, &tag_ids)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}
