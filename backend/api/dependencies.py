"""FastAPI 依赖：数据库 store 注入。"""
from core.db.store import SkillStore, get_store


def get_skill_store() -> SkillStore:
    return get_store()
