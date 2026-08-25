use rusqlite::{Connection, Result as SqlResult};

pub fn ensure_schema(conn: &Connection) -> SqlResult<()> {
    reset_incompatible_schema(conn)?;
    migrate_skill_targets_to_v4_if_old_shape(conn)?;
    self_heal_schema(conn)?;
    initialize_sort_order_columns(conn)?;
    initialize_sort_order_data(conn)?;
    migrate_community_paths(conn)?;
    Ok(())
}

/// Migrate old community_path prefixes to the new directory structure.
/// Old: ~/.skillshub/{name} → New: ~/.skills-hub/skillshub/community-skills/{name}
fn migrate_community_paths(conn: &Connection) -> SqlResult<()> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    if home.is_empty() {
        return Ok(());
    }

    let old_prefix = format!("{}\\.skillshub\\", home.replace('/', "\\"));
    let old_prefix_unix = format!("{}/.skillshub/", home.replace('\\', "/"));
    let new_prefix = format!("{}\\.skills-hub\\skillshub\\community-skills\\", home.replace('/', "\\"));
    let new_prefix_unix = format!("{}/.skills-hub/skillshub/community-skills/", home.replace('\\', "/"));

    // Only migrate if there are records with the old prefix
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM skills WHERE community_path LIKE ?1 OR community_path LIKE ?2",
        rusqlite::params![format!("{}%", old_prefix), format!("{}%", old_prefix_unix)],
        |row| row.get(0),
    )?;

    if count == 0 {
        return Ok(());
    }

    log::info!("migrating {} skill community_path(s) from old prefix", count);

    conn.execute(
        "UPDATE skills SET community_path = ?1 || SUBSTR(community_path, LENGTH(?2) + 1) WHERE community_path LIKE ?2 || '%'",
        rusqlite::params![new_prefix, old_prefix],
    )?;
    conn.execute(
        "UPDATE skills SET community_path = ?1 || SUBSTR(community_path, LENGTH(?2) + 1) WHERE community_path LIKE ?2 || '%'",
        rusqlite::params![new_prefix_unix, old_prefix_unix],
    )?;

    // Also migrate source_ref if it matches
    conn.execute(
        "UPDATE skills SET source_ref = ?1 || SUBSTR(source_ref, LENGTH(?2) + 1) WHERE source_ref LIKE ?2 || '%' AND source_ref IS NOT NULL",
        rusqlite::params![new_prefix, old_prefix],
    )?;
    conn.execute(
        "UPDATE skills SET source_ref = ?1 || SUBSTR(source_ref, LENGTH(?2) + 1) WHERE source_ref LIKE ?2 || '%' AND source_ref IS NOT NULL",
        rusqlite::params![new_prefix_unix, old_prefix_unix],
    )?;

    Ok(())
}

fn reset_incompatible_schema(conn: &Connection) -> SqlResult<()> {
    if !has_development_incompatible_schema(conn)? {
        return Ok(());
    }

    conn.execute_batch(
        "PRAGMA foreign_keys = OFF;
         DROP TABLE IF EXISTS skill_tag_links;
         DROP TABLE IF EXISTS skill_targets;
         DROP TABLE IF EXISTS tool_skill_cache;
         DROP TABLE IF EXISTS tool_scan_state;
         DROP TABLE IF EXISTS discovered_skills;
         DROP TABLE IF EXISTS skill_scope_preference;
         DROP TABLE IF EXISTS tool_adapter_configs;
         DROP TABLE IF EXISTS recent_projects;
         DROP TABLE IF EXISTS skill_tags;
         DROP TABLE IF EXISTS settings;
         DROP TABLE IF EXISTS skills;
         PRAGMA foreign_keys = ON;",
    )
}

fn has_development_incompatible_schema(conn: &Connection) -> SqlResult<bool> {
    let skills_columns = table_columns(conn, "skills")?;
    if !skills_columns.is_empty() && !skills_columns.contains(&"community_path".to_string()) {
        return Ok(true);
    }

    let cache_columns = table_columns(conn, "tool_skill_cache")?;
    if !cache_columns.is_empty() && !cache_columns.contains(&"in_community_repo".to_string()) {
        return Ok(true);
    }

    Ok(false)
}

fn table_columns(conn: &Connection, table: &str) -> SqlResult<Vec<String>> {
    let sql = format!("PRAGMA table_info('{}')", table);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    rows.collect()
}

