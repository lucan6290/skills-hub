use crate::db::{now_ms, Database};
use crate::error::AppResult;
use crate::models::{ToolScanState, ToolSkillCache};

pub struct ToolCacheRepository<'a> {
    db: &'a Database,
}

impl<'a> ToolCacheRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn get_scan_state(&self, tool_key: &str) -> AppResult<Option<ToolScanState>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT tool_key, tool_name, installed, skills_dir, supports_project_scope,
                        dir_mtime_ns, scanned_at, first_seen_at
                 FROM tool_scan_state WHERE tool_key = ?1",
            )?;
            let result = stmt.query_row([tool_key], row_to_tool_scan_state);
            match result {
                Ok(state) => Ok(Some(state)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    pub fn list_skill_cache(&self, tool_key: &str) -> AppResult<Vec<ToolSkillCache>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT tool_key, name, skill_path, is_link, link_target, description,
                        in_community_repo, skill_mtime_ns, scanned_at
                 FROM tool_skill_cache
                 WHERE tool_key = ?1
                 ORDER BY LOWER(name) ASC",
            )?;
            let rows = stmt.query_map([tool_key], row_to_tool_skill_cache)?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn replace_skill_cache(
        &self,
        tool_key: &str,
        tool_name: &str,
        installed: bool,
        skills_dir: Option<&str>,
        supports_project_scope: bool,
        dir_mtime_ns: Option<i64>,
        scanned_at: i64,
        entries: &[ToolSkillCacheEntry],
    ) -> AppResult<()> {
        self.db.with_conn(|conn| {
            let tx = conn.unchecked_transaction()?;

            tx.execute(
                "INSERT INTO tool_scan_state (
                   tool_key, tool_name, installed, skills_dir, supports_project_scope,
                   dir_mtime_ns, scanned_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(tool_key) DO UPDATE SET
                   tool_name = excluded.tool_name,
                   installed = excluded.installed,
                   skills_dir = excluded.skills_dir,
                   supports_project_scope = excluded.supports_project_scope,
                   dir_mtime_ns = excluded.dir_mtime_ns,
                   scanned_at = excluded.scanned_at",
                rusqlite::params![
                    tool_key,
                    tool_name,
                    installed as i32,
                    skills_dir,
                    supports_project_scope as i32,
                    dir_mtime_ns,
                    scanned_at,
                ],
            )?;

            tx.execute(
                "DELETE FROM tool_skill_cache WHERE tool_key = ?1",
                [tool_key],
            )?;

            for entry in entries {
                tx.execute(
                    "INSERT INTO tool_skill_cache (
                       tool_key, skill_path, name, is_link, link_target, description,
                       in_community_repo, skill_mtime_ns, scanned_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    rusqlite::params![
                        tool_key,
                        entry.path,
                        entry.name,
                        entry.is_link as i32,
                        entry.link_target,
                        entry.description,
                        entry.in_community_repo as i32,
                        entry.skill_mtime_ns,
                        scanned_at,
                    ],
                )?;
            }

            tx.commit()?;
            Ok(())
        })
    }

    pub fn clear_cache(&self, tool_key: &str) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "DELETE FROM tool_skill_cache WHERE tool_key = ?1",
                [tool_key],
            )?;
            conn.execute(
                "DELETE FROM tool_scan_state WHERE tool_key = ?1",
                [tool_key],
            )?;
            Ok(())
        })
    }

    pub fn mark_tool_first_seen(&self, tool_key: &str) -> AppResult<Option<i64>> {
        let state = self.get_scan_state(tool_key)?;
        if state.is_none() || state.as_ref().unwrap().first_seen_at.is_some() {
            return Ok(None);
        }

        let now = now_ms();
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE tool_scan_state SET first_seen_at = ?1 WHERE tool_key = ?2",
                rusqlite::params![now, tool_key],
            )?;
            Ok(())
        })?;

        Ok(Some(now))
    }
}

#[derive(Debug, Clone)]
pub struct ToolSkillCacheEntry {
    pub path: String,
    pub name: String,
    pub is_link: bool,
    pub link_target: Option<String>,
    pub description: Option<String>,
    pub in_community_repo: bool,
    pub skill_mtime_ns: Option<i64>,
}

fn row_to_tool_scan_state(row: &rusqlite::Row) -> rusqlite::Result<ToolScanState> {
    Ok(ToolScanState {
        tool_key: row.get(0)?,
        tool_name: row.get(1)?,
        installed: row.get::<_, i32>(2)? != 0,
        skills_dir: row.get(3)?,
        supports_project_scope: row.get::<_, i32>(4)? != 0,
        dir_mtime_ns: row.get(5)?,
        scanned_at: row.get(6)?,
        first_seen_at: row.get(7)?,
    })
}

fn row_to_tool_skill_cache(row: &rusqlite::Row) -> rusqlite::Result<ToolSkillCache> {
    Ok(ToolSkillCache {
        tool_key: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        is_link: row.get::<_, i32>(3)? != 0,
        link_target: row.get(4)?,
        description: row.get(5)?,
        in_community_repo: row.get::<_, i32>(6)? != 0,
        skill_mtime_ns: row.get(7)?,
        scanned_at: row.get(8)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_cache() {
        let db = Database::new_in_memory().unwrap();
        let repo = ToolCacheRepository::new(&db);

        assert!(repo.get_scan_state("cursor").unwrap().is_none());

        let entries = vec![ToolSkillCacheEntry {
            path: "/path/to/skill1".to_string(),
            name: "skill1".to_string(),
            is_link: false,
            link_target: None,
            description: Some("Test skill".to_string()),
            in_community_repo: false,
            skill_mtime_ns: None,
        }];

        repo.replace_skill_cache(
            "cursor",
            "Cursor",
            true,
            Some("/path/to/skills"),
            true,
            None,
            now_ms(),
            &entries,
        )
        .unwrap();

        let state = repo.get_scan_state("cursor").unwrap().unwrap();
        assert_eq!(state.tool_name, "Cursor");
        assert!(state.installed);

        let cache = repo.list_skill_cache("cursor").unwrap();
        assert_eq!(cache.len(), 1);

        repo.clear_cache("cursor").unwrap();
        assert!(repo.get_scan_state("cursor").unwrap().is_none());
    }
}
