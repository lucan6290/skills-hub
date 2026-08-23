use crate::db::{now_ms, Database};
use crate::error::AppResult;
use crate::models::Skill;

pub struct SkillsRepository<'a> {
    db: &'a Database,
}

impl<'a> SkillsRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn upsert(&self, skill: &mut Skill) -> AppResult<()> {
        if skill.sort_order == 0.0 {
            let existing_order = self.db.with_conn(|conn| {
                let mut stmt = conn.prepare("SELECT sort_order FROM skills WHERE id = ?1")?;
                let result = stmt.query_row([&skill.id], |row| row.get(0));
                match result {
                    Ok(order) => Ok(Some(order)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(e),
                }
            })?;

            if let Some(order) = existing_order {
                skill.sort_order = order;
            } else {
                let max_order: f64 = self.db.with_conn(|conn| {
                    conn.query_row(
                        "SELECT COALESCE(MAX(sort_order), 0) FROM skills",
                        [],
                        |row| row.get(0),
                    )
                })?;
                skill.sort_order = max_order + 1.0;
            }
        }

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO skills (
                  id, name, description, frontmatter_extra, version, author, license,
                  category, homepage, skill_file_count, skill_dir_size,
                  source_type, source_ref, source_subpath,
                  source_revision, source_url, community_path, content_hash, created_at, updated_at,
                  last_sync_at, last_seen_at, status, sort_order
                ) VALUES (
                  ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24
                )
                ON CONFLICT(id) DO UPDATE SET
                  name = excluded.name,
                  description = excluded.description,
                  frontmatter_extra = excluded.frontmatter_extra,
                  version = excluded.version,
                  author = excluded.author,
                  license = excluded.license,
                  category = excluded.category,
                  homepage = excluded.homepage,
                  skill_file_count = excluded.skill_file_count,
                  skill_dir_size = excluded.skill_dir_size,
                  source_type = excluded.source_type,
                  source_ref = excluded.source_ref,
                  source_subpath = excluded.source_subpath,
                  source_revision = excluded.source_revision,
                  source_url = excluded.source_url,
                  community_path = excluded.community_path,
                  content_hash = excluded.content_hash,
                  created_at = excluded.created_at,
                  updated_at = excluded.updated_at,
                  last_sync_at = excluded.last_sync_at,
                  last_seen_at = excluded.last_seen_at,
                  status = excluded.status,
                  sort_order = excluded.sort_order",
                rusqlite::params![
                    skill.id, skill.name, skill.description, skill.frontmatter_extra,
                    skill.version, skill.author, skill.license, skill.category, skill.homepage,
                    skill.skill_file_count, skill.skill_dir_size, skill.source_type,
                    skill.source_ref, skill.source_subpath, skill.source_revision, skill.source_url,
                    skill.community_path, skill.content_hash, skill.created_at, skill.updated_at,
                    skill.last_sync_at, skill.last_seen_at, skill.status, skill.sort_order,
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_by_id(&self, skill_id: &str) -> AppResult<Option<Skill>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, frontmatter_extra, version, author, license,
                        category, homepage, skill_file_count, skill_dir_size,
                        source_type, source_ref, source_subpath,
                        source_revision, source_url, community_path, content_hash, created_at,
                        updated_at, last_sync_at, last_seen_at, status, sort_order
                 FROM skills WHERE id = ?1 LIMIT 1",
            )?;
            let result = stmt.query_row([skill_id], row_to_skill);
            match result {
                Ok(skill) => Ok(Some(skill)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    pub fn get_by_content_hash(&self, content_hash: &str) -> AppResult<Option<Skill>> {
        if content_hash.is_empty() {
            return Ok(None);
        }
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, frontmatter_extra, version, author, license,
                        category, homepage, skill_file_count, skill_dir_size,
                        source_type, source_ref, source_subpath,
                        source_revision, source_url, community_path, content_hash, created_at,
                        updated_at, last_sync_at, last_seen_at, status, sort_order
                 FROM skills WHERE content_hash = ?1",
            )?;
            let result = stmt.query_row([content_hash], row_to_skill);
            match result {
                Ok(skill) => Ok(Some(skill)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    pub fn get_by_community_path(&self, community_path: &str) -> AppResult<Option<Skill>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, description, frontmatter_extra, version, author, license,
                        category, homepage, skill_file_count, skill_dir_size,
                        source_type, source_ref, source_subpath,
                        source_revision, source_url, community_path, content_hash, created_at,
                        updated_at, last_sync_at, last_seen_at, status, sort_order
                 FROM skills WHERE community_path = ?1 LIMIT 1",
            )?;
            let result = stmt.query_row([community_path], row_to_skill);
            match result {
                Ok(skill) => Ok(Some(skill)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e),
            }
        })
    }

    pub fn list(&self, sort: &str) -> AppResult<Vec<Skill>> {
        let order_by = match sort {
            "manual" => "sort_order ASC, id ASC",
            "updated" => "updated_at DESC",
            "name" => "LOWER(name) ASC",
            _ => "sort_order ASC, id ASC",
        };

        self.db.with_conn(|conn| {
            let sql = format!(
                "SELECT id, name, description, frontmatter_extra, version, author, license,
                        category, homepage, skill_file_count, skill_dir_size,
                        source_type, source_ref, source_subpath,
                        source_revision, source_url, community_path, content_hash, created_at,
                        updated_at, last_sync_at, last_seen_at, status, sort_order
                 FROM skills ORDER BY {}",
                order_by
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map([], row_to_skill)?;
            rows.collect::<Result<Vec<_>, _>>()
        })
    }

    pub fn update_description(&self, skill_id: &str, description: Option<&str>) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE skills SET description = ?1 WHERE id = ?2",
                rusqlite::params![description, skill_id],
            )?;
            Ok(())
        })
    }

    pub fn update_source_url(&self, skill_id: &str, source_url: Option<&str>) -> AppResult<()> {
        let now = now_ms();
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE skills SET source_url = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![source_url, now, skill_id],
            )?;
            Ok(())
        })
    }

    pub fn delete(&self, skill_id: &str) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute("DELETE FROM skills WHERE id = ?1", [skill_id])?;
            Ok(())
        })
    }
}

