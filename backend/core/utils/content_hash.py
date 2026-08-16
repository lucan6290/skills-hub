"""SHA256 内容哈希"""
from __future__ import annotations

import hashlib
from pathlib import Path

from core.utils.constants import IGNORE_NAMES


def hash_dir(path: str | Path) -> str:
    """计算目录的 SHA256 哈希（相对路径 + 文件内容）"""
    base = Path(path)
    if not base.is_dir():
        raise NotADirectoryError(f"not a directory: {base}")

    sha = hashlib.sha256()
    for entry in sorted(_walk(base)):
        rel = entry.relative_to(base).as_posix()
        sha.update(rel.encode("utf-8"))
        if entry.is_file():
            sha.update(entry.read_bytes())
        sha.update(b"\n")
    return sha.hexdigest()


def _walk(base: Path) -> list[Path]:
    result: list[Path] = []
    for item in sorted(base.iterdir()):
        if item.name in IGNORE_NAMES:
            continue
        if item.is_symlink():
            continue
        if item.is_dir():
            result.extend(_walk(item))
        else:
            result.append(item)
    return result
