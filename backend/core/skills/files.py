"""Skill 文件列表与读取 — 对应 Rust skill_files.rs"""
from __future__ import annotations

from dataclasses import dataclass
import os
from pathlib import Path

from core.utils.constants import IGNORE_NAMES

MAX_FILE_SIZE = 1 * 1024 * 1024  # 1 MB


@dataclass
class FileEntry:
    path: str
    size: int


def list_files(community_path: str | Path) -> list[FileEntry]:
    """遍历 skill 目录，返回排序后的文件列表（SKILL.md 优先）"""
    base = Path(community_path)
    if not base.is_dir():
        raise NotADirectoryError(f"not a directory: {base}")

    entries: list[FileEntry] = []
    for item in sorted(base.rglob("*")):
        if item.name in IGNORE_NAMES:
            continue
        if item.is_dir():
            continue
        rel = item.relative_to(base).as_posix()
        entries.append(FileEntry(path=rel, size=item.stat().st_size))

    # SKILL.md 排在最前面
    entries.sort(key=lambda e: (0 if e.path == "SKILL.md" else 1, e.path))
    return entries


def read_file(community_path: str | Path, relative_path: str) -> str:
    """读取 skill 目录中的文件内容（带路径穿越保护和大小限制）

    先打开文件句柄，通过 fd 校验路径后再读取，消除 TOCTOU 时间窗口。
    """
    base = Path(community_path).resolve()
    target = (base / relative_path).resolve()

    # 路径穿越保护
    try:
        if os.path.commonpath([str(target), str(base)]) != str(base):
            raise ValueError(f"path traversal not allowed: {relative_path}")
    except ValueError:
        raise ValueError(f"path traversal not allowed: {relative_path}")

    if not target.is_file():
        raise FileNotFoundError(f"file not found: {relative_path}")

    if target.stat().st_size > MAX_FILE_SIZE:
        raise ValueError(f"file too large (>1MB): {relative_path}")

    # 通过 fd 读取，避免 TOCTOU：校验后文件可能被替换为符号链接
    # O_NOFOLLOW 在 Windows 上不可用，跳过该保护（Windows 默认不支持文件软链接）
    flags = os.O_RDONLY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        fd = os.open(str(target), flags)
    except OSError as e:
        raise ValueError(f"cannot open file: {relative_path}") from e

    try:
        content = os.read(fd, MAX_FILE_SIZE + 1)
        if len(content) > MAX_FILE_SIZE:
            raise ValueError(f"file too large (>1MB): {relative_path}")
        return content.decode("utf-8", errors="replace")
    finally:
        os.close(fd)


def write_file(community_path: str | Path, relative_path: str, content: str) -> None:
    """写入 skill 目录中的文件内容（带路径穿越保护和大小限制）

    与 read_file 使用一致的路径校验逻辑，只允许写入已存在文件。
    """
    base = Path(community_path).resolve()
    target = (base / relative_path).resolve()

    try:
        if os.path.commonpath([str(target), str(base)]) != str(base):
            raise ValueError(f"path traversal not allowed: {relative_path}")
    except ValueError:
        raise ValueError(f"path traversal not allowed: {relative_path}")

    if not target.is_file():
        raise FileNotFoundError(f"file not found: {relative_path}")

    data = content.encode("utf-8")
    if len(data) > MAX_FILE_SIZE:
        raise ValueError(f"content too large (>1MB): {relative_path}")

    flags = os.O_WRONLY | os.O_TRUNC
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        fd = os.open(str(target), flags)
    except OSError as e:
        raise ValueError(f"cannot open file: {relative_path}") from e

    try:
        os.write(fd, data)
    finally:
        os.close(fd)
