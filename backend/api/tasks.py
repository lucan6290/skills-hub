"""Background task API for long-running local operations."""
from __future__ import annotations

from fastapi import APIRouter, HTTPException

from models.schemas import MigrateCommunityRepoTaskRequest, TaskStartResponse
from core.tasks.manager import get_task_manager

router = APIRouter()


@router.get(
    "/api/tasks",
    summary="获取后台任务列表",
    description="返回当前所有后台任务的摘要信息（含状态与进度）。",
    tags=["Tasks"],
)
async def list_tasks():
    """获取后台任务列表。"""
    return [task.to_dict() for task in get_task_manager().list()]


@router.get(
    "/api/tasks/{task_id}",
    summary="获取单个后台任务",
    description="按任务 ID 返回任务详情；任务不存在时返回 404。",
    tags=["Tasks"],
)
async def get_task(task_id: str):
    """获取指定后台任务的详情。"""
    task = get_task_manager().get(task_id)
    if not task:
        raise HTTPException(status_code=404, detail="task not found")
    return task.to_dict()


@router.post(
    "/api/tasks/{task_id}/cancel",
    summary="取消后台任务",
    description="请求取消指定任务；任务不存在或无法取消时返回 404。",
    tags=["Tasks"],
)
async def cancel_task(task_id: str):
    """取消指定后台任务。"""
    if not get_task_manager().cancel(task_id):
        raise HTTPException(status_code=404, detail="task not found")
    return {"ok": True}


@router.post(
    "/api/tasks/get_tool_skills",
    response_model=TaskStartResponse,
    summary="启动工具技能扫描任务",
    description="提交一个后台任务，扫描各 AI 工具的 skills 目录，并返回任务 ID 与初始状态。",
    tags=["Tasks"],
)
async def start_get_tool_skills():
    """启动工具技能扫描后台任务。"""
    import asyncio
    from api.tools.tool_skills import get_tool_skills

    def run(ctx):
        ctx.set_progress(5, "scanning tool directories")
        ctx.raise_if_cancelled()
        return asyncio.run(get_tool_skills())

    task = get_task_manager().submit("get_tool_skills", run)
    return TaskStartResponse(task_id=task.id, status=task.status)


@router.post(
    "/api/tasks/set_community_repo_path",
    response_model=TaskStartResponse,
    summary="启动社区仓库迁移任务",
    description="提交一个后台任务执行社区仓库路径迁移，支持 dry_run 预演，并返回任务 ID 与初始状态。",
    tags=["Tasks"],
)
async def start_set_community_repo_path(req: MigrateCommunityRepoTaskRequest):
    """启动社区仓库路径迁移后台任务。"""
    from pathlib import Path
    from core.repo.community_migration import execute_community_repo_migration, plan_community_repo_migration
    from core.db.store import get_store
    from core.utils.path_safety import expand_home

    def run(ctx):
        ctx.set_progress(5, "preparing community repo migration")
        ctx.raise_if_cancelled()
        expanded = expand_home(req.path)
        store = get_store()
        if req.dry_run:
            result = {"dry_run": True, **plan_community_repo_migration(store, expanded)}
        else:
            with ctx.exclusive("migrating community repo"):
                result = {"dry_run": False, **execute_community_repo_migration(store, expanded)}
        ctx.raise_if_cancelled()
        return result

    task = get_task_manager().submit("set_community_repo_path", run)
    return TaskStartResponse(task_id=task.id, status=task.status)
