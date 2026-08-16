"""工具技能浏览 API — 扫描各 AI 工具的 skills 目录"""
from __future__ import annotations

import asyncio
import logging
import os
import shutil
import subprocess
import sys
from fastapi import APIRouter, Depends, HTTPException
from pathlib import Path
from typing import Optional

from models.schemas import (
    ClearToolSkillsRequest,
    DeleteToolSkillRequest,
    OpenToolFolderRequest,
    ResetToolAdapterConfigRequest,
    SaveToolAdapterConfigRequest,
    SyncToCommunityRequest,
    ToolAdapterConfigResponse,
    ToolSkillsResponse,
)
from api.dependencies import get_skill_store
from core.db.store import SkillStore
from core.tools.adapters import (
    default_tool_adapters,
    effective_tool_adapters,
    is_tool_installed,
    _normalize_tool_key,
    resolve_default_path,
    scan_tool_dir,
    supports_project_scope,
)
from core.tools.skill_cache import cached_tool_response, refresh_tool_cache
from core.skills.install_service import build_skill_record
from core.skills.installer import parse_skill_md
from core.skills.sync_engine import _is_junction, _remove_path_any
from core.repo.community import resolve_community_repo_path
from core.utils.path_safety import require_path_within, safe_child_path, safe_dir_name

router = APIRouter()

logger = logging.getLogger(__name__)


def _require_installed_tool_source(source_path: str) -> Path:
    source = Path(source_path)
    for adapter in effective_tool_adapters():
        if not is_tool_installed(adapter):
            continue
        skills_dir = resolve_default_path(adapter)
        try:
            return require_path_within(source, skills_dir, "source path")
        except ValueError:
            continue
    raise ValueError("source path is not inside an installed tool skills directory")


def _open_folder(path: Path) -> None:
    if sys.platform == "win32":
        os.startfile(str(path))  # type: ignore[attr-defined]
    elif sys.platform == "darwin":
        subprocess.Popen(["open", str(path)])
    else:
        subprocess.Popen(["xdg-open", str(path)])


def _find_duplicate_by_hash(content_hash: Optional[str]):
    if not content_hash:
        return None
    from core.db.store import get_store
    return get_store().get_skill_by_content_hash(content_hash)


def _resolve_path(relative: str) -> str:
    """将相对/绝对路径解析为绝对路径"""
    p = Path(relative).expanduser()
    if p.is_absolute():
        return str(p)
    return str(Path.home() / p)


def _config_differs_from_default(adapter, default_adapter) -> bool:
    """检查当前 DB 配置是否与默认配置不同"""
    if default_adapter is None:
        return False  # 自定义工具不比较
    return (
        adapter.display_name != default_adapter.display_name
        or adapter.relative_skills_dir != default_adapter.relative_skills_dir
        or adapter.relative_detect_dir != default_adapter.relative_detect_dir
        or adapter.project_relative_skills_dir != default_adapter.project_relative_skills_dir
        or adapter.supports_symlink != default_adapter.supports_symlink
        or adapter.supports_junction != default_adapter.supports_junction
        or adapter.force_copy != default_adapter.force_copy
        or supports_project_scope(adapter) != supports_project_scope(default_adapter)
    )


def _adapter_config_response(adapter, default_adapter=None, sort_order: float = 0.0) -> ToolAdapterConfigResponse:
    return ToolAdapterConfigResponse(
        tool_key=adapter.id.as_key(),
        display_name=adapter.display_name,
        skills_dir=_resolve_path(adapter.relative_skills_dir) if adapter.relative_skills_dir else "",
        detect_dir=_resolve_path(adapter.relative_detect_dir) if adapter.relative_detect_dir else "",
        project_skills_dir=adapter.project_relative_skills_dir,
        default_skills_dir=_resolve_path(default_adapter.relative_skills_dir) if default_adapter and default_adapter.relative_skills_dir else None,
        default_detect_dir=_resolve_path(default_adapter.relative_detect_dir) if default_adapter and default_adapter.relative_detect_dir else None,
        supports_symlink=adapter.supports_symlink,
        supports_junction=adapter.supports_junction,
        force_copy=adapter.force_copy,
        supports_project_scope=supports_project_scope(adapter),
        is_custom=adapter.is_custom,
        has_override=_config_differs_from_default(adapter, default_adapter) if not adapter.is_custom else False,
        sort_order=sort_order,
    )


