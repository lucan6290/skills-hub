"""Repository and sync health checks plus conservative repair actions."""
from __future__ import annotations

import logging
import os
import time
import uuid
from pathlib import Path
from typing import Any, Optional

from core.utils.content_hash import hash_dir
from core.utils.path_safety import is_path_within, norm_path, require_path_within
from core.db.store import SkillRecord, SkillStore, SkillTargetRecord, now_ms
from core.repo.community import resolve_community_repo_path, resolve_custom_repo_path
from core.skills.installer import is_skill_dir, parse_skill_md
from core.skills.install_service import build_skill_record
from core.skills.source_paths import normalize_source_type, resolve_skill_source_path
from core.skills.sync_engine import (
    _is_junction,
    _remove_path_any,
    sync_dir_copy_with_overwrite,
    sync_dir_for_tool_with_overwrite,
)
from core.tools.adapters import (
    adapter_by_key,
    effective_tool_adapters,
    is_tool_installed,
    resolve_default_path,
    resolve_project_path,
    scan_tool_dir,
)

logger = logging.getLogger(__name__)


def _target_base(target: SkillTargetRecord) -> Optional[str]:
    adapter = adapter_by_key(target.tool)
    if not adapter:
        return None
    if (target.scope or "global") == "project":
        if not target.project_path:
            return None
        return resolve_project_path(adapter, target.project_path)
    return resolve_default_path(adapter)


def _issue(
    code: str,
    severity: str,
    message: str,
    repair: Optional[str] = None,
    **data: Any,
) -> dict[str, Any]:
    payload = {
        "code": code,
        "severity": severity,
        "message": message,
        "repair_action": repair,
    }
    payload.update(data)
    return payload


