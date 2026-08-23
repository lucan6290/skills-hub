use crate::db::{now_ms, Database};
use crate::error::AppResult;
use crate::models::SkillUsage;

pub struct SkillUsageRepository<'a> {
    db: &'a Database,
}

impl<'a> SkillUsageRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn record_view(&self, skill_id: &str) -> AppResult<()> {
        let now = now_ms();
        self.db.with_conn(|conn| {
            let existing: Option<i64> = {
                let mut stmt = conn.prepare(
                    "SELECT id FROM skill_usage WHERE skill_id = ?1 AND tool = 'view'"
                )?;
                let result = stmt.query_row([skill_id], |row| row.get(0));
                match result {
                    Ok(id) => Some(id),
                    Err(rusqlite::Error::QueryReturnedNoRows) => None,
                    Err(e) => return Err(e),
                }
            };

            if let Some(id) = existing {
                conn.execute(
                    "UPDATE skill_usage SET view_count = view_count + 1, last_viewed_at = ?1 WHERE id = ?2",
                    rusqlite::params![now, id],
                )?;
            } else {
                conn.execute(
                    "INSERT INTO skill_usage (skill_id, tool, sync_count, last_synced_at, last_viewed_at, view_count)
                     VALUES (?1, 'view', 0, NULL, ?2, 1)",
                    rusqlite::params![skill_id, now],
                )?;
            }
            Ok(())
        })
    }

    pub fn record_sync(&self, skill_id: &str, tool: &str) -> AppResult<()> {
        let now = now_ms();
        self.db.with_conn(|conn| {
            let existing: Option<i64> = {
                let mut stmt = conn.prepare(
                    "SELECT id FROM skill_usage WHERE skill_id = ?1 AND tool = ?2"
                )?;
                let result = stmt.query_row(rusqlite::params![skill_id, tool], |row| row.get(0));
                match result {
                    Ok(id) => Some(id),
                    Err(rusqlite::Error::QueryReturnedNoRows) => None,
                    Err(e) => return Err(e),
                }
            };

            if let Some(id) = existing {
                conn.execute(
                    "UPDATE skill_usage SET sync_count = sync_count + 1, last_synced_at = ?1 WHERE id = ?2",
                    rusqlite::params![now, id],
                )?;
            } else {
                conn.execute(
                    "INSERT INTO skill_usage (skill_id, tool, sync_count, last_synced_at, last_viewed_at, view_count)
                     VALUES (?1, ?2, 1, ?3, NULL, 0)",
                    rusqlite::params![skill_id, tool, now],
                )?;
            }
            Ok(())
        })
    }

    pub fn get_by_skill(&self, skill_id: &str) -> AppResult<Vec<SkillUsage>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, skill_id, tool, sync_count, last_synced_at, last_viewed_at, view_count
                 FROM skill_usage WHERE skill_id = ?1
                 ORDER BY tool ASC",
            )?;
            let rows = stmt.query_map([skill_id], row_to_skill_usage)?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }
}

fn row_to_skill_usage(row: &rusqlite::Row) -> rusqlite::Result<SkillUsage> {
    Ok(SkillUsage {
        id: row.get(0)?,
        skill_id: row.get(1)?,
        tool: row.get(2)?,
        sync_count: row.get(3)?,
        last_synced_at: row.get(4)?,
        last_viewed_at: row.get(5)?,
        view_count: row.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_usage() {
        let db = Database::new_in_memory().unwrap();

        // First create a skill
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO skills (id, name, source_type, community_path, created_at, updated_at, last_seen_at, status)
                 VALUES ('skill-1', 'Test', 'custom', '/path', 0, 0, 0, 'active')",
                [],
            )
        }).unwrap();

        let repo = SkillUsageRepository::new(&db);

        repo.record_view("skill-1").unwrap();
        repo.record_view("skill-1").unwrap();

        let usages = repo.get_by_skill("skill-1").unwrap();
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].view_count, 2);

        repo.record_sync("skill-1", "cursor").unwrap();
        let usages = repo.get_by_skill("skill-1").unwrap();
        assert_eq!(usages.len(), 2);
    }
}
