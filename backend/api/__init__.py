"""API 路由注册 — 显式导入所有路由模块并注册到 FastAPI app。

使用显式导入而非 pkgutil.walk_packages，因为后者在 PyInstaller 等
冻结打包环境中无法可靠枚举子模块，会导致部分路由（如 database）
在安装版本中缺失而返回 404。
"""
import importlib
import logging

logger = logging.getLogger(__name__)

# 所有包含 router = APIRouter() 的模块，必须显式列出以确保打包后可用
_ROUTER_MODULES = [
    "api.health",
    "api.database",
    "api.maintenance",
    "api.onboarding",
    "api.reorder",
    "api.settings",
    "api.tags",
    "api.tasks",
    "api.update",
    "api.skills.crud",
    "api.skills.files",
    "api.skills.sync",
    "api.tools.status",
    "api.tools.tool_skills",
]


def register_all_routers(app):
    """显式导入并注册所有 API 路由模块"""
    for mod_name in _ROUTER_MODULES:
        try:
            mod = importlib.import_module(mod_name)
            if hasattr(mod, "router"):
                app.include_router(mod.router)
        except ImportError:
            logger.warning("Failed to import API module: %s", mod_name, exc_info=True)
