use tauri::{AppHandle, State};

use crate::error::AppResult;
use crate::repositories::SettingsRepository;
use crate::state::AppState;
use crate::update::{self, CheckUpdateResponse, PerformUpdateResponse};

fn read_proxy_url(state: &AppState) -> Option<String> {
    let repo = SettingsRepository::new(&state.db);
    match repo.get("proxy_url") {
        Ok(Some(val)) if !val.is_empty() => Some(val),
        _ => None,
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn check_update(app: AppHandle, state: State<'_, AppState>) -> AppResult<CheckUpdateResponse> {
    let current_version = app.package_info().version.to_string();
    let install_mode = detect_install_mode();
    let proxy_url = read_proxy_url(&state);

    let result = tokio_or_spawn(move || {
        update::check_for_update(&current_version, &install_mode, proxy_url.as_deref())
    });

    Ok(result)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn do_update(app: AppHandle, state: State<'_, AppState>) -> AppResult<PerformUpdateResponse> {
    let current_version = app.package_info().version.to_string();
    let install_mode = detect_install_mode();
    let install_mode2 = install_mode.clone();
    let proxy_url = read_proxy_url(&state);
    let proxy_url2 = proxy_url.clone();

    let check_result = tokio_or_spawn(move || {
        update::check_for_update(&current_version, &install_mode, proxy_url.as_deref())
    });

    if !check_result.update_available {
        return Ok(PerformUpdateResponse {
            ok: false,
            message: "no update available".to_string(),
        });
    }

    let result = tokio_or_spawn(move || {
        update::perform_update(&install_mode2, &check_result.download_urls, proxy_url2.as_deref())
    });

    Ok(result)
}

fn detect_install_mode() -> String {
    // Simple heuristic: check if running from a portable directory or installed location
    if let Ok(exe) = std::env::current_exe() {
        let exe_dir = exe.parent().unwrap_or(std::path::Path::new("."));
        let portable_flag = exe_dir.join("portable.flag");
        if portable_flag.exists() {
            return "portable".to_string();
        }
        // Check if in Program Files (installed via setup)
        let exe_str = exe.to_string_lossy().to_lowercase();
        if exe_str.contains("program files") {
            return "setup".to_string();
        }
    }
    "dev".to_string()
}

/// Run a blocking closure on a separate thread and wait for the result.
fn tokio_or_spawn<F, T>(f: F) -> T
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(f)
        .join()
        .expect("background thread panicked")
}
