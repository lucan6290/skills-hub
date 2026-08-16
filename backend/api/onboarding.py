"""Onboarding API — 对应 Rust get_onboarding_plan command"""
from __future__ import annotations

from fastapi import APIRouter, Depends

from models.schemas import (
    OnboardingGroup,
    OnboardingPlan,
    OnboardingVariant,
)
from api.dependencies import get_skill_store
from core.skills.onboarding import build_onboarding_plan
from core.repo.community import resolve_community_repo_path, resolve_custom_repo_path
from core.db.store import SkillStore

router = APIRouter()


@router.get(
    "/api/get_onboarding_plan",
    response_model=OnboardingPlan,
    summary="获取入门引导计划",
    description="扫描社区与自定义仓库中已存在但未纳入管理的技能，返回分组后的引导计划。",
    tags=["Onboarding"],
)
async def get_onboarding_plan(store: SkillStore = Depends(get_skill_store)):
    """获取入门引导计划（已发现但未管理的技能）。"""
    community_path = str(resolve_community_repo_path(store.db_path))
    custom_path = str(resolve_custom_repo_path(store.db_path))

    # 收集已管理的 target paths
    all_targets = store.list_all_skill_target_paths()
    managed_paths = {t[1] for t in all_targets}

    plan = build_onboarding_plan(community_path, managed_paths, custom_path)
    return OnboardingPlan(
        total_tools_scanned=plan.total_tools_scanned,
        total_skills_found=plan.total_skills_found,
        groups=[
            OnboardingGroup(
                name=g.name,
                variants=[
                    OnboardingVariant(
                        tool=v.tool,
                        name=v.name,
                        path=v.path,
                        fingerprint=v.fingerprint,
                        is_link=v.is_link,
                        link_target=v.link_target,
                    )
                    for v in g.variants
                ],
                has_conflict=g.has_conflict,
            )
            for g in plan.groups
        ],
    )