def scan_sync_health(store: SkillStore) -> dict[str, Any]:
    """Scan DB, community repo and installed tool dirs without mutating state."""
    community_base = resolve_community_repo_path(store.db_path)
    custom_base = resolve_custom_repo_path(store.db_path)
    skills = store.list_skills()
    issues: list[dict[str, Any]] = []
    target_paths: set[str] = set()
    community_paths: set[str] = set()
    custom_paths: set[str] = set()

    for skill in skills:
        source_type = normalize_source_type(skill.source_type)
        source_label = "自制源目录" if source_type == "custom" else "中央仓库"
        try:
            source_path = resolve_skill_source_path(skill, store)
        except ValueError:
            issues.append(_issue(
                "source_path_outside_repo",
                "error",
                f"{source_label}路径超出允许范围 / source path escapes repo",
                skill_id=skill.id,
                skill_name=skill.name,
                community_path=skill.community_path,
            ))
            continue
        if source_type == "community":
            community_paths.add(norm_path(source_path))
        elif source_type == "custom":
            custom_paths.add(norm_path(source_path))
        if not source_path.is_dir():
            missing_code = "missing_source_dir" if source_type == "custom" else "missing_community_dir"
            issues.append(_issue(
                missing_code,
                "error",
                f"{source_label}中的 Skill 目录丢失 / source skill directory is missing",
                "mark_skill_missing",
                skill_id=skill.id,
                skill_name=skill.name,
                community_path=str(source_path),
            ))
            continue
        try:
            source_hash = hash_dir(source_path)
            if skill.content_hash and source_hash != skill.content_hash:
                issues.append(_issue(
                    "source_hash_drift",
                    "info",
                    f"{source_label}内容与数据库哈希不一致 / source content hash differs from DB",
                    "update_content_hash",
                    skill_id=skill.id,
                    skill_name=skill.name,
                    community_path=str(source_path),
                    stored_hash=skill.content_hash,
                    actual_hash=source_hash,
                ))
        except Exception as e:
            issues.append(_issue(
                "source_hash_failed",
                "info",
                f"无法计算{source_label}哈希 / cannot hash source skill",
                skill_id=skill.id,
                skill_name=skill.name,
                community_path=str(source_path),
                error=str(e),
            ))

        targets = store.list_skill_targets(skill.id)
        for target in targets:
            target_paths.add(norm_path(target.target_path))
            base = _target_base(target)
            if not base:
                issues.append(_issue(
                    "invalid_target_record",
                    "error",
                    "同步目标记录无法解析工具或项目路径 / invalid target record",
                    "delete_target_record",
                    skill_id=skill.id,
                    tool=target.tool,
                    target_path=target.target_path,
                ))
                continue
            if not is_path_within(target.target_path, base):
                issues.append(_issue(
                    "target_path_outside_tool_dir",
                    "error",
                    "同步目标路径超出工具目录 / target path escapes tool directory",
                    skill_id=skill.id,
                    tool=target.tool,
                    target_path=target.target_path,
                    expected_base=base,
                ))
                continue

            path = Path(target.target_path)
            exists = path.exists() or path.is_symlink() or _is_junction(path)
            if not exists:
                issues.append(_issue(
                    "missing_target_path",
                    "warning",
                    "数据库记录存在，但工具目录中的目标已丢失 / target path is missing",
                    "resync_target",
                    skill_id=skill.id,
                    tool=target.tool,
                    scope=target.scope,
                    project_path=target.project_path,
                    target_path=target.target_path,
                ))
                continue

            is_link = path.is_symlink() or _is_junction(path)
            if target.mode in {"symlink", "junction"} and not is_link:
                issues.append(_issue(
                    "mode_mismatch",
                    "warning",
                    "数据库记录为链接模式，但目标不是链接 / target mode does not match filesystem",
                    skill_id=skill.id,
                    tool=target.tool,
                    target_path=target.target_path,
                    mode=target.mode,
                ))

            if is_link and not Path(os.path.realpath(path)).exists():
                issues.append(_issue(
                    "broken_link",
                    "warning",
                    "同步链接已断开 / sync link target is missing",
                    "resync_target",
                    skill_id=skill.id,
                    tool=target.tool,
                    target_path=target.target_path,
                ))
                continue

            if is_link and norm_path(os.path.realpath(path)) != norm_path(source_path):
                issues.append(_issue(
                    "wrong_link_target",
                    "warning",
                    "同步链接指向的目录不是当前源路径 / link points to the wrong target",
                    skill_id=skill.id,
                    tool=target.tool,
                    target_path=target.target_path,
                    expected_path=str(source_path),
                    actual_path=os.path.realpath(path),
                ))
                continue

            if target.mode == "copy" and path.is_dir():
                try:
                    source_hash = hash_dir(source_path)
                    # 优先使用存储的 target_content_hash 快速比较，避免重新哈希
                    if target.target_content_hash and target.target_content_hash == source_hash:
                        # 存储的哈希与源一致，无需重新扫描
                        pass
                    else:
                        target_hash = hash_dir(path)
                        if source_hash != target_hash:
                            issues.append(_issue(
                                "stale_copy_target",
                                "warning",
                                "copy 模式目标内容落后于中央仓库 / copied target is stale",
                                "resync_copy_target",
                                skill_id=skill.id,
                                tool=target.tool,
                                target_path=target.target_path,
                                source_hash=source_hash,
                                target_hash=target_hash,
                            ))
                except Exception as e:
                    issues.append(_issue(
                        "target_hash_failed",
                        "info",
                        "无法计算同步目标哈希 / cannot hash target",
                        skill_id=skill.id,
                        tool=target.tool,
                        target_path=target.target_path,
                        error=str(e),
                    ))

    if community_base.is_dir():
        for item in sorted(community_base.iterdir()):
            if not item.is_dir() or item.name == ".snapshots":
                continue
            if not is_skill_dir(item):
                continue
            if norm_path(item) not in community_paths:
                issues.append(_issue(
                    "community_orphan_dir",
                    "info",
                    "Community Repo 中存在未登记的 Skill / community skill is not registered",
                    "register_community_skill",
                    community_path=str(item),
                    skill_name=item.name,
                ))

    if custom_base.is_dir():
        for item in sorted(custom_base.iterdir()):
            if not item.is_dir() or item.name == ".snapshots":
                continue
            if not is_skill_dir(item) and not item.name.endswith("-skills"):
                continue
            if norm_path(item) not in custom_paths:
                # 跳过已注册套件根目录下的子 skill
                if any(is_path_within(item, Path(cp)) for cp in custom_paths):
                    continue
                issues.append(_issue(
                    "custom_orphan_dir",
                    "info",
                    "自制仓库中存在未登记的 Skill / custom skill is not registered",
                    "register_custom_skill",
                    community_path=str(item),
                    skill_name=item.name,
                ))

    for adapter in effective_tool_adapters():
        if not is_tool_installed(adapter):
            continue
        skills_dir = resolve_default_path(adapter)
        for detected in scan_tool_dir(adapter, skills_dir):
            if norm_path(detected.path) in target_paths:
                continue
            if is_path_within(detected.path, community_base):
                continue
            if is_path_within(detected.path, custom_base):
                continue
            issues.append(_issue(
                "unmanaged_tool_skill",
                "info",
                "工具目录中存在未托管 Skill / unmanaged skill in tool directory",
                "import_tool_skill",
                tool=adapter.id.as_key(),
                skill_name=detected.name,
                target_path=detected.path,
                is_link=detected.is_link,
                link_target=detected.link_target,
            ))

    hashes: dict[str, list[SkillRecord]] = {}
    names: dict[str, list[SkillRecord]] = {}
    for skill in skills:
        if skill.content_hash:
            hashes.setdefault(skill.content_hash, []).append(skill)
        names.setdefault(skill.name.lower(), []).append(skill)
    for content_hash, group in hashes.items():
        if len(group) > 1:
            issues.append(_issue(
                "duplicate_content_hash",
                "info",
                "多个 Skill 内容完全相同 / duplicate skill content",
                skill_ids=[s.id for s in group],
                names=[s.name for s in group],
                content_hash=content_hash,
            ))
    for name, group in names.items():
        unique_hashes = {s.content_hash for s in group if s.content_hash}
        if len(group) > 1 and len(unique_hashes) > 1:
            issues.append(_issue(
                "same_name_different_content",
                "info",
                "多个同名 Skill 内容不同 / same name with different content",
                skill_ids=[s.id for s in group],
                name=name,
            ))

    return {
        "community_repo": str(community_base),
        "skills_checked": len(skills),
        "issues": issues,
        "summary": _summarize_issues(issues),
        "generated_at": int(time.time() * 1000),
    }


