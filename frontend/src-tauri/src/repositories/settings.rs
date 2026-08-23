use crate::db::Database;
use crate::error::AppResult;

pub struct SettingsRepository<'a> {
    db: &'a Database,
}

impl<'a> SettingsRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn get(&self, key: &str) -> AppResult<Option<String>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
            let result = stmt.query_row([key], |row| row.get(0));
            match result {
                Ok(value) => Ok(Some(value)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    pub fn set(&self, key: &str, value: &str) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                rusqlite::params![key, value],
            )?;
            Ok(())
        })
    }

    pub fn delete(&self, key: &str) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute("DELETE FROM settings WHERE key = ?1", [key])?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_crud() {
        let db = Database::new_in_memory().unwrap();
        let repo = SettingsRepository::new(&db);

        assert_eq!(repo.get("test_key").unwrap(), None);

        repo.set("test_key", "test_value").unwrap();
        assert_eq!(
            repo.get("test_key").unwrap(),
            Some("test_value".to_string())
        );

        repo.set("test_key", "updated_value").unwrap();
        assert_eq!(
            repo.get("test_key").unwrap(),
            Some("updated_value".to_string())
        );

        repo.delete("test_key").unwrap();
        assert_eq!(repo.get("test_key").unwrap(), None);
    }
}
