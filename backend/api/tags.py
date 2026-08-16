"""标签 API — 对应 Rust tag commands"""
from __future__ import annotations

from fastapi import APIRouter, Depends, HTTPException, Query

from models.schemas import (
    CreateTagRequest,
    DeleteTagRequest,
    RenameTagRequest,
    SetSkillTagsRequest,
    TagDto,
    TagWithCountDto,
)
from api.dependencies import get_skill_store
from core.db.store import SkillStore
from core.skills.source_paths import normalize_source_type

router = APIRouter()


@router.get(
    "/api/get_tags",
    response_model=list[TagWithCountDto],
    summary="获取标签列表",
    description="返回标签及其关联技能数量，可按来源类型过滤并按名称或排序值排序。",
    tags=["Tags"],
)
async def get_tags(source_type: str | None = None, sort: str = "name", store: SkillStore = Depends(get_skill_store)):
    """获取标签列表（含技能数量）。"""
    tags = store.list_tags_with_counts(
        normalize_source_type(source_type) if source_type else None,
        sort=sort,
    )
    return [
        TagWithCountDto(id=t.id, name=t.name, skill_count=t.skill_count, updated_at=t.updated_at, sort_order=t.sort_order)
        for t in tags
    ]


@router.post(
    "/api/create_tag",
    response_model=TagDto,
    summary="创建标签",
    description="根据名称创建新标签；名称重复或非法时返回 400。",
    tags=["Tags"],
)
async def create_tag(req: CreateTagRequest, store: SkillStore = Depends(get_skill_store)):
    """创建标签。"""
    try:
        tag = store.create_tag(req.name)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    return TagDto(id=tag.id, name=tag.name, sort_order=tag.sort_order)


@router.post(
    "/api/rename_tag",
    response_model=TagDto,
    summary="重命名标签",
    description="按标签 ID 修改名称；名称非法或重复时返回 400。",
    tags=["Tags"],
)
async def rename_tag(req: RenameTagRequest, store: SkillStore = Depends(get_skill_store)):
    """重命名标签。"""
    try:
        tag = store.rename_tag(req.tag_id, req.name)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    return TagDto(id=tag.id, name=tag.name, sort_order=tag.sort_order)


@router.post(
    "/api/delete_tag",
    summary="删除标签",
    description="按标签 ID 删除标签；标签不存在时返回 404。",
    tags=["Tags"],
)
async def delete_tag(req: DeleteTagRequest, store: SkillStore = Depends(get_skill_store)):
    """删除标签。"""
    existing = store.get_tags()
    if not any(t.id == req.tag_id for t in existing):
        raise HTTPException(status_code=404, detail=f"tag not found: {req.tag_id}")
    store.delete_tag(req.tag_id)
    return {"ok": True}


@router.get(
    "/api/get_skill_tags",
    response_model=list[TagDto],
    summary="获取技能关联的标签",
    description="返回指定技能已关联的标签列表；技能不存在时返回 404。",
    tags=["Tags"],
)
async def get_skill_tags(skill_id: str = Query(...), store: SkillStore = Depends(get_skill_store)):
    """获取指定技能的标签列表。"""
    if not store.get_skill_by_id(skill_id):
        raise HTTPException(status_code=404, detail="skill not found")
    tags = store.get_skill_tags(skill_id)
    return [TagDto(id=t.id, name=t.name, sort_order=t.sort_order) for t in tags]


@router.post(
    "/api/set_skill_tags",
    summary="设置技能标签",
    description="用给定标签 ID 列表覆盖指定技能的标签；技能或标签不存在时返回错误。",
    tags=["Tags"],
)
async def set_skill_tags(req: SetSkillTagsRequest, store: SkillStore = Depends(get_skill_store)):
    """设置技能的标签集合。"""
    if not store.get_skill_by_id(req.skill_id):
        raise HTTPException(status_code=404, detail="skill not found")
    existing_tags = store.list_tags_with_counts()
    existing_ids = {t.id for t in existing_tags}
    for tag_id in req.tag_ids:
        if tag_id not in existing_ids:
            raise HTTPException(status_code=400, detail=f"tag not found: {tag_id}")
    store.set_skill_tags(req.skill_id, req.tag_ids)
    return {"ok": True}


@router.get(
    "/api/get_untagged_skill_ids",
    summary="获取未打标签的技能 ID",
    description="返回未关联任何标签的技能 ID 列表，可按来源类型过滤。",
    tags=["Tags"],
)
async def get_untagged_skill_ids(source_type: str | None = None, store: SkillStore = Depends(get_skill_store)):
    """获取未打标签的技能 ID 列表。"""
    return store.list_untagged_skill_ids(
        normalize_source_type(source_type) if source_type else None,
    )
