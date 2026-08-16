"""扫描仓库并自动注册未入库的 skills"""
from __future__ import annotations

import logging
import time
import uuid
from dataclasses import replace
from pathlib import Path

from core.utils.content_hash import hash_dir
from core.utils.path_safety import is_path_within, norm_path
from core.repo.community import resolve_community_repo_path, resolve_custom_repo_path
from core.skills.installer import parse_skill_md, is_skill_dir
from core.skills.install_service import build_skill_record
from core.skills.source_paths import infer_source_type_from_path, is_suite_dir, normalize_source_type
from core.db.store import SkillRecord, get_store

logger = logging.getLogger(__name__)


def _is_skill_suite_dir(path: Path) -> bool:
    # any 语义：任一子目录是 skill 即视为套件（仓库扫描的宽松识别）。
    # 注意与 source_paths.is_suite_dir 区分——后者是 all 语义（所有子目录都是 skill 且 ≥2 个），
    # 两者判定条件不同、用途不同，勿合并。
    return any(child.is_dir() and is_skill_dir(child) for child in path.iterdir())


def _upsert_scanned_skill(item: Path, source_type: str, existing_by_path: dict[str, SkillRecord]) -> bool:
    str_path = str(item)
    fm = parse_skill_md(item)
    name = fm.name
    description = fm.description
    content_hash = None
    try:
        content_hash = hash_dir(item)
    except Exception as e:
        logger.warning("failed to hash skill dir %s: %s", item, e)

    now = int(time.time() * 1000)
    existing = existing_by_path.get(norm_path(str_path))
    if existing:
        record = replace(
            existing,
            name=name or item.name,
            description=description,
            frontmatter_extra=fm.frontmatter_extra,
            version=fm.version,
            author=fm.author,
            license=fm.license,
            category=fm.category,
            homepage=fm.homepage,
            source_type=source_type,
            source_ref=str_path,
            community_path=str_path,
            content_hash=content_hash,
            updated_at=now,
            last_seen_at=now,
            status="active",
        )
        get_store().upsert_skill(record)
        return False

    record = build_skill_record(
        skill_id=str(uuid.uuid4()),
        name=name,
        description=description,
        frontmatter=fm,
        source_type=source_type,
        source_ref=str_path,
        community_path=str_path,
        content_hash=content_hash,
    )
    get_store().upsert_skill(record)
    existing_by_path[norm_path(str_path)] = record
    return True


def _scan_and_register_repo(
    repo_path: str | Path | None,
    db_path: str | None,
    source_type: str,
) -> int:
    if repo_path is None:
        base_path = resolve_custom_repo_path(db_path) if source_type == "custom" else resolve_community_repo_path(db_path)
    else:
        base_path = Path(repo_path)

    if not base_path.is_dir():
        return 0

    store = get_store()
    existing_by_path = {norm_path(s.community_path): s for s in store.list_skills() if s.community_path}

    registered = 0

    # 套件根目录：无 SKILL.md，直接子目录全是 skill dirs，作为整体注册
    if source_type == "custom" and is_suite_dir(base_path):
        if _upsert_scanned_skill(base_path, source_type, existing_by_path):
            registered += 1
        logger.info("repo scan finished: source_type=%s repo=%s registered=%d", source_type, base_path, registered)
        return registered

    for item in sorted(base_path.iterdir()):
        if not item.is_dir():
            continue
        if item.name == ".snapshots":
            continue

        if not is_skill_dir(item) and not (
            source_type == "custom" and _is_skill_suite_dir(item)
        ):
            continue

        if _upsert_scanned_skill(item, source_type, existing_by_path):
            registered += 1

    logger.info("repo scan finished: source_type=%s repo=%s registered=%d", source_type, base_path, registered)
    return registered


def _remove_missing_repo_skills(repo_base: Path, source_type: str) -> int:
    store = get_store()
    removed = 0
    for skill in store.list_skills():
        normalized = normalize_source_type(skill.source_type)
        if normalized != source_type:
            continue
        if not skill.community_path:
            continue
        path = Path(skill.community_path)
        if not is_path_within(path, repo_base):
            continue
        if path.exists():
            continue
        store.delete_skill(skill.id)
        removed += 1
    return removed


def remove_missing_community_repo_skills(db_path: str | None = None) -> int:
    return _remove_missing_repo_skills(resolve_community_repo_path(db_path), "community")


def remove_missing_custom_repo_skills(db_path: str | None = None) -> int:
    return _remove_missing_repo_skills(resolve_custom_repo_path(db_path), "custom")


def reconcile_skill_source_types(db_path: str | None = None) -> int:
    store = get_store()
    updated = 0
    now = int(time.time() * 1000)
    for skill in store.list_skills():
        if not skill.community_path:
            continue
        actual_source = infer_source_type_from_path(skill.community_path, store)
        if actual_source is None:
            continue
        if normalize_source_type(skill.source_type) == actual_source:
            continue
        store.upsert_skill(replace(
            skill,
            source_type=actual_source,
            updated_at=now,
            last_seen_at=now,
        ))
        updated += 1
    return updated


def sync_community_repo_registry(
    community_repo_path: str | Path | None = None,
    db_path: str | None = None,
) -> dict[str, int]:
    normalized = reconcile_skill_source_types(db_path)
    removed = remove_missing_community_repo_skills(db_path)
    registered = scan_and_register_community_repo(
        community_repo_path=community_repo_path,
        db_path=db_path,
    )
    return {"removed": removed, "registered": registered, "normalized": normalized}


def sync_custom_repo_registry(
    custom_repo_path: str | Path | None = None,
    db_path: str | None = None,
) -> dict[str, int]:
    normalized = reconcile_skill_source_types(db_path)
    removed = remove_missing_custom_repo_skills(db_path)
    registered = scan_and_register_custom_repo(
        custom_repo_path=custom_repo_path,
        db_path=db_path,
    )
    return {"removed": removed, "registered": registered, "normalized": normalized}


def sync_repo_registry(source_type: str, db_path: str | None = None) -> dict[str, int]:
    if normalize_source_type(source_type) == "custom":
        return sync_custom_repo_registry(db_path=db_path)
    return sync_community_repo_registry(db_path=db_path)


def scan_and_register_community_repo(
    community_repo_path: str | Path | None = None,
    db_path: str | None = None,
) -> int:
    return _scan_and_register_repo(community_repo_path, db_path, "community")


def scan_and_register_custom_repo(
    custom_repo_path: str | Path | None = None,
    db_path: str | None = None,
) -> int:
    return _scan_and_register_repo(custom_repo_path, db_path, "custom")


def sync_all_repo_registries(db_path: str | None = None) -> dict[str, int]:
    normalized = reconcile_skill_source_types(db_path)
    removed = remove_missing_community_repo_skills(db_path) + remove_missing_custom_repo_skills(db_path)
    registered = scan_and_register_community_repo(db_path=db_path) + scan_and_register_custom_repo(db_path=db_path)
    return {"removed": removed, "registered": registered, "normalized": normalized}
