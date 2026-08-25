use crate::db::{now_ms, Database};
use crate::error::AppResult;
use crate::models::PromptFile;

pub struct PromptFilesRepository<'a> {
    db: &'a Database,
}

impl<'a> PromptFilesRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn list(&self) -> AppResult<Vec<PromptFile>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, tool, scope, file_name, file_path, content_hash,
                        exists_on_disk, last_scanned_at, created_at, updated_at
                 FROM prompt_files
                 ORDER BY tool, scope, file_name",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(PromptFile {
                    id: row.get(0)?,
                    tool: row.get(1)?,
                    scope: row.get(2)?,
                    file_name: row.get(3)?,
                    file_path: row.get(4)?,
                    content_hash: row.get(5)?,
                    exists_on_disk: row.get::<_, i64>(6)? != 0,
                    last_scanned_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn list_by_tool(&self, tool: &str) -> AppResult<Vec<PromptFile>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, tool, scope, file_name, file_path, content_hash,
                        exists_on_disk, last_scanned_at, created_at, updated_at
                 FROM prompt_files
                 WHERE tool = ?1
                 ORDER BY scope, file_name",
            )?;
            let rows = stmt.query_map([tool], |row| {
                Ok(PromptFile {
                    id: row.get(0)?,
                    tool: row.get(1)?,
                    scope: row.get(2)?,
                    file_name: row.get(3)?,
                    file_path: row.get(4)?,
                    content_hash: row.get(5)?,
                    exists_on_disk: row.get::<_, i64>(6)? != 0,
                    last_scanned_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn get_by_id(&self, id: &str) -> AppResult<Option<PromptFile>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, tool, scope, file_name, file_path, content_hash,
                        exists_on_disk, last_scanned_at, created_at, updated_at
                 FROM prompt_files
                 WHERE id = ?1",
            )?;
            let result = stmt.query_row([id], |row| {
                Ok(PromptFile {
                    id: row.get(0)?,
                    tool: row.get(1)?,
                    scope: row.get(2)?,
                    file_name: row.get(3)?,
                    file_path: row.get(4)?,
                    content_hash: row.get(5)?,
                    exists_on_disk: row.get::<_, i64>(6)? != 0,
                    last_scanned_at: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            });
            match result {
                Ok(pf) => Ok(Some(pf)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        })
    }

    pub fn upsert(&self, pf: &PromptFile) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO prompt_files (
                  id, tool, scope, file_name, file_path, content_hash,
                  exists_on_disk, last_scanned_at, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                ON CONFLICT(file_path) DO UPDATE SET
                  tool = excluded.tool,
                  scope = excluded.scope,
                  file_name = excluded.file_name,
                  content_hash = excluded.content_hash,
                  exists_on_disk = excluded.exists_on_disk,
                  last_scanned_at = excluded.last_scanned_at,
                  updated_at = excluded.updated_at",
                rusqlite::params![
                    pf.id,
                    pf.tool,
                    pf.scope,
                    pf.file_name,
                    pf.file_path,
                    pf.content_hash,
                    pf.exists_on_disk as i64,
                    pf.last_scanned_at,
                    pf.created_at,
                    pf.updated_at,
                ],
            )?;
            Ok(())
        })
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute("DELETE FROM prompt_files WHERE id = ?1", [id])?;
            Ok(())
        })
    }

    pub fn update_content_hash(&self, id: &str, hash: &str, exists: bool) -> AppResult<()> {
        let now = now_ms();
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE prompt_files SET content_hash = ?1, exists_on_disk = ?2,
                        last_scanned_at = ?3, updated_at = ?3
                 WHERE id = ?4",
                rusqlite::params![hash, exists as i64, now, id],
            )?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_files_crud() {
        let db = Database::new_in_memory().unwrap();
        let repo = PromptFilesRepository::new(&db);

        // Initially empty
        let list = repo.list().unwrap();
        assert!(list.is_empty());

        // Insert
        let pf = PromptFile {
            id: "test-id-1".to_string(),
            tool: "claude_code".to_string(),
            scope: "global".to_string(),
            file_name: "CLAUDE.md".to_string(),
            file_path: "/home/user/.claude/CLAUDE.md".to_string(),
            content_hash: Some("abc123".to_string()),
            exists_on_disk: true,
            last_scanned_at: 1000,
            created_at: 1000,
            updated_at: 1000,
        };
        repo.upsert(&pf).unwrap();

        let list = repo.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].file_name, "CLAUDE.md");

        // Get by id
        let found = repo.get_by_id("test-id-1").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().tool, "claude_code");

        // Update via upsert
        let mut pf2 = pf.clone();
        pf2.content_hash = Some("def456".to_string());
        repo.upsert(&pf2).unwrap();

        let found = repo.get_by_id("test-id-1").unwrap().unwrap();
        assert_eq!(found.content_hash, Some("def456".to_string()));

        // Delete
        repo.delete("test-id-1").unwrap();
        let list = repo.list().unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_list_by_tool() {
        let db = Database::new_in_memory().unwrap();
        let repo = PromptFilesRepository::new(&db);

        for (i, tool) in ["claude_code", "cursor", "claude_code"].iter().enumerate() {
            let pf = PromptFile {
                id: format!("id-{}", i),
                tool: tool.to_string(),
                scope: "project".to_string(),
                file_name: format!("file-{}.md", i),
                file_path: format!("/path/file-{}.md", i),
                content_hash: None,
                exists_on_disk: true,
                last_scanned_at: 1000,
                created_at: 1000,
                updated_at: 1000,
            };
            repo.upsert(&pf).unwrap();
        }

        let claude_files = repo.list_by_tool("claude_code").unwrap();
        assert_eq!(claude_files.len(), 2);

        let cursor_files = repo.list_by_tool("cursor").unwrap();
        assert_eq!(cursor_files.len(), 1);
    }
}
