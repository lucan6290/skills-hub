"""core/tool_adapters.py 单元测试"""
import os
import tempfile
import pytest
from core.tools.adapters import (
    default_tool_adapters, adapter_by_key, resolve_default_path,
    resolve_project_path, supports_project_scope, is_tool_installed,
    adapters_sharing_skills_dir, adapters_sharing_project_skills_dir,
    scan_tool_dir, ToolAdapter, ToolId,
)


class TestToolAdapters:
    def test_default_adapters_non_empty(self):
        adapters = default_tool_adapters()
        assert len(adapters) >= 40

    def test_adapter_by_key_found(self):
        a = adapter_by_key("claude_code")
        assert a is not None
        assert a.display_name == "Claude Code"

    def test_adapter_by_key_not_found(self):
        assert adapter_by_key("nonexistent_tool") is None

    def test_resolve_default_path(self):
        a = adapter_by_key("claude_code")
        path = resolve_default_path(a)
        assert path.endswith(".claude/skills") or path.endswith(".claude\\skills")

    def test_resolve_project_path(self):
        a = adapter_by_key("claude_code")
        path = resolve_project_path(a, "/home/user/project")
        assert ".claude/skills" in path.replace("\\", "/")

    def test_supports_project_scope(self):
        a = adapter_by_key("claude_code")
        assert supports_project_scope(a) is True

    def test_hermes_no_project_scope(self):
        a = adapter_by_key("hermes_agent")
        assert supports_project_scope(a) is False

    def test_amp_kimi_share_skills_dir(self):
        amp = adapter_by_key("amp")
        group = adapters_sharing_skills_dir(amp)
        keys = [a.id.as_key() for a in group]
        assert "amp" in keys
        assert "kimi_cli" in keys

    def test_cursor_always_copy(self):
        a = adapter_by_key("cursor")
        assert a is not None

    def test_is_tool_installed_claude(self):
        a = adapter_by_key("claude_code")
        result = is_tool_installed(a)
        assert isinstance(result, bool)

    def test_scan_tool_dir(self, tmp_path):
        skill_dir = tmp_path / "test-skill"
        skill_dir.mkdir()
        (skill_dir / "SKILL.md").write_text("---\nname: test\n---\n")

        a = adapter_by_key("claude_code")
        results = scan_tool_dir(a, str(tmp_path))
        assert len(results) == 1
        assert results[0].name == "test-skill"

    def test_scan_tool_dir_skips_directories_without_skill_md(self, tmp_path):
        skill_dir = tmp_path / "test-skill"
        skill_dir.mkdir()
        (skill_dir / "SKILL.md").write_text("---\nname: test\n---\n")
        invalid_dir = tmp_path / "not-a-skill"
        invalid_dir.mkdir()

        a = adapter_by_key("claude_code")
        results = scan_tool_dir(a, str(tmp_path))
        names = [result.name for result in results]
        assert names == ["test-skill"]

    def test_all_adapters_have_unique_keys(self):
        adapters = default_tool_adapters()
        keys = [a.id.as_key() for a in adapters]
        assert len(keys) == len(set(keys))
