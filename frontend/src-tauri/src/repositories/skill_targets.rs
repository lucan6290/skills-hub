use crate::db::Database;
use crate::error::AppResult;
use crate::models::SkillTarget;

pub struct SkillTargetsRepository<'a> {
    db: &'a Database,
}

impl<'a> SkillTargetsRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn upsert(&self, target: &SkillTarget) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO skill_targets (
                  id, skill_id, tool, scope, project_path, target_path,
                  mode, status, last_error, synced_at, target_content_hash, target_updated_at,
                  suite_skill_id
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                ON CONFLICT DO UPDATE SET
                  target_path = excluded.target_path,
                  mode = excluded.mode,
                  status = excluded.status,
                  last_error = excluded.last_error,
                  synced_at = excluded.synced_at,
                  target_content_hash = excluded.target_content_hash,
                  target_updated_at = excluded.target_updated_at,
                  suite_skill_id = excluded.suite_skill_id",
                rusqlite::params![
                    target.id,
                    target.skill_id,
                    target.tool,
                    target.scope,
                    target.project_path,
                    target.target_path,
                    target.mode,
                    target.status,
                    target.last_error,
                    target.synced_at,
                    target.target_content_hash,
                    target.target_updated_at,
                    target.suite_skill_id,
                ],
            )?;
            Ok(())
        })
    }

    pub fn list_by_skill(&self, skill_id: &str) -> AppResult<Vec<SkillTarget>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, skill_id, tool, scope, project_path, target_path,
                        mode, status, last_error, synced_at,
                        target_content_hash, target_updated_at, suite_skill_id
                 FROM skill_targets WHERE skill_id = ?1
                 ORDER BY tool ASC, scope ASC, project_path ASC",
            )?;
            let rows = stmt.query_map([skill_id], row_to_target)?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn list_suite_sub_targets(&self, suite_skill_id: &str) -> AppResult<Vec<SkillTarget>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, skill_id, tool, scope, project_path, target_path,
                        mode, status, last_error, synced_at,
                        target_content_hash, target_updated_at, suite_skill_id
                 FROM skill_targets WHERE suite_skill_id = ?1
                 ORDER BY tool ASC, scope ASC, project_path ASC",
            )?;
            let rows = stmt.query_map([suite_skill_id], row_to_target)?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn list_all_paths(&self) -> AppResult<Vec<(String, String)>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT tool, target_path FROM skill_targets")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn get(
        &self,
        skill_id: &str,
        tool: &str,
        scope: &str,
        project_path: Option<&str>,
    ) -> AppResult<Option<SkillTarget>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, skill_id, tool, scope, project_path, target_path,
                        mode, status, last_error, synced_at,
                        target_content_hash, target_updated_at, suite_skill_id
                 FROM skill_targets
                 WHERE skill_id = ?1 AND tool = ?2 AND scope = ?3
                   AND ((?4 IS NULL AND project_path IS NULL) OR project_path = ?4)",
            )?;
            let result = stmt.query_row(
                rusqlite::params![skill_id, tool, scope, project_path],
                row_to_target,
            );
            match result {
                Ok(target) => Ok(Some(target)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    pub fn get_by_path(&self, target_path: &str) -> AppResult<Option<SkillTarget>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, skill_id, tool, scope, project_path, target_path,
                        mode, status, last_error, synced_at,
                        target_content_hash, target_updated_at, suite_skill_id
                 FROM skill_targets WHERE target_path = ?1",
            )?;
            let result = stmt.query_row([target_path], row_to_target);
            match result {
                Ok(target) => Ok(Some(target)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    pub fn delete(
        &self,
        skill_id: &str,
        tool: &str,
        scope: &str,
        project_path: Option<&str>,
    ) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "DELETE FROM skill_targets
                 WHERE skill_id = ?1 AND tool = ?2 AND scope = ?3
                   AND ((?4 IS NULL AND project_path IS NULL) OR project_path = ?4)",
                rusqlite::params![skill_id, tool, scope, project_path],
            )?;
            Ok(())
        })
    }

    pub fn delete_suite_targets(
        &self,
        suite_skill_id: &str,
        tool: &str,
        scope: &str,
        project_path: Option<&str>,
    ) -> AppResult<Vec<SkillTarget>> {
        let records = self.list_suite_sub_targets(suite_skill_id)?;
        let matching: Vec<_> = records
            .into_iter()
            .filter(|r| {
                r.tool == tool
                    && r.scope == scope
                    && ((project_path.is_none() && r.project_path.is_none())
                        || r.project_path.as_deref() == project_path)
            })
            .collect();

        for r in &matching {
            self.db.with_conn(|conn| {
                conn.execute("DELETE FROM skill_targets WHERE id = ?1", [&r.id])?;
                Ok::<_, rusqlite::Error>(())
            })?;
        }

        Ok(matching)
    }
}

fn row_to_target(row: &rusqlite::Row) -> rusqlite::Result<SkillTarget> {
    Ok(SkillTarget {
        id: row.get(0)?,
        skill_id: row.get(1)?,
        tool: row.get(2)?,
        scope: row.get(3)?,
        project_path: row.get(4)?,
        target_path: row.get(5)?,
        mode: row.get(6)?,
        status: row.get(7)?,
        last_error: row.get(8)?,
        synced_at: row.get(9)?,
        target_content_hash: row.get(10)?,
        target_updated_at: row.get(11)?,
        suite_skill_id: row.get(12)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_targets_crud() {
        let db = Database::new_in_memory().unwrap();

        // First create a skill
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO skills (id, name, source_type, community_path, created_at, updated_at, last_seen_at, status)
                 VALUES ('skill-1', 'Test', 'custom', '/path', 0, 0, 0, 'active')",
                [],
            )
        }).unwrap();

        let repo = SkillTargetsRepository::new(&db);

        let target = SkillTarget {
            id: "target-1".to_string(),
            skill_id: "skill-1".to_string(),
            tool: "cursor".to_string(),
            scope: "global".to_string(),
            target_path: "/target/path".to_string(),
            mode: "copy".to_string(),
            status: "ok".to_string(),
            ..Default::default()
        };

        repo.upsert(&target).unwrap();

        let targets = repo.list_by_skill("skill-1").unwrap();
        assert_eq!(targets.len(), 1);

        let fetched = repo
            .get("skill-1", "cursor", "global", None)
            .unwrap()
            .unwrap();
        assert_eq!(fetched.target_path, "/target/path");

        repo.delete("skill-1", "cursor", "global", None).unwrap();
        assert!(repo
            .get("skill-1", "cursor", "global", None)
            .unwrap()
            .is_none());
    }
}
