"""设置 API — Community Repo 管理"""
from __future__ import annotations

import json
import logging
import os
import subprocess
import sys
from pathlib import Path

from fastapi import APIRouter, Depends, HTTPException

from models.schemas import (
    OpenSettingsFolderRequest,
    SetAutoCheckUpdateRequest,
    SetCommunityRepoPathRequest,
    SetCustomRepoPathRequest,
)
from api.dependencies import get_skill_store
from core.repo.community import resolve_community_repo_path, resolve_custom_repo_path
from core.repo.community_migration import execute_community_repo_migration, plan_community_repo_migration
from core.db.store import SkillStore
from core.utils.path_safety import expand_home

router = APIRouter()

logger = logging.getLogger(__name__)


def _pick_folder_dialog() -> str | None:
    """打开系统原生文件夹选择对话框，返回所选路径。"""
    import tkinter as tk
    from tkinter import filedialog
    root = tk.Tk()
    root.withdraw()
    root.attributes('-topmost', True)
    root.lift()
    try:
        path = filedialog.askdirectory(title="选择文件夹")
        return path if path else None
    finally:
        root.destroy()


@router.get(
    "/api/pick_folder",
    summary="弹出系统文件夹选择对话框",
    description="打开原生文件夹选择对话框并返回用户所选路径，未选择时返回空。",
    tags=["Settings"],
)
async def pick_folder():
    """弹出系统文件夹选择对话框并返回所选路径。"""
    return {"path": _pick_folder_dialog()}


def _open_folder(path: Path) -> None:
    """跨平台在系统文件管理器中打开指定目录。"""
    if sys.platform == "win32":
        os.startfile(str(path))  # type: ignore[attr-defined]
    elif sys.platform == "darwin":
        subprocess.Popen(["open", str(path)])
    else:
        subprocess.Popen(["xdg-open", str(path)])


def _check_dir_writable(path: Path) -> bool:
    """校验目录可读写：存在 + 可读 + 可写（通过写探针验证）。"""
    if not path.is_dir():
        return False
    if not os.access(path, os.R_OK | os.W_OK):
        return False
    probe = path / f".write_probe_{os.getpid()}"
    try:
        probe.write_text("")
        probe.unlink()
        return True
    except OSError:
        return False


@router.post(
    "/api/open_settings_folder",
    summary="打开设置相关目录",
    description="在系统文件管理器中打开指定目录；目录不存在时返回 400。",
    tags=["Settings"],
)
async def open_settings_folder(req: OpenSettingsFolderRequest):
    """在系统文件管理器中打开指定目录。"""
    path = Path(expand_home(req.path))
    if not path.is_dir():
        raise HTTPException(status_code=400, detail=f"目录不存在：{path}")
    try:
        _open_folder(path)
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))
    return {"ok": True, "path": str(path)}


DEFAULT_SYNC_TOOLS_KEY = "default_sync_tools"


@router.get(
    "/api/get_default_sync_tools",
    summary="获取默认同步工具列表",
    description="返回保存的默认同步工具 ID 列表；未设置或解析失败时返回空列表。",
    tags=["Settings"],
)
async def get_default_sync_tools(store: SkillStore = Depends(get_skill_store)):
    """获取默认同步工具 ID 列表。"""
    raw = store.get_setting(DEFAULT_SYNC_TOOLS_KEY)
    try:
        return json.loads(raw) if raw else []
    except (json.JSONDecodeError, TypeError):
        return []


@router.post(
    "/api/save_default_sync_tools",
    summary="保存默认同步工具列表",
    description="用给定的工具 ID 列表覆盖保存的默认同步工具设置。",
    tags=["Settings"],
)
async def save_default_sync_tools(tool_ids: list[str], store: SkillStore = Depends(get_skill_store)):
    """保存默认同步工具 ID 列表。"""
    store.set_setting(DEFAULT_SYNC_TOOLS_KEY, json.dumps(tool_ids))
    return {"ok": True}


AUTO_CHECK_UPDATE_KEY = "auto_check_update"


@router.get(
    "/api/get_auto_check_update",
    summary="获取是否启动时自动检查更新",
    description="返回启动时自动检查更新开关状态；未设置时默认为 true。",
    tags=["Settings"],
)
async def get_auto_check_update(store: SkillStore = Depends(get_skill_store)):
    """获取自动检查更新开关状态。"""
    raw = store.get_setting(AUTO_CHECK_UPDATE_KEY)
    return raw != "false"


