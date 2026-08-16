"""批量排序 API — POST /api/reorder"""
from __future__ import annotations

from fastapi import APIRouter, Depends, HTTPException

from models.schemas import ReorderRequest
from api.dependencies import get_skill_store
from core.db.store import SkillStore

router = APIRouter()

_VALID_ENTITIES = {"skills", "tags", "tools"}


@router.post(
    "/api/reorder",
    summary="批量重排实体顺序",
    description="按传入的顺序更新 skills、tags 或 tools 的排序值；实体类型非法时返回 400。",
    tags=["Skills"],
)
async def reorder(req: ReorderRequest, store: SkillStore = Depends(get_skill_store)):
    """批量更新技能/标签/工具的排序顺序。"""
    if req.entity not in _VALID_ENTITIES:
        raise HTTPException(status_code=400, detail=f"invalid entity: {req.entity}")

    # tags 的 id 是 int，需要转换
    items = req.items
    if req.entity == "tags":
        items = [(int(item.id), item.sort_order) for item in items]
    else:
        items = [(item.id, item.sort_order) for item in items]

    store.reorder_entities(req.entity, items)
    return {"ok": True}