def _summarize_issues(issues: list[dict[str, Any]]) -> dict[str, int]:
    summary = {"error": 0, "warning": 0, "info": 0, "repairable": 0}
    for issue in issues:
        severity = issue.get("severity", "info")
        summary[severity] = summary.get(severity, 0) + 1
        if issue.get("repair_action"):
            summary["repairable"] += 1
    return summary


def repair_sync_health(store: SkillStore, dry_run: bool = True) -> dict[str, Any]:
    """Run conservative repairs. Destructive deletes are limited to DB records or managed targets."""
    report = scan_sync_health(store)
    operations: list[dict[str, Any]] = []

    for issue in report["issues"]:
        action = issue.get("repair_action")
        if action in {"register_community_skill", "register_custom_skill"}:
            operations.append({"action": action, "community_path": issue["community_path"]})
            if not dry_run:
                source_type = "custom" if action == "register_custom_skill" else "community"
                record = create_import_record_from_community(store, issue["community_path"], source_type)
                store.upsert_skill(record)
        elif action == "mark_skill_missing":
            operations.append({"action": action, "skill_id": issue["skill_id"]})
            if not dry_run:
                skill = store.get_skill_by_id(issue["skill_id"])
                if skill:
                    skill.status = "missing"
                    skill.updated_at = now_ms()
                    store.upsert_skill(skill)
        elif action == "update_content_hash":
            operations.append({"action": action, "skill_id": issue["skill_id"]})
            if not dry_run:
                skill = store.get_skill_by_id(issue["skill_id"])
                if skill:
                    skill.content_hash = issue.get("actual_hash")
                    skill.updated_at = now_ms()
                    store.upsert_skill(skill)
        elif action in {"delete_target_record", "resync_target", "resync_copy_target"}:
            operations.append({
                "action": action,
                "skill_id": issue.get("skill_id"),
                "tool": issue.get("tool"),
                "target_path": issue.get("target_path"),
            })
            if not dry_run and issue.get("skill_id") and issue.get("tool"):
                _repair_target_issue(store, issue)

    logger.info(
        "sync health repair %s: %d operations",
        "executed" if not dry_run else "planned (dry-run)",
        len(operations),
    )
    return {
        "dry_run": dry_run,
        "operations": operations,
        "operation_count": len(operations),
        "before": report,
        "after": scan_sync_health(store) if not dry_run else None,
    }


def _repair_target_issue(store: SkillStore, issue: dict[str, Any]) -> None:
    skill_id = issue["skill_id"]
    tool = issue["tool"]
    skill = store.get_skill_by_id(skill_id)
    if not skill:
        return
    try:
        source_path = resolve_skill_source_path(skill, store)
    except ValueError:
        return
    if not source_path.is_dir():
        return

    matching = [t for t in store.list_skill_targets(skill_id) if t.tool == tool]
    for target in matching:
        if issue.get("target_path") and norm_path(target.target_path) != norm_path(issue["target_path"]):
            continue
        base = _target_base(target)
        if not base:
            store.delete_skill_target(target.skill_id, target.tool, target.scope, target.project_path)
            continue
        target_path = require_path_within(target.target_path, base, "target path")
        if issue["repair_action"] == "delete_target_record":
            store.delete_skill_target(target.skill_id, target.tool, target.scope, target.project_path)
            continue
        _remove_path_any(target_path)
        if target.mode == "copy":
            result = sync_dir_copy_with_overwrite(source_path, target_path, overwrite=True)
        else:
            result = sync_dir_for_tool_with_overwrite(target.tool, str(source_path), target_path, overwrite=True)
        target.mode = result.mode_used.value
        target.status = "ok"
        target.last_error = None
        target.synced_at = now_ms()
        store.upsert_skill_target(target)
        logger.info("repair resynced target: skill=%s tool=%s mode=%s", skill_id, target.tool, target.mode)


def create_import_record_from_community(
    store: SkillStore, community_path: str | Path, source_type: str = "community"
) -> SkillRecord:
    path = Path(community_path)
    fm = parse_skill_md(path)
    content_hash = None
    try:
        content_hash = hash_dir(path)
    except Exception:
        pass
    return build_skill_record(
        skill_id=str(uuid.uuid4()),
        name=fm.name,
        description=fm.description,
        frontmatter=fm,
        source_type=source_type,
        source_ref=str(path),
        community_path=str(path),
        content_hash=content_hash,
    )
