"""pytest 共享 fixtures"""
import os

# 测试统一以开发模式运行：生产模式的根路由依赖 backend/static（构建产物），
# 在 CI / 干净 clone 中不存在，会导致 GET / 返回 404。必须在导入 core.config 之前设置。
os.environ["SKILLS_HUB_DEV"] = "1"

import pytest
from fastapi.testclient import TestClient

import core.db.store as store_module
from core.db.store import SkillStore
from main import app


@pytest.fixture
def client():
    return TestClient(app)


@pytest.fixture
def isolated_store(tmp_path, monkeypatch):
    """使用临时 DB 的 SkillStore，并替换全局单例，隔离真实数据库。"""
    store = SkillStore(str(tmp_path / "isolated_skills_hub.db"))
    store.ensure_schema()
    monkeypatch.setattr(store_module, "_store_instance", store)
    yield store
    store.close()


@pytest.fixture
def isolated_client(isolated_store):
    """基于隔离 store 的 TestClient，使 Depends(get_skill_store) 拿到临时 DB。"""
    return TestClient(app)
