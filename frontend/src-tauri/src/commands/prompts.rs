use std::path::PathBuf;

use sha2::{Digest, Sha256};
use tauri::State;
use uuid::Uuid;

use crate::config::default_tool_adapters;
use crate::db::now_ms;
use crate::error::{AppError, AppResult};
use crate::models::PromptFile;
use crate::repositories::PromptFilesRepository;
use crate::state::AppState;

/// Compute SHA256 hash of a single file's contents.
fn hash_file(path: &std::path::Path) -> Result<String, String> {
    let content =
        std::fs::read(path).map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Resolve home directory.
fn home_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn scan_prompt_files(state: State<'_, AppState>) -> AppResult<Vec<PromptFile>> {
    let adapters = default_tool_adapters();
    let home = home_dir();
    let now = now_ms();
    let repo = PromptFilesRepository::new(&state.db);

    for (tool_key, adapter) in &adapters {
        for spec in &adapter.prompt_files {
            // Handle global scope
            if spec.scope == "global" || spec.scope == "both" {
                if let Some(global_rel) = spec.global_rel {
                    let file_path = home.join(global_rel);
                    let exists = file_path.exists();
                    let content_hash = if exists {
                        hash_file(&file_path).ok()
                    } else {
                        None
                    };

                    let pf = PromptFile {
                        id: Uuid::new_v4().to_string(),
                        tool: tool_key.clone(),
                        scope: "global".to_string(),
                        file_name: spec.file_name.to_string(),
                        file_path: file_path.to_string_lossy().to_string(),
                        content_hash,
                        exists_on_disk: exists,
                        last_scanned_at: now,
                        created_at: now,
                        updated_at: now,
                    };
                    repo.upsert(&pf)?;
                }
            }

            // Handle project scope - we only record the template, actual scanning
            // requires a project path which will be handled by scan_project_prompt_files
            if spec.scope == "project" || spec.scope == "both" {
                // Project-level prompt files are scanned on-demand via scan_project_prompt_files
                // We don't auto-scan projects here since we don't know which projects to scan
            }
        }
    }

    repo.list()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn scan_project_prompt_files(
    state: State<'_, AppState>,
    project_path: String,
) -> AppResult<Vec<PromptFile>> {
    let adapters = default_tool_adapters();
    let now = now_ms();
    let repo = PromptFilesRepository::new(&state.db);
    let proj = PathBuf::from(&project_path);

    for (tool_key, adapter) in &adapters {
        for spec in &adapter.prompt_files {
            if spec.scope != "project" && spec.scope != "both" {
                continue;
            }
            if let Some(project_rel) = spec.project_rel {
                let file_path = proj.join(project_rel);
                let exists = file_path.exists();
                let content_hash = if exists {
                    hash_file(&file_path).ok()
                } else {
                    None
                };

                let pf = PromptFile {
                    id: Uuid::new_v4().to_string(),
                    tool: tool_key.clone(),
                    scope: "project".to_string(),
                    file_name: spec.file_name.to_string(),
                    file_path: file_path.to_string_lossy().to_string(),
                    content_hash,
                    exists_on_disk: exists,
                    last_scanned_at: now,
                    created_at: now,
                    updated_at: now,
                };
                repo.upsert(&pf)?;
            }
        }
    }

    repo.list()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_prompt_files(
    state: State<'_, AppState>,
    tool: Option<String>,
) -> AppResult<Vec<PromptFile>> {
    let repo = PromptFilesRepository::new(&state.db);
    match tool {
        Some(t) => repo.list_by_tool(&t),
        None => repo.list(),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn read_prompt_file(file_path: String) -> AppResult<String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(AppError::NotFound(format!(
            "prompt file not found: {}",
            file_path
        )));
    }
    std::fs::read_to_string(&path)
        .map_err(|e| AppError::FileSystemError(format!("failed to read {}: {}", file_path, e)))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn write_prompt_file(
    state: State<'_, AppState>,
    file_path: String,
    content: String,
) -> AppResult<()> {
    let path = PathBuf::from(&file_path);

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::FileSystemError(format!(
                "failed to create directory {}: {}",
                parent.display(),
                e
            ))
        })?;
    }

    std::fs::write(&path, &content).map_err(|e| {
        AppError::FileSystemError(format!("failed to write {}: {}", file_path, e))
    })?;

    // Update content hash in DB
    let new_hash = hash_file(&path).ok();
    let repo = PromptFilesRepository::new(&state.db);
    // Find the record by file_path and update its hash
    let list = repo.list()?;
    for pf in &list {
        if pf.file_path == file_path {
            if let Some(ref h) = new_hash {
                repo.update_content_hash(&pf.id, h, true)?;
            }
            break;
        }
    }

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_prompt_file(
    state: State<'_, AppState>,
    id: String,
    delete_from_disk: Option<bool>,
) -> AppResult<()> {
    let repo = PromptFilesRepository::new(&state.db);
    let pf = repo
        .get_by_id(&id)?
        .ok_or_else(|| AppError::NotFound(format!("prompt file not found: {}", id)))?;

    if delete_from_disk.unwrap_or(false) {
        let path = PathBuf::from(&pf.file_path);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| {
                AppError::FileSystemError(format!(
                    "failed to delete {}: {}",
                    pf.file_path, e
                ))
            })?;
        }
    }

    repo.delete(&id)
}
