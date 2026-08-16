"""工具技能目录扫描与缓存（tool_skill_cache 表）的 core 层实现。"""
from __future__ import annotations

import logging
from pathlib import Path
from typing import Optional

from models.schemas import ToolSkillEntry, ToolSkillsResponse
from core.db.store import now_ms
from core.repo.community import resolve_community_repo_path, resolve_custom_repo_path
from core.skills.installer import parse_skill_md
from core.tools.adapters import adapter_by_key, scan_tool_dir, supports_project_scope
from core.utils.path_safety import is_path_within

logger = logging.getLogger(__name__)


def check_in_managed_repo(skill_path: str) -> bool:
    """判断技能路径是否位于社区或自定义托管仓库内。"""
    from core.db.store import get_store
    store = get_store()
    community = str(resolve_community_repo_path(store.db_path))
    if is_path_within(skill_path, community):
        return True
    custom = str(resolve_custom_repo_path(store.db_path))
    return is_path_within(skill_path, custom)


def path_mtime_ns(path: Optional[str]) -> Optional[int]:
    """返回路径的 mtime（纳秒），路径不存在或不可访问时返回 None。"""
    if not path:
        return None
    try:
        return Path(path).stat().st_mtime_ns
    except OSError:
        return None


def skill_mtime_ns(skill_path: str) -> Optional[int]:
    """返回技能目录与其 SKILL.md 的最新 mtime（纳秒）。"""
    mtimes = []
    for path in (Path(skill_path), Path(skill_path) / "SKILL.md"):
        try:
            mtimes.append(path.stat().st_mtime_ns)
        except OSError:
            continue
    return max(mtimes) if mtimes else None


def cache_payload_from_entries(entries: list[ToolSkillEntry]) -> list[dict]:
    """把技能条目转换为缓存 payload（含 mtime）。"""
    return [
        {
            "name": entry.name,
            "path": entry.path,
            "is_link": entry.is_link,
            "link_target": entry.link_target,
            "description": entry.description,
            "in_community_repo": entry.in_community_repo,
            "skill_mtime_ns": skill_mtime_ns(entry.path),
        }
        for entry in entries
    ]


def build_skill_entries(skills_dir: str, tool_key: str) -> list[ToolSkillEntry]:
    """扫描工具 skills 目录并构建技能条目列表。"""
    from core.db.store import get_store
    store = get_store()
    managed_source_paths = {s.community_path for s in store.list_skills()}

    adapter = adapter_by_key(tool_key)
    if not adapter:
        return []
    detected = scan_tool_dir(adapter, skills_dir)
    entries = []
    for skill in detected:
        desc = None
        try:
            fm = parse_skill_md(skill.path)
            desc = fm.description
        except Exception as e:
            logger.debug("failed to parse SKILL.md for %s: %s", skill.path, e)

        normalized = skill.path.replace("\\", "/")
        in_community = any(
            cp.replace("\\", "/") == normalized
            or normalized.startswith(cp.replace("\\", "/").rstrip("/") + "/")
            for cp in managed_source_paths
        ) or check_in_managed_repo(skill.path)

        # 对于 symlink 技能，额外检查 link_target 是否在托管仓库中
        if not in_community and skill.is_link and skill.link_target:
            in_community = check_in_managed_repo(skill.link_target)

        entries.append(ToolSkillEntry(
            name=skill.name,
            path=skill.path,
            is_link=skill.is_link,
            link_target=skill.link_target,
            description=desc,
            in_community_repo=in_community,
        ))
    return entries


def entries_from_cache(tool_key: str) -> list[ToolSkillEntry]:
    """从 tool_skill_cache 表读取指定工具的技能条目列表。"""
    from core.db.store import get_store
    return [
        ToolSkillEntry(
            name=row.name,
            path=row.path,
            is_link=row.is_link,
            link_target=row.link_target,
            description=row.description,
            in_community_repo=row.in_community_repo,
        )
        for row in get_store().list_tool_skill_cache(tool_key)
    ]


def refresh_tool_cache(adapter, installed: bool, skills_dir: Optional[str]) -> ToolSkillsResponse:
    """重新扫描指定工具并写入技能缓存，返回响应对象。"""
    from core.db.store import get_store

    tool_key = adapter.id.as_key()
    scanned_at = now_ms()
    entries: list[ToolSkillEntry] = []
    dir_mtime_ns = path_mtime_ns(skills_dir)

    if installed and skills_dir:
        entries = build_skill_entries(skills_dir, tool_key)

    get_store().replace_tool_skill_cache(
        tool_key=tool_key,
        tool_name=adapter.display_name,
        installed=installed,
        skills_dir=skills_dir,
        supports_project_scope=supports_project_scope(adapter),
        dir_mtime_ns=dir_mtime_ns,
        scanned_at=scanned_at,
        entries=cache_payload_from_entries(entries),
    )

    return ToolSkillsResponse(
        tool_key=tool_key,
        tool_name=adapter.display_name,
        installed=installed,
        skills_dir=skills_dir,
        supports_project_scope=supports_project_scope(adapter),
        skills=entries,
        cached=False,
        scanned_at=scanned_at,
    )


def cached_tool_response(adapter) -> ToolSkillsResponse:
    """基于 tool_skill_cache 表构造工具技能响应对象。"""
    from core.db.store import get_store

    store = get_store()
    tool_key = adapter.id.as_key()
    state = store.get_tool_scan_state(tool_key)

    return ToolSkillsResponse(
        tool_key=tool_key,
        tool_name=state.tool_name if state else adapter.display_name,
        installed=state.installed if state else False,
        skills_dir=state.skills_dir if state else None,
        supports_project_scope=(
            state.supports_project_scope if state else supports_project_scope(adapter)
        ),
        skills=entries_from_cache(tool_key) if state else [],
        cached=True,
        scanned_at=state.scanned_at if state else None,
    )
