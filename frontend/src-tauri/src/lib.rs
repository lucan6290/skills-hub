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

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

pub fn run() {
    let log_dir = crate::config::resolve_data_dir().join("logs");
    let _ = std::fs::create_dir_all(&log_dir);

    // Install a panic hook so that crashes are captured in the dedicated error
    // log file. Without this, panics only print to stderr and are lost in a
    // GUI application.
    let error_log_path = log_dir.join("skills-hub-error.log");
    std::panic::set_hook(Box::new(move |panic_info| {
        use std::io::Write;
        let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
        let timestamp = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            now.year(),
            u8::from(now.month()),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );
        // force_capture() ignores the RUST_BACKTRACE env var, which is
        // typically unset in a desktop GUI application.
        let backtrace = std::backtrace::Backtrace::force_capture();
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&error_log_path)
            .and_then(|mut f| {
                writeln!(f, "[{}] [PANIC] {}", timestamp, panic_info)?;
                writeln!(f, "Backtrace:\n{}", backtrace)?;
                writeln!(f, "--------------------------------------------------")?;
                Ok(())
            });
        // Also attempt to route through the regular logger (may fail if the
        // panic originated inside the logger itself).
        log::error!("PANIC: {}", panic_info);
    }));

    // Read log level from DB before Tauri initializes (use a temporary connection).
    let log_level = {
        let db_path = crate::config::default_db_path();
        if let Ok(db) = crate::db::Database::new(&db_path) {
            let repo = crate::repositories::SettingsRepository::new(&db);
            repo.get("log_level").ok().flatten().unwrap_or_else(|| "info".to_string())
        } else {
            "info".to_string()
        }
    };
    let level_filter = match log_level.as_str() {
        "debug" => log::LevelFilter::Debug,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new()
            .targets([
                Target::new(TargetKind::Stdout),
                Target::new(TargetKind::Folder {
                    path: log_dir.clone(),
                    file_name: Some("skills-hub".to_string()),
                }),
                // Dedicated error log: only captures Error-level messages
                // → skills-hub-error.log
                Target::new(TargetKind::Folder {
                    path: log_dir,
                    file_name: Some("skills-hub-error".to_string()),
                })
                .filter(|metadata| metadata.level() == log::Level::Error),
                Target::new(TargetKind::Webview),
            ])
            .level(level_filter)
            .max_file_size(10_000_000) // 10 MB per file
            .rotation_strategy(RotationStrategy::KeepSome(7)) // keep 7 rotated files
            .timezone_strategy(TimezoneStrategy::UseLocal)
            .build())
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
        .plugin(tauri_plugin_dialog::init())
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

            // Intercept the close button based on user setting:
            // "minimize_to_tray" (default) → hide window; "quit" → exit app.
            let main_window = app
                .get_webview_window("main")
                .unwrap_or_else(|| {
                    log::error!("setup 阶段未找到主窗口");
                    panic!("main window not found");
                });
            let app_handle = app.handle().clone();
            let db_for_close = state::AppState::default_db_ref(&app_handle);
            main_window.clone().on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let behavior = db_for_close.as_ref()
                        .and_then(|db| {
                            let repo = crate::repositories::SettingsRepository::new(db);
                            repo.get("close_behavior").ok().flatten()
                        })
                        .unwrap_or_else(|| "minimize_to_tray".to_string());

                    if behavior == "quit" {
                        // Allow the window to close and quit the app
                        app_handle.exit(0);
                    } else {
                        api.prevent_close();
                        let _ = main_window.hide();
                    }
                }
            });

            #[cfg(desktop)]
            {
                // Non-fatal: if the hotkey is already taken by another app,
                // log a warning instead of crashing the entire setup.
                if let Err(e) = register_global_shortcut(app.handle()) {
                    log::warn!("全局快捷键注册失败: {e}");
                }
            }

            // Register the skillshub:// scheme at runtime on Windows/Linux.
            // macOS uses the Info.plist entry generated from the config.
            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let _ = app.deep_link().register_all();
            }

            // Auto-refresh repo registries on startup if enabled.
            {
                let state = app.state::<state::AppState>();
                let repo = crate::repositories::SettingsRepository::new(&state.db);
                let auto_refresh = repo.get("auto_refresh_on_startup")
                    .ok()
                    .flatten()
                    .map(|v| v == "true")
                    .unwrap_or(false);
                if auto_refresh {
                    let db = state.db.clone();
                    std::thread::spawn(move || {
                        match crate::repo::scanner::sync_all_repo_registries(&db) {
                            Ok(result) => log::info!(
                                "启动时自动刷新完成: registered={}, removed={}",
                                result.registered, result.removed
                            ),
                            Err(e) => log::warn!("启动时自动刷新失败: {}", e),
                        }
                    });
                }
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
            crate::commands::settings::get_close_behavior,
            crate::commands::settings::set_close_behavior,
            crate::commands::settings::get_show_tray_icon,
            crate::commands::settings::set_show_tray_icon,
            crate::commands::settings::get_log_level,
            crate::commands::settings::set_log_level,
            crate::commands::settings::get_auto_refresh_on_startup,
            crate::commands::settings::set_auto_refresh_on_startup,
            crate::commands::settings::reset_general_settings,
            // database
            crate::commands::database::db_overview,
            crate::commands::database::db_table_data,
            crate::commands::database::db_maintenance,
            crate::commands::database::db_reset,
            crate::commands::database::db_export,
            crate::commands::database::db_open_folder,
            crate::commands::database::db_import,
            // onboarding
            crate::commands::onboarding::get_onboarding_plan,
            // tasks
            crate::commands::tasks::get_task_list,
            crate::commands::tasks::get_task,
            crate::commands::tasks::cancel_task,
            // update
            crate::commands::update::check_update,
            crate::commands::update::do_update,
            // prompts
            crate::commands::prompts::scan_prompt_files,
            crate::commands::prompts::scan_project_prompt_files,
            crate::commands::prompts::get_prompt_files,
            crate::commands::prompts::read_prompt_file,
            crate::commands::prompts::write_prompt_file,
            crate::commands::prompts::delete_prompt_file,
            // misc
            crate::commands::misc::pick_folder,
            crate::commands::misc::cancel_current_operation,
            crate::commands::misc::reorder,
            crate::commands::misc::open_new_window,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            log::error!("应用启动失败: {}", e);
            panic!("failed to run Skills Hub: {}", e);
        });
}

