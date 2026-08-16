"""Skill 文件浏览 API — 对应 Rust list_skill_files, read_skill_file"""
from __future__ import annotations

import logging
import traceback

from fastapi import APIRouter, HTTPException, Query
from pathlib import Path

from models.schemas import SkillFileEntry, WriteSkillFileRequest
from core.skills.files import list_files, read_file, write_file
from core.skills.source_paths import resolve_skill_source_path
from core.db.store import get_store

logger = logging.getLogger(__name__)

router = APIRouter()


def _resolve_allowed_skill_path(skill_id: str | None) -> Path:
    store = get_store()
    if not skill_id:
        raise ValueError("skill_id is required")
    skill = store.get_skill_by_id(skill_id)
    if not skill:
        raise ValueError("skill not found")
    return resolve_skill_source_path(skill, store)


@router.get(
    "/api/list_skill_files",
    response_model=list[SkillFileEntry],
    summary="列出技能文件",
    description="返回指定技能源目录下的文件列表（含相对路径与大小）；技能不存在时返回错误。",
    tags=["Skills"],
)
async def list_skill_files(skill_id: str = Query(...)):
    """列出指定技能的文件列表。"""
    try:
        entries = list_files(_resolve_allowed_skill_path(skill_id))
    except (ValueError, NotADirectoryError) as e:
        raise HTTPException(status_code=400, detail=str(e))
    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    except Exception:
        logger.error("list_skill_files failed: skill_id=%s\n%s", skill_id, traceback.format_exc())
        raise HTTPException(status_code=500, detail="internal error")
    return [SkillFileEntry(path=e.path, size=e.size) for e in entries]


@router.get(
    "/api/read_skill_file",
    summary="读取技能文件内容",
    description="读取指定技能源目录中文件的内容；技能或文件不存在时返回错误。",
    tags=["Skills"],
)
async def read_skill_file(skill_id: str = Query(...), file_path: str = Query(...)):
    """读取指定技能文件的内容。"""
    try:
        content = read_file(_resolve_allowed_skill_path(skill_id), file_path)
    except (ValueError, NotADirectoryError) as e:
        raise HTTPException(status_code=400, detail=str(e))
    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    except Exception:
        logger.error("read_skill_file failed: skill_id=%s file_path=%s\n%s", skill_id, file_path, traceback.format_exc())
        raise HTTPException(status_code=500, detail="internal error")
    return content


@router.post(
    "/api/write_skill_file",
    summary="写入技能文件",
    description="将给定内容写入指定技能源目录中的文件；技能或文件不存在时返回错误。",
    tags=["Skills"],
)
async def write_skill_file(req: WriteSkillFileRequest):
    """写入指定技能文件的内容。"""
    try:
        write_file(_resolve_allowed_skill_path(req.skill_id), req.file_path, req.content)
    except (ValueError, NotADirectoryError) as e:
        raise HTTPException(status_code=400, detail=str(e))
    except FileNotFoundError as e:
        raise HTTPException(status_code=404, detail=str(e))
    except Exception:
        logger.error("write_skill_file failed: skill_id=%s file_path=%s\n%s", req.skill_id, req.file_path, traceback.format_exc())
        raise HTTPException(status_code=500, detail="internal error")
    return {"ok": True}
