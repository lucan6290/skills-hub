"""设置 API 测试：打开目录 / 自定义仓库校验 / 恢复默认设置"""
from pathlib import Path

from api import settings as settings_module


def test_open_settings_folder_missing_dir(isolated_client):
    response = isolated_client.post(
        "/api/open_settings_folder",
        json={"path": "Z:/definitely/not/exist"},
    )
    assert response.status_code == 400


def test_open_settings_folder_ok(isolated_client, tmp_path, monkeypatch):
    monkeypatch.setattr(settings_module, "_open_folder", lambda p: None)
    response = isolated_client.post(
        "/api/open_settings_folder",
        json={"path": str(tmp_path)},
    )
    assert response.status_code == 200
    data = response.json()
    assert data["ok"] is True
    assert data["path"] == str(tmp_path)


def test_set_custom_repo_path_not_dir(isolated_client, tmp_path):
    missing = tmp_path / "missing"
    response = isolated_client.post(
        "/api/set_custom_repo_path",
        json={"path": str(missing)},
    )
    assert response.status_code == 400


def test_set_custom_repo_path_no_permission(isolated_client, tmp_path, monkeypatch):
    monkeypatch.setattr(settings_module, "_check_dir_writable", lambda p: False)
    response = isolated_client.post(
        "/api/set_custom_repo_path",
        json={"path": str(tmp_path)},
    )
    assert response.status_code == 400


def test_set_custom_repo_path_empty_dir(isolated_client, tmp_path):
    target = tmp_path / "custom"
    target.mkdir()
    response = isolated_client.post(
        "/api/set_custom_repo_path",
        json={"path": str(target)},
    )
    assert response.status_code == 200
    data = response.json()
    assert data["ok"] is True
    assert data["empty"] is True
    assert data["path"] == str(target)


def test_set_custom_repo_path_non_empty_dir(isolated_client, tmp_path):
    target = tmp_path / "custom"
    target.mkdir()
    (target / "skill.md").write_text("hello")
    response = isolated_client.post(
        "/api/set_custom_repo_path",
        json={"path": str(target)},
    )
    assert response.status_code == 200
    assert response.json()["empty"] is False


def test_reset_general_settings(isolated_client, isolated_store, tmp_path):
    isolated_store.set_setting("community_repo_path", str(tmp_path / "community"))
    isolated_store.set_setting("custom_repo_path", str(tmp_path / "custom"))

    response = isolated_client.post("/api/reset_general_settings")
    assert response.status_code == 200
    data = response.json()
    assert data["ok"] is True
    assert isolated_store.get_setting("community_repo_path") is None
    assert isolated_store.get_setting("custom_repo_path") is None
    assert Path(data["community_repo_path"]).is_absolute()
    assert Path(data["custom_repo_path"]).is_absolute()
