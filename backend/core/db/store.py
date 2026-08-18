"""SQLite data access layer -- self-healing schema mode.

Schema is maintained by _self_heal_schema() which ensures every table/column
physically exists via idempotent DDL. No version-number trust. To add a table
or column, add one entry to _self_heal_schema() -- that's the only change needed.
"""
from __future__ import annotations

import os
import sqlite3
import threading
from dataclasses import dataclass, field
from typing import Optional, Any

from core.config import LEGACY_APP_IDENTIFIERS


# ── Data Records ────────────────────────────────────────

@dataclass
class SkillRecord:
    id: str
    name: str
    description: Optional[str] = None
    frontmatter_extra: Optional[str] = None
    version: Optional[str] = None
    author: Optional[str] = None
    license: Optional[str] = None
    category: Optional[str] = None
    homepage: Optional[str] = None
    skill_file_count: Optional[int] = None
    skill_dir_size: Optional[int] = None
    source_type: str = "community"
    source_ref: Optional[str] = None
    source_subpath: Optional[str] = None
    source_revision: Optional[str] = None
    source_url: Optional[str] = None
    community_path: str = ""
    content_hash: Optional[str] = None
    created_at: int = 0
    updated_at: int = 0
    last_sync_at: Optional[int] = None
    last_seen_at: int = 0
    status: str = "active"
    sort_order: float = 0.0


@dataclass
class SkillUsageRecord:
    id: int
    skill_id: str
    tool: str
    sync_count: int
    last_synced_at: Optional[int]
    last_viewed_at: Optional[int]
    view_count: int


@dataclass
class SkillTargetRecord:
    id: str
    skill_id: str
    tool: str
    scope: str
    project_path: Optional[str]
    target_path: str
    mode: str
    status: str
    last_error: Optional[str]
    synced_at: Optional[int]
    target_content_hash: Optional[str]
    target_updated_at: Optional[int]
    suite_skill_id: Optional[str] = None


@dataclass
class TagRecord:
    id: int
    name: str
    sort_order: float = 0.0


@dataclass
class TagWithCountRecord:
    id: int
    name: str
    skill_count: int
    updated_at: int
    sort_order: float = 0.0


@dataclass
class ToolScanStateRecord:
    tool_key: str
    tool_name: str
    installed: bool
    skills_dir: Optional[str]
    supports_project_scope: bool
    dir_mtime_ns: Optional[int]
    scanned_at: int
    first_seen_at: Optional[int]


@dataclass
class ToolSkillCacheRecord:
    tool_key: str
    name: str
    path: str
    is_link: bool
    link_target: Optional[str]
    description: Optional[str]
    in_community_repo: bool
    skill_mtime_ns: Optional[int]
    scanned_at: int


@dataclass
class ToolAdapterConfigRecord:
    tool_key: str
    display_name: str
    skills_dir: str
    detect_dir: str
    project_skills_dir: Optional[str]
    supports_symlink: bool
    supports_junction: bool
    force_copy: bool
    supports_project_scope: Optional[bool]
    is_custom: bool
    enabled: bool
    updated_at: int
    sort_order: float = 0.0
@dataclass
class ScopePreferenceRecord:
    skill_id: str
    scope: str
    project_paths: str  # JSON array
    updated_at: int


