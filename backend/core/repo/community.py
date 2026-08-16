"""Community Repo 路径管理。"""
from __future__ import annotations

import os
from pathlib import Path

from core.config import DEFAULT_COMMUNITY_REPO_NAME


def resolve_community_repo_path(db_path: str | None = None) -> Path:
    """解析 Community Repo 路径：DB 设置 > ~/.skillshub > app data dir"""
    if db_path:
        from core.db.store import SkillStore
        store = SkillStore(db_path)
        try:
            stored = store.get_setting("community_repo_path")
            if stored:
                p = Path(stored)
                if p.is_absolute():
                    return p
        finally:
            store.close()

    home = Path.home()
    default = home / DEFAULT_COMMUNITY_REPO_NAME
    if default.exists():
        return default

    return default


def resolve_custom_repo_path(db_path: str | None = None) -> Path:
    """解析自制 Skill 仓库路径：DB 设置 > ~/.skills-hub-custom"""
    if db_path:
        from core.db.store import SkillStore
        store = SkillStore(db_path)
        try:
            stored = store.get_setting("custom_repo_path")
            if stored:
                p = Path(stored)
                if p.is_absolute():
                    return p
        finally:
            store.close()

    return Path.home() / ".skills-hub-custom"


def ensure_community_repo(path: str | Path) -> None:
    """确保 Community Repo 目录存在"""
    p = Path(path)
    p.mkdir(parents=True, exist_ok=True)
