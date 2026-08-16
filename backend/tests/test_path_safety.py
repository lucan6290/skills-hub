import os

import pytest

from core.utils.path_safety import (
    expand_home,
    require_path_within,
    safe_child_path,
    safe_dir_name,
)


def test_safe_dir_name_returns_single_component():
    name = safe_dir_name("../bad/name")
    assert "/" not in name
    assert "\\" not in name
    assert name not in {"", ".", ".."}


def test_require_path_within_rejects_parent_escape(tmp_path):
    with pytest.raises(ValueError):
        require_path_within(tmp_path / ".." / "outside", tmp_path)


def test_require_path_within_accepts_child(tmp_path):
    child = tmp_path / "child"
    assert require_path_within(child, tmp_path) == child


def test_safe_child_path_joins(tmp_path):
    child = safe_child_path(tmp_path, "child")
    assert child == tmp_path / "child"


def test_safe_child_path_rejects_parent_escape(tmp_path):
    with pytest.raises(ValueError):
        safe_child_path(tmp_path, "..")


def test_expand_home_expands_tilde():
    assert expand_home("~") == os.path.expanduser("~")


def test_expand_home_keeps_absolute_path(tmp_path):
    absolute = str(tmp_path)
    assert expand_home(absolute) == absolute


@pytest.mark.parametrize("reserved", ["CON", "PRN", "AUX", "NUL"])
def test_safe_dir_name_rewrites_windows_reserved_names(reserved):
    result = safe_dir_name(reserved)
    assert result != reserved
    assert result.upper().split(".", 1)[0] != reserved

