"""更新检查与执行 API"""
import logging
import sys

from fastapi import APIRouter

from core.config import _get_exe_dir
from core.update.checker import check_for_update
from core.update.updater import perform_update
from core.version import __version__

router = APIRouter()
logger = logging.getLogger(__name__)


def get_install_mode() -> str:
    """检测当前安装模式: dev / setup / portable / naked"""
    if not getattr(sys, "frozen", False):
        return "dev"
    exe_dir = _get_exe_dir()
    if exe_dir is None:
        return "naked"
    if (exe_dir / "portable.flag").exists():
        return "portable"
    if (exe_dir / "installed.flag").exists():
        return "setup"
    return "naked"


@router.get("/api/check_update", summary="检查更新")
async def check_update():
    """检查 GitHub Releases 是否有新版本"""
    result = check_for_update(__version__)
    result["install_mode"] = get_install_mode()
    return result


@router.post("/api/perform_update", summary="执行更新")
async def do_update():
    """下载并执行更新（仅安装版/便携版/裸exe可用）"""
    mode = get_install_mode()
    if mode == "dev":
        return {"ok": False, "message": "开发模式不支持自动更新"}

    check_result = check_for_update(__version__)
    if not check_result.get("update_available"):
        return {"ok": False, "message": "当前已是最新版本"}

    download_urls = check_result.get("download_urls", {})
    return perform_update(mode, download_urls)
