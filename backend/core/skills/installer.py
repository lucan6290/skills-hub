"""Skill 安装器：处理本地安装和复制目标重试。"""
from __future__ import annotations

import json
import logging
import re
import shutil
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from core.utils.constants import IGNORE_NAMES, KNOWN_FRONTMATTER_KEYS, SKILL_SCAN_BASES
from core.utils.content_hash import hash_dir
from core.utils.path_safety import require_path_within, safe_child_path, safe_dir_name
from core.db.store import SkillStore, SkillTargetRecord, now_ms
from core.skills.source_paths import is_suite_dir, normalize_source_type, resolve_skill_source_path
from core.skills.sync_engine import _remove_path_any
from core.tools.adapters import adapter_by_key, resolve_default_path, resolve_project_path, _target_base_for_record

logger = logging.getLogger(__name__)


@dataclass
class SkillFrontmatter:
    """从 SKILL.md frontmatter 解析出的完整信息。"""
    name: Optional[str] = None
    description: Optional[str] = None
    version: Optional[str] = None
    author: Optional[str] = None
    license: Optional[str] = None
    category: Optional[str] = None
    homepage: Optional[str] = None
    frontmatter_extra: Optional[str] = None  # JSON string of unknown fields


@dataclass
class InstallResult:
    skill_id: str
    name: str
    community_path: str
    content_hash: Optional[str]
    description: Optional[str] = None
    source_subpath: Optional[str] = None
    frontmatter: Optional[SkillFrontmatter] = None
    skill_file_count: Optional[int] = None
    skill_dir_size: Optional[int] = None


@dataclass
class LocalSkillCandidate:
    name: str
    description: Optional[str]
    subpath: str
    valid: bool
    reason: Optional[str]


def parse_skill_md(path: str | Path) -> SkillFrontmatter:
    """解析 SKILL.md 的 YAML frontmatter，返回所有提取字段。"""
    skill_md = Path(path) / "SKILL.md"
    if not skill_md.exists():
        return SkillFrontmatter()

    try:
        content = skill_md.read_text(encoding="utf-8")
    except Exception as e:
        logger.debug("failed to read SKILL.md %s: %s", skill_md, e)
        return SkillFrontmatter()

    return _extract_frontmatter(content)


def _extract_frontmatter(content: str) -> SkillFrontmatter:
    """从 SKILL.md 内容中提取所有 frontmatter 字段。"""
    result = SkillFrontmatter()
    all_fields: dict[str, str] = {}

    m = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
    if m:
        frontmatter_text = m.group(1)
        for line in frontmatter_text.split("\n"):
            line = line.strip()
            if ":" not in line:
                continue
            key, val = line.split(":", 1)
            key = key.strip()
            val = val.strip().strip("\"'")
            if not key or not val:
                continue
            all_fields[key] = val

    # 提取已知字段
    for key in KNOWN_FRONTMATTER_KEYS:
        if key in all_fields:
            setattr(result, key, all_fields.pop(key))

    # 剩余未知字段存入 frontmatter_extra (JSON)
    if all_fields:
        result.frontmatter_extra = json.dumps(all_fields, ensure_ascii=False)

    # 降级：如果 frontmatter 中没有 name，尝试从第一个 # 标题提取
    if not result.name:
        for line in content.split("\n"):
            line = line.strip()
            if line.startswith("# "):
                result.name = line[2:].strip()
                break

    return result


def compute_skill_file_stats(dir_path: str | Path) -> tuple[int, int]:
    """递归统计技能目录中的文件数和总大小（字节）。

    排除 .git、.DS_Store、Thumbs.db、.gitignore。
    返回 (file_count, total_size_bytes)。
    """
    base = Path(dir_path)
    file_count = 0
    total_size = 0
    try:
        for entry in base.rglob("*"):
            if entry.name in IGNORE_NAMES:
                continue
            if entry.is_file():
                try:
                    total_size += entry.stat().st_size
                except OSError:
                    pass
                file_count += 1
    except Exception:
        pass
    return file_count, total_size


def is_skill_dir(path: str | Path) -> bool:
    """检查目录是否是一个有效的 skill。"""
    p = Path(path)
    if not p.is_dir():
        return False
    if (p / "SKILL.md").exists():
        return True
    claude_skills = p / ".claude" / "skills"
    if claude_skills.is_dir():
        for item in claude_skills.iterdir():
            if item.is_dir() and (item / "SKILL.md").exists():
                return True
    return False


def collect_skill_dirs(repo_dir: str | Path) -> list[Path]:
    """扫描目录中的 skill 目录。"""
    base = Path(repo_dir)
    results: list[Path] = []

    if is_skill_dir(base):
        results.append(base)

    for scan_base in SKILL_SCAN_BASES:
        skills_dir = base / scan_base
        if skills_dir.is_dir():
            for item in sorted(skills_dir.iterdir()):
                if item.is_dir() and is_skill_dir(item):
                    results.append(item)

    return results


