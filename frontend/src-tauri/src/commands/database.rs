use tauri::State;

use crate::contracts::{
    DbColumnInfo, DbMaintenanceResult, DbOverview, DbTableData, DbTableInfo, OkResponse,
};
use crate::error::{AppError, AppResult};
use crate::repositories::MaintenanceRepository;
use crate::state::AppState;

const ALLOWED_TABLES: &[&str] = &[
    "skills",
    "skill_targets",
    "skill_tags",
    "skill_tag_links",
    "settings",
    "discovered_skills",
    "tool_scan_state",
    "tool_skill_cache",
    "tool_adapter_configs",
    "skill_scope_preference",
    "recent_projects",
    "skill_usage",
];

fn table_display_name(table: &str) -> &str {
    match table {
        "skills" => "Skills",
        "skill_targets" => "Sync Targets",
        "skill_tags" => "Tags",
        "skill_tag_links" => "Tag Links",
        "settings" => "Settings",
        "discovered_skills" => "Discovered Skills",
        "tool_scan_state" => "Tool Scan State",
        "tool_skill_cache" => "Tool Skill Cache",
        "tool_adapter_configs" => "Tool Adapter Configs",
        "skill_scope_preference" => "Scope Preferences",
        "recent_projects" => "Recent Projects",
        "skill_usage" => "Skill Usage",
        _ => table,
    }
}

