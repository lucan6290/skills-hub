"""Helpers for safe filesystem path handling."""
from __future__ import annotations

import os
import re
from pathlib import Path


WINDOWS_RESERVED_NAMES = {
    "CON",
    "PRN",
    "AUX",
    "NUL",
    *(f"COM{i}" for i in range(1, 10)),
    *(f"LPT{i}" for i in range(1, 10)),
}


def safe_dir_name(name: str | None, fallback: str = "skill") -> str:
    """Return a single safe directory component derived from a display name."""
    raw = (name or "").strip()
    component = re.sub(r'[<>:"/\\|?*\x00-\x1f]+', "-", raw)
    component = re.sub(r"\s+", " ", component).strip(" .")

    if not component or component in {".", ".."}:
        component = fallback

    stem = component.split(".", 1)[0].upper()
    if stem in WINDOWS_RESERVED_NAMES:
        component = f"{component}-skill"

    return component[:120].rstrip(" .") or fallback


def is_path_within(path: str | Path, base: str | Path) -> bool:
    """Check lexical containment without following the final symlink target."""
    candidate = os.path.normcase(os.path.abspath(os.fspath(path)))
    root = os.path.normcase(os.path.abspath(os.fspath(base)))
    try:
        return os.path.commonpath([candidate, root]) == root
    except ValueError:
        return False


def require_path_within(path: str | Path, base: str | Path, label: str = "path") -> Path:
    if not is_path_within(path, base):
        raise ValueError(f"{label} escapes base directory: {path}")
    return Path(path)


def safe_child_path(base: str | Path, child_name: str, label: str = "path") -> Path:
    target = Path(base) / child_name
    return require_path_within(target, base, label)


def norm_path(path: str | Path) -> str:
    """规范化路径：绝对路径 + 大小写规范化（Windows 不区分大小写比较）。"""
    return os.path.normcase(os.path.abspath(os.fspath(path)))


def expand_home(input_path: str) -> str:
    """展开 ~ 和 ~/ 为完整路径，不改变其他路径"""
    p = input_path.strip()
    home = str(Path.home())
    if p == "~":
        return home
    if p.startswith("~/"):
        return str(Path.home() / p[2:])
    if p.startswith("~\\"):
        return str(Path.home() / p[2:])
    return p