@router.post(
    "/api/set_auto_check_update",
    summary="设置是否启动时自动检查更新",
    description="保存启动时自动检查更新开关状态。",
    tags=["Settings"],
)
async def set_auto_check_update(req: SetAutoCheckUpdateRequest, store: SkillStore = Depends(get_skill_store)):
    """保存自动检查更新开关状态。"""
    store.set_setting(AUTO_CHECK_UPDATE_KEY, "true" if req.enabled else "false")
    return {"ok": True}


@router.get(
    "/api/get_community_repo_path",
    summary="获取社区仓库路径",
    description="返回当前社区仓库的本地目录路径（字符串）。",
    tags=["Settings"],
)
async def get_community_repo_path(store: SkillStore = Depends(get_skill_store)):
    """获取社区仓库本地路径。"""
    path = resolve_community_repo_path(store.db_path)
    return str(path)


@router.post(
    "/api/set_community_repo_path",
    summary="设置社区仓库路径",
    description="设置新的社区仓库根路径；支持 dry_run 预演迁移计划，实际迁移失败时返回 400。",
    tags=["Settings"],
)
async def set_community_repo_path(req: SetCommunityRepoPathRequest, store: SkillStore = Depends(get_skill_store)):
    """设置社区仓库路径（支持 dry_run 预演）。"""
    new_base = Path(expand_home(req.path))
    try:
        if req.dry_run:
            plan = plan_community_repo_migration(store, new_base)
            return {"dry_run": True, **plan}

        result = execute_community_repo_migration(store, new_base)
        return {"dry_run": False, **result}
    except (ValueError, FileNotFoundError, FileExistsError, PermissionError) as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception:
        logger.error("set_community_repo_path failed: path=%s", req.path, exc_info=True)
        raise HTTPException(status_code=400, detail="社区仓库路径设置失败，请稍后重试")


@router.get(
    "/api/get_custom_repo_path",
    summary="获取自定义仓库路径",
    description="返回当前自定义仓库的本地目录路径（字符串）。",
    tags=["Settings"],
)
async def get_custom_repo_path(store: SkillStore = Depends(get_skill_store)):
    """获取自定义仓库本地路径。"""
    return str(resolve_custom_repo_path(store.db_path))


@router.post(
    "/api/set_custom_repo_path",
    summary="设置自定义仓库路径",
    description="设置自定义仓库根路径；目标必须是已存在的目录，否则返回 400。",
    tags=["Settings"],
)
async def set_custom_repo_path(req: SetCustomRepoPathRequest, store: SkillStore = Depends(get_skill_store)):
    """设置自定义仓库本地路径。"""
    new_base = Path(expand_home(req.path))
    if not new_base.is_dir():
        raise HTTPException(status_code=400, detail=f"目录不存在：{new_base}")
    if not _check_dir_writable(new_base):
        raise HTTPException(status_code=400, detail=f"目录不可读写：{new_base}")
    store.set_setting("custom_repo_path", str(new_base))
    empty = not any(new_base.iterdir())
    return {"ok": True, "path": str(new_base), "empty": empty}


@router.post(
    "/api/scan_community_repo",
    summary="扫描社区仓库",
    description="重新扫描社区仓库目录并刷新技能注册表，返回扫描结果。",
    tags=["Settings"],
)
async def scan_community_repo(store: SkillStore = Depends(get_skill_store)):
    """扫描社区仓库并刷新注册表。"""
    from core.repo.scanner import sync_community_repo_registry
    return sync_community_repo_registry(db_path=store.db_path)


@router.post(
    "/api/scan_all_repos",
    summary="扫描全部仓库",
    description="重新扫描社区与自定义仓库并刷新全部技能注册表，返回扫描结果。",
    tags=["Settings"],
)
async def scan_all_repos(store: SkillStore = Depends(get_skill_store)):
    """扫描全部仓库并刷新注册表。"""
    from core.repo.scanner import sync_all_repo_registries
    return sync_all_repo_registries(db_path=store.db_path)


@router.post(
    "/api/reset_general_settings",
    summary="恢复通用默认设置",
    description="清除自定义仓库与社区仓库路径设置，回退到默认目录。",
    tags=["Settings"],
)
async def reset_general_settings(store: SkillStore = Depends(get_skill_store)):
    """清除存储路径设置并返回默认路径。"""
    store.delete_setting("community_repo_path")
    store.delete_setting("custom_repo_path")
    return {
        "ok": True,
        "community_repo_path": str(resolve_community_repo_path(store.db_path)),
        "custom_repo_path": str(resolve_custom_repo_path(store.db_path)),
    }
