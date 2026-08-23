use tauri::State;

use crate::error::AppResult;
use crate::services::onboarding::{self, OnboardingPlan};
use crate::state::AppState;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_onboarding_plan(state: State<'_, AppState>) -> AppResult<OnboardingPlan> {
    let community_path = crate::repo::community::resolve_community_repo_path(&state.db);
    let custom_path = crate::repo::community::resolve_custom_repo_path(&state.db);

    // Get managed target paths to exclude
    use crate::repositories::SkillTargetsRepository;
    let targets_repo = SkillTargetsRepository::new(&state.db);
    let all_paths = targets_repo.list_all_paths().unwrap_or_default();
    let target_set: std::collections::HashSet<String> =
        all_paths.into_iter().map(|(_, p)| p).collect();

    let plan = onboarding::build_onboarding_plan(
        &state.db,
        Some(&community_path.to_string_lossy()),
        Some(&target_set),
        Some(&custom_path.to_string_lossy()),
    );

    Ok(plan)
}