fn row_to_skill(row: &rusqlite::Row) -> rusqlite::Result<Skill> {
    Ok(Skill {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        frontmatter_extra: row.get(3)?,
        version: row.get(4)?,
        author: row.get(5)?,
        license: row.get(6)?,
        category: row.get(7)?,
        homepage: row.get(8)?,
        skill_file_count: row.get(9)?,
        skill_dir_size: row.get(10)?,
        source_type: row.get(11)?,
        source_ref: row.get(12)?,
        source_subpath: row.get(13)?,
        source_revision: row.get(14)?,
        source_url: row.get(15)?,
        community_path: row.get(16)?,
        content_hash: row.get(17)?,
        created_at: row.get(18)?,
        updated_at: row.get(19)?,
        last_sync_at: row.get(20)?,
        last_seen_at: row.get(21)?,
        status: row.get(22)?,
        sort_order: row.get(23)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skills_crud() {
        let db = Database::new_in_memory().unwrap();
        let repo = SkillsRepository::new(&db);

        let mut skill = Skill {
            id: "test-skill-1".to_string(),
            name: "Test Skill".to_string(),
            community_path: "/test/path".to_string(),
            source_type: "custom".to_string(),
            status: "active".to_string(),
            ..Default::default()
        };

        repo.upsert(&mut skill).unwrap();
        assert!(skill.sort_order > 0.0);

        let fetched = repo.get_by_id("test-skill-1").unwrap().unwrap();
        assert_eq!(fetched.name, "Test Skill");

        let skills = repo.list("manual").unwrap();
        assert_eq!(skills.len(), 1);

        repo.delete("test-skill-1").unwrap();
        assert!(repo.get_by_id("test-skill-1").unwrap().is_none());
    }
}
