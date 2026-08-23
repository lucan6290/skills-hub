use crate::db::{now_ms, Database};
use crate::error::{AppError, AppResult};
use crate::models::{Tag, TagWithCount};

pub struct TagsRepository<'a> {
    db: &'a Database,
}

impl<'a> TagsRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn create(&self, name: &str) -> AppResult<Tag> {
        let normalized = normalize_tag_name(name)?;
        let now = now_ms();

        self.db.with_conn(|conn| {
            let max_order: f64 = conn
                .query_row("SELECT COALESCE(MAX(sort_order), 0) FROM skill_tags", [], |row| {
                    row.get(0)
                })?;
            let sort_order = max_order + 1.0;

            conn.execute(
                "INSERT INTO skill_tags (name, created_at, updated_at, sort_order) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![normalized, now, now, sort_order],
            ).map_err(|e| {
                if e.to_string().contains("UNIQUE constraint") {
                    rusqlite::Error::InvalidParameterName(format!("tag already exists: {}", normalized))
                } else {
                    e
                }
            })?;

            let id = conn.last_insert_rowid();
            Ok(Tag {
                id,
                name: normalized,
                sort_order,
            })
        })
    }

    pub fn rename(&self, tag_id: i64, name: &str) -> AppResult<Tag> {
        let normalized = normalize_tag_name(name)?;
        let now = now_ms();

        self.db.with_conn(|conn| {
            let affected = conn.execute(
                "UPDATE skill_tags SET name = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![normalized, now, tag_id],
            )?;

            if affected == 0 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }

            Ok(Tag {
                id: tag_id,
                name: normalized,
                sort_order: 0.0,
            })
        })
    }

    pub fn delete(&self, tag_id: i64) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute("DELETE FROM skill_tags WHERE id = ?1", [tag_id])?;
            Ok(())
        })
    }

    pub fn list_with_counts(
        &self,
        source_type: Option<&str>,
        sort: &str,
    ) -> AppResult<Vec<TagWithCount>> {
        self.db.with_conn(|conn| {
            let (source_join_filter, count_expr, last_used_expr, params): (
                String,
                &str,
                &str,
                Vec<String>,
            ) = match source_type {
                Some("custom") => (
                    " AND s.source_type = ?".to_string(),
                    "s.id",
                    "CASE WHEN s.id IS NOT NULL THEN l.created_at END",
                    vec!["custom".to_string()],
                ),
                Some("community") => (
                    " AND s.source_type != ?".to_string(),
                    "s.id",
                    "CASE WHEN s.id IS NOT NULL THEN l.created_at END",
                    vec!["custom".to_string()],
                ),
                _ => (String::new(), "l.skill_id", "l.created_at", vec![]),
            };

            let order_by = match sort {
                "manual" => "t.sort_order ASC, t.id ASC",
                "name" => "LOWER(t.name) ASC",
                _ => "LOWER(t.name) ASC",
            };

            let sql = format!(
                "SELECT t.id, t.name, t.sort_order, COUNT({}) AS skill_count,
                        COALESCE(MAX({}), t.updated_at) AS last_used_at
                 FROM skill_tags t
                 LEFT JOIN skill_tag_links l ON l.tag_id = t.id
                 LEFT JOIN skills s ON s.id = l.skill_id{}
                 GROUP BY t.id, t.name, t.sort_order, t.updated_at
                 ORDER BY {}",
                count_expr, last_used_expr, source_join_filter, order_by
            );

            let mut stmt = conn.prepare(&sql)?;
            let params_refs: Vec<&dyn rusqlite::types::ToSql> = params
                .iter()
                .map(|s| s as &dyn rusqlite::types::ToSql)
                .collect();
            let rows = stmt.query_map(params_refs.as_slice(), |row| {
                Ok(TagWithCount {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    sort_order: row.get(2)?,
                    skill_count: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?;

            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn get_skill_tags(&self, skill_id: &str) -> AppResult<Vec<Tag>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT t.id, t.name, t.sort_order
                 FROM skill_tags t
                 INNER JOIN skill_tag_links l ON l.tag_id = t.id
                 WHERE l.skill_id = ?1
                 ORDER BY t.sort_order ASC, LOWER(t.name) ASC",
            )?;
            let rows = stmt.query_map([skill_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    sort_order: row.get(2)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn set_skill_tags(&self, skill_id: &str, tag_ids: &[i64]) -> AppResult<()> {
        let now = now_ms();
        self.db.with_conn(|conn| {
            let tx = conn.unchecked_transaction()?;
            tx.execute("DELETE FROM skill_tag_links WHERE skill_id = ?1", [skill_id])?;
            for tag_id in tag_ids {
                tx.execute(
                    "INSERT INTO skill_tag_links (skill_id, tag_id, created_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![skill_id, tag_id, now],
                )?;
            }
            tx.commit()?;
            Ok(())
        })
    }

    pub fn list_untagged_skill_ids(&self, source_type: Option<&str>) -> AppResult<Vec<String>> {
        self.db.with_conn(|conn| {
            let (source_filter, params): (String, Vec<String>) = match source_type {
                Some("custom") => (
                    " AND s.source_type = ?".to_string(),
                    vec!["custom".to_string()],
                ),
                Some("community") => (
                    " AND s.source_type != ?".to_string(),
                    vec!["custom".to_string()],
                ),
                _ => (String::new(), vec![]),
            };

            let sql = format!(
                "SELECT s.id FROM skills s
                 WHERE NOT EXISTS (
                   SELECT 1 FROM skill_tag_links l WHERE l.skill_id = s.id
                 ){}
                 ORDER BY s.updated_at DESC",
                source_filter
            );

            let mut stmt = conn.prepare(&sql)?;
            let params_refs: Vec<&dyn rusqlite::types::ToSql> = params
                .iter()
                .map(|s| s as &dyn rusqlite::types::ToSql)
                .collect();
            let rows = stmt.query_map(params_refs.as_slice(), |row| row.get(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }
}

fn normalize_tag_name(name: &str) -> AppResult<String> {
    let normalized = name.trim();
    if normalized.is_empty() {
        return Err(AppError::Unexpected("tag name cannot be empty".to_string()));
    }
    Ok(normalized.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tags_crud() {
        let db = Database::new_in_memory().unwrap();
        let repo = TagsRepository::new(&db);

        let tag = repo.create("Test Tag").unwrap();
        assert_eq!(tag.name, "Test Tag");
        assert!(tag.id > 0);

        let renamed = repo.rename(tag.id, "Renamed Tag").unwrap();
        assert_eq!(renamed.name, "Renamed Tag");

        let tags = repo.list_with_counts(None, "name").unwrap();
        assert_eq!(tags.len(), 1);

        repo.delete(tag.id).unwrap();
        let tags = repo.list_with_counts(None, "name").unwrap();
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_normalize_tag_name() {
        assert!(normalize_tag_name("").is_err());
        assert!(normalize_tag_name("   ").is_err());
        assert_eq!(normalize_tag_name("  test  ").unwrap(), "test");
    }
}
