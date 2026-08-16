"""Onboarding — 对应 Rust onboarding.rs
扫描已安装工具中的现有 skills"""
from __future__ import annotations

import logging
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from core.utils.content_hash import hash_dir
from core.tools.adapters import (
    ToolAdapter,
    DetectedSkill,
    effective_tool_adapters,
    is_tool_installed,
    resolve_default_path,
    scan_tool_dir,
)

logger = logging.getLogger(__name__)


@dataclass
class OnboardingVariant:
    tool: str
    name: str
    path: str
    fingerprint: Optional[str]
    is_link: bool
    link_target: Optional[str]


@dataclass
class OnboardingGroup:
    name: str
    variants: list[OnboardingVariant]
    has_conflict: bool


@dataclass
class OnboardingPlan:
    total_tools_scanned: int
    total_skills_found: int
    groups: list[OnboardingGroup]


def _normalize_path_for_compare(p: str) -> str:
    r"""规范化路径用于前缀比较：统一为小写正斜杠，并剥离 Windows 扩展路径前缀。

    不能用 pathlib 的 Path.resolve()：它对带扩展前缀（双反斜杠加问号那种）的
    Windows 路径会误解析、吃掉反斜杠（例如 \\?\C:\Users\... 被解析成 C:Users\...），
    导致 community repo 过滤失灵。这里改用纯字符串规范化后做前缀比较。
    """
    s = str(p).replace("\\", "/").lower()
    for prefix in ("//?/", "//./"):
        if s.startswith(prefix):
            s = s[len(prefix):]
    return s.rstrip("/")


def _path_starts_with(path: str, prefix: str) -> bool:
    """判断 path 是否位于 prefix 目录下（用于识别指向 community repo 的软链接）。

    加 '/' 边界避免 '.skillshub-evil' 误匹配 '.skillshub'。
    """
    np = _normalize_path_for_compare(path)
    nprefix = _normalize_path_for_compare(prefix)
    return np == nprefix or np.startswith(nprefix + "/")


def build_onboarding_plan(
    community_repo_path: Optional[str] = None,
    managed_target_paths: Optional[set[str]] = None,
    custom_repo_path: Optional[str] = None,
) -> OnboardingPlan:
    """扫描所有已安装工具的 skills 目录，分组并检测冲突。

    优先使用 tool_skill_cache 表数据，仅当缓存 stale（dir_mtime_ns 变化）
    或未安装时才回退到文件系统扫描。
    """
    from core.db.store import get_store

    store = get_store()
    adapters = effective_tool_adapters()
    all_variants: list[OnboardingVariant] = []
    scanned = 0

    for adapter in adapters:
        if not is_tool_installed(adapter):
            continue
        scanned += 1

        tool_key = adapter.id.as_key()
        try:
            skills_dir = resolve_default_path(adapter)
        except Exception as e:
            logger.warning("failed to resolve skills dir for tool %s: %s", tool_key, e)
            continue

        # 优先使用缓存数据
        detected = _detect_from_cache(store, adapter, tool_key, skills_dir, custom_repo_path)
        if detected is None:
            # 缓存 stale 或不存在，回退 FS 扫描
            detected = scan_tool_dir(adapter, skills_dir)

        for skill in detected:
            # 过滤 community repo 下的
            if community_repo_path and _path_starts_with(skill.path, community_repo_path):
                continue
            # 过滤已管理的 targets
            if managed_target_paths and skill.path in managed_target_paths:
                continue
            # 过滤已经是软链接且指向已管理源仓库的（已由 Hub 管理）
            if skill.is_link and skill.link_target:
                if community_repo_path and _path_starts_with(skill.link_target, community_repo_path):
                    continue
                if custom_repo_path and _path_starts_with(skill.link_target, custom_repo_path):
                    continue

            # 计算指纹
            fingerprint = None
            try:
                if Path(skill.path).is_dir():
                    fingerprint = hash_dir(skill.path)
            except Exception:
                pass

            all_variants.append(OnboardingVariant(
                tool=skill.tool.value if hasattr(skill.tool, 'value') else str(skill.tool),
                name=skill.name,
                path=skill.path,
                fingerprint=fingerprint,
                is_link=skill.is_link,
                link_target=skill.link_target,
            ))

    # 按名称分组
    groups_map: dict[str, list[OnboardingVariant]] = {}
    for v in all_variants:
        groups_map.setdefault(v.name, []).append(v)

    groups = []
    for name, variants in sorted(groups_map.items()):
        # 检测冲突：相同名称但不同 fingerprint
        fingerprints = [v.fingerprint for v in variants if v.fingerprint]
        has_conflict = len(set(fingerprints)) > 1 and len(fingerprints) > 1
        groups.append(OnboardingGroup(
            name=name,
            variants=variants,
            has_conflict=has_conflict,
        ))

    return OnboardingPlan(
        total_tools_scanned=scanned,
        total_skills_found=len(groups_map),
        groups=groups,
    )


def _detect_from_cache(store, adapter: ToolAdapter, tool_key: str, skills_dir: str, custom_repo_path: Optional[str] = None) -> Optional[list[DetectedSkill]]:
    """从 tool_skill_cache 构造 DetectedSkill 列表。

    缓存有效条件：
    - tool_scan_state 存在且 installed=True
    - dir_mtime_ns 与实际目录 mtime 一致且都不为 None
    - tool_skill_cache 中有数据

    返回 None 表示缓存不可用，调用方应回退 FS 扫描。
    """
    state = store.get_tool_scan_state(tool_key)
    if not state or not state.installed:
        return None

    actual_mtime = _dir_mtime_ns(skills_dir)
    if actual_mtime is None or state.dir_mtime_ns is None:
        return None
    if actual_mtime != state.dir_mtime_ns:
        return None

    cache_entries = store.list_tool_skill_cache(tool_key)
    if not cache_entries:
        return None

    tool_id = adapter.id

    def _clean_prefix(p: Optional[str]) -> Optional[str]:
        if not p:
            return p
        for prefix in ("\\\\?\\", "\\??\\"):
            if p.startswith(prefix):
                p = p[len(prefix):]
        return p

    return [
        DetectedSkill(
            tool=tool_id,
            name=entry.name,
            path=entry.path,
            is_link=entry.is_link,
            link_target=_clean_prefix(entry.link_target) if entry.is_link else None,
        )
        for entry in cache_entries
        if not entry.in_community_repo
        if not (custom_repo_path and entry.is_link and entry.link_target
                and _path_starts_with(_clean_prefix(entry.link_target), custom_repo_path))
    ]


def _dir_mtime_ns(path: Optional[str]) -> Optional[int]:
    if not path:
        return None
    try:
        return os.stat(path).st_mtime_ns
    except OSError:
        return None