from core.db.store import now_ms as _now_ms


def _tool_response(adapter, refresh: bool = False) -> ToolSkillsResponse:
    if not refresh:
        return cached_tool_response(adapter)

    installed = is_tool_installed(adapter)
    skills_dir = resolve_default_path(adapter) if installed else None
    return refresh_tool_cache(adapter, installed, skills_dir)


@router.get(
    "/api/get_tool_skills",
    response_model=list[ToolSkillsResponse],
    summary="获取工具技能列表",
    description="返回各工具 skills 目录下的技能列表；refresh=true 时重新扫描并写入缓存。",
    tags=["Tools"],
)
async def get_tool_skills(refresh: bool = False):
    """读取工具技能缓存；refresh=true 时重新扫描并写入缓存。"""
    adapters = effective_tool_adapters()
    if refresh:
        return await asyncio.to_thread(
            lambda: [_tool_response(adapter, refresh=True) for adapter in adapters]
        )
    return [cached_tool_response(adapter) for adapter in adapters]


@router.get(
    "/api/get_tool_adapter_configs",
    response_model=list[ToolAdapterConfigResponse],
    summary="获取工具适配器配置",
    description="返回所有工具适配器的当前配置（含默认值与自定义覆盖标记）。",
    tags=["Tools"],
)
async def get_tool_adapter_configs(store: SkillStore = Depends(get_skill_store)):
    """获取工具适配器配置列表。"""
    defaults = {adapter.id.as_key(): adapter for adapter in default_tool_adapters()}
    configs = {c.tool_key: c for c in store.list_tool_adapter_configs()}
    results = []
    for adapter in effective_tool_adapters():
        key = adapter.id.as_key()
        db_config = configs.get(key)
        results.append(
            _adapter_config_response(
                adapter,
                defaults.get(key),
                sort_order=db_config.sort_order if db_config else 0.0,
            )
        )
    return results


@router.post(
    "/api/save_tool_adapter_config",
    summary="保存工具适配器配置",
    description="保存（新增或更新）指定工具适配器的配置并清空其技能缓存；必填字段缺失时返回 400。",
    tags=["Tools"],
)
async def save_tool_adapter_config(req: SaveToolAdapterConfigRequest, store: SkillStore = Depends(get_skill_store)):
    """保存工具适配器配置。"""
    from core.db.store import ToolAdapterConfigRecord

    key = _normalize_tool_key(req.tool_key)
    if not key:
        raise HTTPException(status_code=400, detail="tool key is required")
    if not req.display_name.strip():
        raise HTTPException(status_code=400, detail="display name is required")
    if not req.skills_dir.strip() or not req.detect_dir.strip():
        raise HTTPException(status_code=400, detail="skills and detect paths are required")

    defaults = {adapter.id.as_key(): adapter for adapter in default_tool_adapters()}
    is_custom = req.is_custom and key not in defaults
    record = ToolAdapterConfigRecord(
        tool_key=key,
        display_name=req.display_name.strip(),
        skills_dir=req.skills_dir.strip(),
        detect_dir=req.detect_dir.strip(),
        project_skills_dir=req.project_skills_dir.strip() if req.project_skills_dir else None,
        supports_symlink=req.supports_symlink,
        supports_junction=req.supports_junction,
        force_copy=req.force_copy,
        supports_project_scope=req.supports_project_scope,
        is_custom=is_custom,
        enabled=True,
        updated_at=_now_ms(),
    )
    store.upsert_tool_adapter_config(record)
    store.clear_tool_skill_cache(key)
    return {"ok": True, "tool_key": key}


