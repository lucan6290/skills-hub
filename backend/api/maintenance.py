"""Maintenance and sync health APIs."""
from __future__ import annotations

import logging

from fastapi import APIRouter, Depends, HTTPException

from models.schemas import RepairSyncHealthRequest
from api.dependencies import get_skill_store
from core.skills.maintenance import repair_sync_health, scan_sync_health
from core.db.store import SkillStore

router = APIRouter()

logger = logging.getLogger(__name__)


@router.get(
    "/api/sync_health",
    summary="扫描同步健康状态",
    description="扫描所有技能的同步目标健康情况并返回报告；扫描异常时返回 400。",
    tags=["Maintenance"],
)
async def get_sync_health(store: SkillStore = Depends(get_skill_store)):
    """扫描同步健康状态。"""
    try:
        return scan_sync_health(store)
    except Exception:
        logger.error("scan_sync_health failed", exc_info=True)
        raise HTTPException(status_code=400, detail="扫描同步健康状态失败，请稍后重试")


@router.post(
    "/api/sync_health/repair",
    summary="修复同步健康问题",
    description="按扫描结果修复同步目标；支持 dry_run 预演，异常时返回 400。",
    tags=["Maintenance"],
)
async def repair_sync_health_api(req: RepairSyncHealthRequest, store: SkillStore = Depends(get_skill_store)):
    """修复同步健康问题（支持 dry_run 预演）。"""
    try:
        return repair_sync_health(store, dry_run=req.dry_run)
    except Exception:
        logger.error("repair_sync_health failed: dry_run=%s", req.dry_run, exc_info=True)
        raise HTTPException(status_code=400, detail="修复同步健康失败，请稍后重试")
