"""同步编排服务：把 skill 同步到工具的完整业务逻辑（不依赖 fastapi）。"""
from __future__ import annotations

import logging
import os
import time
import uuid
from pathlib import Path

from models.schemas import SyncRequest, SyncResultDto
from core.db.store import SkillStore, SkillTargetRecord
from core.utils.path_safety import expand_home, safe_child_path, safe_dir_name
from core.skills.source_paths import resolve_skill_source_path
from core.skills.sync_engine import (
    sync_dir_for_tool_with_overwrite,
    _is_junction,
)
from core.tools.adapters import (
    adapter_by_key,
    adapters_sharing_project_skills_dir,
    adapters_sharing_skills_dir,
    is_tool_installed,
    resolve_default_path,
    resolve_project_path,
    supports_project_scope,
    _normalize_scope,
)
from core.utils.content_hash import hash_dir
from core.error_codes import ErrorCode

logger = logging.getLogger(__name__)


class SkillSyncError(Exception):
    def __init__(self, status_code: int, detail):
        super().__init__(str(detail))
        self.status_code = status_code
        self.detail = detail  # 与原来 HTTPException 的 detail 完全一致，str 或 dict


def _refresh_global_tool_cache(tool_key: str) -> None:
    adapter = adapter_by_key(tool_key)
    if not adapter:
        return
    if not is_tool_installed(adapter):
        return

    from core.tools.skill_cache import refresh_tool_cache

    refresh_tool_cache(adapter, True, resolve_default_path(adapter))


def sync_skill(req: SyncRequest, store: SkillStore) -> SyncResultDto:
    adapter = adapter_by_key(req.tool)
    if not adapter:
        raise SkillSyncError(status_code=400, detail="unknown tool")

    scope = _normalize_scope(req.scope)
    if scope == "project" and not supports_project_scope(adapter):
        raise SkillSyncError(status_code=400, detail={
            "code": ErrorCode.PROJECT_SCOPE_UNSUPPORTED,
            "tool_key": adapter.id.as_key(),
        })

    project_root = None
    if scope == "project":
        raw = req.project_path
        if not raw:
            raise SkillSyncError(status_code=400, detail="project_path is required for project scope")
        expanded = expand_home(raw)
        if not os.path.isdir(expanded):
            raise SkillSyncError(status_code=400, detail=f"project_path must be an existing directory: {expanded}")
        project_root = expanded

    if scope == "global" and not is_tool_installed(adapter):
        raise SkillSyncError(status_code=400, detail={
            "code": ErrorCode.TOOL_NOT_INSTALLED,
            "tool_key": adapter.id.as_key(),
        })

    if project_root:
        tool_root = resolve_project_path(adapter, project_root)
    else:
        tool_root = resolve_default_path(adapter)

    # 确保目录可写
    try:
        os.makedirs(tool_root, exist_ok=True)
    except PermissionError:
        raise SkillSyncError(
            status_code=400,
            detail={
                "code": ErrorCode.TOOL_NOT_WRITABLE,
                "tool": adapter.display_name,
                "path": str(tool_root),
            },
        )

    try:
        target = safe_child_path(tool_root, safe_dir_name(req.name), "skill name")
    except ValueError as e:
        raise SkillSyncError(status_code=400, detail=str(e))

    # 检查是否已同步
    project_path_for_record = project_root
    existing = store.get_skill_target(
        req.skill_id, req.tool, scope, project_path_for_record
    )
    if existing and existing.target_path == str(target) and target.exists():
        return SyncResultDto(
            mode_used=existing.mode,
            target_path=existing.target_path,
        )

    skill = store.get_skill_by_id(req.skill_id)
    if not skill:
        raise SkillSyncError(status_code=404, detail="skill not found")
    try:
        source_path = resolve_skill_source_path(skill, store)
    except ValueError as e:
        raise SkillSyncError(status_code=400, detail=str(e))

    # 判断是否需要覆盖
    overwrite = req.overwrite or False
    if req.overwrite_if_same_content and target.exists():
        # 如果目标是真实目录（非符号链接），说明原始文件还在，需要被替换为符号链接
        if not os.path.islink(str(target)) and not _is_junction(target):
            overwrite = True
        else:
            try:
                source_hash = hash_dir(source_path)
                target_hash = hash_dir(str(target))
                if source_hash == target_hash:
                    overwrite = True
            except Exception as e:
                logger.warning("same-content hash compare failed for %s: %s", target, e)

    try:
        result = sync_dir_for_tool_with_overwrite(req.tool, str(source_path), str(target), overwrite)
    except FileExistsError:
        raise SkillSyncError(status_code=400, detail={
            "code": ErrorCode.TARGET_EXISTS,
            "path": str(target),
        })
    except PermissionError:
        raise SkillSyncError(
            status_code=400,
            detail={
                "code": ErrorCode.TOOL_NOT_WRITABLE,
                "tool": adapter.display_name,
                "path": str(tool_root),
            },
        )
    except Exception as e:
        raise SkillSyncError(status_code=400, detail=str(e))

    # 为共享目录的工具创建记录
    if scope == "project":
        group = adapters_sharing_project_skills_dir(adapter)
    else:
        group = adapters_sharing_skills_dir(adapter)

    for a in group:
        if scope == "global" and not is_tool_installed(a):
            continue
        # 仅在 copy 模式下计算目标内容哈希（symlink/junction 不存在内容不一致问题）
        target_hash = None
        target_hash_time = None
        if result.mode_used.value == "copy":
            try:
                target_hash = hash_dir(str(result.target_path))
                target_hash_time = int(time.time() * 1000)
            except Exception as e:
                logger.warning("failed to hash synced copy target %s: %s", result.target_path, e)
        record = SkillTargetRecord(
            id=str(uuid.uuid4()),
            skill_id=req.skill_id,
            tool=a.id.as_key(),
            scope=scope,
            project_path=project_path_for_record,
            target_path=str(result.target_path),
            mode=result.mode_used.value,
            status="ok",
            last_error=None,
            synced_at=int(time.time() * 1000),
            target_content_hash=target_hash,
            target_updated_at=target_hash_time,
        )
        store.upsert_skill_target(record)
        if scope == "global":
            _refresh_global_tool_cache(a.id.as_key())

    # 记录同步使用统计
    try:
        store.record_skill_sync(req.skill_id, req.tool)
    except Exception as e:
        logger.warning("failed to record skill sync: skill_id=%s tool=%s: %s", req.skill_id, req.tool, e)

    return SyncResultDto(
        mode_used=result.mode_used.value,
        target_path=str(result.target_path),
    )