@router.post(
    "/api/reset_tool_adapter_config",
    summary="重置工具适配器配置",
    description="内置工具重置为默认配置，自定义工具直接删除配置记录。",
    tags=["Tools"],
)
async def reset_tool_adapter_config(req: ResetToolAdapterConfigRequest, store: SkillStore = Depends(get_skill_store)):
    """重置工具适配器配置。"""
    key = _normalize_tool_key(req.tool_key)
    if not key:
        raise HTTPException(status_code=400, detail="tool key is required")
    from core.config import get_default_tool_config
    # 内置工具：重置为 config 中的默认值（保留记录）
    if get_default_tool_config(key) is not None:
        store.reset_tool_adapter_to_default(key)
        return {"ok": True, "tool_key": key, "reset": True}
    # 自定义工具：直接删除记录
    store.delete_tool_adapter_config(key)
    return {"ok": True, "tool_key": key, "deleted": True}


@router.get(
    "/api/get_tool_skills/{tool_key}",
    response_model=ToolSkillsResponse,
    summary="获取指定工具技能",
    description="返回指定工具的 skills 目录技能列表；refresh=true 时重新扫描，未知工具返回 404。",
    tags=["Tools"],
)
async def get_tool_skills_detail(tool_key: str, refresh: bool = False):
    """读取指定工具技能缓存；refresh=true 时重新扫描并写入缓存。"""
    adapters = effective_tool_adapters()
    adapter = None
    for a in adapters:
        if a.id.as_key() == tool_key:
            adapter = a
            break

    if not adapter:
        raise HTTPException(status_code=404, detail=f"unknown tool: {tool_key}")

    if refresh:
        return await asyncio.to_thread(lambda: _tool_response(adapter, refresh=True))
    return cached_tool_response(adapter)


@router.post(
    "/api/delete_tool_skill",
    summary="删除工具目录中的技能",
    description="删除指定工具 skills 目录中的技能（符号链接仅删除链接本身）；同时清理对应同步目标记录。",
    tags=["Tools"],
)
async def delete_tool_skill(req: DeleteToolSkillRequest, store: SkillStore = Depends(get_skill_store)):
    """删除工具目录中的 skill（符号链接只删链接本身，不影响源文件）"""
    from pathlib import Path
    adapters = effective_tool_adapters()
    adapter = next((a for a in adapters if a.id.as_key() == req.tool_key), None)
    if not adapter:
        raise HTTPException(status_code=404, detail=f"unknown tool: {req.tool_key}")
    if not is_tool_installed(adapter):
        raise HTTPException(status_code=400, detail="tool not installed")

    skills_dir = resolve_default_path(adapter)
    try:
        p = require_path_within(Path(req.skill_path), skills_dir, "skill path")
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

    if not p.exists() and not p.is_symlink() and not _is_junction(p):
        raise HTTPException(status_code=404, detail="skill not found")

    try:
        if p.is_symlink() or _is_junction(p):
            try:
                p.unlink()
            except (IsADirectoryError, PermissionError):
                os.rmdir(str(p))
        elif p.is_dir():
            shutil.rmtree(str(p))
        else:
            p.unlink()
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

    target = store.get_skill_target_by_path(str(p))
    if target:
        store.delete_skill_target(target.skill_id, target.tool, target.scope, target.project_path)
    refresh_tool_cache(adapter, True, skills_dir)

    return {"ok": True}


@router.post(
    "/api/open_tool_skills_dir",
    summary="打开工具 skills 目录",
    description="在系统文件管理器中打开指定工具的 skills 目录；目录不存在时先创建。",
    tags=["Tools"],
)
async def open_tool_skills_dir(req: OpenToolFolderRequest):
    """在文件管理器中打开工具 skills 目录。"""
    adapters = effective_tool_adapters()
    adapter = next((a for a in adapters if a.id.as_key() == req.tool_key), None)
    if not adapter:
        raise HTTPException(status_code=404, detail=f"unknown tool: {req.tool_key}")
    if not is_tool_installed(adapter):
        raise HTTPException(status_code=400, detail="tool not installed")

    path = Path(resolve_default_path(adapter))
    try:
        path.mkdir(parents=True, exist_ok=True)
        _open_folder(path)
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

    return {"ok": True, "path": str(path)}


