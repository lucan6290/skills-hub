pub mod commands;
pub mod contracts;
pub mod error;
pub mod state;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![commands::health_check])
        .run(tauri::generate_context!())
        .expect("failed to run Skills Hub");
}
