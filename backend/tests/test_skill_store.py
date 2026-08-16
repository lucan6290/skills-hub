"""core/skill_store.py 单元测试"""
import os
import sqlite3
import tempfile
import pytest
from core.db.store import SkillStore, SkillRecord, SkillTargetRecord, ToolAdapterConfigRecord, now_ms


@pytest.fixture
def store():
    """创建临时数据库的 SkillStore"""
    fd, path = tempfile.mkstemp(suffix=".db")
    os.close(fd)
    store = SkillStore(path)
    store.ensure_schema()
    yield store
    store.close()
    os.unlink(path)


def make_skill(**overrides) -> SkillRecord:
    now = now_ms()
    defaults = {
        "id": "test-id-1",
        "name": "test-skill",
        "description": None,
        "source_type": "local",
        "source_ref": None,
        "source_subpath": None,
        "source_revision": None,
        "community_path": "/tmp/test-skill",
        "content_hash": None,
        "created_at": now,
        "updated_at": now,
        "last_sync_at": None,
        "last_seen_at": now,
        "status": "ok",
    }
    defaults.update(overrides)
    return SkillRecord(**defaults)


class TestSkillStore:
    def test_ensure_schema_creates_tables(self, store):
        tables = store._fetch_all(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
        )
        names = [r[0] for r in tables]
        assert "skills" in names
        assert "skill_targets" in names
        assert "settings" in names
        assert "skill_tags" in names
        assert "skill_tag_links" in names

    def test_ensure_schema_resets_development_incompatible_schema(self, tmp_path):
        db_path = tmp_path / "legacy.db"
        conn = sqlite3.connect(db_path)
        conn.executescript(
            """
            CREATE TABLE skills (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              source_type TEXT NOT NULL,
              central_path TEXT NOT NULL UNIQUE,
              created_at INTEGER NOT NULL,
              updated_at INTEGER NOT NULL,
              last_seen_at INTEGER NOT NULL,
              status TEXT NOT NULL
            );
            CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO skills (
              id, name, source_type, central_path, created_at, updated_at, last_seen_at, status
            ) VALUES ('legacy', 'legacy', 'community', '/tmp/legacy', 1, 1, 1, 'active');
            INSERT INTO settings (key, value) VALUES ('central_repo_path', '/tmp/community');
            """
        )
        conn.commit()
        conn.close()

        legacy_store = SkillStore(str(db_path))
        try:
            legacy_store.ensure_schema()
            columns = {r[1] for r in legacy_store._fetch_all("PRAGMA table_info('skills')")}
            assert "community_path" in columns
            assert "central_path" not in columns
            assert legacy_store.list_skills() == []
            assert legacy_store.get_setting("central_repo_path") is None
        finally:
            legacy_store.close()

    def test_ensure_schema_delegates_to_focused_schema_steps_in_order(self, tmp_path, monkeypatch):
        expected_steps = [
            "_reset_incompatible_schema",
            "_migrate_data_if_needed",
            "_self_heal_schema_structure",
            "_initialize_sort_order_columns",
            "_initialize_sort_order_data",
            "_initialize_tool_adapter_configs",
        ]
        missing_steps = [name for name in expected_steps if not hasattr(SkillStore, name)]
        assert missing_steps == []

        calls = []
        schema_store = SkillStore(str(tmp_path / "ordered.db"))
        try:
            for name in expected_steps:
                monkeypatch.setattr(schema_store, name, lambda name=name: calls.append(name))

            schema_store.ensure_schema()

            assert calls == expected_steps
        finally:
            schema_store.close()

    def test_upsert_and_list_skills(self, store):
        skill = make_skill()
        store.upsert_skill(skill)
        skills = store.list_skills()
        assert len(skills) == 1
        assert skills[0].name == "test-skill"

    def test_get_skill_by_id(self, store):
        store.upsert_skill(make_skill())
        found = store.get_skill_by_id("test-id-1")
        assert found is not None
        assert found.name == "test-skill"

    def test_get_skill_by_id_not_found(self, store):
        assert store.get_skill_by_id("nonexistent") is None

    def test_delete_skill(self, store):
        store.upsert_skill(make_skill())
        store.delete_skill("test-id-1")
        assert store.get_skill_by_id("test-id-1") is None

    def test_settings_get_set(self, store):
        assert store.get_setting("foo") is None
        store.set_setting("foo", "bar")
        assert store.get_setting("foo") == "bar"

    def test_settings_overwrite(self, store):
        store.set_setting("key", "v1")
        store.set_setting("key", "v2")
        assert store.get_setting("key") == "v2"

    def test_tags_create_and_list(self, store):
        tag = store.create_tag("My Tag")
        assert tag.name == "My Tag"
        tags = store.list_tags_with_counts()
        assert len(tags) == 1
        assert tags[0].name == "My Tag"

    def test_create_duplicate_tag_raises(self, store):
        store.create_tag("mytag")
        with pytest.raises(Exception):
            store.create_tag("mytag")

    def test_rename_tag(self, store):
        tag = store.create_tag("old")
        renamed = store.rename_tag(tag.id, "new")
        assert renamed.name == "new"

    def test_delete_tag(self, store):
        tag = store.create_tag("tmp")
        store.delete_tag(tag.id)
        tags = store.list_tags_with_counts()
        assert len(tags) == 0

    def test_set_and_get_skill_tags(self, store):
        store.upsert_skill(make_skill())
        tag1 = store.create_tag("python")
        tag2 = store.create_tag("web")
        store.set_skill_tags("test-id-1", [tag1.id, tag2.id])
        tags = store.get_skill_tags("test-id-1")
        assert len(tags) == 2
        assert {t.name for t in tags} == {"python", "web"}

    def test_untagged_skill_ids(self, store):
        store.upsert_skill(make_skill(id="s1", name="s1", community_path="/tmp/test-skill-1"))
        store.upsert_skill(make_skill(id="s2", name="s2", community_path="/tmp/test-skill-2"))
        tag = store.create_tag("tag")
        store.set_skill_tags("s1", [tag.id])
        untagged = store.list_untagged_skill_ids()
        assert untagged == ["s2"]

    def test_list_tags_with_counts(self, store):
        store.upsert_skill(make_skill(id="s1", community_path="/tmp/test-skill-1"))
        store.upsert_skill(make_skill(id="s2", community_path="/tmp/test-skill-2"))
        tag = store.create_tag("shared")
        store.set_skill_tags("s1", [tag.id])
        store.set_skill_tags("s2", [tag.id])
        tags = store.list_tags_with_counts()
        assert tags[0].skill_count == 2

    def test_list_tags_with_counts_filters_by_source_type(self, store):
        store.upsert_skill(make_skill(id="community", source_type="local", community_path="/tmp/community-skill"))
        store.upsert_skill(make_skill(id="custom", source_type="custom", community_path="/tmp/custom-skill"))
        shared = store.create_tag("shared")
        community_only = store.create_tag("community-only")
        custom_only = store.create_tag("custom-only")
        store.set_skill_tags("community", [shared.id, community_only.id])
        store.set_skill_tags("custom", [shared.id, custom_only.id])

        community_tags = {t.name: t.skill_count for t in store.list_tags_with_counts("community")}
        custom_tags = {t.name: t.skill_count for t in store.list_tags_with_counts("custom")}

        assert community_tags == {"community-only": 1, "custom-only": 0, "shared": 1}
        assert custom_tags == {"community-only": 0, "custom-only": 1, "shared": 1}

    def test_list_untagged_skill_ids_filters_by_source_type(self, store):
        store.upsert_skill(make_skill(id="community", source_type="local", community_path="/tmp/community-skill"))
        store.upsert_skill(make_skill(id="custom", source_type="custom", community_path="/tmp/custom-skill"))

        assert store.list_untagged_skill_ids("community") == ["community"]
        assert store.list_untagged_skill_ids("custom") == ["custom"]

    def test_upsert_and_list_skill_targets(self, store):
        store.upsert_skill(make_skill())
        target = SkillTargetRecord(
            id="target-1",
            skill_id="test-id-1",
            tool="claude_code",
            scope="global",
            project_path=None,
            target_path="/home/.claude/skills/test-skill",
            mode="symlink",
            status="ok",
            last_error=None,
            synced_at=now_ms(),
            target_content_hash=None,
            target_updated_at=None,
        )
        store.upsert_skill_target(target)
        targets = store.list_skill_targets("test-id-1")
        assert len(targets) == 1
        assert targets[0].tool == "claude_code"

    def test_get_skill_target(self, store):
        store.upsert_skill(make_skill())
        store.upsert_skill_target(SkillTargetRecord(
            id="t1", skill_id="test-id-1", tool="cursor",
            scope="global", project_path=None,
            target_path="/tmp/target", mode="copy",
            status="ok", last_error=None, synced_at=None,
            target_content_hash=None, target_updated_at=None,
        ))
        t = store.get_skill_target("test-id-1", "cursor", "global", None)
        assert t is not None
        assert t.tool == "cursor"

    def test_delete_skill_target(self, store):
        store.upsert_skill(make_skill())
        store.upsert_skill_target(SkillTargetRecord(
            id="t1", skill_id="test-id-1", tool="cursor",
            scope="global", project_path=None,
            target_path="/tmp/target", mode="copy",
            status="ok", last_error=None, synced_at=None,
            target_content_hash=None, target_updated_at=None,
        ))
        store.delete_skill_target("test-id-1", "cursor", "global", None)
        assert store.get_skill_target("test-id-1", "cursor", "global", None) is None

    def test_update_skill_description(self, store):
        store.upsert_skill(make_skill(description=None))
        store.update_skill_description("test-id-1", "new desc")
        skill = store.get_skill_by_id("test-id-1")
        assert skill.description == "new desc"

    # ── sort_order tests ───────────────────────────────

    def test_new_skill_gets_sequential_sort_order(self, store):
        """新 skill 自动获得 MAX(sort_order) + 1"""
        s1 = make_skill(id="s1", name="first", community_path="/tmp/s1")
        s2 = make_skill(id="s2", name="second", community_path="/tmp/s2")
        store.upsert_skill(s1)
        store.upsert_skill(s2)
        skill1 = store.get_skill_by_id("s1")
        skill2 = store.get_skill_by_id("s2")
        assert skill1.sort_order > 0
        assert skill2.sort_order > 0
        assert skill2.sort_order > skill1.sort_order

    def test_upsert_skill_preserves_existing_sort_order(self, store):
        """更新已有 skill 且 sort_order=0 时，保留 DB 中的已有值"""
        store.upsert_skill(make_skill(id="s1", name="original", community_path="/tmp/s1"))
        original = store.get_skill_by_id("s1")
        original_order = original.sort_order
        assert original_order > 0

        store.upsert_skill(make_skill(
            id="s1", name="updated", community_path="/tmp/s1",
            description="should preserve sort_order",
        ))
        updated = store.get_skill_by_id("s1")
        assert updated.name == "updated"
        assert updated.sort_order == original_order

    def test_new_tag_gets_sequential_sort_order(self, store):
        """新 tag 自动获得 MAX(sort_order) + 1"""
        t1 = store.create_tag("alpha")
        t2 = store.create_tag("beta")
        assert t1.sort_order > 0
        assert t2.sort_order > 0
        assert t2.sort_order > t1.sort_order

    def test_ensure_schema_migrates_skills_sort_order(self, tmp_path):
        """旧 DB 缺少 sort_order 列时，ensure_schema 应自动迁移"""
        db_path = tmp_path / "old.db"
        conn = sqlite3.connect(db_path)
        conn.executescript("""
            CREATE TABLE skills (
              id TEXT PRIMARY KEY,
              name TEXT NOT NULL,
              source_type TEXT NOT NULL,
              source_ref TEXT NULL,
              source_revision TEXT NULL,
              community_path TEXT NOT NULL UNIQUE,
              content_hash TEXT NULL,
              created_at INTEGER NOT NULL,
              updated_at INTEGER NOT NULL,
              last_sync_at INTEGER NULL,
              last_seen_at INTEGER NOT NULL,
              status TEXT NOT NULL
            );
            INSERT INTO skills (id, name, source_type, source_ref, source_revision, community_path, created_at, updated_at, last_seen_at, status)
            VALUES ('old1', 'Old Skill 1', 'local', NULL, NULL, '/tmp/old1', 100, 200, 100, 'active');
            INSERT INTO skills (id, name, source_type, source_ref, source_revision, community_path, created_at, updated_at, last_seen_at, status)
            VALUES ('old2', 'Old Skill 2', 'local', NULL, NULL, '/tmp/old2', 100, 300, 100, 'active');
        """)
        conn.commit()
        conn.close()

        old_store = SkillStore(str(db_path))
        try:
            old_store.ensure_schema()
            skills = old_store.list_skills()
            assert len(skills) >= 2
            skill1 = next(s for s in skills if s.id == "old1")
            skill2 = next(s for s in skills if s.id == "old2")
            assert skill1.sort_order > 0
            assert skill2.sort_order > 0
            assert skill2.sort_order < skill1.sort_order
        finally:
            old_store.close()

    def test_ensure_schema_migrates_tags_sort_order(self, tmp_path):
        """旧 DB skill_tags 缺少 sort_order 列时，ensure_schema 应自动迁移"""
        db_path = tmp_path / "old_tags.db"
        conn = sqlite3.connect(db_path)
        conn.executescript("""
            CREATE TABLE skills (
              id TEXT PRIMARY KEY, name TEXT NOT NULL, source_type TEXT NOT NULL,
              community_path TEXT NOT NULL UNIQUE, content_hash TEXT NULL,
              created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
              last_sync_at INTEGER NULL, last_seen_at INTEGER NOT NULL, status TEXT NOT NULL
            );
            CREATE TABLE skill_tags (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              name TEXT NOT NULL UNIQUE COLLATE NOCASE,
              created_at INTEGER NOT NULL,
              updated_at INTEGER NOT NULL
            );
            INSERT INTO skill_tags (name, created_at, updated_at) VALUES ('ZZZ', 100, 100);
            INSERT INTO skill_tags (name, created_at, updated_at) VALUES ('AAA', 100, 100);
        """)
        conn.commit()
        conn.close()

        old_store = SkillStore(str(db_path))
        try:
            old_store.ensure_schema()
            tags = old_store.list_tags_with_counts()
            assert len(tags) >= 2
            aaa = next(t for t in tags if t.name == "AAA")
            zzz = next(t for t in tags if t.name == "ZZZ")
            assert aaa.sort_order > 0
            assert zzz.sort_order > 0
            assert aaa.sort_order < zzz.sort_order
        finally:
            old_store.close()

    def _make_tool_config(self, tool_key="test_tool", display_name="Test Tool", **overrides) -> ToolAdapterConfigRecord:
        defaults = {
            "tool_key": tool_key,
            "display_name": display_name,
            "skills_dir": "/tmp/skills",
            "detect_dir": "/tmp/detect",
            "project_skills_dir": None,
            "supports_symlink": True,
            "supports_junction": True,
            "force_copy": False,
            "supports_project_scope": None,
            "is_custom": False,
            "enabled": True,
            "updated_at": now_ms(),
        }
        defaults.update(overrides)
        return ToolAdapterConfigRecord(**defaults)

    def test_new_tool_config_gets_sequential_sort_order(self, store):
        """新 tool adapter config 自动获得 MAX(sort_order) + 1"""
        c1 = self._make_tool_config(tool_key="tool_a", display_name="A")
        c2 = self._make_tool_config(tool_key="tool_b", display_name="B")
        store.upsert_tool_adapter_config(c1)
        store.upsert_tool_adapter_config(c2)
        configs = store.list_tool_adapter_configs()
        a_config = next(c for c in configs if c.tool_key == "tool_a")
        b_config = next(c for c in configs if c.tool_key == "tool_b")
        assert a_config.sort_order > 0
        assert b_config.sort_order > 0
        assert b_config.sort_order > a_config.sort_order

    def test_upsert_tool_config_preserves_existing_sort_order(self, store):
        """更新已有 tool config 且 sort_order=0 时，保留 DB 中的已有值"""
        c1 = self._make_tool_config(tool_key="preserve_me", display_name="Original")
        store.upsert_tool_adapter_config(c1)
        configs = store.list_tool_adapter_configs()
        original = next(c for c in configs if c.tool_key == "preserve_me")
        original_order = original.sort_order
        assert original_order > 0

        c2 = self._make_tool_config(tool_key="preserve_me", display_name="Updated")
        store.upsert_tool_adapter_config(c2)
        configs = store.list_tool_adapter_configs()
        updated = next(c for c in configs if c.tool_key == "preserve_me")
        assert updated.display_name == "Updated"
        assert updated.sort_order == original_order

    def test_ensure_schema_initializes_tool_adapter_sort_order(self, tmp_path):
        """旧 DB tool_adapter_configs 缺少 sort_order 列时，ensure_schema 应初始化"""
        db_path = tmp_path / "old_tools.db"
        conn = sqlite3.connect(db_path)
        conn.executescript("""
            CREATE TABLE skills (
              id TEXT PRIMARY KEY, name TEXT NOT NULL, source_type TEXT NOT NULL,
              source_ref TEXT NULL, source_revision TEXT NULL,
              community_path TEXT NOT NULL UNIQUE, content_hash TEXT NULL,
              created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
              last_sync_at INTEGER NULL, last_seen_at INTEGER NOT NULL, status TEXT NOT NULL
            );
            CREATE TABLE tool_adapter_configs (
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
              updated_at INTEGER NOT NULL
            );
            INSERT INTO tool_adapter_configs (tool_key, display_name, skills_dir, detect_dir, is_custom, enabled, updated_at)
            VALUES ('old_custom', 'Old Custom Tool', '/tmp', '/tmp', 1, 1, 100);
        """)
        conn.commit()
        conn.close()

        old_store = SkillStore(str(db_path))
        try:
            old_store.ensure_schema()
            configs = old_store.list_tool_adapter_configs()
            old_config = next((c for c in configs if c.tool_key == "old_custom"), None)
            assert old_config is not None
            assert old_config.sort_order > 0
        finally:
            old_store.close()