@router.post(
    "/api/skill_to_community_repo",
    summary="同步技能到社区仓库",
    description="将工具目录中的技能复制到中央社区仓库并登记为受管技能；重复内容会被去重。",
    tags=["Tools"],
)
async def skill_to_community_repo(req: SyncToCommunityRequest, store: SkillStore = Depends(get_skill_store)):
    """将工具目录中的 skill 同步到中央仓库"""
    from pathlib import Path
    import uuid
    from core.utils.content_hash import hash_dir

    try:
        source = _require_installed_tool_source(req.source_path)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    if not source.is_dir():
        raise HTTPException(status_code=400, detail="source is not a directory")

    community_path = resolve_community_repo_path(store.db_path)
    community_base = Path(community_path)
    community_base.mkdir(parents=True, exist_ok=True)

    fm = parse_skill_md(source)
    skill_name = req.name or fm.name or source.name

    target_dir = safe_child_path(community_base, safe_dir_name(skill_name), "skill name")
    if target_dir.exists():
        skill_name = safe_dir_name(f"{skill_name}-{uuid.uuid4().hex[:8]}")
        target_dir = safe_child_path(community_base, skill_name, "skill name")

    try:
        shutil.copytree(str(source), str(target_dir))
    except Exception:
        if target_dir.exists():
            shutil.rmtree(str(target_dir))
        raise

    try:
        content_hash = hash_dir(target_dir)
    except Exception:
        content_hash = None

    duplicate = _find_duplicate_by_hash(content_hash)
    if duplicate:
        shutil.rmtree(str(target_dir))
        return {
            "ok": True,
            "deduped": True,
            "skill_id": duplicate.id,
            "name": duplicate.name,
            "community_path": duplicate.community_path,
        }

    skill_id = str(uuid.uuid4())

    record = build_skill_record(
        skill_id=skill_id,
        name=skill_name,
        description=fm.description,
        frontmatter=fm,
        source_type="local",
        source_ref=str(target_dir),
        community_path=str(target_dir),
        content_hash=content_hash,
    )
    store.upsert_skill(record)

    return {"ok": True, "skill_id": skill_id, "name": skill_name, "community_path": str(target_dir)}


@router.post(
    "/api/clear_tool_skills",
    summary="清空工具技能",
    description="删除指定工具 skills 目录下的所有技能；dry_run=true 时仅返回操作计划。",
    tags=["Tools"],
)
async def clear_tool_skills(req: ClearToolSkillsRequest, store: SkillStore = Depends(get_skill_store)):
    """删除工具目录下的所有 skill（符号链接只删链接本身）"""
    from pathlib import Path

    adapters = effective_tool_adapters()
    adapter = None
    for a in adapters:
        if a.id.as_key() == req.tool_key:
            adapter = a
            break
    if not adapter:
        raise HTTPException(status_code=404, detail=f"unknown tool: {req.tool_key}")

    if not is_tool_installed(adapter):
        raise HTTPException(status_code=400, detail="tool not installed")

    skills_dir = resolve_default_path(adapter)
    detected = scan_tool_dir(adapter, skills_dir)

    removed = 0
    operations = []
    for skill in detected:
        p = Path(skill.path)
        try:
            p = require_path_within(p, skills_dir, "skill path")
        except Exception as e:
            logger.debug("skip skill path outside tool dir %s: %s", skill.path, e)
            continue

        operations.append({"action": "delete_tool_skill", "path": str(p), "skill_name": skill.name})
        if req.dry_run:
            continue
        try:
            _remove_path_any(p)
            removed += 1
        except Exception as e:
            logger.debug("failed to remove tool skill %s: %s", p, e)

        target = store.get_skill_target_by_path(str(p))
        if target:
            store.delete_skill_target(target.skill_id, target.tool, target.scope, target.project_path)

    if req.dry_run:
        return {"ok": True, "dry_run": True, "operations": operations, "operation_count": len(operations)}

    refresh_tool_cache(adapter, True, skills_dir)

    return {"ok": True, "dry_run": False, "removed": removed}
