"""Skills 管理 API — 对应 Rust get_managed_skills, delete_managed_skill 等"""
from __future__ import annotations

from pathlib import Path
import logging
import os

from fastapi import APIRouter, Depends, HTTPException

from models.schemas import (
    DeleteManagedSkillRequest,
    ImportExistingRequest,
    InstallLocalRequest,
    InstallLocalSelectionRequest,
    InstallResultDto,
    ListLocalSkillsRequest,
    LocalSkillCandidate,
    ManagedSkillDto,
    RetryCopyTargetRequest,
    SkillTargetDto,
    SkillUsageDto,
    TagDto,
    UpdateSourceUrlRequest,
)
from api.dependencies import get_skill_store
from core.error_codes import ErrorCode
from core.utils.path_safety import require_path_within
from core.repo.community import resolve_community_repo_path
from core.repo.scanner import sync_all_repo_registries, sync_repo_registry
from core.db.store import SkillRecord, SkillStore
from core.skills.installer import (
    install_local_skill,
    install_local_skill_from_selection,
    list_local_skills,
    retry_copy_target,
)
from core.skills.source_paths import normalize_source_type
from core.skills.sync_engine import _remove_path_any
from core.skills.install_service import (
    build_skill_record,
    dedupe_install_result,
    upsert_skill_from_install,
)
from core.tools.adapters import _target_base_for_record

router = APIRouter()

_logger = logging.getLogger(__name__)


def _to_managed_dto(skill: SkillRecord, store) -> ManagedSkillDto:
    targets = store.list_skill_targets(skill.id)
    tags = store.get_skill_tags(skill.id)
    usage = store.get_skill_usage(skill.id)
    return ManagedSkillDto(
        id=skill.id,
        name=skill.name,
        description=skill.description,
        frontmatter_extra=skill.frontmatter_extra,
        version=skill.version,
        author=skill.author,
        license=skill.license,
        category=skill.category,
        homepage=skill.homepage,
        skill_file_count=skill.skill_file_count,
        skill_dir_size=skill.skill_dir_size,
        source_type=skill.source_type,
        source_ref=skill.source_ref,
        source_subpath=skill.source_subpath,
        source_url=skill.source_url,
        community_path=skill.community_path,
        created_at=skill.created_at,
        updated_at=skill.updated_at,
        last_sync_at=skill.last_sync_at,
        status=skill.status,
        tags=[TagDto(id=t.id, name=t.name, sort_order=t.sort_order) for t in tags],
        targets=[
            SkillTargetDto(
                tool=t.tool,
                scope=t.scope,
                project_path=t.project_path,
                mode=t.mode,
                status=t.status,
                target_path=t.target_path,
                synced_at=t.synced_at,
            )
            for t in targets
        ],
        usage=[
            SkillUsageDto(
                id=u.id,
                skill_id=u.skill_id,
                tool=u.tool,
                sync_count=u.sync_count,
                last_synced_at=u.last_synced_at,
                last_viewed_at=u.last_viewed_at,
                view_count=u.view_count,
            )
            for u in usage
        ],
        sort_order=skill.sort_order,
    )


@router.get(
    "/api/get_managed_skills",
    response_model=list[ManagedSkillDto],
    summary="获取受管技能列表",
    description="返回受管技能及其标签、同步目标与使用统计；refresh=true 时先刷新仓库注册表，sort 控制排序方式。",
    tags=["Skills"],
)
async def get_managed_skills(refresh: bool = False, source_type: str | None = None, sort: str = "manual", store: SkillStore = Depends(get_skill_store)):
    """获取受管技能列表。"""
    if refresh:
        if source_type:
            sync_repo_registry(source_type, db_path=store.db_path)
        else:
            sync_all_repo_registries(db_path=store.db_path)
    skills = store.list_skills(sort=sort)
    # 记录每个技能的查看
    for s in skills:
        try:
            store.record_skill_view(s.id)
        except Exception as e:
            _logger.debug("failed to record skill view: skill_id=%s: %s", s.id, e)
    return [_to_managed_dto(s, store) for s in skills]


@router.post(
    "/api/delete_managed_skill",
    summary="删除受管技能",
    description="删除技能及其同步目标与社区仓库源目录；dry_run=true 时仅返回操作计划，技能不存在时返回 404。",
    tags=["Skills"],
)
async def delete_managed_skill(req: DeleteManagedSkillRequest, store: SkillStore = Depends(get_skill_store)):
    """删除受管技能（支持 dry_run 预演）。"""
    record = store.get_skill_by_id(req.skill_id)
    if not record:
        raise HTTPException(status_code=404, detail="skill not found")

    targets = store.list_skill_targets(req.skill_id)
    planned = []
    for target in targets:
        base = _target_base_for_record(target)
        target_path = require_path_within(Path(target.target_path), base, "target path")
        planned.append({"target": target, "path": target_path})

    operations = [{
        "action": "delete_target",
        "tool": p["target"].tool,
        "scope": p["target"].scope,
        "path": str(p["path"]),
    } for p in planned]

    source_type = normalize_source_type(record.source_type)
    community_path = None
    if source_type == "custom":
        operations.append({"action": "keep_custom_source", "path": record.community_path})
    else:
        try:
            community_base = resolve_community_repo_path(store.db_path)
            community_path = require_path_within(Path(record.community_path), community_base, "community path")
        except ValueError as e:
            raise HTTPException(status_code=400, detail=str(e))
        operations.append({"action": "delete_community", "path": str(community_path)})

    if req.dry_run:
        return {
            "dry_run": True,
            "skill_id": req.skill_id,
            "operations": operations,
            "operation_count": len(operations),
        }

    removed_paths: set[str] = set()
    for p in planned:
        try:
            target_path = p["path"]
            normalized = os.path.normcase(os.path.abspath(os.fspath(target_path)))
            if normalized in removed_paths:
                continue
            _remove_path_any(target_path)
            removed_paths.add(normalized)
        except Exception as e:
            _logger.error(
                "failed to delete target at %s: %s",
                p["target"].target_path,
                e,
                exc_info=True,
            )
            raise HTTPException(
                status_code=500,
                detail=f"删除同步目标失败（已成功 {len(removed_paths)} 个），请手动检查残留目标后重试",
            )

    if community_path is not None:
        _remove_path_any(community_path)

    store.delete_skill(req.skill_id)
    return {"ok": True}