/// Build the system tray icon with a context menu.
fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let version = app.package_info().version.to_string();
    let version_label = format!("Skills Hub v{}", version);

    let menu = Menu::new(app)?;

    // --- Version info (disabled item) ---
    menu.append(&MenuItem::with_id(
        app,
        "version",
        &version_label,
        false,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // --- Window controls ---
    menu.append(&MenuItem::with_id(
        app,
        "show",
        "显示窗口",
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

    // --- Update & web ---
    menu.append(&MenuItem::with_id(
        app,
        "check_update",
        "检查更新",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "open_website",
        "打开官方网站",
        true,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // --- Open directory submenu ---
    let dir_submenu = Submenu::with_items(
        app,
        "打开目录",
        true,
        &[
            &MenuItem::with_id(app, "open_app_dir", "应用目录", true, None::<&str>)?,
            &MenuItem::with_id(app, "open_data_dir", "工作目录", true, None::<&str>)?,
            &MenuItem::with_id(app, "open_resource_dir", "内核目录", true, None::<&str>)?,
            &MenuItem::with_id(app, "open_log_dir", "日志目录", true, None::<&str>)?,
        ],
    )?;
    menu.append(&dir_submenu)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // --- App lifecycle ---
    menu.append(&MenuItem::with_id(
        app,
        "restart",
        "重启应用",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?)?;

    let icon = app
        .default_window_icon()
        .cloned()
        .unwrap_or_else(|| {
            log::error!("窗口图标未配置");
            panic!("window icon must be configured");
        });

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("Skills Hub")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "new_window" => {
                let _ = crate::commands::misc::create_new_window(app);
            }
            "check_update" => tray_check_update(app),
            "open_website" => open_url("https://github.com/lucan6290/skills-hub"),
            "open_app_dir" => open_app_directory(app, AppDir::App),
            "open_data_dir" => open_app_directory(app, AppDir::Data),
            "open_resource_dir" => open_app_directory(app, AppDir::Resource),
            "open_log_dir" => open_app_directory(app, AppDir::Log),
            "restart" => app.restart(),
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

/// Directory types for the tray "打开目录" submenu.
enum AppDir {
    App,
    Data,
    Resource,
    Log,
}

/// Resolve and open a specific app directory in the system file manager.
fn open_app_directory(app: &tauri::AppHandle, dir: AppDir) {
    let path = match dir {
        AppDir::App => std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf())),
        AppDir::Data => Some(crate::config::resolve_data_dir()),
        AppDir::Resource => app.path().resource_dir().ok(),
        AppDir::Log => Some(crate::config::resolve_data_dir().join("logs")),
    };

    let Some(path) = path else {
        log::warn!("无法解析目录路径");
        return;
    };

    if !path.exists() {
        if let Err(e) = std::fs::create_dir_all(&path) {
            log::warn!("无法创建目录 {}: {}", path.display(), e);
            return;
        }
    }

    if let Err(e) = crate::filesystem::open_folder(&path) {
        log::warn!("无法打开目录 {}: {}", path.display(), e);
    }
}

/// Open a URL in the system default browser.
fn open_url(url: &str) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

/// Check for updates from the tray menu, showing a system notification with the result.
fn tray_check_update(app: &tauri::AppHandle) {
    use tauri_plugin_notification::NotificationExt;

    let app_handle = app.clone();
    std::thread::spawn(move || {
        let version = app_handle.package_info().version.to_string();
        let result = crate::update::check_for_update(&version, "tray");

        let (title, body) = if let Some(err) = &result.error {
            ("检查更新失败".to_string(), err.clone())
        } else if result.update_available {
            (
                "发现新版本".to_string(),
                format!("v{} → v{}\n点击应用内更新按钮进行安装", result.current_version, result.latest_version),
            )
        } else {
            ("已是最新版本".to_string(), format!("v{}", result.current_version))
        };

        let _ = app_handle
            .notification()
            .builder()
            .title(&title)
            .body(&body)
            .show();
    });
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
