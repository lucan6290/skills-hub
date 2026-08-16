"""Safe community repository migration planning and execution."""
from __future__ import annotations

import logging
import shutil
from pathlib import Path
from typing import Any

from core.utils.path_safety import is_path_within, norm_path, require_path_within
from core.repo.community import resolve_community_repo_path
from core.db.store import SkillStore, now_ms

logger = logging.getLogger(__name__)


def _reject_dangerous_destination(path: Path) -> None:
    resolved = path.resolve() if path.exists() else path.absolute()
    home = Path.home().resolve()
    anchor = Path(resolved.anchor).resolve()
    if resolved == home:
        raise ValueError("storage path cannot be the home directory")
    if resolved == anchor:
        raise ValueError("storage path cannot be a filesystem root")


def plan_community_repo_migration(store: SkillStore, new_base: str | Path) -> dict[str, Any]:
    new_path = Path(new_base)
    if not new_path.is_absolute():
        raise ValueError("storage path must be absolute")
    _reject_dangerous_destination(new_path)

    current_base = resolve_community_repo_path(store.db_path)
    if norm_path(current_base) == norm_path(new_path):
        return {
            "current_path": str(current_base),
            "new_path": str(new_path),
            "operations": [],
            "operation_count": 0,
        }
    if is_path_within(new_path, current_base) or is_path_within(current_base, new_path):
        raise ValueError("new storage path cannot be inside the current repo, or contain it")

    operations: list[dict[str, Any]] = []
    for skill in store.list_skills():
        old_path = require_path_within(Path(skill.community_path), current_base, "community path")
        if not old_path.exists():
            raise FileNotFoundError(f"community path not found: {old_path}")
        new_path_for_skill = new_path / old_path.name
        if new_path_for_skill.exists():
            raise FileExistsError(f"target path already exists: {new_path_for_skill}")
        operations.append({
            "action": "move",
            "skill_id": skill.id,
            "skill_name": skill.name,
            "from": str(old_path),
            "to": str(new_path_for_skill),
        })

    return {
        "current_path": str(current_base),
        "new_path": str(new_path),
        "operations": operations,
        "operation_count": len(operations),
    }


def execute_community_repo_migration(store: SkillStore, new_base: str | Path) -> dict[str, Any]:
    plan = plan_community_repo_migration(store, new_base)
    new_path = Path(plan["new_path"])
    new_path.mkdir(parents=True, exist_ok=True)
    logger.info(
        "community repo migration started: %s -> %s (operations=%d)",
        plan["current_path"], plan["new_path"], plan["operation_count"],
    )

    moved: list[dict[str, str]] = []
    try:
        for op in plan["operations"]:
            src = Path(op["from"])
            dst = Path(op["to"])
            try:
                src.rename(dst)
            except OSError:
                shutil.copytree(str(src), str(dst))
                shutil.rmtree(str(src))
            moved.append({"from": str(src), "to": str(dst)})
    except Exception as e:
        logger.warning("community repo migration failed, rolling back %d moved dirs: %s", len(moved), e)
        for item in reversed(moved):
            src = Path(item["from"])
            dst = Path(item["to"])
            try:
                if src.exists() or not dst.exists():
                    continue
                dst.rename(src)
            except OSError:
                try:
                    shutil.copytree(str(dst), str(src))
                    shutil.rmtree(str(dst))
                except Exception as rollback_error:
                    logger.warning("rollback cleanup failed: %s", rollback_error)
        raise

    by_id = {s.id: s for s in store.list_skills()}
    for op in plan["operations"]:
        skill = by_id.get(op["skill_id"])
        if not skill:
            continue
        skill.community_path = op["to"]
        skill.updated_at = now_ms()
        store.upsert_skill(skill)

    store.set_setting("community_repo_path", str(new_path))
    logger.info("community repo migration completed: moved=%d", len(moved))
    return plan