fn self_heal_schema(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS skills (
          id TEXT PRIMARY KEY,
          name TEXT NOT NULL,
          source_type TEXT NOT NULL,
          source_ref TEXT NULL,
          source_revision TEXT NULL,
          source_url TEXT NULL,
          community_path TEXT NOT NULL UNIQUE,
          content_hash TEXT NULL,
          created_at INTEGER NOT NULL,
          updated_at INTEGER NOT NULL,
          last_sync_at INTEGER NULL,
          last_seen_at INTEGER NOT NULL,
          status TEXT NOT NULL,
          sort_order REAL NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS skill_targets (
          id TEXT PRIMARY KEY,
          skill_id TEXT NOT NULL,
          tool TEXT NOT NULL,
          scope TEXT NOT NULL DEFAULT 'global',
          project_path TEXT NULL,
          target_path TEXT NOT NULL,
          mode TEXT NOT NULL,
          status TEXT NOT NULL,
          last_error TEXT NULL,
          synced_at INTEGER NULL,
          FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_skill_targets_unique_scope
        ON skill_targets(skill_id, tool, scope, COALESCE(project_path, ''));

        CREATE TABLE IF NOT EXISTS settings (
          key TEXT PRIMARY KEY,
          value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS discovered_skills (
          id TEXT PRIMARY KEY,
          tool TEXT NOT NULL,
          found_path TEXT NOT NULL,
          name_guess TEXT NULL,
          fingerprint TEXT NULL,
          found_at INTEGER NOT NULL,
          imported_skill_id TEXT NULL,
          FOREIGN KEY(imported_skill_id) REFERENCES skills(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_skills_name ON skills(name);
        CREATE INDEX IF NOT EXISTS idx_skills_updated_at ON skills(updated_at);

        CREATE TABLE IF NOT EXISTS skill_tags (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL UNIQUE COLLATE NOCASE,
          created_at INTEGER NOT NULL,
          updated_at INTEGER NOT NULL,
          sort_order REAL NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS skill_tag_links (
          skill_id TEXT NOT NULL,
          tag_id INTEGER NOT NULL,
          created_at INTEGER NOT NULL,
          PRIMARY KEY (skill_id, tag_id),
          FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE,
          FOREIGN KEY(tag_id) REFERENCES skill_tags(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS tool_scan_state (
          tool_key TEXT PRIMARY KEY,
          tool_name TEXT NOT NULL,
          installed INTEGER NOT NULL,
          skills_dir TEXT NULL,
          supports_project_scope INTEGER NOT NULL DEFAULT 1,
          dir_mtime_ns INTEGER NULL,
          scanned_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tool_skill_cache (
          tool_key TEXT NOT NULL,
          skill_path TEXT NOT NULL,
          name TEXT NOT NULL,
          is_link INTEGER NOT NULL,
          link_target TEXT NULL,
          description TEXT NULL,
          in_community_repo INTEGER NOT NULL DEFAULT 0,
          skill_mtime_ns INTEGER NULL,
          scanned_at INTEGER NOT NULL,
          PRIMARY KEY (tool_key, skill_path),
          FOREIGN KEY(tool_key) REFERENCES tool_scan_state(tool_key) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_tool_skill_cache_tool_name
        ON tool_skill_cache(tool_key, name);

        CREATE TABLE IF NOT EXISTS tool_adapter_configs (
          tool_key TEXT PRIMARY KEY,
          display_name TEXT NOT NULL,
          skills_dir TEXT NOT NULL,
          detect_dir TEXT NOT NULL,
          supports_symlink INTEGER NOT NULL DEFAULT 1,
          supports_junction INTEGER NOT NULL DEFAULT 1,
          force_copy INTEGER NOT NULL DEFAULT 0,
          supports_project_scope INTEGER NULL,
          is_custom INTEGER NOT NULL DEFAULT 0,
          enabled INTEGER NOT NULL DEFAULT 1,
          sort_order REAL NOT NULL DEFAULT 0,
          updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS skill_scope_preference (
          skill_id TEXT NOT NULL,
          scope TEXT NOT NULL DEFAULT 'global',
          project_paths TEXT NOT NULL DEFAULT '[]',
          updated_at INTEGER NOT NULL,
          PRIMARY KEY (skill_id)
        );

        CREATE TABLE IF NOT EXISTS recent_projects (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          project_path TEXT NOT NULL UNIQUE,
          last_used_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS skill_usage (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          skill_id TEXT NOT NULL,
          tool TEXT NOT NULL,
          sync_count INTEGER NOT NULL DEFAULT 0,
          last_synced_at INTEGER NULL,
          last_viewed_at INTEGER NULL,
          view_count INTEGER NOT NULL DEFAULT 0,
          FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_skill_usage_skill_tool ON skill_usage(skill_id, tool);",
    )?;

    add_column_if_missing(conn, "skills", "description", "TEXT NULL")?;
    add_column_if_missing(conn, "skills", "source_subpath", "TEXT NULL")?;
    add_column_if_missing(conn, "skills", "frontmatter_extra", "TEXT NULL")?;
    add_column_if_missing(conn, "skills", "version", "TEXT NULL")?;
    add_column_if_missing(conn, "skills", "author", "TEXT NULL")?;
    add_column_if_missing(conn, "skills", "license", "TEXT NULL")?;
    add_column_if_missing(conn, "skills", "category", "TEXT NULL")?;
    add_column_if_missing(conn, "skills", "homepage", "TEXT NULL")?;
    add_column_if_missing(conn, "skills", "skill_file_count", "INTEGER NULL")?;
    add_column_if_missing(conn, "skills", "skill_dir_size", "INTEGER NULL")?;
    add_column_if_missing(conn, "skills", "source_url", "TEXT NULL")?;
    add_column_if_missing(conn, "tool_adapter_configs", "project_skills_dir", "TEXT")?;
    add_column_if_missing(conn, "skill_targets", "target_content_hash", "TEXT")?;
    add_column_if_missing(conn, "skill_targets", "target_updated_at", "INTEGER")?;
    add_column_if_missing(conn, "skill_targets", "suite_skill_id", "TEXT NULL")?;
    add_column_if_missing(conn, "tool_scan_state", "first_seen_at", "INTEGER")?;

    Ok(())
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    col_type: &str,
) -> SqlResult<()> {
    let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, col_type);
    match conn.execute_batch(&sql) {
        Ok(_) => Ok(()),
        Err(e) => {
            let msg = e.to_string().to_lowercase();
            if msg.contains("duplicate column") {
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

fn initialize_sort_order_columns(conn: &Connection) -> SqlResult<()> {
    for table in &["skills", "skill_tags", "tool_adapter_configs"] {
        let _ = add_column_if_missing(conn, table, "sort_order", "REAL NOT NULL DEFAULT 0");
    }
    Ok(())
}

fn initialize_sort_order_data(conn: &Connection) -> SqlResult<()> {
    let skills_rows: Vec<String> = {
        let mut stmt =
            conn.prepare("SELECT id FROM skills WHERE sort_order = 0 ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect::<SqlResult<Vec<_>>>()?
    };

    for (i, id) in skills_rows.iter().enumerate() {
        conn.execute(
            "UPDATE skills SET sort_order = ?1 WHERE id = ?2",
            rusqlite::params![(i + 1) as f64, id],
        )?;
    }

    let tag_rows: Vec<i64> = {
        let mut stmt = conn
            .prepare("SELECT id FROM skill_tags WHERE sort_order = 0 ORDER BY LOWER(name) ASC")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect::<SqlResult<Vec<_>>>()?
    };

    for (i, id) in tag_rows.iter().enumerate() {
        conn.execute(
            "UPDATE skill_tags SET sort_order = ?1 WHERE id = ?2",
            rusqlite::params![(i + 1) as f64, id],
        )?;
    }

    Ok(())
}

fn migrate_skill_targets_to_v4_if_old_shape(conn: &Connection) -> SqlResult<()> {
    let columns = table_columns(conn, "skill_targets")?;
    if columns.is_empty() || columns.contains(&"scope".to_string()) {
        return Ok(());
    }

    conn.execute_batch(
        "BEGIN;
         DROP INDEX IF EXISTS idx_skill_targets_unique_scope;
         CREATE TABLE skill_targets_new (
           id TEXT PRIMARY KEY,
           skill_id TEXT NOT NULL,
           tool TEXT NOT NULL,
           scope TEXT NOT NULL DEFAULT 'global',
           project_path TEXT NULL,
           target_path TEXT NOT NULL,
           mode TEXT NOT NULL,
           status TEXT NOT NULL,
           last_error TEXT NULL,
           synced_at INTEGER NULL,
           FOREIGN KEY(skill_id) REFERENCES skills(id) ON DELETE CASCADE
         );
         INSERT INTO skill_targets_new (
           id, skill_id, tool, scope, project_path, target_path, mode, status, last_error, synced_at
         )
         SELECT id, skill_id, tool, 'global', NULL, target_path, mode, status, last_error, synced_at
         FROM skill_targets;
         DROP TABLE skill_targets;
         ALTER TABLE skill_targets_new RENAME TO skill_targets;
         CREATE UNIQUE INDEX idx_skill_targets_unique_scope
         ON skill_targets(skill_id, tool, scope, COALESCE(project_path, ''));
         COMMIT;",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_ensure_schema_creates_all_tables() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        ensure_schema(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(count, 12);
    }

    #[test]
    fn test_ensure_schema_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        ensure_schema(&conn).unwrap();
        ensure_schema(&conn).unwrap();
        ensure_schema(&conn).unwrap();
    }
}
