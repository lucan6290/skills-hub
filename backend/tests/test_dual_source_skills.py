from pathlib import Path

import pytest

from core.skills.maintenance import scan_sync_health
from core.repo.scanner import (
    reconcile_skill_source_types,
    sync_all_repo_registries,
    sync_repo_registry,
)
from core.db.store import SkillRecord, SkillStore, now_ms
from core.skills.installer import install_local_skill
from core.skills.source_paths import normalize_source_type, resolve_skill_source_path


def make_store(tmp_path):
    store = SkillStore(str(tmp_path / "skills_hub.db"))
    store.ensure_schema()
    return store


def make_skill_dir(path: Path, name: str = "demo"):
    path.mkdir(parents=True, exist_ok=True)
    (path / "SKILL.md").write_text(
        f"---\nname: {name}\ndescription: {name} description\n---\n\n# {name}\n",
        encoding="utf-8",
    )
    return path


def make_skill(**overrides):
    now = now_ms()
    defaults = {
        "id": "skill-1",
        "name": "demo",
        "description": None,
        "source_type": "community",
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


def test_normalize_source_type_treats_legacy_local_as_community():
    assert normalize_source_type("custom") == "custom"
    assert normalize_source_type("community") == "community"
    assert normalize_source_type("local") == "community"
    assert normalize_source_type(None) == "community"


def test_resolve_skill_source_path_uses_custom_repo_boundary(tmp_path):
    store = make_store(tmp_path)
    community = tmp_path / "community"
    custom = tmp_path / "custom"
    community.mkdir()
    custom_skill = make_skill_dir(custom / "custom-skill", "custom-skill")
    store.set_setting("community_repo_path", str(community))
    store.set_setting("custom_repo_path", str(custom))

    skill = make_skill(source_type="custom", community_path=str(custom_skill))

    assert resolve_skill_source_path(skill, store) == custom_skill


def test_custom_install_registers_source_without_copying_to_community(tmp_path):
    source = make_skill_dir(tmp_path / "source" / "custom-skill", "custom-skill")
    community = tmp_path / "community"

    result = install_local_skill(str(source), community_repo=community, source_type="custom")

    assert Path(result.community_path) == source
    assert not community.exists()


def test_sync_all_repo_registries_keeps_missing_custom_records_isolated(tmp_path, monkeypatch):
    store = make_store(tmp_path)
    community = tmp_path / "community"
    custom = tmp_path / "custom"
    community_skill = make_skill_dir(community / "community-skill", "community-skill")
    custom_skill = make_skill_dir(custom / "custom-skill", "custom-skill")
    store.set_setting("community_repo_path", str(community))
    store.set_setting("custom_repo_path", str(custom))
    store.upsert_skill(make_skill(id="missing-custom", source_type="custom", community_path=str(custom / "missing")))
    monkeypatch.setattr("core.repo.scanner.get_store", lambda: store)

    result = sync_all_repo_registries(db_path=store.db_path)

    records = store.list_skills()
    by_path = {Path(skill.community_path): skill for skill in records}
    assert store.get_skill_by_id("missing-custom") is None
    assert by_path[community_skill].source_type == "community"
    assert by_path[custom_skill].source_type == "custom"
    assert result["registered"] == 2
    assert result["removed"] == 1


def test_sync_all_repo_registries_registers_custom_suite_dir(tmp_path, monkeypatch):
    store = make_store(tmp_path)
    community = tmp_path / "community"
    custom = tmp_path / "custom"
    suite = custom / "job-hunting-skills"
    make_skill_dir(suite / "interview-prep", "interview-prep")
    make_skill_dir(suite / "resume-polish", "resume-polish")
    store.set_setting("community_repo_path", str(community))
    store.set_setting("custom_repo_path", str(custom))
    monkeypatch.setattr("core.repo.scanner.get_store", lambda: store)

    result = sync_all_repo_registries(db_path=store.db_path)

    records = store.list_skills()
    assert len(records) == 1
    assert Path(records[0].community_path) == suite
    assert records[0].name == "job-hunting-skills"
    assert records[0].source_type == "custom"
    assert result["registered"] == 1


def test_sync_repo_registry_community_does_not_scan_custom_repo(tmp_path, monkeypatch):
    store = make_store(tmp_path)
    community = tmp_path / "community"
    custom = tmp_path / "custom"
    community_skill = make_skill_dir(community / "community-skill", "community-skill")
    custom_skill = make_skill_dir(custom / "custom-skill", "custom-skill")
    store.set_setting("community_repo_path", str(community))
    store.set_setting("custom_repo_path", str(custom))
    monkeypatch.setattr("core.repo.scanner.get_store", lambda: store)

    result = sync_repo_registry("community", db_path=store.db_path)

    records = store.list_skills()
    assert len(records) == 1
    assert Path(records[0].community_path) == community_skill
    assert Path(records[0].community_path) != custom_skill
    assert records[0].source_type == "community"
    assert result["registered"] == 1


def test_reconcile_skill_source_types_uses_repo_path_ownership(tmp_path, monkeypatch):
    store = make_store(tmp_path)
    community = tmp_path / "community"
    custom = tmp_path / "custom"
    dirty_custom_skill = make_skill_dir(custom / "job-hunting-skills" / "work-experience-extractor", "work-experience-extractor")
    dirty_community_skill = make_skill_dir(community / "community-skill", "community-skill")
    store.set_setting("community_repo_path", str(community))
    store.set_setting("custom_repo_path", str(custom))
    store.upsert_skill(make_skill(
        id="dirty-community",
        name="work-experience-extractor",
        source_type="community",
        community_path=str(dirty_custom_skill),
    ))
    store.upsert_skill(make_skill(
        id="dirty-custom",
        name="community-skill",
        source_type="custom",
        community_path=str(dirty_community_skill),
    ))
    monkeypatch.setattr("core.repo.scanner.get_store", lambda: store)

    updated = reconcile_skill_source_types(db_path=store.db_path)

    custom_record = store.get_skill_by_id("dirty-community")
    community_record = store.get_skill_by_id("dirty-custom")
    assert updated == 2
    assert custom_record is not None
    assert community_record is not None
    assert custom_record.source_type == "custom"
    assert community_record.source_type == "community"


def test_sync_repo_registry_updates_existing_record_without_losing_tags(tmp_path, monkeypatch):
    store = make_store(tmp_path)
    community = tmp_path / "community"
    custom = tmp_path / "custom"
    community_skill = make_skill_dir(community / "community-skill", "community-skill")
    tag = store.create_tag("keep-tag")
    store.set_setting("community_repo_path", str(community))
    store.set_setting("custom_repo_path", str(custom))
    store.upsert_skill(make_skill(
        id="existing-community",
        name="old-name",
        description="old description",
        source_type="custom",
        community_path=str(community_skill),
    ))
    store.set_skill_tags("existing-community", [tag.id])
    monkeypatch.setattr("core.repo.scanner.get_store", lambda: store)

    result = sync_repo_registry("community", db_path=store.db_path)

    record = store.get_skill_by_id("existing-community")
    assert result["registered"] == 0
    assert result["normalized"] == 1
    assert record is not None
    assert record.name == "community-skill"
    assert record.source_type == "community"
    assert [tag.name for tag in store.get_skill_tags("existing-community")] == ["keep-tag"]


def test_sync_health_accepts_custom_source_outside_community_repo(tmp_path):
    store = make_store(tmp_path)
    community = tmp_path / "community"
    custom = tmp_path / "custom"
    community.mkdir()
    custom_skill = make_skill_dir(custom / "custom-skill", "custom-skill")
    store.set_setting("community_repo_path", str(community))
    store.set_setting("custom_repo_path", str(custom))
    store.upsert_skill(make_skill(source_type="custom", community_path=str(custom_skill)))

    report = scan_sync_health(store)

    assert not any(issue["code"] == "community_path_outside_repo" for issue in report["issues"])
    assert not any(issue["code"] == "missing_community_dir" for issue in report["issues"])
