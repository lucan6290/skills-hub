use crate::db::{now_ms, Database};
use crate::error::AppResult;

pub struct RecentProjectsRepository<'a> {
    db: &'a Database,
}

impl<'a> RecentProjectsRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn list(&self, limit: i64) -> AppResult<Vec<String>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT project_path FROM recent_projects ORDER BY last_used_at DESC LIMIT ?1",
            )?;
            let rows = stmt.query_map([limit], |row| row.get(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn touch(&self, project_path: &str) -> AppResult<()> {
        let now = now_ms();
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO recent_projects (project_path, last_used_at)
                 VALUES (?1, ?2)
                 ON CONFLICT(project_path) DO UPDATE SET last_used_at = excluded.last_used_at",
                rusqlite::params![project_path, now],
            )?;

            conn.execute_batch(
                "DELETE FROM recent_projects WHERE id NOT IN (
                   SELECT id FROM recent_projects ORDER BY last_used_at DESC LIMIT 8
                 )",
            )?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_projects() {
        let db = Database::new_in_memory().unwrap();
        let repo = RecentProjectsRepository::new(&db);

        let projects = repo.list(8).unwrap();
        assert!(projects.is_empty());

        repo.touch("/path/to/project1").unwrap();
        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(2));
        repo.touch("/path/to/project2").unwrap();

        let projects = repo.list(8).unwrap();
        assert_eq!(projects.len(), 2);
        // project2 should be first since it was touched later
        assert_eq!(projects[0], "/path/to/project2");

        // Touch project1 again to make it most recent
        std::thread::sleep(std::time::Duration::from_millis(2));
        repo.touch("/path/to/project1").unwrap();
        let projects = repo.list(8).unwrap();
        assert_eq!(projects[0], "/path/to/project1");
    }

    #[test]
    fn test_lru_eviction() {
        let db = Database::new_in_memory().unwrap();
        let repo = RecentProjectsRepository::new(&db);

        for i in 0..10 {
            repo.touch(&format!("/path/{}", i)).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        let projects = repo.list(100).unwrap();
        assert_eq!(projects.len(), 8);
    }
}
