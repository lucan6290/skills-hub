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
            // health
            crate::commands::health::health_check,
            // skills
            crate::commands::skills::get_managed_skills,
            crate::commands::skills::delete_managed_skill,
            crate::commands::skills::update_skill_source_url,
            crate::commands::skills::import_existing_skill,
            crate::commands::skills::list_local_skills_cmd,
            crate::commands::skills::install_local_selection,
            // tags
            crate::commands::tags::get_tags,
            crate::commands::tags::create_tag,
            crate::commands::tags::rename_tag,
            crate::commands::tags::delete_tag,
            crate::commands::tags::get_skill_tags,
            crate::commands::tags::set_skill_tags,
            // sync
            crate::commands::sync::sync_skill_to_tool,
            crate::commands::sync::unsync_skill_from_tool,
            crate::commands::sync::sync_suite_to_tool,
            crate::commands::sync::unsync_suite_from_tool,
            crate::commands::sync::get_scope_preferences,
            crate::commands::sync::set_scope_preference,
            crate::commands::sync::get_recent_projects,
            crate::commands::sync::save_recent_project,
            crate::commands::sync::list_suite_sub_skills,
            // files
            crate::commands::files::list_skill_files,
            crate::commands::files::read_skill_file,
            crate::commands::files::write_skill_file,
            // tools
            crate::commands::tools::get_tool_status,
            crate::commands::tools::get_tool_skills,
            crate::commands::tools::get_tool_adapter_configs,
            crate::commands::tools::save_tool_adapter_config,
            crate::commands::tools::reset_tool_adapter_config,
            crate::commands::tools::delete_tool_skill,
            crate::commands::tools::open_tool_skills_dir,
            crate::commands::tools::skill_to_community_repo,
            crate::commands::tools::clear_tool_skills,
            // settings
            crate::commands::settings::get_default_sync_tools,
            crate::commands::settings::save_default_sync_tools,
            crate::commands::settings::get_auto_check_update,
            crate::commands::settings::set_auto_check_update,
            crate::commands::settings::get_community_repo_path,
            crate::commands::settings::set_community_repo_path,
            crate::commands::settings::get_custom_repo_path,
            crate::commands::settings::set_custom_repo_path,
            crate::commands::settings::open_settings_folder,
            crate::commands::settings::reset_general_settings,
            // database
            crate::commands::database::db_overview,
            crate::commands::database::db_table_data,
            crate::commands::database::db_maintenance,
            crate::commands::database::db_reset,
            crate::commands::database::db_export,
            crate::commands::database::db_open_folder,
            // onboarding
            crate::commands::onboarding::get_onboarding_plan,
            // tasks
            crate::commands::tasks::get_task_list,
            crate::commands::tasks::get_task,
            crate::commands::tasks::cancel_task,
            // update
            crate::commands::update::check_update,
            crate::commands::update::do_update,
            // misc
            crate::commands::misc::pick_folder,
            crate::commands::misc::cancel_current_operation,
            crate::commands::misc::reorder,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Skills Hub");
}
