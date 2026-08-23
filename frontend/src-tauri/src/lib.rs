pub mod commands;
pub mod config;
pub mod contracts;
pub mod db;
pub mod error;
pub mod filesystem;
pub mod models;
pub mod platform;
pub mod repo;
pub mod repositories;
pub mod services;
pub mod skills;
pub mod state;
pub mod tasks;
pub mod tools;
pub mod update;
pub mod utils;

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
        .invoke_handler(tauri::generate_handler![
            crate::commands::health::health_check
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Skills Hub");
}
