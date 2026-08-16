"""工具状态 API — 对应 Rust get_tool_status command"""
from __future__ import annotations

from fastapi import APIRouter, Depends

from models.schemas import ToolInfo, ToolStatusResponse
from api.dependencies import get_skill_store
from core.tools.adapters import (
    effective_tool_adapters,
    is_tool_installed,
    resolve_default_path,
    supports_project_scope,
    tool_sync_capabilities,
)
from core.db.store import SkillStore

router = APIRouter()


@router.get(
    "/api/get_tool_status",
    response_model=ToolStatusResponse,
    summary="获取工具安装状态",
    description="返回各 AI 工具的安装状态、skills 目录与同步能力，并标记新发现的已安装工具。",
    tags=["Tools"],
)
async def get_tool_status(store: SkillStore = Depends(get_skill_store)):
    """获取各工具的安装与同步能力状态。"""
    adapters = effective_tool_adapters()
    tools = []
    installed_keys = []
    newly = []

    for adapter in adapters:
        ok = is_tool_installed(adapter)
        key = adapter.id.as_key()
        skills_dir = resolve_default_path(adapter)
        capabilities = tool_sync_capabilities(adapter)
        tools.append(ToolInfo(
            key=key,
            label=adapter.display_name,
            installed=ok,
            skills_dir=skills_dir,
            supports_project_scope=capabilities["supports_project_scope"],
            supports_symlink=capabilities["supports_symlink"],
            supports_junction=capabilities["supports_junction"],
            force_copy=capabilities["force_copy"],
        ))
        if ok:
            installed_keys.append(key)
            # 使用 first_seen_at 替代 installed_tools_v1 JSON blob
            first_seen = store.mark_tool_first_seen(key)
            if first_seen is not None:
                # first_seen_at 刚刚被设置（即新增工具）
                state = store.get_tool_scan_state(key)
                if state and state.first_seen_at == first_seen:
                    newly.append(key)

    installed_keys = list(dict.fromkeys(installed_keys))  # dedup

    return ToolStatusResponse(
        tools=tools,
        installed=installed_keys,
        newly_installed=newly,
    )
