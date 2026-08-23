use tauri::State;

use crate::contracts::ManagedSkillDto;
use crate::error::{AppError, AppResult};
use crate::models::Skill;
use crate::repositories::{
    SkillTargetsRepository, SkillUsageRepository, SkillsRepository, TagsRepository,
};
use crate::services::install::{
    install_local_skill_from_selection, list_local_skills, upsert_skill_from_install,
    LocalSkillCandidate,
};
use crate::state::AppState;

#[tauri::command]
pub async fn get_managed_skills(
    state: State<'_, AppState>,
    refresh: Option<bool>,
    source_type: Option<String>,
    sort: Option<String>,
) -> AppResult<Vec<ManagedSkillDto>> {
    let _refresh = refresh.unwrap_or(false);
    let sort = sort.unwrap_or_else(|| "manual".to_string());

    let repo = SkillsRepository::new(&state.db);
    let tags_repo = TagsRepository::new(&state.db);
    let targets_repo = SkillTargetsRepository::new(&state.db);
    let usage_repo = SkillUsageRepository::new(&state.db);

    let skills = repo
        .list(&sort)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let skills: Vec<Skill> = if let Some(st) = source_type {
        let normalized = match st.as_str() {
            "custom" => "custom",
            _ => "community",
        };
        skills
            .into_iter()
            .filter(|s| {
                let s_type = crate::repo::scanner::normalize_source_type(&s.source_type);
                s_type == normalized
            })
            .collect()
    } else {
        skills
    };

    let mut dtos = Vec::with_capacity(skills.len());
    for skill in skills {
        let tags = tags_repo
            .get_skill_tags(&skill.id)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let targets = targets_repo
            .list_by_skill(&skill.id)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let usage = usage_repo
            .get_by_skill(&skill.id)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let is_suite = crate::repo::scanner::has_sub_skills(std::path::Path::new(
            &skill.community_path,
        ));

        dtos.push(ManagedSkillDto {
            skill,
            tags,
            targets,
            usage,
            is_suite,
        });
    }

    Ok(dtos)
}

#[tauri::command]
pub async fn delete_managed_skill(state: State<'_, AppState>, skill_id: String) -> AppResult<()> {
    let repo = SkillsRepository::new(&state.db);
    // Also remove targets and tag links
    state
        .db
        .with_conn(|conn| {
            conn.execute("DELETE FROM skill_targets WHERE skill_id = ?1", [&skill_id])?;
            conn.execute(
                "DELETE FROM skill_tag_links WHERE skill_id = ?1",
                [&skill_id],
            )?;
            conn.execute(
                "DELETE FROM skill_scope_preference WHERE skill_id = ?1",
                [&skill_id],
            )?;
            conn.execute("DELETE FROM skill_usage WHERE skill_id = ?1", [&skill_id])?;
            Ok::<_, rusqlite::Error>(())
        })
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    repo.delete(&skill_id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))
}

#[tauri::command]
pub async fn update_skill_source_url(
    state: State<'_, AppState>,
    skill_id: String,
    source_url: Option<String>,
) -> AppResult<Skill> {
    let repo = SkillsRepository::new(&state.db);
    repo.update_source_url(&skill_id, source_url.as_deref())
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    repo.get_by_id(&skill_id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("skill not found: {}", skill_id)))
}

#[tauri::command]
pub async fn import_existing_skill(
    state: State<'_, AppState>,
    source_path: String,
    name: Option<String>,
    source_type: Option<String>,
) -> AppResult<serde_json::Value> {
    let source_type = source_type.unwrap_or_else(|| "community".to_string());
    let path = std::path::Path::new(&source_path);

    if !path.is_dir() {
        return Err(AppError::InvalidInput(format!(
            "source path is not a directory: {}",
            source_path
        )));
    }

    let result = crate::services::install::install_local_skill(
        &state.db,
        path,
        name.as_deref(),
        None,
        &source_type,
    )
    .map_err(|e| AppError::FileSystemError(e))?;

    upsert_skill_from_install(&state.db, &result, &source_path, &source_type)
        .map_err(|e| AppError::DatabaseError(e))?;

    Ok(serde_json::json!({
        "skill_id": result.skill_id,
        "name": result.name,
        "community_path": result.community_path,
    }))
}

#[tauri::command]
pub async fn list_local_skills_cmd(base_path: String) -> AppResult<Vec<LocalSkillCandidate>> {
    let path = std::path::Path::new(&base_path);
    list_local_skills(path).map_err(|e| AppError::FileSystemError(e))
}

#[tauri::command]
pub async fn install_local_selection(
    state: State<'_, AppState>,
    base_path: String,
    subpath: String,
    name: Option<String>,
    source_type: Option<String>,
) -> AppResult<serde_json::Value> {
    let source_type = source_type.unwrap_or_else(|| "custom".to_string());
    let base = std::path::Path::new(&base_path);

    let result = install_local_skill_from_selection(
        &state.db,
        base,
        &subpath,
        name.as_deref(),
        None,
        &source_type,
    )
    .map_err(|e| AppError::FileSystemError(e))?;

    let full_source = base.join(&subpath);
    upsert_skill_from_install(
        &state.db,
        &result,
        &full_source.to_string_lossy(),
        &source_type,
    )
    .map_err(|e| AppError::DatabaseError(e))?;

    Ok(serde_json::json!({
        "skill_id": result.skill_id,
        "name": result.name,
        "community_path": result.community_path,
        "description": result.description,
        "content_hash": result.content_hash,
    }))
}