# ── sort 参数 + reorder_entities 测试 ──────────────────

class TestSortParamAndReorder:
    """Phase 2: list_skills / list_tags_with_counts / get_skill_tags /
    list_tool_adapter_configs 的 sort 参数，以及 reorder_entities 方法。"""

    def test_list_skills_default_sort_is_manual(self, store):
        """list_skills 默认按 sort_order ASC（manual）排序"""
        store.upsert_skill(make_skill(id="s1", name="charlie", community_path="/tmp/s1", updated_at=300))
        store.upsert_skill(make_skill(id="s2", name="alpha", community_path="/tmp/s2", updated_at=100))
        store.upsert_skill(make_skill(id="s3", name="bravo", community_path="/tmp/s3", updated_at=200))
        skills = store.list_skills()  # default = 'manual'
        assert [s.id for s in skills[:3]] == ["s1", "s2", "s3"]

    def test_list_skills_sort_by_updated(self, store):
        store.upsert_skill(make_skill(id="s1", name="charlie", community_path="/tmp/s1", updated_at=300))
        store.upsert_skill(make_skill(id="s2", name="alpha", community_path="/tmp/s2", updated_at=100))
        store.upsert_skill(make_skill(id="s3", name="bravo", community_path="/tmp/s3", updated_at=200))
        skills = store.list_skills(sort="updated")
        assert [s.id for s in skills[:3]] == ["s1", "s3", "s2"]

    def test_list_skills_sort_by_name(self, store):
        store.upsert_skill(make_skill(id="s1", name="charlie", community_path="/tmp/s1"))
        store.upsert_skill(make_skill(id="s2", name="alpha", community_path="/tmp/s2"))
        store.upsert_skill(make_skill(id="s3", name="bravo", community_path="/tmp/s3"))
        skills = store.list_skills(sort="name")
        assert [s.id for s in skills[:3]] == ["s2", "s3", "s1"]

    def test_list_tags_with_counts_default_sort_is_name(self, store):
        store.create_tag("charlie")
        store.create_tag("alpha")
        store.create_tag("bravo")
        tags = store.list_tags_with_counts()  # default = 'name'
        assert [t.name for t in tags] == ["alpha", "bravo", "charlie"]

    def test_list_tags_with_counts_sort_by_manual(self, store):
        store.create_tag("charlie")  # sort_order=1
        store.create_tag("alpha")    # sort_order=2
        store.create_tag("bravo")    # sort_order=3
        tags = store.list_tags_with_counts(sort="manual")
        assert [t.name for t in tags] == ["charlie", "alpha", "bravo"]

    def test_get_skill_tags_ordered_by_sort_order(self, store):
        store.upsert_skill(make_skill())
        t1 = store.create_tag("charlie")
        t2 = store.create_tag("alpha")
        t3 = store.create_tag("bravo")
        store.set_skill_tags("test-id-1", [t1.id, t2.id, t3.id])
        tags = store.get_skill_tags("test-id-1")
        assert [t.name for t in tags] == ["charlie", "alpha", "bravo"]

    def test_list_tool_adapter_configs_ordered_by_sort_order(self, store):
        c1 = ToolAdapterConfigRecord(
            tool_key="sort_zebra", display_name="Z", skills_dir="/tmp/z",
            detect_dir="/tmp/zd", project_skills_dir=None,
            supports_symlink=True, supports_junction=True, force_copy=False,
            supports_project_scope=None, is_custom=False, enabled=True,
            updated_at=now_ms(),
        )
        c2 = ToolAdapterConfigRecord(
            tool_key="sort_alpha", display_name="A", skills_dir="/tmp/a",
            detect_dir="/tmp/ad", project_skills_dir=None,
            supports_symlink=True, supports_junction=True, force_copy=False,
            supports_project_scope=None, is_custom=False, enabled=True,
            updated_at=now_ms(),
        )
        store.upsert_tool_adapter_config(c1)  # sort_order after defaults
        store.upsert_tool_adapter_config(c2)
        configs = store.list_tool_adapter_configs()
        zebra_idx = next(i for i, c in enumerate(configs) if c.tool_key == "sort_zebra")
        alpha_idx = next(i for i, c in enumerate(configs) if c.tool_key == "sort_alpha")
        assert zebra_idx < alpha_idx  # zebra inserted first → lower sort_order

    def test_reorder_skills(self, store):
        store.upsert_skill(make_skill(id="s1", name="a", community_path="/tmp/s1"))
        store.upsert_skill(make_skill(id="s2", name="b", community_path="/tmp/s2"))
        store.upsert_skill(make_skill(id="s3", name="c", community_path="/tmp/s3"))
        store.reorder_entities("skills", [("s3", 1.0), ("s1", 2.0), ("s2", 3.0)])
        skills = store.list_skills(sort="manual")
        assert [s.id for s in skills[:3]] == ["s3", "s1", "s2"]

    def test_reorder_tags(self, store):
        t1 = store.create_tag("a")
        t2 = store.create_tag("b")
        t3 = store.create_tag("c")
        store.reorder_entities("tags", [(t3.id, 1.0), (t1.id, 2.0), (t2.id, 3.0)])
        tags = store.list_tags_with_counts(sort="manual")
        assert [t.name for t in tags[:3]] == ["c", "a", "b"]

    def test_reorder_tools(self, store):
        c1 = ToolAdapterConfigRecord(
            tool_key="reorder_t1", display_name="T1", skills_dir="/tmp/t1",
            detect_dir="/tmp/t1d", project_skills_dir=None,
            supports_symlink=True, supports_junction=True, force_copy=False,
            supports_project_scope=None, is_custom=True, enabled=True,
            updated_at=now_ms(),
        )
        c2 = ToolAdapterConfigRecord(
            tool_key="reorder_t2", display_name="T2", skills_dir="/tmp/t2",
            detect_dir="/tmp/t2d", project_skills_dir=None,
            supports_symlink=True, supports_junction=True, force_copy=False,
            supports_project_scope=None, is_custom=True, enabled=True,
            updated_at=now_ms(),
        )
        store.upsert_tool_adapter_config(c1)
        store.upsert_tool_adapter_config(c2)
        store.reorder_entities("tools", [("reorder_t2", 0.5), ("reorder_t1", 0.6)])
        configs = store.list_tool_adapter_configs()
        t1_idx = next(i for i, c in enumerate(configs) if c.tool_key == "reorder_t1")
        t2_idx = next(i for i, c in enumerate(configs) if c.tool_key == "reorder_t2")
        assert t2_idx < t1_idx

    def test_reorder_unknown_entity_raises(self, store):
        with pytest.raises(ValueError):
            store.reorder_entities("unknown", [("x", 1.0)])

