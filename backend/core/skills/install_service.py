"""安装编排服务：统一的 SkillRecord 构造与安装结果去重/入库。"""
from __future__ import annotations

import logging
import time
from pathlib import Path
from typing import Optional

from models.schemas import InstallResultDto
from core.db.store import SkillRecord, SkillStore
from core.skills.installer import SkillFrontmatter
from core.skills.source_paths import normalize_source_type
from core.skills.sync_engine import _remove_path_any

logger = logging.getLogger(__name__)


def build_skill_record(
    *,
    skill_id: str,
    name: Optional[str] = None,
    description: Optional[str] = None,
    frontmatter: Optional[SkillFrontmatter] = None,
    skill_file_count: Optional[int] = None,
    skill_dir_size: Optional[int] = None,
    source_type: str = "community",
    source_ref: Optional[str] = None,
    source_subpath: Optional[str] = None,
    community_path: str = "",
    content_hash: Optional[str] = None,
) -> SkillRecord:
    """统一构造 SkillRecord，字段映射与各安装/扫描路径保持一致。

    name 为空时回退到 source_ref 对应的目录名；时间戳统一使用当前毫秒，
    status 固定为 "active"。
    """
    now = int(time.time() * 1000)
    resolved_name = name or (Path(source_ref).name if source_ref else "")
    return SkillRecord(
        id=skill_id,
        name=resolved_name,
        description=description,
        frontmatter_extra=frontmatter.frontmatter_extra if frontmatter else None,
        version=frontmatter.version if frontmatter else None,
        author=frontmatter.author if frontmatter else None,
        license=frontmatter.license if frontmatter else None,
        category=frontmatter.category if frontmatter else None,
        homepage=frontmatter.homepage if frontmatter else None,
        skill_file_count=skill_file_count,
        skill_dir_size=skill_dir_size,
        source_type=normalize_source_type(source_type),
        source_ref=source_ref,
        source_subpath=source_subpath,
        source_revision=None,
        community_path=community_path,
        content_hash=content_hash,
        created_at=now,
        updated_at=now,
        last_sync_at=None,
        last_seen_at=now,
        status="active",
    )


def upsert_skill_from_install(result, source_path: str, store, source_type: str = "community") -> InstallResultDto:
    record = build_skill_record(
        skill_id=result.skill_id,
        name=result.name,
        description=result.description,
        frontmatter=getattr(result, "frontmatter", None),
        skill_file_count=getattr(result, "skill_file_count", None),
        skill_dir_size=getattr(result, "skill_dir_size", None),
        source_type=source_type,
        source_ref=source_path,
        community_path=result.community_path,
        content_hash=result.content_hash,
    )
    store.upsert_skill(record)
    return InstallResultDto(
        skill_id=result.skill_id,
        name=result.name,
        community_path=result.community_path,
        content_hash=result.content_hash,
    )


def dedupe_install_result(store, result: InstallResultDto | object, source_type: str = "community"):
    content_hash = getattr(result, "content_hash", None)
    if not content_hash:
        return None
    normalized_source_type = normalize_source_type(source_type)
    for skill in store.list_skills():
        if skill.content_hash != content_hash:
            continue
        if normalize_source_type(skill.source_type) != normalized_source_type:
            continue
        community_path = getattr(result, "community_path", None)
        if normalized_source_type != "custom" and community_path and community_path != skill.community_path:
            try:
                _remove_path_any(Path(community_path))
            except Exception:
                logger.warning(
                    "failed to remove duplicate skill dir %s", community_path, exc_info=True
                )
        return InstallResultDto(
            skill_id=skill.id,
            name=skill.name,
            community_path=skill.community_path,
            content_hash=skill.content_hash,
        )
    return None
