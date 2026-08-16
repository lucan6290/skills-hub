"""同步引擎 — 对应 Rust sync_engine.rs
支持 symlink / junction (Windows) / copy 三级回退"""
from __future__ import annotations

import logging
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from enum import Enum
from pathlib import Path

logger = logging.getLogger(__name__)


class SyncMode(Enum):
    AUTO = "auto"
    SYMLINK = "symlink"
    JUNCTION = "junction"
    COPY = "copy"


@dataclass
class SyncOutcome:
    mode_used: SyncMode
    target_path: Path
    replaced: bool


SKIP_COPY_NAMES = {".git"}


def sync_dir_hybrid(source: str | Path, target: str | Path) -> SyncOutcome:
    """使用 symlink 优先的混合同步策略"""
    return _sync_hybrid(Path(source), Path(target), overwrite=False)


def sync_dir_hybrid_with_overwrite(
    source: str | Path, target: str | Path, overwrite: bool
) -> SyncOutcome:
    return _sync_hybrid(Path(source), Path(target), overwrite=overwrite)


def _sync_hybrid(source: Path, target: Path, overwrite: bool) -> SyncOutcome:
    if not source.is_dir():
        raise NotADirectoryError(f"source is not a directory: {source}")

    if _is_link_or_junction(target):
        if _same_resolved_path(source, target):
            return SyncOutcome(mode_used=SyncMode.SYMLINK, target_path=target, replaced=False)
        if not overwrite:
            raise FileExistsError(f"target already exists: {target}")
        _remove_path_any(target)
    elif target.exists():
        if not overwrite:
            raise FileExistsError(f"target already exists: {target}")
        _remove_path_any(target)

    # 尝试 symlink
    try:
        os.symlink(str(source), str(target))
        return SyncOutcome(mode_used=SyncMode.SYMLINK, target_path=target, replaced=overwrite)
    except (OSError, NotImplementedError):
        pass

    # 尝试 junction (Windows)
    if sys.platform == "win32":
        try:
            _create_junction(source, target)
            return SyncOutcome(mode_used=SyncMode.JUNCTION, target_path=target, replaced=overwrite)
        except (OSError, NotImplementedError):
            pass

    # 回退到 copy
    logger.debug("symlink/junction unavailable, falling back to copy: %s", target)
    copy_dir_recursive(source, target)
    return SyncOutcome(mode_used=SyncMode.COPY, target_path=target, replaced=overwrite)


def sync_dir_copy_with_overwrite(
    source: str | Path, target: str | Path, overwrite: bool
) -> SyncOutcome:
    """强制使用 copy 模式（Cursor 等不支持 symlink 的工具）"""
    source, target = Path(source), Path(target)
    if not source.is_dir():
        raise NotADirectoryError(f"source is not a directory: {source}")

    if _is_link_or_junction(target):
        if _same_resolved_path(source, target):
            return SyncOutcome(mode_used=SyncMode.COPY, target_path=target, replaced=False)
        if not overwrite:
            raise FileExistsError(f"target already exists: {target}")
        _remove_path_any(target)
    elif target.exists():
        if not overwrite:
            raise FileExistsError(f"target already exists: {target}")
        _remove_path_any(target)

    copy_dir_recursive(source, target)
    return SyncOutcome(mode_used=SyncMode.COPY, target_path=target, replaced=overwrite)


def sync_dir_for_tool_with_overwrite(
    tool_key: str, source: str | Path, target: str | Path, overwrite: bool
) -> SyncOutcome:
    """工具感知的同步：根据 adapter 的 force_copy 决定 copy 或 hybrid，异常时降级为 hybrid"""
    force_copy = False
    try:
        from core.tools.adapters import adapter_by_key
        adapter = adapter_by_key(tool_key)
        force_copy = bool(adapter and adapter.force_copy)
    except Exception as e:
        logger.warning("adapter lookup failed for tool %s, falling back to hybrid sync: %s", tool_key, e)
        force_copy = False

    if force_copy:
        result = sync_dir_copy_with_overwrite(source, target, overwrite)
    else:
        result = _sync_hybrid(Path(source), Path(target), overwrite)
    logger.info("skill synced: tool=%s mode=%s target=%s", tool_key, result.mode_used.value, result.target_path)
    return result


def copy_dir_recursive(source: Path, target: Path) -> None:
    """递归复制目录（跳过 .git，跳过符号链接避免无限递归）"""
    target.mkdir(parents=True, exist_ok=True)
    for item in source.iterdir():
        if item.name in SKIP_COPY_NAMES:
            continue
        dst = target / item.name
        if item.is_symlink():
            continue
        if item.is_dir():
            copy_dir_recursive(item, dst)
        else:
            shutil.copy2(str(item), str(dst))


def _remove_path_any(path: Path) -> None:
    """删除文件、目录、符号链接或 Windows junction。

    链接/junction 只删除 reparse point 本身，绝不递归进目标内容——
    shutil.rmtree 会拒绝 junction，且对真实符号链接有误删目标的风险。
    注意不能仅靠 exists() 判存在：broken junction 跟随后会返回 False 而漏删。
    """
    is_link = _is_link_or_junction(path)
    if not is_link and not path.exists():
        return
    try:
        if is_link:
            try:
                os.unlink(str(path))
            except (IsADirectoryError, PermissionError):
                # 目录型链接/junction：unlink 不可用，改用 rmdir 只删 reparse point
                os.rmdir(str(path))
        elif os.path.isdir(str(path)):
            shutil.rmtree(str(path))
        else:
            path.unlink()
    except FileNotFoundError:
        pass
    except Exception as e:
        logger.warning("failed to remove target path %s: %s", path, e)
        raise


def _is_link_or_junction(path: Path) -> bool:
    return os.path.islink(str(path)) or _is_junction(path)


def _same_resolved_path(left: Path, right: Path) -> bool:
    try:
        left_path = os.path.normcase(os.path.abspath(os.path.realpath(os.fspath(left))))
        right_path = os.path.normcase(os.path.abspath(os.path.realpath(os.fspath(right))))
        return left_path == right_path
    except (OSError, ValueError):
        return False


def _is_junction(path: Path) -> bool:
    """检测 Windows junction（兼容 Python < 3.12）"""
    if sys.platform != "win32":
        return False
    try:
        # Python 3.12+ 有 Path.is_junction()
        if hasattr(path, 'is_junction'):
            return path.is_junction()
        # 旧版本：用 os.path.isdir + lstat 检查 reparse point
        import stat
        st = path.lstat()
        return bool(st.st_file_attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT)
    except (OSError, ValueError):
        return False
    except AttributeError:
        return False


def _create_junction(source: Path, target: Path) -> None:
    """创建 Windows junction"""
    if sys.platform != "win32":
        raise NotImplementedError("junctions are only supported on Windows")
    source_str = str(source)
    target_str = str(target)
    # 路径含特殊字符时 cmd /c 可能解析为命令分隔符，必须加引号
    # 同时校验路径长度，超限时提前失败而非让 cmd 静默出错
    if len(target_str) > 259 or len(source_str) > 259:
        raise OSError(f"mklink /J failed: path too long")
    result = subprocess.run(
        ["cmd", "/c", "mklink", "/J", f'"{target_str}"', f'"{source_str}"'],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip()
        raise OSError(f"mklink /J failed: {detail}")