@router.post(
    "/api/update_skill_source_url",
    summary="更新技能源地址",
    description="更新技能的源 URL 并返回更新后的技能信息；技能不存在时返回 404。",
    tags=["Skills"],
)
async def update_skill_source_url(req: UpdateSourceUrlRequest, store: SkillStore = Depends(get_skill_store)):
    """更新技能的源 URL。"""
    record = store.get_skill_by_id(req.skill_id)
    if not record:
        raise HTTPException(status_code=404, detail="skill not found")

    raw = (req.source_url or "").strip()
    lines = [ln.strip() for ln in raw.splitlines() if ln.strip()] if raw else None
    value = "\n".join(lines) if lines else None

    store.update_skill_source_url(req.skill_id, value)
    return _to_managed_dto(store.get_skill_by_id(req.skill_id), store)


@router.post(
    "/api/import_existing_skill",
    response_model=InstallResultDto,
    summary="导入已有技能",
    description="将已有技能目录（含 SKILL.md）导入为受管技能；缺少 SKILL.md 时返回 400。",
    tags=["Skills"],
)
async def import_existing_skill(req: ImportExistingRequest, store: SkillStore = Depends(get_skill_store)):
    """导入已有技能到受管列表。"""
    source = Path(req.source_path)
    if not (source / "SKILL.md").exists():
        raise HTTPException(status_code=400, detail={
            "code": ErrorCode.SKILL_INVALID,
            "reason": "missing_skill_md",
        })

    result = install_local_skill(req.source_path, req.name, source_type=req.source_type)
    duplicate = dedupe_install_result(store, result, req.source_type)
    if duplicate:
        return duplicate

    return upsert_skill_from_install(result, req.source_path, store, req.source_type)


@router.post(
    "/api/list_local_skills_cmd",
    response_model=list[LocalSkillCandidate],
    summary="列出本地技能目录",
    description="扫描指定基础路径下的本地技能候选目录并返回列表；扫描异常时返回 400。",
    tags=["Skills"],
)
async def list_local_skills_api(req: ListLocalSkillsRequest):
    """列出本地技能候选目录。"""
    try:
        return list_local_skills(req.base_path)
    except (ValueError, FileNotFoundError, NotADirectoryError, PermissionError) as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception:
        _logger.error("list_local_skills failed: base_path=%s", req.base_path, exc_info=True)
        raise HTTPException(status_code=400, detail="扫描本地技能失败，请稍后重试")


@router.post(
    "/api/install_local",
    response_model=InstallResultDto,
    summary="安装本地技能",
    description="从本地路径安装技能并登记为受管技能；重复安装时返回已有记录。",
    tags=["Skills"],
)
async def install_local(req: InstallLocalRequest, store: SkillStore = Depends(get_skill_store)):
    """从本地路径安装技能。"""
    result = install_local_skill(req.source_path, req.name, source_type=req.source_type)
    duplicate = dedupe_install_result(store, result, req.source_type)
    if duplicate:
        return duplicate

    return upsert_skill_from_install(result, req.source_path, store, req.source_type)


@router.post(
    "/api/install_local_selection",
    response_model=InstallResultDto,
    summary="安装本地选中的技能",
    description="从基础路径的子目录安装选中的技能并登记为受管技能。",
    tags=["Skills"],
)
async def install_local_selection(req: InstallLocalSelectionRequest, store: SkillStore = Depends(get_skill_store)):
    """安装本地目录下选中的技能。"""
    result = install_local_skill_from_selection(
        req.base_path, req.subpath, req.name, source_type=req.source_type
    )
    duplicate = dedupe_install_result(store, result, req.source_type)
    if duplicate:
        return duplicate

    selected_path = str(Path(req.base_path) / req.subpath)
    record = build_skill_record(
        skill_id=result.skill_id,
        name=result.name,
        description=result.description,
        frontmatter=getattr(result, "frontmatter", None),
        skill_file_count=getattr(result, "skill_file_count", None),
        skill_dir_size=getattr(result, "skill_dir_size", None),
        source_type=req.source_type,
        source_ref=selected_path,
        source_subpath=req.subpath,
        community_path=result.community_path,
        content_hash=result.content_hash,
    )
    store.upsert_skill(record)

    return InstallResultDto(
        skill_id=result.skill_id,
        name=result.name,
        community_path=result.community_path,
        content_hash=result.content_hash,
    )


@router.post(
    "/api/retry_copy_target",
    summary="重试复制同步目标",
    description="针对指定技能与工具重试复制同步目标，返回最终目标路径。",
    tags=["Skills"],
)
async def retry_copy_target_api(req: RetryCopyTargetRequest, store: SkillStore = Depends(get_skill_store)):
    """重试指定技能到工具的复制同步目标。"""
    target_path = retry_copy_target(req.skill_id, req.tool, store)
    return {"ok": True, "target_path": target_path}