class SkillStore:
    """SQLite 数据访问层"""

    def __init__(self, db_path: str):
        self._db_path = db_path
        self._local = threading.local()

    @property
    def db_path(self) -> str:
        return self._db_path

    # ── Connection management ───────────────────────────

    def _get_conn(self) -> sqlite3.Connection:
        """获取当前线程的数据库连接（线程安全）"""
        if not hasattr(self._local, "conn") or self._local.conn is None:
            conn = sqlite3.connect(self._db_path)
            conn.execute("PRAGMA foreign_keys = ON")
            conn.row_factory = sqlite3.Row
            self._local.conn = conn
        return self._local.conn

    def close(self):
        """关闭当前线程的数据库连接"""
        if hasattr(self._local, "conn") and self._local.conn is not None:
            self._local.conn.close()
            self._local.conn = None

    def _fetch_all(self, sql: str, params: tuple = ()) -> list[sqlite3.Row]:
        conn = self._get_conn()
        return conn.execute(sql, params).fetchall()

    def _fetch_one(self, sql: str, params: tuple = ()) -> Optional[sqlite3.Row]:
        conn = self._get_conn()
        return conn.execute(sql, params).fetchone()

    def _execute(self, sql: str, params: tuple = ()) -> sqlite3.Cursor:
        conn = self._get_conn()
        cur = conn.execute(sql, params)
        conn.commit()  # 显式提交，避免写入只停留在未提交事务里（重启即丢、外部进程读不到）
        return cur

    # ── Schema ──────────────────────────────────────────

    def ensure_schema(self) -> None:
        """单点自愈：按顺序确保 Schema、迁移数据和默认配置完成。"""
        self._reset_incompatible_schema()
        self._migrate_data_if_needed()
        self._self_heal_schema_structure()
        self._initialize_sort_order_columns()
        self._initialize_sort_order_data()
        self._initialize_tool_adapter_configs()

    def _reset_incompatible_schema(self) -> None:
        conn = self._get_conn()
        _reset_schema_if_incompatible(conn)

    def _migrate_data_if_needed(self) -> None:
        conn = self._get_conn()
        _migrate_skill_targets_to_v4_if_old_shape(conn)

    def _self_heal_schema_structure(self) -> None:
        conn = self._get_conn()
        _self_heal_schema(conn)

    def _initialize_sort_order_columns(self) -> None:
        for table in ("skills", "skill_tags", "tool_adapter_configs"):
            try:
                self._execute(f"ALTER TABLE {table} ADD COLUMN sort_order REAL NOT NULL DEFAULT 0")
            except sqlite3.OperationalError:
                pass  # 列已存在

    def _initialize_sort_order_data(self) -> None:
        skills_rows = self._fetch_all("SELECT id FROM skills WHERE sort_order = 0 ORDER BY updated_at DESC")
        for i, row in enumerate(skills_rows):
            self._execute("UPDATE skills SET sort_order = ? WHERE id = ?", (float(i + 1), row["id"]))

        tag_rows = self._fetch_all("SELECT id FROM skill_tags WHERE sort_order = 0 ORDER BY LOWER(name) ASC")
        for i, row in enumerate(tag_rows):
            self._execute("UPDATE skill_tags SET sort_order = ? WHERE id = ?", (float(i + 1), row["id"]))

    def _initialize_tool_adapter_configs(self) -> None:
        from core.config import DEFAULT_TOOL_ADAPTERS

        existing_keys = {r["tool_key"] for r in self._fetch_all("SELECT tool_key FROM tool_adapter_configs")}
        order = 1.0
        now = now_ms()
        for key, cfg in DEFAULT_TOOL_ADAPTERS.items():
            if key not in existing_keys:
                self._insert_default_tool_adapter_config(key, cfg, order, now)
            else:
                self._update_existing_tool_adapter_config(key, cfg, order, now)
            order += 1.0
        self._update_custom_tool_adapter_sort_orders(order)

        self._get_conn().commit()

    def _insert_default_tool_adapter_config(self, key: str, cfg: dict[str, Any], order: float, now: int) -> None:
        self._execute(
            """INSERT OR IGNORE INTO tool_adapter_configs
               (tool_key, display_name, skills_dir, detect_dir, project_skills_dir,
                supports_symlink, supports_junction, force_copy, supports_project_scope,
                is_custom, enabled, sort_order, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 1, ?, ?)""",
            (
                key,
                cfg["display_name"],
                cfg["skills_dir"],
                cfg["detect_dir"],
                cfg.get("project_skills_dir"),
                1 if cfg.get("supports_symlink", True) else 0,
                1 if cfg.get("supports_junction", True) else 0,
                1 if cfg.get("force_copy", False) else 0,
                None if cfg.get("supports_project_scope") is None
                else 1 if cfg.get("supports_project_scope") else 0,
                order,
                now,
            ),
        )

    def _update_existing_tool_adapter_config(self, key: str, cfg: dict[str, Any], order: float, now: int) -> None:
        self._execute(
            """UPDATE tool_adapter_configs
               SET skills_dir = CASE WHEN skills_dir = '' AND detect_dir = '' THEN ? ELSE skills_dir END,
                   detect_dir = CASE WHEN skills_dir = '' AND detect_dir = '' THEN ? ELSE detect_dir END,
                   project_skills_dir = CASE WHEN skills_dir = '' AND detect_dir = '' AND project_skills_dir IS NULL THEN ? ELSE project_skills_dir END,
                   supports_symlink = CASE WHEN skills_dir = '' AND detect_dir = '' THEN ? ELSE supports_symlink END,
                   supports_junction = CASE WHEN skills_dir = '' AND detect_dir = '' THEN ? ELSE supports_junction END,
                   force_copy = CASE WHEN skills_dir = '' AND detect_dir = '' THEN ? ELSE force_copy END,
                   supports_project_scope = CASE WHEN skills_dir = '' AND detect_dir = '' THEN ? ELSE supports_project_scope END,
                   sort_order = CASE WHEN sort_order = 0 THEN ? ELSE sort_order END,
                   updated_at = CASE WHEN (skills_dir = '' AND detect_dir = '') OR sort_order = 0 THEN ? ELSE updated_at END
               WHERE tool_key = ?""",
            (
                cfg["skills_dir"],
                cfg["detect_dir"],
                cfg.get("project_skills_dir"),
                1 if cfg.get("supports_symlink", True) else 0,
                1 if cfg.get("supports_junction", True) else 0,
                1 if cfg.get("force_copy", False) else 0,
                None if cfg.get("supports_project_scope") is None
                else 1 if cfg.get("supports_project_scope") else 0,
                order,
                now,
                key,
            ),
        )

    def _update_custom_tool_adapter_sort_orders(self, order: float) -> None:
        custom_rows = self._fetch_all(
            "SELECT tool_key FROM tool_adapter_configs WHERE is_custom = 1 AND sort_order = 0"
        )
        for row in custom_rows:
            self._execute(
                "UPDATE tool_adapter_configs SET sort_order = ? WHERE tool_key = ?",
                (order, row["tool_key"]),
            )
            order += 1.0

    # ── Scope Preferences ───────────────────────────────

    def get_scope_preference(self, skill_id: str) -> Optional[ScopePreferenceRecord]:
        row = self._fetch_one(
            """SELECT skill_id, scope, project_paths, updated_at
               FROM skill_scope_preference WHERE skill_id = ?""",
            (skill_id,),
        )
        return _row_to_scope_preference(row) if row else None

    def set_scope_preference(self, skill_id: str, scope: str, project_paths: str) -> None:
        now = now_ms()
        self._execute(
            """INSERT INTO skill_scope_preference (skill_id, scope, project_paths, updated_at)
               VALUES (?, ?, ?, ?)
               ON CONFLICT(skill_id) DO UPDATE SET
                 scope = excluded.scope,
                 project_paths = excluded.project_paths,
                 updated_at = excluded.updated_at""",
            (skill_id, scope, project_paths, now),
        )

    def list_all_scope_preferences(self) -> list[ScopePreferenceRecord]:
        rows = self._fetch_all(
            """SELECT skill_id, scope, project_paths, updated_at
               FROM skill_scope_preference"""
        )
        return [_row_to_scope_preference(r) for r in rows]

    # ── Settings ────────────────────────────────────────

    def get_setting(self, key: str) -> Optional[str]:
        row = self._fetch_one("SELECT value FROM settings WHERE key = ?", (key,))
        return row["value"] if row else None

    def set_setting(self, key: str, value: str) -> None:
        self._execute(
            "INSERT INTO settings (key, value) VALUES (?, ?) "
            "ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            (key, value),
        )

    def delete_setting(self, key: str) -> None:
        self._execute("DELETE FROM settings WHERE key = ?", (key,))

    # ── Skills CRUD ─────────────────────────────────────

    def upsert_skill(self, record: SkillRecord) -> None:
        # 新增 skill 时如果 sort_order 为 0，则设为 MAX + 1
        # 已存在的 skill：保留已有 sort_order，不重置手动排序
        if record.sort_order == 0:
            existing = self._fetch_one(
                "SELECT sort_order FROM skills WHERE id = ?", (record.id,)
            )
            if existing is not None:
                record.sort_order = existing["sort_order"]
            else:
                max_row = self._fetch_one("SELECT MAX(sort_order) AS m FROM skills")
                record.sort_order = (max_row["m"] or 0) + 1.0

        self._execute(
            """INSERT INTO skills (
              id, name, description, frontmatter_extra, version, author, license,
              category, homepage, skill_file_count, skill_dir_size,
              source_type, source_ref, source_subpath,
              source_revision, source_url, community_path, content_hash, created_at, updated_at,
              last_sync_at, last_seen_at, status, sort_order
            ) VALUES (
              ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
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
              sort_order = excluded.sort_order""",
            (
                record.id, record.name, record.description,
                record.frontmatter_extra, record.version, record.author,
                record.license, record.category, record.homepage,
                record.skill_file_count, record.skill_dir_size,
                record.source_type,
                record.source_ref, record.source_subpath, record.source_revision,
                record.source_url,
                record.community_path, record.content_hash, record.created_at,
                record.updated_at, record.last_sync_at, record.last_seen_at,
                record.status, record.sort_order,
            ),
        )

    def get_skill_by_content_hash(self, content_hash: str) -> Optional[SkillRecord]:
        """按 content_hash 查重，避免全表扫描"""
        if not content_hash:
            return None
        row = self._fetch_one(
            """SELECT id, name, description, frontmatter_extra, version, author, license,
                      category, homepage, skill_file_count, skill_dir_size,
                      source_type, source_ref, source_subpath,
                      source_revision, community_path, content_hash, created_at,
                      updated_at, last_sync_at, last_seen_at, status, sort_order
               FROM skills WHERE content_hash = ?""",
            (content_hash,),
        )
        return _row_to_skill(row) if row else None

    def list_skills(self, sort: str = "manual") -> list[SkillRecord]:
        order_by = {
            "manual": "sort_order ASC, id ASC",
            "updated": "updated_at DESC",
            "name": "LOWER(name) ASC",
        }.get(sort, "sort_order ASC, id ASC")
        rows = self._fetch_all(
            f"""SELECT id, name, description, frontmatter_extra, version, author, license,
                      category, homepage, skill_file_count, skill_dir_size,
                      source_type, source_ref, source_subpath,
                      source_revision, source_url, community_path, content_hash, created_at,
                      updated_at, last_sync_at, last_seen_at, status, sort_order
               FROM skills ORDER BY {order_by}"""
        )
        return [_row_to_skill(r) for r in rows]

    def get_skill_by_id(self, skill_id: str) -> Optional[SkillRecord]:
        row = self._fetch_one(
            """SELECT id, name, description, frontmatter_extra, version, author, license,
                      category, homepage, skill_file_count, skill_dir_size,
                      source_type, source_ref, source_subpath,
                      source_revision, source_url, community_path, content_hash, created_at,
                      updated_at, last_sync_at, last_seen_at, status, sort_order
               FROM skills WHERE id = ? LIMIT 1""",
            (skill_id,),
        )
        return _row_to_skill(row) if row else None

    def get_skill_by_community_path(self, community_path: str) -> Optional[SkillRecord]:
        row = self._fetch_one(
            """SELECT id, name, description, frontmatter_extra, version, author, license,
                      category, homepage, skill_file_count, skill_dir_size,
                      source_type, source_ref, source_subpath,
                      source_revision, source_url, community_path, content_hash, created_at,
                      updated_at, last_sync_at, last_seen_at, status, sort_order
               FROM skills WHERE community_path = ? LIMIT 1""",
            (community_path,),
        )
        return _row_to_skill(row) if row else None

    def update_skill_description(self, skill_id: str, description: Optional[str]) -> None:
        self._execute(
            "UPDATE skills SET description = ? WHERE id = ?",
            (description, skill_id),
        )

    def update_skill_source_url(self, skill_id: str, source_url: Optional[str]) -> None:
        now = now_ms()
        self._execute(
            "UPDATE skills SET source_url = ?, updated_at = ? WHERE id = ?",
            (source_url, now, skill_id),
        )

    def delete_skill(self, skill_id: str) -> None:
        self._execute("DELETE FROM skills WHERE id = ?", (skill_id,))

    # ── Skill Usage ───────────────────────────────────────

    def record_skill_view(self, skill_id: str) -> None:
        """记录或更新技能查看次数和时间"""
        now = now_ms()
        existing = self._fetch_one(
            "SELECT id, view_count FROM skill_usage WHERE skill_id = ? AND tool = ?",
            (skill_id, "view"),
        )
        if existing:
            self._execute(
                "UPDATE skill_usage SET view_count = view_count + 1, last_viewed_at = ? WHERE id = ?",
                (now, existing["id"]),
            )
        else:
            self._execute(
                """INSERT INTO skill_usage (skill_id, tool, sync_count, last_synced_at, last_viewed_at, view_count)
                   VALUES (?, 'view', 0, NULL, ?, 1)""",
                (skill_id, now),
            )

    def record_skill_sync(self, skill_id: str, tool: str) -> None:
        """记录或更新技能同步次数和时间"""
        now = now_ms()
        existing = self._fetch_one(
            "SELECT id FROM skill_usage WHERE skill_id = ? AND tool = ?",
            (skill_id, tool),
        )
        if existing:
            self._execute(
                "UPDATE skill_usage SET sync_count = sync_count + 1, last_synced_at = ? WHERE id = ?",
                (now, existing["id"]),
            )
        else:
            self._execute(
                """INSERT INTO skill_usage (skill_id, tool, sync_count, last_synced_at, last_viewed_at, view_count)
                   VALUES (?, ?, 1, ?, NULL, 0)""",
                (skill_id, tool, now),
            )

    def get_skill_usage(self, skill_id: str) -> list[SkillUsageRecord]:
        """获取技能的所有使用记录"""
        rows = self._fetch_all(
            """SELECT id, skill_id, tool, sync_count, last_synced_at, last_viewed_at, view_count
               FROM skill_usage WHERE skill_id = ?
               ORDER BY tool ASC""",
            (skill_id,),
        )
        return [_row_to_skill_usage(r) for r in rows]

    # ── Skill Targets ───────────────────────────────────

    def upsert_skill_target(self, record: SkillTargetRecord) -> None:
        self._execute(
            """INSERT INTO skill_targets (
              id, skill_id, tool, scope, project_path, target_path,
              mode, status, last_error, synced_at, target_content_hash, target_updated_at,
              suite_skill_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT DO UPDATE SET
              target_path = excluded.target_path,
              mode = excluded.mode,
              status = excluded.status,
              last_error = excluded.last_error,
              synced_at = excluded.synced_at,
              target_content_hash = excluded.target_content_hash,
              target_updated_at = excluded.target_updated_at,
              suite_skill_id = excluded.suite_skill_id""",
            (
                record.id, record.skill_id, record.tool, record.scope,
                record.project_path, record.target_path, record.mode,
                record.status, record.last_error, record.synced_at,
                record.target_content_hash, record.target_updated_at,
                record.suite_skill_id,
            ),
        )

    def list_skill_targets(self, skill_id: str) -> list[SkillTargetRecord]:
        rows = self._fetch_all(
            """SELECT id, skill_id, tool, scope, project_path, target_path,
                      mode, status, last_error, synced_at,
                      target_content_hash, target_updated_at, suite_skill_id
               FROM skill_targets WHERE skill_id = ?
               ORDER BY tool ASC, scope ASC, project_path ASC""",
            (skill_id,),
        )
        return [_row_to_target(r) for r in rows]

    def list_suite_sub_targets(self, suite_skill_id: str) -> list[SkillTargetRecord]:
        """列出属于某个套件的所有子 skill 同步记录。"""
        rows = self._fetch_all(
            """SELECT id, skill_id, tool, scope, project_path, target_path,
                      mode, status, last_error, synced_at,
                      target_content_hash, target_updated_at, suite_skill_id
               FROM skill_targets WHERE suite_skill_id = ?
               ORDER BY tool ASC, scope ASC, project_path ASC""",
            (suite_skill_id,),
        )
        return [_row_to_target(r) for r in rows]

    def list_all_skill_target_paths(self) -> list[tuple[str, str]]:
        rows = self._fetch_all("SELECT tool, target_path FROM skill_targets")
        return [(r["tool"], r["target_path"]) for r in rows]

    def get_skill_target(
        self,
        skill_id: str,
        tool: str,
        scope: str,
        project_path: Optional[str],
    ) -> Optional[SkillTargetRecord]:
        row = self._fetch_one(
            """SELECT id, skill_id, tool, scope, project_path, target_path,
                      mode, status, last_error, synced_at,
                      target_content_hash, target_updated_at, suite_skill_id
               FROM skill_targets
               WHERE skill_id = ? AND tool = ? AND scope = ?
                 AND ((? IS NULL AND project_path IS NULL) OR project_path = ?)""",
            (skill_id, tool, scope, project_path, project_path),
        )
        return _row_to_target(row) if row else None

    def get_skill_target_by_path(self, target_path: str) -> Optional[SkillTargetRecord]:
        row = self._fetch_one(
            """SELECT id, skill_id, tool, scope, project_path, target_path,
                      mode, status, last_error, synced_at,
                      target_content_hash, target_updated_at, suite_skill_id
               FROM skill_targets
               WHERE target_path = ?""",
            (target_path,),
        )
        return _row_to_target(row) if row else None

    def delete_skill_target(
        self,
        skill_id: str,
        tool: str,
        scope: str,
        project_path: Optional[str],
    ) -> None:
        self._execute(
            """DELETE FROM skill_targets
               WHERE skill_id = ? AND tool = ? AND scope = ?
                 AND ((? IS NULL AND project_path IS NULL) OR project_path = ?)""",
            (skill_id, tool, scope, project_path, project_path),
        )

    def delete_suite_targets(self, suite_skill_id: str, tool: str, scope: str, project_path: Optional[str]) -> list[SkillTargetRecord]:
        """删除属于某个套件的所有子 skill 同步记录，返回被删除的记录。"""
        records = self.list_suite_sub_targets(suite_skill_id)
        matching = [
            r for r in records
            if r.tool == tool and r.scope == scope
            and ((project_path is None and r.project_path is None) or r.project_path == project_path)
        ]
        for r in matching:
            self._execute(
                "DELETE FROM skill_targets WHERE id = ?",
                (r.id,),
            )
        return matching

    # ── Tags ────────────────────────────────────────────

    def create_tag(self, name: str) -> TagRecord:
        normalized = _normalize_tag_name(name)
        now = now_ms()
        max_row = self._fetch_one("SELECT MAX(sort_order) AS m FROM skill_tags")
        sort_order = (max_row["m"] or 0) + 1.0
        try:
            self._execute(
                "INSERT INTO skill_tags (name, created_at, updated_at, sort_order) VALUES (?, ?, ?, ?)",
                (normalized, now, now, sort_order),
            )
        except sqlite3.IntegrityError:
            raise ValueError(f"tag already exists: {normalized}")
        tag_id = self._get_conn().execute("SELECT last_insert_rowid()").fetchone()[0]
        return TagRecord(id=tag_id, name=normalized, sort_order=sort_order)

    def rename_tag(self, tag_id: int, name: str) -> TagRecord:
        normalized = _normalize_tag_name(name)
        now = now_ms()
        try:
            cur = self._execute(
                "UPDATE skill_tags SET name = ?, updated_at = ? WHERE id = ?",
                (normalized, now, tag_id),
            )
        except sqlite3.IntegrityError:
            raise ValueError(f"tag already exists: {normalized}")
        if cur.rowcount == 0:
            raise ValueError(f"tag not found: {tag_id}")
        return TagRecord(id=tag_id, name=normalized)

    def delete_tag(self, tag_id: int) -> None:
        self._execute("DELETE FROM skill_tags WHERE id = ?", (tag_id,))

    def list_tags_with_counts(self, source_type: str | None = None, sort: str = "name") -> list[TagWithCountRecord]:
        source_join_filter = ""
        count_expr = "l.skill_id"
        last_used_expr = "l.created_at"
        params: tuple = ()
        if source_type == "custom":
            source_join_filter = " AND s.source_type = ?"
            count_expr = "s.id"
            last_used_expr = "CASE WHEN s.id IS NOT NULL THEN l.created_at END"
            params = (source_type,)
        elif source_type == "community":
            source_join_filter = " AND s.source_type != ?"
            count_expr = "s.id"
            last_used_expr = "CASE WHEN s.id IS NOT NULL THEN l.created_at END"
            params = ("custom",)
        order_by = {
            "manual": "t.sort_order ASC, t.id ASC",
            "name": "LOWER(t.name) ASC",
        }.get(sort, "LOWER(t.name) ASC")
        rows = self._fetch_all(
            f"""SELECT t.id, t.name, t.sort_order, COUNT({count_expr}) AS skill_count,
                      COALESCE(MAX({last_used_expr}), t.updated_at) AS last_used_at
               FROM skill_tags t
               LEFT JOIN skill_tag_links l ON l.tag_id = t.id
               LEFT JOIN skills s ON s.id = l.skill_id{source_join_filter}
               GROUP BY t.id, t.name, t.sort_order, t.updated_at
               ORDER BY {order_by}""",
            params,
        )
        return [_row_to_tag_with_count(r) for r in rows]

    def get_skill_tags(self, skill_id: str) -> list[TagRecord]:
        rows = self._fetch_all(
            """SELECT t.id, t.name, t.sort_order
               FROM skill_tags t
               INNER JOIN skill_tag_links l ON l.tag_id = t.id
               WHERE l.skill_id = ?
               ORDER BY t.sort_order ASC, LOWER(t.name) ASC""",
            (skill_id,),
        )
        return [TagRecord(id=r["id"], name=r["name"],
                sort_order=r["sort_order"] if "sort_order" in r.keys() else 0.0) for r in rows]

    def set_skill_tags(self, skill_id: str, tag_ids: list[int]) -> None:
        now = now_ms()
        conn = self._get_conn()
        with conn:
            conn.execute("DELETE FROM skill_tag_links WHERE skill_id = ?", (skill_id,))
            for tag_id in tag_ids:
                conn.execute(
                    "INSERT INTO skill_tag_links (skill_id, tag_id, created_at) "
                    "VALUES (?, ?, ?)",
                    (skill_id, tag_id, now),
                )

    def list_untagged_skill_ids(self, source_type: str | None = None) -> list[str]:
        source_filter = ""
        params: tuple = ()
        if source_type == "custom":
            source_filter = " AND s.source_type = ?"
            params = (source_type,)
        elif source_type == "community":
            source_filter = " AND s.source_type != ?"
            params = ("custom",)
        rows = self._fetch_all(
            f"""SELECT s.id FROM skills s
               WHERE NOT EXISTS (
                 SELECT 1 FROM skill_tag_links l WHERE l.skill_id = s.id
               ){source_filter}
               ORDER BY s.updated_at DESC""",
            params,
        )
        return [r["id"] for r in rows]

    # 工具 Skills 缓存

    def get_tool_scan_state(self, tool_key: str) -> Optional[ToolScanStateRecord]:
        row = self._fetch_one(
            """SELECT tool_key, tool_name, installed, skills_dir, supports_project_scope,
                      dir_mtime_ns, scanned_at, first_seen_at
               FROM tool_scan_state WHERE tool_key = ?""",
            (tool_key,),
        )
        return _row_to_tool_scan_state(row) if row else None

    def list_tool_skill_cache(self, tool_key: str) -> list[ToolSkillCacheRecord]:
        rows = self._fetch_all(
            """SELECT tool_key, name, skill_path, is_link, link_target, description,
                      in_community_repo, skill_mtime_ns, scanned_at
               FROM tool_skill_cache
               WHERE tool_key = ?
               ORDER BY LOWER(name) ASC""",
            (tool_key,),
        )
        return [_row_to_tool_skill_cache(r) for r in rows]

    def replace_tool_skill_cache(
        self,
        *,
        tool_key: str,
        tool_name: str,
        installed: bool,
        skills_dir: Optional[str],
        supports_project_scope: bool,
        dir_mtime_ns: Optional[int],
        scanned_at: int,
        entries: list[dict[str, Any]],
    ) -> None:
        conn = self._get_conn()
        with conn:
            conn.execute(
                """INSERT INTO tool_scan_state (
                     tool_key, tool_name, installed, skills_dir, supports_project_scope,
                     dir_mtime_ns, scanned_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?)
                   ON CONFLICT(tool_key) DO UPDATE SET
                     tool_name = excluded.tool_name,
                     installed = excluded.installed,
                     skills_dir = excluded.skills_dir,
                     supports_project_scope = excluded.supports_project_scope,
                     dir_mtime_ns = excluded.dir_mtime_ns,
                     scanned_at = excluded.scanned_at
                     -- first_seen_at 保持原值，不覆盖""",
                (
                    tool_key, tool_name, 1 if installed else 0, skills_dir,
                    1 if supports_project_scope else 0, dir_mtime_ns, scanned_at,
                ),
            )
            conn.execute("DELETE FROM tool_skill_cache WHERE tool_key = ?", (tool_key,))
            conn.executemany(
                """INSERT INTO tool_skill_cache (
                     tool_key, skill_path, name, is_link, link_target, description,
                     in_community_repo, skill_mtime_ns, scanned_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
                [
                    (
                        tool_key,
                        entry["path"],
                        entry["name"],
                        1 if entry.get("is_link") else 0,
                        entry.get("link_target"),
                        entry.get("description"),
                        1 if entry.get("in_community_repo") else 0,
                        entry.get("skill_mtime_ns"),
                        scanned_at,
                    )
                    for entry in entries
                ],
            )

    def clear_tool_skill_cache(self, tool_key: str) -> None:
        conn = self._get_conn()
        with conn:
            conn.execute("DELETE FROM tool_skill_cache WHERE tool_key = ?", (tool_key,))
            conn.execute("DELETE FROM tool_scan_state WHERE tool_key = ?", (tool_key,))

    # ── Recent Projects ──────────────────────────────────

    def list_recent_projects(self, limit: int = 8) -> list[str]:
        rows = self._fetch_all(
            "SELECT project_path FROM recent_projects ORDER BY last_used_at DESC LIMIT ?",
            (limit,),
        )
        return [r["project_path"] for r in rows]

    def touch_recent_project(self, project_path: str) -> None:
        now = now_ms()
        self._execute(
            """INSERT INTO recent_projects (project_path, last_used_at)
               VALUES (?, ?)
               ON CONFLICT(project_path) DO UPDATE SET last_used_at = excluded.last_used_at""",
            (project_path, now),
        )
        # LRU 淘汰：保留最近 8 条，删除多余的
        self._execute(
            """DELETE FROM recent_projects WHERE id NOT IN (
                 SELECT id FROM recent_projects ORDER BY last_used_at DESC LIMIT 8
               )"""
        )

    def mark_tool_first_seen(self, tool_key: str) -> Optional[int]:
        """为首次出现的工具设置 first_seen_at；已设置则跳过。新增时返回已设置的 first_seen_at，否则返回 None。"""
        state = self.get_tool_scan_state(tool_key)
        if not state:
            return None
        if state.first_seen_at is not None:
            return None
        now = now_ms()
        self._execute(
            "UPDATE tool_scan_state SET first_seen_at = ? WHERE tool_key = ?",
            (now, tool_key),
        )
        return now

    # ── 工具 Adapter 配置 ────────────────────────────────

    def list_tool_adapter_configs(self) -> list[ToolAdapterConfigRecord]:
        rows = self._fetch_all(
            """SELECT tool_key, display_name, skills_dir, detect_dir,
                      project_skills_dir,
                      supports_symlink, supports_junction, force_copy,
                      supports_project_scope, is_custom, enabled, sort_order, updated_at
               FROM tool_adapter_configs
               WHERE enabled = 1
               ORDER BY sort_order ASC, is_custom ASC"""
        )
        return [_row_to_tool_adapter_config(r) for r in rows]

    def upsert_tool_adapter_config(self, record: ToolAdapterConfigRecord) -> None:
        # 新增 tool config 时如果 sort_order 为 0，则设为 MAX + 1
        # 已存在的 tool config：保留已有 sort_order，不重置手动排序
        if record.sort_order == 0:
            existing = self._fetch_one(
                "SELECT sort_order FROM tool_adapter_configs WHERE tool_key = ?", (record.tool_key,)
            )
            if existing is not None:
                record.sort_order = existing["sort_order"]
            else:
                max_row = self._fetch_one("SELECT MAX(sort_order) AS m FROM tool_adapter_configs")
                record.sort_order = (max_row["m"] or 0) + 1.0

        self._execute(
            """INSERT INTO tool_adapter_configs (
                 tool_key, display_name, skills_dir, detect_dir,
                 project_skills_dir,
                 supports_symlink, supports_junction, force_copy,
                 supports_project_scope, is_custom, enabled, sort_order, updated_at
               ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(tool_key) DO UPDATE SET
                 display_name = excluded.display_name,
                 skills_dir = excluded.skills_dir,
                 detect_dir = excluded.detect_dir,
                 project_skills_dir = excluded.project_skills_dir,
                 supports_symlink = excluded.supports_symlink,
                 supports_junction = excluded.supports_junction,
                 force_copy = excluded.force_copy,
                 supports_project_scope = excluded.supports_project_scope,
                 is_custom = excluded.is_custom,
                 enabled = excluded.enabled,
                 sort_order = excluded.sort_order,
                 updated_at = excluded.updated_at""",
            (
                record.tool_key,
                record.display_name,
                record.skills_dir,
                record.detect_dir,
                record.project_skills_dir,
                1 if record.supports_symlink else 0,
                1 if record.supports_junction else 0,
                1 if record.force_copy else 0,
                None if record.supports_project_scope is None else 1 if record.supports_project_scope else 0,
                1 if record.is_custom else 0,
                1 if record.enabled else 0,
                record.sort_order,
                record.updated_at,
            ),
        )

    def delete_tool_adapter_config(self, tool_key: str) -> None:
        self._execute("DELETE FROM tool_adapter_configs WHERE tool_key = ?", (tool_key,))
        self.clear_tool_skill_cache(tool_key)

    def reset_tool_adapter_to_default(self, tool_key: str) -> bool:
        """将内置工具重置为 config.DEFAULT_TOOL_ADAPTERS 中的默认值。
        返回 True 表示成功重置；False 表示该工具不是内置工具（应使用 delete）。"""
        from core.config import get_default_tool_config
        default_cfg = get_default_tool_config(tool_key)
        if default_cfg is None:
            return False
        now = now_ms()
        # 保留已有 sort_order
        existing = self._fetch_one(
            "SELECT sort_order FROM tool_adapter_configs WHERE tool_key = ?",
            (tool_key,),
        )
        sort_order = existing["sort_order"] if existing else 0.0
        self._execute(
            """INSERT INTO tool_adapter_configs
               (tool_key, display_name, skills_dir, detect_dir, project_skills_dir,
                supports_symlink, supports_junction, force_copy, supports_project_scope,
                is_custom, enabled, sort_order, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 1, ?, ?)
               ON CONFLICT(tool_key) DO UPDATE SET
                 display_name = excluded.display_name,
                 skills_dir = excluded.skills_dir,
                 detect_dir = excluded.detect_dir,
                 project_skills_dir = excluded.project_skills_dir,
                 supports_symlink = excluded.supports_symlink,
                 supports_junction = excluded.supports_junction,
                 force_copy = excluded.force_copy,
                 supports_project_scope = excluded.supports_project_scope,
                 is_custom = 0,
                 enabled = 1,
                 sort_order = excluded.sort_order,
                 updated_at = excluded.updated_at""",
            (
                tool_key,
                default_cfg["display_name"],
                default_cfg["skills_dir"],
                default_cfg["detect_dir"],
                default_cfg.get("project_skills_dir"),
                1 if default_cfg.get("supports_symlink", True) else 0,
                1 if default_cfg.get("supports_junction", True) else 0,
                1 if default_cfg.get("force_copy", False) else 0,
                None if default_cfg.get("supports_project_scope") is None
                else 1 if default_cfg.get("supports_project_scope") else 0,
                sort_order,
                now,
            ),
        )
        self.clear_tool_skill_cache(tool_key)
        return True

    # ── Reorder (批量排序) ──────────────────────────────

    _REORDER_TABLES = {
        "skills": ("skills", "id"),
        "tags": ("skill_tags", "id"),
        "tools": ("tool_adapter_configs", "tool_key"),
    }

    def reorder_entities(self, entity: str, items: list[tuple]) -> None:
        """批量更新 sort_order。items = [(id, sort_order), ...]"""
        table_info = self._REORDER_TABLES.get(entity)
        if table_info is None:
            raise ValueError(f"unknown entity: {entity}")
        table, id_col = table_info
        conn = self._get_conn()
        with conn:
            for entity_id, sort_order in items:
                conn.execute(
                    f"UPDATE {table} SET sort_order = ? WHERE {id_col} = ?",
                    (float(sort_order), entity_id),
                )


# ── Self-healing schema ────────────────────────────────

def _self_heal_schema(conn: sqlite3.Connection) -> None:
    """Ensure every table and column physically exists (idempotent).

    No version numbers. No trust. To add a table or column, add one entry here.
    All DDL uses IF NOT EXISTS / column-missing guards so it's safe to re-run.
    """
    conn.executescript("""
        CREATE TABLE IF NOT EXISTS skills (
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

        CREATE UNIQUE INDEX IF NOT EXISTS idx_skill_usage_skill_tool ON skill_usage(skill_id, tool);
    """)

    # Columns added to existing tables via ALTER (may already exist)
    _add_column_if_missing(conn, "skills", "description", "TEXT NULL")
    _add_column_if_missing(conn, "skills", "source_subpath", "TEXT NULL")
    _add_column_if_missing(conn, "skills", "frontmatter_extra", "TEXT NULL")
    _add_column_if_missing(conn, "skills", "version", "TEXT NULL")
    _add_column_if_missing(conn, "skills", "author", "TEXT NULL")
    _add_column_if_missing(conn, "skills", "license", "TEXT NULL")
    _add_column_if_missing(conn, "skills", "category", "TEXT NULL")
    _add_column_if_missing(conn, "skills", "homepage", "TEXT NULL")
    _add_column_if_missing(conn, "skills", "skill_file_count", "INTEGER NULL")
    _add_column_if_missing(conn, "skills", "skill_dir_size", "INTEGER NULL")
    _add_column_if_missing(conn, "skills", "source_url", "TEXT NULL")
    _add_column_if_missing(conn, "tool_adapter_configs", "project_skills_dir", "TEXT")
    _add_column_if_missing(conn, "skill_targets", "target_content_hash", "TEXT")
    _add_column_if_missing(conn, "skill_targets", "target_updated_at", "INTEGER")
    _add_column_if_missing(conn, "skill_targets", "suite_skill_id", "TEXT NULL")
    _add_column_if_missing(conn, "tool_scan_state", "first_seen_at", "INTEGER")


def _reset_schema_if_incompatible(conn: sqlite3.Connection) -> None:
    if not _has_development_incompatible_schema(conn):
        return

    conn.executescript("""
        PRAGMA foreign_keys = OFF;
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
        PRAGMA foreign_keys = ON;
    """)


def _has_development_incompatible_schema(conn: sqlite3.Connection) -> bool:
    skills_columns = _table_columns(conn, "skills")
    if skills_columns and "community_path" not in skills_columns:
        return True

    cache_columns = _table_columns(conn, "tool_skill_cache")
    return bool(cache_columns) and "in_community_repo" not in cache_columns


def _table_columns(conn: sqlite3.Connection, table: str) -> set[str]:
    return {r[1] for r in conn.execute(f"PRAGMA table_info('{table}')").fetchall()}


def _add_column_if_missing(conn: sqlite3.Connection, table: str, column: str, col_type: str) -> None:
    """Add a column if it doesn't exist — silently skip duplicate."""
    try:
        conn.execute(f"ALTER TABLE {table} ADD COLUMN {column} {col_type}")
    except sqlite3.OperationalError as e:
        if "duplicate column name" not in str(e).lower():
            raise


def _migrate_skill_targets_to_v4_if_old_shape(conn: sqlite3.Connection) -> None:
    """Data-reshaping migration: add scope/project_path columns to skill_targets.

    This is the only migration that cannot be expressed as idempotent DDL
    because it involves copying and transforming row data. Detects old shape
    by checking whether the 'scope' column already exists — safe to re-run.
    """
    columns = {r[1] for r in conn.execute("PRAGMA table_info('skill_targets')").fetchall()}
    if not columns or "scope" in columns:
        return

    conn.executescript("""
        BEGIN;
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
        COMMIT;
    """)


def migrate_legacy_db_if_needed(target_db_path: str) -> None:
    """从旧版 app identifier 迁移数据库"""
    import os as _os
    if _os.path.exists(target_db_path) and _db_has_any_skills(target_db_path):
        return

    data_dir = _get_legacy_data_dir()
    if not data_dir:
        return

    for app_id in LEGACY_APP_IDENTIFIERS:
        legacy = _os.path.join(data_dir, app_id, "skills_hub.db")
        if _os.path.exists(legacy) and legacy != target_db_path:
            parent = _os.path.dirname(target_db_path)
            _os.makedirs(parent, exist_ok=True)
            if _os.path.exists(target_db_path):
                import time
                backup = f"{target_db_path}.bak-{int(time.time())}"
                _os.rename(target_db_path, backup)
            import shutil
            shutil.copy2(legacy, target_db_path)
            return


def _get_legacy_data_dir() -> Optional[str]:
    from core.config import resolve_data_dir
    return resolve_data_dir()


def _db_has_any_skills(db_path: str) -> bool:
    import os as _os
    if not _os.path.exists(db_path):
        return False
    conn = sqlite3.connect(db_path)
    try:
        has_table = conn.execute(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='skills'"
        ).fetchone()[0]
        if not has_table:
            return False
        count = conn.execute("SELECT COUNT(*) FROM skills").fetchone()[0]
        return count > 0
    finally:
        conn.close()


# ── Helpers ─────────────────────────────────────────────

def _row_to_skill(row: sqlite3.Row) -> SkillRecord:
    return SkillRecord(
        id=row["id"], name=row["name"], description=row["description"],
        frontmatter_extra=row["frontmatter_extra"] if "frontmatter_extra" in row.keys() else None,
        version=row["version"] if "version" in row.keys() else None,
        author=row["author"] if "author" in row.keys() else None,
        license=row["license"] if "license" in row.keys() else None,
        category=row["category"] if "category" in row.keys() else None,
        homepage=row["homepage"] if "homepage" in row.keys() else None,
        skill_file_count=row["skill_file_count"] if "skill_file_count" in row.keys() else None,
        skill_dir_size=row["skill_dir_size"] if "skill_dir_size" in row.keys() else None,
        source_type=row["source_type"], source_ref=row["source_ref"],
        source_subpath=row["source_subpath"] if "source_subpath" in row.keys() else None,
        source_revision=row["source_revision"],
        source_url=row["source_url"] if "source_url" in row.keys() else None,
        community_path=row["community_path"], content_hash=row["content_hash"],
        created_at=row["created_at"], updated_at=row["updated_at"],
        last_sync_at=row["last_sync_at"], last_seen_at=row["last_seen_at"],
        status=row["status"],
        sort_order=row["sort_order"] if "sort_order" in row.keys() else 0.0,
    )


def _row_to_target(row: sqlite3.Row) -> SkillTargetRecord:
    return SkillTargetRecord(
        id=row["id"], skill_id=row["skill_id"], tool=row["tool"],
        scope=row["scope"], project_path=row["project_path"],
        target_path=row["target_path"], mode=row["mode"],
        status=row["status"], last_error=row["last_error"],
        synced_at=row["synced_at"],
        target_content_hash=row["target_content_hash"],
        target_updated_at=row["target_updated_at"],
        suite_skill_id=row["suite_skill_id"],
    )


def _row_to_tool_scan_state(row: sqlite3.Row) -> ToolScanStateRecord:
    return ToolScanStateRecord(
        tool_key=row["tool_key"],
        tool_name=row["tool_name"],
        installed=bool(row["installed"]),
        skills_dir=row["skills_dir"],
        supports_project_scope=bool(row["supports_project_scope"]),
        dir_mtime_ns=row["dir_mtime_ns"],
        scanned_at=row["scanned_at"],
        first_seen_at=row["first_seen_at"],
    )


def _row_to_tool_skill_cache(row: sqlite3.Row) -> ToolSkillCacheRecord:
    return ToolSkillCacheRecord(
        tool_key=row["tool_key"],
        name=row["name"],
        path=row["skill_path"],
        is_link=bool(row["is_link"]),
        link_target=row["link_target"],
        description=row["description"],
        in_community_repo=bool(row["in_community_repo"]),
        skill_mtime_ns=row["skill_mtime_ns"],
        scanned_at=row["scanned_at"],
    )


def _row_to_tool_adapter_config(row: sqlite3.Row) -> ToolAdapterConfigRecord:
    supports_project_scope = row["supports_project_scope"]
    return ToolAdapterConfigRecord(
        tool_key=row["tool_key"],
        display_name=row["display_name"],
        skills_dir=row["skills_dir"],
        detect_dir=row["detect_dir"],
        project_skills_dir=row["project_skills_dir"],
        supports_symlink=bool(row["supports_symlink"]),
        supports_junction=bool(row["supports_junction"]),
        force_copy=bool(row["force_copy"]),
        supports_project_scope=None if supports_project_scope is None else bool(supports_project_scope),
        is_custom=bool(row["is_custom"]),
        enabled=bool(row["enabled"]),
        updated_at=row["updated_at"],
        sort_order=row["sort_order"] if "sort_order" in row.keys() else 0.0,
    )


def _normalize_tag_name(name: str) -> str:
    normalized = name.strip()
    if not normalized:
        raise ValueError("tag name cannot be empty")
    return normalized


def _row_to_tag_with_count(row: sqlite3.Row) -> TagWithCountRecord:
    return TagWithCountRecord(
        id=row["id"], name=row["name"],
        skill_count=row["skill_count"], updated_at=row["last_used_at"],
        sort_order=row["sort_order"] if "sort_order" in row.keys() else 0.0,
    )


def _row_to_scope_preference(row: sqlite3.Row) -> ScopePreferenceRecord:
    return ScopePreferenceRecord(
        skill_id=row["skill_id"],
        scope=row["scope"],
        project_paths=row["project_paths"],
        updated_at=row["updated_at"],
    )


def _row_to_skill_usage(row: sqlite3.Row) -> SkillUsageRecord:
    return SkillUsageRecord(
        id=row["id"],
        skill_id=row["skill_id"],
        tool=row["tool"],
        sync_count=row["sync_count"],
        last_synced_at=row["last_synced_at"],
        last_viewed_at=row["last_viewed_at"],
        view_count=row["view_count"],
    )


def now_ms() -> int:
    """返回当前 Unix 时间戳（毫秒）"""
    import time
    return int(time.time() * 1000)


# ── Global Store Singleton ─────────────────────────────

_store_instance: Optional["SkillStore"] = None
_init_lock = threading.Lock()


def get_store() -> "SkillStore":
    """获取全局 SkillStore 单例（线程安全）"""
    global _store_instance
    if _store_instance is None:
        with _init_lock:
            if _store_instance is None:
                from core.config import default_db_path
                db_path = default_db_path()
                migrate_legacy_db_if_needed(db_path)
                _store_instance = SkillStore(db_path)
                _store_instance.ensure_schema()
    return _store_instance
