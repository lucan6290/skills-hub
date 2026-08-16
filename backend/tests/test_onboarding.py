from core.skills.onboarding import build_onboarding_plan
from core.tools.adapters import DetectedSkill, ToolAdapter, ToolId


class FakeStore:
    def get_tool_scan_state(self, tool_key):
        return None

    def list_tool_skill_cache(self, tool_key):
        return []


def make_skill_dir(path):
    path.mkdir(parents=True, exist_ok=True)
    (path / "SKILL.md").write_text("---\nname: demo\n---\n", encoding="utf-8")
    return path


def test_onboarding_filters_symlink_to_custom_repo(tmp_path, monkeypatch):
    community_repo = tmp_path / "community"
    custom_repo = tmp_path / "custom"
    tool_skills = tmp_path / "tool-skills"
    custom_skill = make_skill_dir(custom_repo / "agent-workspace")
    linked_skill = make_skill_dir(tool_skills / "agent-workspace")
    external_skill = make_skill_dir(tool_skills / "external-skill")
    adapter = ToolAdapter(
        id=ToolId.ClaudeCode,
        display_name="Claude Code",
        relative_skills_dir=str(tool_skills),
        relative_detect_dir=str(tmp_path),
    )

    monkeypatch.setattr("core.db.store.get_store", lambda: FakeStore())
    monkeypatch.setattr("core.skills.onboarding.effective_tool_adapters", lambda: [adapter])
    monkeypatch.setattr("core.skills.onboarding.is_tool_installed", lambda _: True)
    monkeypatch.setattr("core.skills.onboarding.resolve_default_path", lambda _: str(tool_skills))
    monkeypatch.setattr(
        "core.skills.onboarding.scan_tool_dir",
        lambda *_: [
            DetectedSkill(
                tool=ToolId.ClaudeCode,
                name="agent-workspace",
                path=str(linked_skill),
                is_link=True,
                link_target=str(custom_skill),
            ),
            DetectedSkill(
                tool=ToolId.ClaudeCode,
                name="external-skill",
                path=str(external_skill),
                is_link=False,
                link_target=None,
            ),
        ],
    )

    plan = build_onboarding_plan(
        community_repo_path=str(community_repo),
        managed_target_paths=set(),
        custom_repo_path=str(custom_repo),
    )

    assert plan.total_skills_found == 1
    assert [group.name for group in plan.groups] == ["external-skill"]
