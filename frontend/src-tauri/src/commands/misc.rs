use tauri::State;

use crate::contracts::{PickFolderResult, ReorderItem};
use crate::error::{AppError, AppResult};
use crate::repositories::ToolAdapterConfigsRepository;
use crate::state::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn pick_folder() -> AppResult<PickFolderResult> {
    // Tauri dialog plugin is not available; return None to trigger fallback in frontend
    // TODO: Integrate tauri-plugin-dialog for native folder picker
    Ok(PickFolderResult { path: None })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn cancel_current_operation(state: State<'_, AppState>) -> AppResult<()> {
    state.task_manager.cancel_all_running();
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn reorder(
    state: State<'_, AppState>,
    entity: String,
    items: Vec<ReorderItem>,
) -> AppResult<()> {
    match entity.as_str() {
        "skills" => {
            let pairs: Vec<(String, f64)> = items
                .iter()
                .map(|item| (item.id.clone(), item.sort_order))
                .collect();
            state
                .db
                .with_conn(|conn| {
                    for (id, sort_order) in &pairs {
                        conn.execute(
                            "UPDATE skills SET sort_order = ?1 WHERE id = ?2",
                            rusqlite::params![sort_order, id],
                        )?;
                    }
                    Ok::<_, rusqlite::Error>(())
                })
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }
        "tags" => {
            let pairs: Vec<(i64, f64)> = items
                .iter()
                .filter_map(|item| item.id.parse::<i64>().ok().map(|id| (id, item.sort_order)))
                .collect();
            state
                .db
                .with_conn(|conn| {
                    for (id, sort_order) in &pairs {
                        conn.execute(
                            "UPDATE skill_tags SET sort_order = ?1 WHERE id = ?2",
                            rusqlite::params![sort_order, id],
                        )?;
                    }
                    Ok::<_, rusqlite::Error>(())
                })
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }
        "tools" => {
            let pairs: Vec<(String, f64)> = items
                .iter()
                .map(|item| (item.id.clone(), item.sort_order))
                .collect();
            let repo = ToolAdapterConfigsRepository::new(&state.db);
            repo.reorder(&pairs)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }
        _ => {
            return Err(AppError::InvalidInput(format!(
                "unknown entity type: {}",
                entity
            )));
        }
    }
    Ok(())
}
