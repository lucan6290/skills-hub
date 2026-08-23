use tauri::State;

use crate::contracts::{OkNameResponse, OkPathResponse, OkRemovedResponse};
use crate::db::now_ms;
use crate::error::{AppError, AppResult};
use crate::models::ToolAdapterConfig;
use crate::repositories::ToolAdapterConfigsRepository;
use crate::state::AppState;
use crate::tools::adapter::{self, effective_tool_adapters, resolve_default_path};
use crate::tools::skill_cache::{self, ToolSkillsResponse};

#[tauri::command(rename_all = "snake_case")]
pub async fn get_tool_status(
    state: State<'_, AppState>,
) -> AppResult<crate::contracts::ToolStatusDto> {
    let adapters = effective_tool_adapters(&state.db);
    let mut tools = Vec::new();
    let mut installed = Vec::new();
    let mut newly_installed = Vec::new();

    for adapter in &adapters {
        let is_installed = adapter::is_tool_installed(adapter);
        let skills_dir = if is_installed {
            resolve_default_path(adapter)
        } else {
            String::new()
        };

        if is_installed {
            installed.push(adapter.tool_key.clone());

            // Check if first time seen
            use crate::repositories::ToolCacheRepository;
            let cache_repo = ToolCacheRepository::new(&state.db);
            if let Ok(Some(first_seen)) = cache_repo.mark_tool_first_seen(&adapter.tool_key) {
                // Only mark as newly installed if first_seen was just set (was None before)
                if first_seen > 0 {
                    newly_installed.push(adapter.tool_key.clone());
                }
            }
        }

        tools.push(crate::contracts::ToolStatusInfo {
            key: adapter.tool_key.clone(),
            label: adapter.display_name.clone(),
            installed: is_installed,
            skills_dir,
            supports_project_scope: adapter::supports_project_scope(adapter),
        });
    }

    Ok(crate::contracts::ToolStatusDto {
        tools,
        installed,
        newly_installed,
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_tool_skills(
    state: State<'_, AppState>,
    refresh: Option<bool>,
) -> AppResult<Vec<ToolSkillsResponse>> {
    let do_refresh = refresh.unwrap_or(false);
    let adapters = effective_tool_adapters(&state.db);
    let mut results = Vec::new();

    for adapter in &adapters {
        let is_installed = adapter::is_tool_installed(adapter);
        let skills_dir = if is_installed {
            Some(resolve_default_path(adapter))
        } else {
            None
        };

        let response = if do_refresh {
            skill_cache::refresh_tool_cache(&state.db, adapter, is_installed, skills_dir.as_deref())
                .unwrap_or_else(|_| skill_cache::cached_tool_response(&state.db, adapter))
        } else {
            skill_cache::cached_tool_response(&state.db, adapter)
        };

        results.push(response);
    }

    Ok(results)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_tool_adapter_configs(
    state: State<'_, AppState>,
) -> AppResult<Vec<serde_json::Value>> {
    let repo = ToolAdapterConfigsRepository::new(&state.db);
    let configs = repo
        .list_enabled()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let defaults = crate::config::default_tool_adapters();

    let result: Vec<serde_json::Value> = configs
        .iter()
        .map(|c| {
            let default_cfg = defaults.get(&c.tool_key);
            let has_override = default_cfg
                .map(|d| d.skills_dir != c.skills_dir || d.detect_dir != c.detect_dir)
                .unwrap_or(false);

            serde_json::json!({
                "tool_key": c.tool_key,
                "display_name": c.display_name,
                "skills_dir": c.skills_dir,
                "detect_dir": c.detect_dir,
                "project_skills_dir": c.project_skills_dir,
                "default_skills_dir": default_cfg.map(|d| &d.skills_dir),
                "default_detect_dir": default_cfg.map(|d| &d.detect_dir),
                "supports_symlink": c.supports_symlink,
                "supports_junction": c.supports_junction,
                "force_copy": c.force_copy,
                "supports_project_scope": c.supports_project_scope.unwrap_or(true),
                "is_custom": c.is_custom,
                "has_override": has_override,
                "sort_order": c.sort_order,
            })
        })
        .collect();

    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn save_tool_adapter_config(
    state: State<'_, AppState>,
    tool_key: String,
    display_name: String,
    skills_dir: String,
    detect_dir: String,
    project_skills_dir: Option<String>,
    supports_symlink: Option<bool>,
    supports_junction: Option<bool>,
    force_copy: Option<bool>,
    supports_project_scope: Option<bool>,
    is_custom: Option<bool>,
) -> AppResult<()> {
    let now = now_ms();
    let mut config = ToolAdapterConfig {
        tool_key,
        display_name,
        skills_dir,
        detect_dir,
        project_skills_dir,
        supports_symlink: supports_symlink.unwrap_or(true),
        supports_junction: supports_junction.unwrap_or(true),
        force_copy: force_copy.unwrap_or(false),
        supports_project_scope,
        is_custom: is_custom.unwrap_or(false),
        enabled: true,
        sort_order: 0.0,
        updated_at: now,
    };

    let repo = ToolAdapterConfigsRepository::new(&state.db);
    repo.upsert(&mut config)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_tool_adapter_config(
    state: State<'_, AppState>,
    tool_key: String,
) -> AppResult<()> {
    let repo = ToolAdapterConfigsRepository::new(&state.db);
    // Check if it's a custom tool — delete instead of reset
    let configs = repo
        .list_enabled()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let is_custom = configs
        .iter()
        .any(|c| c.tool_key == tool_key && c.is_custom);

    if is_custom {
        repo.delete(&tool_key)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    } else {
        repo.reset_to_default(&tool_key)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn delete_tool_skill(tool_key: String, skill_path: String) -> AppResult<()> {
    let _ = tool_key; // tool_key used for context but deletion is path-based
    crate::filesystem::remove_link_or_directory(&skill_path)
        .map_err(|e| AppError::FileSystemError(e))
}

#[tauri::command(rename_all = "snake_case")]
pub async fn open_tool_skills_dir(
    state: State<'_, AppState>,
    tool_key: String,
) -> AppResult<OkPathResponse> {
    let adapters = effective_tool_adapters(&state.db);
    let adapter = adapter::adapter_by_key(&adapters, &tool_key)
        .ok_or_else(|| AppError::InvalidInput(format!("unknown tool: {}", tool_key)))?;

    let dir = resolve_default_path(adapter);
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::FileSystemError(format!("failed to create dir: {}", e)))?;

    crate::filesystem::open_folder(&dir).map_err(|e| AppError::FileSystemError(e))?;

    Ok(OkPathResponse {
        ok: true,
        path: dir,
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn skill_to_community_repo(
    state: State<'_, AppState>,
    source_path: String,
    name: String,
) -> AppResult<OkNameResponse> {
    let community_base = crate::repo::community::resolve_community_repo_path(&state.db);
    std::fs::create_dir_all(&community_base)
        .map_err(|e| AppError::FileSystemError(format!("failed to create repo dir: {}", e)))?;

    let dir_name = crate::utils::path_safety::safe_dir_name(Some(&name));
    let target =
        crate::utils::path_safety::safe_child_path(&community_base, &dir_name, "skill name")
            .map_err(|e| AppError::PathError(e))?;

    if target.exists() {
        return Err(AppError::InvalidInput(format!(
            "skill directory already exists: {}",
            target.display()
        )));
    }

    crate::filesystem::copy_directory(&source_path, &target)
        .map_err(|e| AppError::FileSystemError(e))?;

    Ok(OkNameResponse { ok: true, name })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn clear_tool_skills(
    state: State<'_, AppState>,
    tool_key: String,
) -> AppResult<OkRemovedResponse> {
    let adapters = effective_tool_adapters(&state.db);
    let adapter = adapter::adapter_by_key(&adapters, &tool_key)
        .ok_or_else(|| AppError::InvalidInput(format!("unknown tool: {}", tool_key)))?;

    let skills_dir = resolve_default_path(adapter);
    let dir = std::path::Path::new(&skills_dir);

    let mut removed: i64 = 0;
    if dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    continue;
                }
                let path = entry.path();
                if path.join("SKILL.md").exists() {
                    if crate::filesystem::remove_link_or_directory(&path).is_ok() {
                        removed += 1;
                    }
                }
            }
        }
    }

    // Clear cache
    use crate::repositories::ToolCacheRepository;
    let cache_repo = ToolCacheRepository::new(&state.db);
    let _ = cache_repo.clear_cache(&tool_key);

    Ok(OkRemovedResponse { ok: true, removed })
}
