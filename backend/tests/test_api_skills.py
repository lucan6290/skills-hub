"""Skills API 端点集成测试（使用隔离 store）"""


def test_get_managed_skills_returns_list(isolated_client):
    resp = isolated_client.get("/api/get_managed_skills")

    assert resp.status_code == 200
    assert isinstance(resp.json(), list)


def test_delete_managed_skill_missing_returns_404(isolated_client):
    resp = isolated_client.post(
        "/api/delete_managed_skill",
        json={"skill_id": "nonexistent-skill", "dry_run": True},
    )

    assert resp.status_code == 404
    assert "skill not found" in resp.json()["detail"]


def test_sync_skill_to_tool_unknown_tool_returns_400(isolated_client, tmp_path):
    resp = isolated_client.post(
        "/api/sync_skill_to_tool",
        json={
            "source_path": str(tmp_path),
            "skill_id": "any-skill",
            "tool": "nonexistent_tool",
            "name": "test-skill",
        },
    )

    assert resp.status_code == 400
    assert resp.json()["detail"] == "unknown tool"