def list_local_skills(base_path: str | Path) -> list[LocalSkillCandidate]:
    """扫描本地目录查找 skill 候选。"""
    base = Path(base_path)
    if not base.is_dir():
        raise NotADirectoryError(f"not a directory: {base}")

    candidates = []
    for item in sorted(base.iterdir()):
        if not item.is_dir():
            continue
        fm = parse_skill_md(item)
        name = fm.name
        desc = fm.description
        if name:
            valid = True
            reason = None
        elif is_suite_dir(item):
            # 套件目录（无 SKILL.md，子目录全是 skill dirs），作为整体展示
            name = item.name
            desc = None
            valid = True
            reason = None
        else:
            claude_skills = item / ".claude" / "skills"
            if claude_skills.is_dir():
                sub_items = [i for i in claude_skills.iterdir() if i.is_dir()]
                if sub_items:
                    first = sub_items[0]
                    fm2 = parse_skill_md(first)
                    name = fm2.name
                    desc = fm2.description
                    if name:
                        valid = True
                        reason = None
                    else:
                        valid = False
                        reason = "missing_skill_md"
                else:
                    valid = False
                    reason = "missing_skill_md"
            else:
                valid = False
                reason = "missing_skill_md"

        candidates.append(LocalSkillCandidate(
            name=name or item.name,
            description=desc,
            subpath=item.name,
            valid=valid,
            reason=reason,
        ))

    return candidates


def install_local_skill(
    source_path: str,
    name: Optional[str] = None,
    community_repo: str | Path | None = None,
    source_type: str = "community",
) -> InstallResult:
    """从本地目录安装 skill。"""
    source = Path(source_path)
    if not source.is_dir():
        raise NotADirectoryError(f"source is not a directory: {source}")

    fm = parse_skill_md(source)
    skill_name = name or fm.name or source.name
    skill_id = str(uuid.uuid4())

    # 计算文件统计
    file_count, dir_size = compute_skill_file_stats(source)

    if normalize_source_type(source_type) == "custom":
        try:
            content = hash_dir(source)
        except Exception:
            content = None
        logger.info("skill installed (custom source): name=%s path=%s", skill_name, source)
        return InstallResult(
            skill_id=skill_id,
            name=skill_name,
            community_path=str(source),
            content_hash=content,
            description=fm.description,
            source_subpath=None,
            frontmatter=fm,
            skill_file_count=file_count,
            skill_dir_size=dir_size,
        )

    if community_repo is None:
        from core.repo.community import resolve_community_repo_path
        community_repo = resolve_community_repo_path()
    community_base = Path(community_repo)
    community_base.mkdir(parents=True, exist_ok=True)

    dir_name = safe_dir_name(skill_name)
    target_dir = safe_child_path(community_base, dir_name, "skill name")

    if target_dir.exists():
        dir_name = safe_dir_name(f"{dir_name}-{skill_id[:8]}")
        target_dir = safe_child_path(community_base, dir_name, "skill name")

    shutil.copytree(str(source), str(target_dir))

    try:
        content = hash_dir(target_dir)
    except Exception:
        content = None

    # 重新计算目标目录的文件统计
    target_file_count, target_dir_size = compute_skill_file_stats(target_dir)

    logger.info("skill installed: name=%s community_path=%s", skill_name, target_dir)
    return InstallResult(
        skill_id=skill_id,
        name=skill_name,
        community_path=str(target_dir),
        content_hash=content,
        description=fm.description,
        source_subpath=None,
        frontmatter=fm,
        skill_file_count=target_file_count,
        skill_dir_size=target_dir_size,
    )


def install_local_skill_from_selection(
    base_path: str,
    subpath: str,
    name: Optional[str] = None,
    community_repo: str | Path | None = None,
    source_type: str = "community",
) -> InstallResult:
    """从本地目录安装选中的 skill。"""
    base = Path(base_path)
    source = require_path_within(base / subpath, base, "skill selection")
    if not source.is_dir():
        raise NotADirectoryError(f"skill not found: {source}")

    fm = parse_skill_md(source)
    skill_name = name or fm.name or Path(subpath).name

    return install_local_skill(str(source), skill_name, community_repo, source_type)




def _refresh_copy_targets(skill_id: str, store: SkillStore, community_path: Path) -> tuple[set[str], list[str]]:
    updated_targets_set: set[str] = set()
    failed_targets: list[str] = []
    for target in store.list_skill_targets(skill_id):
        if target.mode != "copy":
            continue
        try:
            target_path = require_path_within(Path(target.target_path), _target_base_for_record(target), "target path")
            _remove_path_any(target_path)
            shutil.copytree(str(community_path), str(target_path))
            target.status = "ok"
            target.last_error = None
            target.synced_at = now_ms()
            store.upsert_skill_target(target)
            updated_targets_set.add(target.tool)
        except Exception as e:
            target.status = "error"
            target.last_error = str(e)
            store.upsert_skill_target(target)
            failed_targets.append(f"{target.tool}:{target.target_path}:{e}")
    return updated_targets_set, failed_targets


def retry_copy_target(skill_id: str, tool: str, store: SkillStore) -> str:
    record = store.get_skill_by_id(skill_id)
    if not record:
        raise ValueError(f"skill not found: {skill_id}")
    community_path = resolve_skill_source_path(record, store)
    if not community_path.is_dir():
        raise ValueError(f"source path not found: {community_path}")
    for target in store.list_skill_targets(skill_id):
        if target.tool != tool or target.mode != "copy":
            continue
        target_path = require_path_within(Path(target.target_path), _target_base_for_record(target), "target path")
        _remove_path_any(target_path)
        shutil.copytree(str(community_path), str(target_path))
        target.status = "ok"
        target.last_error = None
        target.synced_at = now_ms()
        store.upsert_skill_target(target)
        return str(target_path)
    raise ValueError(f"copy target not found: {skill_id}/{tool}")
