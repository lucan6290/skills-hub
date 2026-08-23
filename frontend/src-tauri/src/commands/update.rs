use tauri::AppHandle;

use crate::error::AppResult;
use crate::update::{self, CheckUpdateResponse, PerformUpdateResponse};

#[tauri::command]
pub async fn check_update(app: AppHandle) -> AppResult<CheckUpdateResponse> {
    let current_version = app.package_info().version.to_string();
    // Determine install mode
    let install_mode = detect_install_mode();

    // Run blocking HTTP call on a background thread
    let result = tokio_or_spawn(move || update::check_for_update(&current_version, &install_mode));

    Ok(result)
}

#[tauri::command]
pub async fn do_update(app: AppHandle) -> AppResult<PerformUpdateResponse> {
    let current_version = app.package_info().version.to_string();
    let install_mode = detect_install_mode();
    let install_mode2 = install_mode.clone();

    // First check for update to get download URLs
    let check_result =
        tokio_or_spawn(move || update::check_for_update(&current_version, &install_mode));

    if !check_result.update_available {
        return Ok(PerformUpdateResponse {
            ok: false,
            message: "no update available".to_string(),
        });
    }

    let result =
        tokio_or_spawn(move || update::perform_update(&install_mode2, &check_result.download_urls));

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
