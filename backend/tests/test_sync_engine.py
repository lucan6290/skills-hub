"""core/skills/sync_engine.py 单元测试"""
from pathlib import Path

import pytest

from core.skills.sync_engine import (
    SyncMode,
    _remove_path_any,
    copy_dir_recursive,
    sync_dir_copy_with_overwrite,
    sync_dir_hybrid,
)


def make_source_tree(root: Path) -> Path:
    """创建含 a.txt、sub/b.txt 以及 .git 目录的源目录。"""
    root.mkdir(parents=True, exist_ok=True)
    (root / "a.txt").write_text("alpha", encoding="utf-8")
    (root / "sub").mkdir()
    (root / "sub" / "b.txt").write_text("beta", encoding="utf-8")
    (root / ".git").mkdir()
    (root / ".git" / "config").write_text("git", encoding="utf-8")
    return root


def assert_tree_content(source: Path, target: Path) -> None:
    assert (target / "a.txt").read_bytes() == (source / "a.txt").read_bytes()
    assert (target / "sub" / "b.txt").read_bytes() == (source / "sub" / "b.txt").read_bytes()


def test_copy_dir_recursive_copies_content_and_skips_git(tmp_path):
    source = make_source_tree(tmp_path / "source")
    target = tmp_path / "target"

    copy_dir_recursive(source, target)

    assert_tree_content(source, target)
    assert not (target / ".git").exists()


def test_sync_dir_copy_with_overwrite_uses_copy_mode(tmp_path):
    source = make_source_tree(tmp_path / "source")
    target = tmp_path / "target"

    result = sync_dir_copy_with_overwrite(source, target, overwrite=False)

    assert result.mode_used == SyncMode.COPY
    assert result.target_path == target
    assert_tree_content(source, target)
    assert not (target / ".git").exists()


def test_sync_dir_copy_with_overwrite_fails_when_target_exists(tmp_path):
    source = make_source_tree(tmp_path / "source")
    target = tmp_path / "target"
    target.mkdir()
    (target / "existing.txt").write_text("existing", encoding="utf-8")

    with pytest.raises(FileExistsError):
        sync_dir_copy_with_overwrite(source, target, overwrite=False)


def test_sync_dir_hybrid_syncs_content(tmp_path):
    source = make_source_tree(tmp_path / "source")
    target = tmp_path / "target"

    result = sync_dir_hybrid(source, target)

    assert result.target_path == target
    # Windows 下 symlink/junction 可能因权限失败而回退 copy，不强制断言具体模式
    assert result.mode_used in {SyncMode.SYMLINK, SyncMode.JUNCTION, SyncMode.COPY}
    assert_tree_content(source, target)


def test_remove_path_any_removes_file(tmp_path):
    path = tmp_path / "file.txt"
    path.write_text("hello", encoding="utf-8")

    _remove_path_any(path)

    assert not path.exists()


def test_remove_path_any_removes_empty_dir(tmp_path):
    path = tmp_path / "empty"
    path.mkdir()

    _remove_path_any(path)

    assert not path.exists()


def test_remove_path_any_removes_dir_with_contents(tmp_path):
    path = tmp_path / "with-content"
    path.mkdir()
    (path / "nested.txt").write_text("nested", encoding="utf-8")

    _remove_path_any(path)

    assert not path.exists()
