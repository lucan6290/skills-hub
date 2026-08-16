import os

import pytest

from core.skills.maintenance import scan_sync_health
from core.repo.community_migration import plan_community_repo_migration
from core.db.store import SkillRecord, SkillStore, now_ms


def make_store(tmp_path):
    store = SkillStore(str(tmp_path / "skills_hub.db"))
    store.ensure_schema()
    return store


def make_skill(**overrides):
    now = now_ms()
    defaults = {
        "id": "skill-1",
        "name": "demo",
        "description": None,
        "source_type": "local",
        "source_ref": None,
        "source_subpath": None,
        "source_revision": None,
        "community_path": "",
        "content_hash": None,
        "created_at": now,
        "updated_at": now,
        "last_sync_at": None,
        "last_seen_at": now,
        "status": "active",
    }
    defaults.update(overrides)
    return SkillRecord(**defaults)


def test_community_migration_rejects_polluted_community_path(tmp_path):
    store = make_store(tmp_path)
    community = tmp_path / "community"
    community.mkdir()
    outside = tmp_path / "outside"
    outside.mkdir()
    (outside / "SKILL.md").write_text("---\nname: outside\n---\n", encoding="utf-8")
    store.set_setting("community_repo_path", str(community))
    store.upsert_skill(make_skill(community_path=str(outside)))

    with pytest.raises(ValueError):
        plan_community_repo_migration(store, tmp_path / "new-community")


def test_sync_health_reports_missing_community_dir(tmp_path):
    store = make_store(tmp_path)
    community = tmp_path / "community"
    community.mkdir()
    store.set_setting("community_repo_path", str(community))
    missing = community / "missing-skill"
    store.upsert_skill(make_skill(community_path=str(missing)))

    report = scan_sync_health(store)

    assert any(issue["code"] == "missing_community_dir" for issue in report["issues"])
