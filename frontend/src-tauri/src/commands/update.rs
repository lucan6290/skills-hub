use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::error::{AppError, AppResult};
use crate::update::{self, CheckUpdateResponse, PerformUpdateResponse};

/// Check for updates via the GitHub Releases API (for display: version + release notes).
///
/// The actual install path is handled by the native `tauri-plugin-updater` in `do_update`.
#[tauri::command(rename_all = "snake_case")]
pub async fn check_update(app: AppHandle) -> AppResult<CheckUpdateResponse> {
    let current_version = app.package_info().version.to_string();
    let install_mode = detect_install_mode();

    let result = tokio_or_spawn(move || update::check_for_update(&current_version, &install_mode));

    Ok(result)
}

/// Download and install the latest update via the native `tauri-plugin-updater`.
///
/// Requires release infrastructure: a signed `latest.json` manifest hosted at the
/// configured `plugins.updater.endpoints` URL and the matching public key in
/// `plugins.updater.pubkey`. Until that is set up, this returns a graceful error.
#[tauri::command(rename_all = "snake_case")]
pub async fn do_update(app: AppHandle) -> AppResult<PerformUpdateResponse> {
    let updater = app
        .updater()
        .map_err(|e| {
            log::warn!("[UPDATE_ERROR] do_update: updater not configured | {}", e);
            AppError::UpdateError(format!("updater 未配置: {e}"))
        })?;

    let update = updater
        .check()
        .await
        .map_err(|e| {
            log::warn!("[UPDATE_ERROR] do_update: check failed | {}", e);
            AppError::UpdateError(format!("检查更新失败: {e}"))
        })?;

    let Some(update) = update else {
        return Ok(PerformUpdateResponse {
            ok: false,
            message: "当前已是最新版本".to_string(),
        });
    };

    update
        .download_and_install(|_chunk, _total| {}, || {})
        .await
        .map_err(|e| {
            log::error!("[UPDATE_ERROR] do_update: download_and_install failed | version={} {}", update.version, e);
            AppError::UpdateError(format!("下载/安装更新失败: {e}"))
        })?;

    app.restart()
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
