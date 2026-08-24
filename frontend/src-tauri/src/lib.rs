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

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        // single-instance must be the first plugin; the deep-link feature forwards
        // scheme URLs from a second process to the running instance.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    if event.state == ShortcutState::Pressed {
                        show_main_window(app);
                    }
                })
                .build(),
        )
        .manage(state::AppState::default())
        .setup(|app| {
            build_tray(app.handle())?;

            // Intercept the close button: hide the window instead of quitting
            // so the app stays resident in the system tray.
            let main_window = app
                .get_webview_window("main")
                .expect("main window not found");
            main_window.clone().on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = main_window.hide();
                }
            });

            #[cfg(desktop)]
            {
                // Non-fatal: if the hotkey is already taken by another app,
                // log a warning instead of crashing the entire setup.
                if let Err(e) = register_global_shortcut(app.handle()) {
                    eprintln!("[warn] 全局快捷键注册失败: {e}");
                }
            }

            // Register the skillshub:// scheme at runtime on Windows/Linux.
            // macOS uses the Info.plist entry generated from the config.
            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let _ = app.deep_link().register_all();
            }

            Ok(())
        })
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
            crate::commands::misc::open_new_window,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Skills Hub");
}

/// Build the system tray icon with a context menu.
fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let menu = Menu::new(app)?;
    menu.append(&MenuItem::with_id(
        app,
        "show",
        "显示 Skills Hub",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "new_window",
        "新建窗口",
        true,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?)?;

    let icon = app
        .default_window_icon()
        .cloned()
        .expect("window icon must be configured");

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("Skills Hub")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "new_window" => {
                let _ = crate::commands::misc::create_new_window(app);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

/// Register the global hotkey Ctrl+Shift+Space to show/focus the main window.
#[cfg(desktop)]
fn register_global_shortcut(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

    let hotkey = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
    app.global_shortcut().register(hotkey).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;
    Ok(())
}

/// Show, unminimize and focus the main window. No-op if the window is gone.
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}
