"""reorder API 测试"""
import pytest

from fastapi.testclient import TestClient

import core.db.store as ss_module
from core.db.store import SkillStore, SkillRecord, ToolAdapterConfigRecord, now_ms


@pytest.fixture
def temp_store(tmp_path, monkeypatch):
    """创建临时 store 并替换全局 _store_instance"""
    store = SkillStore(str(tmp_path / "reorder_test.db"))
    store.ensure_schema()
    monkeypatch.setattr(ss_module, "_store_instance", store)
    return store


def make_skill(**overrides):
    now = now_ms()
    defaults = {
        "id": "test-skill",
        "name": "test",
        "source_type": "local",
        "community_path": "/tmp/test",
        "created_at": now,
        "updated_at": now,
        "last_seen_at": now,
        "status": "active",
    }
    defaults.update(overrides)
    return SkillRecord(**defaults)


class TestReorderAPI:
    def test_reorder_skills(self, client, temp_store):
        temp_store.upsert_skill(make_skill(id="s1", name="a", community_path="/tmp/s1"))
        temp_store.upsert_skill(make_skill(id="s2", name="b", community_path="/tmp/s2"))
        temp_store.upsert_skill(make_skill(id="s3", name="c", community_path="/tmp/s3"))

        resp = client.post("/api/reorder", json={
            "entity": "skills",
            "items": [
                {"id": "s3", "sort_order": 1.0},
                {"id": "s1", "sort_order": 2.0},
                {"id": "s2", "sort_order": 3.0},
            ],
        })
        assert resp.status_code == 200
        # 验证 store 中的顺序已更新
        skills = temp_store.list_skills(sort="manual")
        assert [s.id for s in skills[:3]] == ["s3", "s1", "s2"]

    def test_reorder_tags(self, client, temp_store):
        t1 = temp_store.create_tag("a")
        t2 = temp_store.create_tag("b")
        t3 = temp_store.create_tag("c")

        resp = client.post("/api/reorder", json={
            "entity": "tags",
            "items": [
                {"id": str(t3.id), "sort_order": 1.0},
                {"id": str(t1.id), "sort_order": 2.0},
                {"id": str(t2.id), "sort_order": 3.0},
            ],
        })
        assert resp.status_code == 200
        tags = temp_store.list_tags_with_counts(sort="manual")
        assert [t.name for t in tags[:3]] == ["c", "a", "b"]

    def test_reorder_tools(self, client, temp_store):
        c1 = ToolAdapterConfigRecord(
            tool_key="reorder_tool_a", display_name="A", skills_dir="/tmp/a",
            detect_dir="/tmp/ad", project_skills_dir=None,
            supports_symlink=True, supports_junction=True, force_copy=False,
            supports_project_scope=None, is_custom=True, enabled=True,
            updated_at=now_ms(),
        )
        c2 = ToolAdapterConfigRecord(
            tool_key="reorder_tool_b", display_name="B", skills_dir="/tmp/b",
            detect_dir="/tmp/bd", project_skills_dir=None,
            supports_symlink=True, supports_junction=True, force_copy=False,
            supports_project_scope=None, is_custom=True, enabled=True,
            updated_at=now_ms(),
        )
        temp_store.upsert_tool_adapter_config(c1)
        temp_store.upsert_tool_adapter_config(c2)

        resp = client.post("/api/reorder", json={
            "entity": "tools",
            "items": [
                {"id": "reorder_tool_b", "sort_order": 0.5},
                {"id": "reorder_tool_a", "sort_order": 0.6},
            ],
        })
        assert resp.status_code == 200
        configs = temp_store.list_tool_adapter_configs()
        a_idx = next(i for i, c in enumerate(configs) if c.tool_key == "reorder_tool_a")
        b_idx = next(i for i, c in enumerate(configs) if c.tool_key == "reorder_tool_b")
        assert b_idx < a_idx

    def test_reorder_invalid_entity_returns_400(self, client, temp_store):
        resp = client.post("/api/reorder", json={
            "entity": "unknown",
            "items": [],
        })
        assert resp.status_code == 400
