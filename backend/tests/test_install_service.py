"""install_service 单元测试（不依赖现有 DB）"""
import pytest

from core.db.store import SkillRecord, SkillStore
from core.skills.installer import install_local_skill
from core.skills.install_service import (
    build_skill_record,
    dedupe_install_result,
)
from models.schemas import InstallResultDto


@pytest.fixture
def store(tmp_path):
    store = SkillStore(str(tmp_path / "test.db"))
    store.ensure_schema()
    yield store
    store.close()


def test_build_skill_record_field_mapping(tmp_path):
    skill_dir = tmp_path / "my-skill"
    skill_dir.mkdir()
    (skill_dir / "SKILL.md").write_text(
        "---\n"
        "name: My Skill\n"
        "description: A test skill\n"
        "version: 1.0.0\n"
        "author: Alice\n"
        "license: MIT\n"
        "category: dev\n"
        "homepage: https://example.com\n"
        "---\n"
        "# My Skill\n",
        encoding="utf-8",
    )
    result = install_local_skill(str(skill_dir), source_type="custom")

    record = build_skill_record(
        skill_id=result.skill_id,
        name=result.name,
        description=result.description,
        frontmatter=result.frontmatter,
        skill_file_count=result.skill_file_count,
        skill_dir_size=result.skill_dir_size,
        source_type="custom",
        source_ref=str(skill_dir),
        community_path=result.community_path,
        content_hash=result.content_hash,
    )

    assert record.id == result.skill_id
    assert record.name == "My Skill"
    assert record.description == "A test skill"
    assert record.version == "1.0.0"
    assert record.author == "Alice"
    assert record.license == "MIT"
    assert record.category == "dev"
    assert record.homepage == "https://example.com"
    assert record.source_type == "custom"
    assert record.source_ref == str(skill_dir)
    assert record.community_path == result.community_path
    assert record.content_hash == result.content_hash
    assert record.skill_file_count == result.skill_file_count
    assert record.skill_dir_size == result.skill_dir_size
    assert record.status == "active"
    assert record.created_at == record.updated_at == record.last_seen_at
    assert record.last_sync_at is None


def test_build_skill_record_name_fallback():
    record = build_skill_record(
        skill_id="id-1",
        name=None,
        source_ref="/some/path/fallback-skill",
        community_path="/some/path/fallback-skill",
    )
    assert record.name == "fallback-skill"


def test_build_skill_record_normalizes_source_type():
    record = build_skill_record(skill_id="id-1", name="n", source_type="Custom")
    assert record.source_type == "community"
    record = build_skill_record(skill_id="id-2", name="n", source_type="custom")
    assert record.source_type == "custom"


def test_dedupe_install_result_by_content_hash(store):
    store.upsert_skill(SkillRecord(
        id="existing-1",
        name="existing",
        source_type="community",
        community_path="/tmp/existing-skill",
        content_hash="same-hash",
        created_at=1,
        updated_at=1,
        last_seen_at=1,
        status="active",
    ))

    result = InstallResultDto(
        skill_id="new-1",
        name="new",
        community_path="/tmp/existing-skill",
        content_hash="same-hash",
    )
    dup = dedupe_install_result(store, result, "community")
    assert dup is not None
    assert dup.skill_id == "existing-1"
    assert dup.name == "existing"
    assert dup.content_hash == "same-hash"


def test_dedupe_install_result_no_match(store):
    result = InstallResultDto(
        skill_id="new-1",
        name="new",
        community_path="/tmp/new-skill",
        content_hash="different-hash",
    )
    assert dedupe_install_result(store, result, "community") is None
