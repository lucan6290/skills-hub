use tauri::State;

use crate::error::{AppError, AppResult};
use crate::repositories::SkillsRepository;
use crate::skills::files::{self, FileEntry};
use crate::state::AppState;

#[tauri::command]
pub async fn list_skill_files(
    state: State<'_, AppState>,
    skill_id: String,
) -> AppResult<Vec<FileEntry>> {
    let repo = SkillsRepository::new(&state.db);
    let skill = repo
        .get_by_id(&skill_id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("skill not found: {}", skill_id)))?;

    files::list_files(&skill.community_path).map_err(|e| AppError::FileSystemError(e))
}

#[tauri::command]
pub async fn read_skill_file(
    state: State<'_, AppState>,
    skill_id: String,
    file_path: String,
) -> AppResult<String> {
    let repo = SkillsRepository::new(&state.db);
    let skill = repo
        .get_by_id(&skill_id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("skill not found: {}", skill_id)))?;

    files::read_file(&skill.community_path, &file_path).map_err(|e| AppError::FileSystemError(e))
}

#[tauri::command]
pub async fn write_skill_file(
    state: State<'_, AppState>,
    skill_id: String,
    file_path: String,
    content: String,
) -> AppResult<()> {
    let repo = SkillsRepository::new(&state.db);
    let skill = repo
        .get_by_id(&skill_id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("skill not found: {}", skill_id)))?;

    files::write_file(&skill.community_path, &file_path, &content)
        .map_err(|e| AppError::FileSystemError(e))
}