fn format_size(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_overview(state: State<'_, AppState>) -> AppResult<DbOverview> {
    let db_path = crate::config::default_db_path();
    let db_path_str = db_path.to_string_lossy().to_string();

    let file_meta = std::fs::metadata(&db_path).ok();
    let file_size = file_meta.as_ref().map(|m| m.len() as i64).unwrap_or(0);
    let last_modified = file_meta
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    // Get SQLite pragmas
    let (sqlite_version, page_size, page_count, freelist_count) = state
        .db
        .with_conn(|conn| {
            let version: String = conn
                .query_row("SELECT sqlite_version()", [], |row| row.get(0))
                .unwrap_or_default();
            let ps: i64 = conn
                .query_row("PRAGMA page_size", [], |row| row.get(0))
                .unwrap_or(4096);
            let pc: i64 = conn
                .query_row("PRAGMA page_count", [], |row| row.get(0))
                .unwrap_or(0);
            let fl: i64 = conn
                .query_row("PRAGMA freelist_count", [], |row| row.get(0))
                .unwrap_or(0);
            Ok((version, ps, pc, fl))
        })
        .unwrap_or_else(|_| (String::new(), 4096, 0, 0));

    let free_size = freelist_count * page_size;
    let fragmentation_pct = if page_count > 0 {
        (freelist_count as f64 / page_count as f64) * 100.0
    } else {
        0.0
    };

    let maint = MaintenanceRepository::new(&state.db);
    let overview = maint
        .get_overview()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let tables: Vec<DbTableInfo> = overview
        .tables
        .iter()
        .map(|(name, count)| DbTableInfo {
            table_name: name.clone(),
            display_name: table_display_name(name).to_string(),
            row_count: *count,
            size_bytes: 0, // Per-table size estimation is expensive; skip for now
            size_human: String::new(),
        })
        .collect();

    Ok(DbOverview {
        db_path: db_path_str,
        file_size,
        file_size_human: format_size(file_size),
        last_modified,
        sqlite_version,
        page_size,
        page_count,
        freelist_count,
        free_size,
        free_size_human: format_size(free_size),
        fragmentation_pct,
        tables,
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_table_data(
    state: State<'_, AppState>,
    table_name: String,
    page: Option<i64>,
    page_size: Option<i64>,
    sort_col: Option<String>,
    sort_dir: Option<String>,
    filter_text: Option<String>,
) -> AppResult<DbTableData> {
    if !ALLOWED_TABLES.contains(&table_name.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "invalid table name: {}",
            table_name
        )));
    }

    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * page_size;

    // Get column info
    let columns: Vec<DbColumnInfo> = state
        .db
        .with_conn(|conn| {
            let sql = format!("PRAGMA table_info({})", table_name);
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map([], |row| {
                Ok(DbColumnInfo {
                    cid: row.get(0)?,
                    name: row.get(1)?,
                    col_type: row.get(2)?,
                    notnull: row.get::<_, i32>(3)? != 0,
                    default: row.get(4)?,
                    pk: row.get::<_, i32>(5)? != 0,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Build query
    let (where_clause, params): (String, Vec<String>) = if let Some(ref filter) = filter_text {
        if !filter.is_empty() {
            // Simple text filter on first text column
            let text_cols: Vec<&str> = columns
                .iter()
                .filter(|c| c.col_type.to_uppercase().contains("TEXT") || c.col_type.is_empty())
                .map(|c| c.name.as_str())
                .collect();
            if !text_cols.is_empty() {
                let conditions: Vec<String> = text_cols
                    .iter()
                    .map(|col| format!("{} LIKE ?1", col))
                    .collect();
                (
                    format!(" WHERE {}", conditions.join(" OR ")),
                    vec![format!("%{}%", filter)],
                )
            } else {
                (String::new(), vec![])
            }
        } else {
            (String::new(), vec![])
        }
    } else {
        (String::new(), vec![])
    };

    // Get total count
    let total: i64 = state
        .db
        .with_conn(|conn| {
            let sql = format!("SELECT COUNT(*) FROM {}{}", table_name, where_clause);
            let params_refs: Vec<&dyn rusqlite::types::ToSql> = params
                .iter()
                .map(|s| s as &dyn rusqlite::types::ToSql)
                .collect();
            conn.query_row(&sql, params_refs.as_slice(), |row| row.get(0))
        })
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;

    // Sort
    let order_by = if let Some(ref col) = sort_col {
        let dir = sort_dir.as_deref().unwrap_or("asc");
        let safe_dir = if dir == "desc" { "DESC" } else { "ASC" };
        // Validate column name exists
        if columns.iter().any(|c| c.name == *col) {
            format!(" ORDER BY {} {}", col, safe_dir)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Fetch rows
    let rows: Vec<serde_json::Value> = state
        .db
        .with_conn(|conn| {
            let sql = format!(
                "SELECT * FROM {}{}{} LIMIT ?{} OFFSET ?{}",
                table_name,
                where_clause,
                order_by,
                params.len() + 1,
                params.len() + 2,
            );
            let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = params
                .iter()
                .map(|s| Box::new(s.clone()) as Box<dyn rusqlite::types::ToSql>)
                .collect();
            all_params.push(Box::new(page_size));
            all_params.push(Box::new(offset));

            let params_refs: Vec<&dyn rusqlite::types::ToSql> =
                all_params.iter().map(|b| b.as_ref()).collect();

            let mut stmt = conn.prepare(&sql)?;
            let col_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();

            let result_rows = stmt.query_map(params_refs.as_slice(), |row| {
                let mut map = serde_json::Map::new();
                for (i, name) in col_names.iter().enumerate() {
                    let json_val = match row.get::<_, rusqlite::types::Value>(i) {
                        Ok(rusqlite::types::Value::Null) => serde_json::Value::Null,
                        Ok(rusqlite::types::Value::Integer(n)) => {
                            serde_json::Value::Number(n.into())
                        }
                        Ok(rusqlite::types::Value::Real(f)) => serde_json::Number::from_f64(f)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null),
                        Ok(rusqlite::types::Value::Text(s)) => serde_json::Value::String(s),
                        Ok(rusqlite::types::Value::Blob(b)) => {
                            serde_json::Value::String(format!("<blob {} bytes>", b.len()))
                        }
                        Err(_) => serde_json::Value::Null,
                    };
                    map.insert(name.clone(), json_val);
                }
                Ok(serde_json::Value::Object(map))
            })?;
            result_rows.collect::<Result<Vec<_>, _>>()
        })
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(DbTableData {
        table: table_name.clone(),
        display_name: table_display_name(&table_name).to_string(),
        columns,
        rows,
        total,
        page,
        page_size,
        total_pages,
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_maintenance(
    state: State<'_, AppState>,
    action: String,
) -> AppResult<DbMaintenanceResult> {
    let maint = MaintenanceRepository::new(&state.db);

    match action.as_str() {
        "vacuum" => {
            maint
                .vacuum()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "vacuum".to_string(),
                message: "VACUUM completed".to_string(),
                integrity_result: None,
            })
        }
        "analyze" => {
            maint
                .analyze()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "analyze".to_string(),
                message: "ANALYZE completed".to_string(),
                integrity_result: None,
            })
        }
        "integrity_check" => {
            let results = maint
                .integrity_check()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            let result_str = results.join(", ");
            Ok(DbMaintenanceResult {
                ok: result_str == "ok",
                action: "integrity_check".to_string(),
                message: result_str.clone(),
                integrity_result: Some(result_str),
            })
        }
        "clear_cache" => {
            maint
                .clear_cache()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "clear_cache".to_string(),
                message: "Cache cleared".to_string(),
                integrity_result: None,
            })
        }
        "clear_discovered" => {
            maint
                .clear_discovered()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "clear_discovered".to_string(),
                message: "Discovered skills cleared".to_string(),
                integrity_result: None,
            })
        }
        "wal_checkpoint" => {
            maint
                .wal_checkpoint()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "wal_checkpoint".to_string(),
                message: "WAL checkpoint completed".to_string(),
                integrity_result: None,
            })
        }
        "reindex" => {
            maint
                .reindex()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "reindex".to_string(),
                message: "REINDEX completed".to_string(),
                integrity_result: None,
            })
        }
        "optimize" => {
            maint
                .optimize()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "optimize".to_string(),
                message: "PRAGMA optimize completed".to_string(),
                integrity_result: None,
            })
        }
        "clear_usage" => {
            maint
                .clear_usage()
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            Ok(DbMaintenanceResult {
                ok: true,
                action: "clear_usage".to_string(),
                message: "Usage records cleared".to_string(),
                integrity_result: None,
            })
        }
        _ => Err(AppError::InvalidInput(format!(
            "unknown maintenance action: {}",
            action
        ))),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_reset(state: State<'_, AppState>, confirm_text: String) -> AppResult<OkResponse> {
    if confirm_text != "RESET" && confirm_text != "reset" {
        return Err(AppError::InvalidInput(
            "confirmation text must be 'RESET'".into(),
        ));
    }

    // Delete all data from all tables
    state
        .db
        .with_conn(|conn| {
            for table in ALLOWED_TABLES {
                let sql = format!("DELETE FROM {}", table);
                conn.execute_batch(&sql)?;
            }
            Ok::<_, rusqlite::Error>(())
        })
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // VACUUM to reclaim space
    state
        .db
        .with_conn(|conn| {
            conn.execute_batch("VACUUM")?;
            Ok::<_, rusqlite::Error>(())
        })
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Re-initialize tool adapter configs with defaults
    // The Database struct's initialize_tool_adapter_configs is private,
    // so we replicate the logic here using the public config function.
    {
        use crate::config::default_tool_adapters;
        let adapters = default_tool_adapters();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        state
            .db
            .with_conn(|conn| {
                let mut order = 1.0f64;
                for (key, cfg) in &adapters {
                    conn.execute(
                        "INSERT OR REPLACE INTO tool_adapter_configs
                         (tool_key, display_name, skills_dir, detect_dir, project_skills_dir,
                          supports_symlink, supports_junction, force_copy, supports_project_scope,
                          is_custom, enabled, sort_order, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 1, ?10, ?11)",
                        rusqlite::params![
                            key,
                            cfg.display_name,
                            cfg.skills_dir,
                            cfg.detect_dir,
                            cfg.project_skills_dir,
                            cfg.supports_symlink as i32,
                            cfg.supports_junction as i32,
                            cfg.force_copy as i32,
                            cfg.supports_project_scope.map(|b| b as i32),
                            order,
                            now,
                        ],
                    )?;
                    order += 1.0;
                }
                Ok::<_, rusqlite::Error>(())
            })
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    }

    Ok(OkResponse {
        ok: true,
        message: "database has been reset".to_string(),
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_export(app: tauri::AppHandle) -> AppResult<OkResponse> {
    use tauri_plugin_dialog::DialogExt;

    let db_path = crate::config::default_db_path();
    let default_name = format!(
        "skills_hub_backup_{}.db",
        chrono_like_timestamp()
    );

    let file_path = app
        .dialog()
        .file()
        .set_title("Export Database Backup")
        .set_file_name(&default_name)
        .add_filter("SQLite Database", &["db"])
        .blocking_save_file();

    match file_path {
        Some(path) => {
            let dest = path.as_path().unwrap_or_else(|| std::path::Path::new(""));
            if dest.as_os_str().is_empty() {
                return Ok(OkResponse {
                    ok: false,
                    message: "No file selected".to_string(),
                });
            }
            // Ensure .db extension
            let dest = if dest.extension().is_none() {
                dest.with_extension("db")
            } else {
                dest.to_path_buf()
            };

            std::fs::copy(&db_path, &dest).map_err(|e| {
                AppError::FileSystemError(format!("Failed to copy database: {}", e))
            })?;

            Ok(OkResponse {
                ok: true,
                message: format!("Database exported to: {}", dest.display()),
            })
        }
        None => Ok(OkResponse {
            ok: false,
            message: "Export cancelled".to_string(),
        }),
    }
}

fn chrono_like_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // Simple YYYYMMDD_HHMMSS format
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    // Approximate date from epoch days (good enough for filename)
    let (year, month, day) = epoch_days_to_ymd(days);
    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn epoch_days_to_ymd(days_since_epoch: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_import(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> AppResult<OkResponse> {
    use tauri_plugin_dialog::DialogExt;

    let file_path = app
        .dialog()
        .file()
        .set_title("Import Database Backup")
        .add_filter("SQLite Database", &["db"])
        .blocking_pick_file();

    match file_path {
        Some(path) => {
            let src = path.as_path().unwrap_or_else(|| std::path::Path::new(""));
            if src.as_os_str().is_empty() {
                return Ok(OkResponse {
                    ok: false,
                    message: "No file selected".to_string(),
                });
            }

            // Validate it's a valid SQLite file by checking magic bytes
            let data = std::fs::read(src).map_err(|e| {
                AppError::FileSystemError(format!("Failed to read backup file: {}", e))
            })?;
            if data.len() < 16 || &data[..16] != b"SQLite format 3\0" {
                return Err(AppError::InvalidInput(
                    "Selected file is not a valid SQLite database".into(),
                ));
            }

            let db_path = crate::config::default_db_path();

            // Create a backup of current database first
            let backup_path = db_path.with_extension("db.pre_import_backup");
            if db_path.exists() {
                std::fs::copy(&db_path, &backup_path).map_err(|e| {
                    AppError::FileSystemError(format!("Failed to backup current database: {}", e))
                })?;
            }

            // Copy the imported file over the current database
            std::fs::write(&db_path, &data).map_err(|e| {
                AppError::FileSystemError(format!("Failed to write imported database: {}", e))
            })?;

            Ok(OkResponse {
                ok: true,
                message: format!(
                    "Database imported from: {}. Previous database backed up to: {}. Restart the app to apply changes.",
                    src.display(),
                    backup_path.display()
                ),
            })
        }
        None => Ok(OkResponse {
            ok: false,
            message: "Import cancelled".to_string(),
        }),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn db_open_folder() -> AppResult<OkResponse> {
    let db_path = crate::config::default_db_path();
    let folder = db_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    if !folder.exists() {
        std::fs::create_dir_all(folder)
            .map_err(|e| AppError::FileSystemError(format!("failed to create dir: {}", e)))?;
    }

    crate::filesystem::open_folder(folder).map_err(|e| AppError::FileSystemError(e))?;

    Ok(OkResponse {
        ok: true,
        message: "opened".to_string(),
    })
}
