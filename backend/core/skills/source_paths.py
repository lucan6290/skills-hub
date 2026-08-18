from __future__ import annotations

from pathlib import Path

from core.utils.path_safety import is_path_within, require_path_within
from core.repo.community import resolve_community_repo_path, resolve_custom_repo_path
from core.db.store import SkillRecord, SkillStore


SourceType = str


def normalize_source_type(source_type: str | None) -> str:
    if source_type == "custom":
        return "custom"
    return "community"


def infer_source_type_from_path(path: str | Path, store: SkillStore) -> SourceType | None:
    custom_base = resolve_custom_repo_path(store.db_path)
    if is_path_within(path, custom_base):
        return "custom"

    community_base = resolve_community_repo_path(store.db_path)
    if is_path_within(path, community_base):
        return "community"

    return None


def resolve_skill_source_path(skill: SkillRecord, store: SkillStore) -> Path:
    source_type = normalize_source_type(skill.source_type)
    source_path = Path(skill.community_path)
    if source_type == "custom":
        custom_base = resolve_custom_repo_path(store.db_path)
        return require_path_within(source_path, custom_base, "custom skill path")
    community_base = resolve_community_repo_path(store.db_path)
    return require_path_within(source_path, community_base, "community path")


def is_suite_dir(path: str | Path) -> bool:
    """检测目录是否为套件根目录（无 SKILL.md，直接子目录均为 skill dirs）。"""
    # 延迟导入避免与 core.skills.installer 的循环导入
    from core.skills.installer import is_skill_dir

    p = Path(path)
    if not p.is_dir():
        return False
    if (p / "SKILL.md").exists():
        return False
    children = [c for c in p.iterdir() if c.is_dir()]
    if len(children) < 2:
        return False
    return all(is_skill_dir(c) for c in children)


def has_sub_skills(path: str | Path) -> bool:
    """检测目录是否包含子 skill（无论自身是否有 SKILL.md）。"""
    from core.skills.installer import is_skill_dir

    p = Path(path)
    if not p.is_dir():
        return False
    for child in p.iterdir():
        if child.is_dir() and is_skill_dir(child):
            return True
    return False
