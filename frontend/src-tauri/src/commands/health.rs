use tauri::{AppHandle, State};

use crate::{contracts::HealthCheckResponse, error::AppResult, state::AppState};

#[tauri::command]
pub fn health_check(app: AppHandle, state: State<'_, AppState>) -> AppResult<HealthCheckResponse> {
    let _started_at = state.started_at;

    Ok(build_response(app.package_info().version.to_string()))
}

fn build_response(version: String) -> HealthCheckResponse {
    HealthCheckResponse {
        status: "ok".to_owned(),
        version,
    }
}

#[cfg(test)]
mod tests {
    use super::build_response;

    #[test]
    fn builds_health_check_contract() {
        let response = build_response("0.1.1".to_owned());

        assert_eq!(response.status, "ok");
        assert_eq!(response.version, "0.1.1");
    }
}
