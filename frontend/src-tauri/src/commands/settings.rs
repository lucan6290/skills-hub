use tauri::State;

use crate::contracts::{OkResponse, SetCustomRepoPathResponse, SetRepoPathResponse};
use crate::error::{AppError, AppResult};
use crate::repositories::SettingsRepository;
use crate::state::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_default_sync_tools(state: State<'_, AppState>) -> AppResult<Vec<String>> {
    let repo = SettingsRepository::new(&state.db);
    match repo.get("default_sync_tools") {
        Ok(Some(val)) => {
            let tools: Vec<String> = serde_json::from_str(&val).unwrap_or_default();
            Ok(tools)
        }
        _ => Ok(Vec::new()),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn save_default_sync_tools(
    state: State<'_, AppState>,
    tools: Vec<String>,
) -> AppResult<()> {
    let repo = SettingsRepository::new(&state.db);
    let json = serde_json::to_string(&tools).unwrap_or_else(|_| "[]".to_string());
    repo.set("default_sync_tools", &json)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_auto_check_update(state: State<'_, AppState>) -> AppResult<bool> {
    let repo = SettingsRepository::new(&state.db);
    match repo.get("auto_check_update") {
        Ok(Some(val)) => Ok(val != "false" && val != "0"),
        _ => Ok(true), // default to true
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_auto_check_update(state: State<'_, AppState>, enabled: bool) -> AppResult<()> {
    let repo = SettingsRepository::new(&state.db);
    let val = if enabled { "true" } else { "false" };
    repo.set("auto_check_update", val)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_community_repo_path(state: State<'_, AppState>) -> AppResult<String> {
    let path = crate::repo::community::resolve_community_repo_path(&state.db);
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_community_repo_path(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<SetRepoPathResponse> {
    let p = std::path::Path::new(&path);
    if !p.is_absolute() {
        return Err(AppError::InvalidInput("path must be absolute".into()));
    }

    std::fs::create_dir_all(p)
        .map_err(|e| AppError::FileSystemError(format!("failed to create dir: {}", e)))?;

    let repo = SettingsRepository::new(&state.db);
    repo.set("community_repo_path", &path)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(SetRepoPathResponse { new_path: path })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_custom_repo_path(state: State<'_, AppState>) -> AppResult<String> {
    let path = crate::repo::community::resolve_custom_repo_path(&state.db);
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_custom_repo_path(
    state: State<'_, AppState>,
    path: String,
) -> AppResult<SetCustomRepoPathResponse> {
    let p = std::path::Path::new(&path);
    if !p.is_absolute() {
        return Err(AppError::InvalidInput("path must be absolute".into()));
    }

    std::fs::create_dir_all(p)
        .map_err(|e| AppError::FileSystemError(format!("failed to create dir: {}", e)))?;

    let is_empty = std::fs::read_dir(p)
        .map(|mut d| d.next().is_none())
        .unwrap_or(true);

    let repo = SettingsRepository::new(&state.db);
    repo.set("custom_repo_path", &path)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(SetCustomRepoPathResponse {
        ok: true,
        path,
        empty: Some(is_empty),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn open_settings_folder(path: Option<String>) -> AppResult<OkResponse> {
    let folder = path.unwrap_or_else(|| {
        crate::config::resolve_data_dir()
            .to_string_lossy()
            .to_string()
    });

    let p = std::path::Path::new(&folder);
    if !p.exists() {
        std::fs::create_dir_all(p)
            .map_err(|e| AppError::FileSystemError(format!("failed to create dir: {}", e)))?;
    }

    crate::filesystem::open_folder(p).map_err(|e| AppError::FileSystemError(e))?;

    Ok(OkResponse {
        ok: true,
        message: "opened".to_string(),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_proxy_url(state: State<'_, AppState>) -> AppResult<String> {
    let repo = SettingsRepository::new(&state.db);
    match repo.get("proxy_url") {
        Ok(Some(val)) => Ok(val),
        _ => Ok(String::new()),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_proxy_url(state: State<'_, AppState>, url: String) -> AppResult<()> {
    let repo = SettingsRepository::new(&state.db);
    if url.is_empty() {
        let _ = repo.delete("proxy_url");
    } else {
        repo.set("proxy_url", &url)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_close_behavior(state: State<'_, AppState>) -> AppResult<String> {
    let repo = SettingsRepository::new(&state.db);
    match repo.get("close_behavior") {
        Ok(Some(val)) => Ok(val),
        _ => Ok("minimize_to_tray".to_string()),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_close_behavior(state: State<'_, AppState>, behavior: String) -> AppResult<()> {
    if behavior != "minimize_to_tray" && behavior != "quit" {
        return Err(AppError::InvalidInput(
            "behavior must be 'minimize_to_tray' or 'quit'".into(),
        ));
    }
    let repo = SettingsRepository::new(&state.db);
    repo.set("close_behavior", &behavior)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_show_tray_icon(state: State<'_, AppState>) -> AppResult<bool> {
    let repo = SettingsRepository::new(&state.db);
    match repo.get("show_tray_icon") {
        Ok(Some(val)) => Ok(val != "false" && val != "0"),
        _ => Ok(true),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_show_tray_icon(state: State<'_, AppState>, enabled: bool) -> AppResult<()> {
    let repo = SettingsRepository::new(&state.db);
    let val = if enabled { "true" } else { "false" };
    repo.set("show_tray_icon", val)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_log_level(state: State<'_, AppState>) -> AppResult<String> {
    let repo = SettingsRepository::new(&state.db);
    match repo.get("log_level") {
        Ok(Some(val)) => Ok(val),
        _ => Ok("info".to_string()),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_log_level(state: State<'_, AppState>, level: String) -> AppResult<()> {
    if level != "debug" && level != "info" && level != "warn" && level != "error" {
        return Err(AppError::InvalidInput(
            "level must be one of 'debug', 'info', 'warn', 'error'".into(),
        ));
    }
    let repo = SettingsRepository::new(&state.db);
    repo.set("log_level", &level)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_auto_refresh_on_startup(state: State<'_, AppState>) -> AppResult<bool> {
    let repo = SettingsRepository::new(&state.db);
    match repo.get("auto_refresh_on_startup") {
        Ok(Some(val)) => Ok(val != "false" && val != "0"),
        _ => Ok(false),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_auto_refresh_on_startup(
    state: State<'_, AppState>,
    enabled: bool,
) -> AppResult<()> {
    let repo = SettingsRepository::new(&state.db);
    let val = if enabled { "true" } else { "false" };
    repo.set("auto_refresh_on_startup", val)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_general_settings(state: State<'_, AppState>) -> AppResult<OkResponse> {
    let repo = SettingsRepository::new(&state.db);
    let keys_to_reset = [
        "community_repo_path",
        "custom_repo_path",
        "default_sync_tools",
        "auto_check_update",
        "proxy_url",
        "close_behavior",
        "show_tray_icon",
        "log_level",
        "auto_refresh_on_startup",
    ];
    for key in &keys_to_reset {
        let _ = repo.delete(key);
    }

    Ok(OkResponse {
        ok: true,
        message: "settings reset to defaults".to_string(),
    })
}
