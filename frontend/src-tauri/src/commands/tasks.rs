use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::tasks::TaskRecord;

#[tauri::command]
pub async fn get_task_list(state: State<'_, AppState>) -> AppResult<Vec<TaskRecord>> {
    Ok(state.task_manager.list())
}

#[tauri::command]
pub async fn get_task(state: State<'_, AppState>, task_id: String) -> AppResult<TaskRecord> {
    state
        .task_manager
        .get(&task_id)
        .ok_or_else(|| AppError::NotFound(format!("task not found: {}", task_id)))
}

#[tauri::command]
pub async fn cancel_task(state: State<'_, AppState>, task_id: String) -> AppResult<bool> {
    Ok(state.task_manager.cancel(&task_id))
}
