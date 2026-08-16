"""同步 API — 对应 Rust sync_skill_to_tool, unsync_skill_from_tool, sync_skill_dir"""
from __future__ import annotations

import logging
import os
from pathlib import Path

from fastapi import APIRouter, Depends, HTTPException

from models.schemas import (
    SaveRecentProjectRequest,
    ScopePreferenceDto,
    SetScopePreferenceRequest,
    SyncDirRequest,
    SyncRequest,
    SyncResultDto,
    UnsyncRequest,
)
from api.dependencies import get_skill_store
from core.db.store import SkillStore
from core.config import IS_DEV_MODE
from core.utils.path_safety import expand_home, require_path_within
from core.skills.sync_engine import (
    sync_dir_hybrid,
    _remove_path_any,
)
from core.tools.adapters import (
    adapter_by_key,
    adapters_sharing_project_skills_dir,
    adapters_sharing_skills_dir,
    is_tool_installed,
    _normalize_scope,
    _target_base_for_record,
)
from core.skills import sync_service

router = APIRouter()

logger = logging.getLogger(__name__)

RECENT_PROJECTS_SETTING = "recent_projects_v1"




@router.post(
    "/api/sync_skill_dir",
    response_model=SyncResultDto,
    summary="同步技能目录",
    description="仅开发模式可用：将源目录同步到目标目录，并对源路径做社区仓库范围校验。",
    tags=["Sync"],
)
async def sync_skill_dir(req: SyncDirRequest, store: SkillStore = Depends(get_skill_store)):
    """将源目录同步到目标目录（仅开发模式）。"""
    if not IS_DEV_MODE:
        raise HTTPException(status_code=403, detail="endpoint only available in dev mode")
    # 开发模式也做基本路径校验，防止生产环境误设 DEV 模式时的任意文件写入
    community_path = Path(store.get_setting("community_repo_path") or os.path.expanduser("~/.skillshub"))
    require_path_within(Path(req.source_path), community_path, "source_path")
    result = sync_dir_hybrid(req.source_path, req.target_path)
    return SyncResultDto(
        mode_used=result.mode_used.value,
        target_path=str(result.target_path),
    )


@router.post(
    "/api/sync_skill_to_tool",
    response_model=SyncResultDto,
    summary="同步技能到工具",
    description="将指定技能同步到目标 AI 工具，支持全局或项目作用域；失败时返回相应错误码。",
    tags=["Sync"],
)
async def sync_skill_to_tool(req: SyncRequest, store: SkillStore = Depends(get_skill_store)):
    """将技能同步到指定工具。"""
    try:
        return sync_service.sync_skill(req, store)
    except sync_service.SkillSyncError as e:
        raise HTTPException(status_code=e.status_code, detail=e.detail)


@router.post(
    "/api/unsync_skill_from_tool",
    summary="取消技能同步",
    description="从目标工具（或其共享目录组）移除已同步的技能；project 作用域必须提供 project_path。",
    tags=["Sync"],
)
async def unsync_skill_from_tool(req: UnsyncRequest, store: SkillStore = Depends(get_skill_store)):
    """从目标工具取消技能同步。"""
    scope = _normalize_scope(req.scope)

    project_path = None
    if scope == "project":
        raw = req.project_path
        if not raw:
            raise HTTPException(status_code=400, detail="project_path is required for project scope")
        project_path = expand_home(raw)

    # 获取共享目录组的工具
    adapter = adapter_by_key(req.tool)
    if adapter:
        if scope == "project":
            group = adapters_sharing_project_skills_dir(adapter)
        else:
            group = adapters_sharing_skills_dir(adapter)
        group_keys = [a.id.as_key() for a in group]

        if scope == "global":
            any_installed = any(is_tool_installed(a) for a in group)
            if not any_installed:
                return {"ok": True}
    else:
        group_keys = [req.tool]

    removed = False
    for key in group_keys:
        target = store.get_skill_target(req.skill_id, key, scope, project_path)
        if target:
            if not removed:
                try:
                    target_path = require_path_within(
                        Path(target.target_path),
                        _target_base_for_record(target),
                        "target path",
                    )
                except ValueError as e:
                    raise HTTPException(status_code=400, detail=str(e))
                try:
                    _remove_path_any(target_path)
                except Exception:
                    logger.error(
                        "unsync failed to remove target: skill_id=%s tool=%s scope=%s",
                        req.skill_id, req.tool, scope, exc_info=True,
                    )
                    raise HTTPException(status_code=400, detail="取消同步失败，请稍后重试")
                removed = True
            store.delete_skill_target(req.skill_id, key, scope, project_path)
            if scope == "global":
                sync_service._refresh_global_tool_cache(key)

    return {"ok": True}


@router.post(
    "/api/save_recent_project",
    summary="保存最近项目",
    description="记录项目路径到最近项目列表并返回更新后的列表；路径必须为已存在目录。",
    tags=["Sync"],
)
async def save_recent_project(req: SaveRecentProjectRequest, store: SkillStore = Depends(get_skill_store)):
    """保存最近项目路径并返回列表。"""
    expanded = expand_home(req.project_path)
    if not os.path.isdir(expanded):
        raise HTTPException(status_code=400, detail=f"project_path must be an existing directory: {expanded}")

    normalized = str(Path(expanded))
    store.touch_recent_project(normalized)
    return store.list_recent_projects()


@router.get(
    "/api/get_recent_projects",
    summary="获取最近项目列表",
    description="返回最近使用的项目路径列表。",
    tags=["Sync"],
)
async def get_recent_projects(store: SkillStore = Depends(get_skill_store)):
    """获取最近项目列表。"""
    return store.list_recent_projects()


@router.get(
    "/api/get_scope_preferences",
    response_model=list[ScopePreferenceDto],
    summary="获取作用域偏好",
    description="返回所有技能的作用域偏好（全局或项目及其项目路径列表）。",
    tags=["Sync"],
)
async def get_scope_preferences(store: SkillStore = Depends(get_skill_store)):
    """获取技能作用域偏好列表。"""
    records = store.list_all_scope_preferences()
    return [
        ScopePreferenceDto(
            skill_id=r.skill_id,
            scope=r.scope,
            project_paths=r.project_paths,
        )
        for r in records
    ]


@router.post(
    "/api/set_scope_preference",
    summary="设置作用域偏好",
    description="设置指定技能的作用域及其项目路径列表。",
    tags=["Sync"],
)
async def set_scope_preference(req: SetScopePreferenceRequest, store: SkillStore = Depends(get_skill_store)):
    """设置技能的作用域偏好。"""
    store.set_scope_preference(req.skill_id, req.scope, req.project_paths)
    return {"ok": True}
