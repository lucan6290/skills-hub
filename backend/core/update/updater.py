"""更新执行模块 — 下载新版本并启动更新脚本"""
import logging
import os
import subprocess
import sys
import tempfile
import urllib.request

logger = logging.getLogger(__name__)


def _get_app_exe_path() -> str:
    """获取当前应用 exe 路径（仅在 PyInstaller 打包后有效）"""
    return sys.executable


def _generate_updater_bat(
    pid: str, file_path: str, install_mode: str, app_exe_path: str
) -> str:
    """生成 updater.bat 更新脚本内容

    参数:
        pid: 当前进程 PID
        file_path: 下载的更新文件路径
        install_mode: 安装模式 (setup / portable / naked)
        app_exe_path: 应用 exe 路径

    bat 脚本逻辑:
        1. 循环等待主进程退出
        2. 根据安装模式执行更新（静默安装 / 解压覆盖 / 替换 exe）
        3. 重启应用
        4. 自删除
    """
    # bat 文件注释用英文（bat 文件不支持中文注释）
    # 使用 %~dp4 获取应用 exe 所在目录，避免在 bat 中嵌入路径
    return f"""@echo off
:: SkillsHub Updater Script
:: Args: %1=current PID, %2=downloaded file, %3=install mode, %4=app exe path

echo Waiting for application to exit...

:wait_loop
tasklist /FI "PID eq %1" 2>nul | find "%1" >nul
if not errorlevel 1 (
    timeout /t 1 /nobreak >nul
    goto wait_loop
)

echo Application exited, starting update...

if "%3"=="setup" (
    echo Running NSIS silent install...
    "%2" /S
) else if "%3"=="portable" (
    echo Extracting portable ZIP...
    powershell -Command "Expand-Archive -Path '%2' -DestinationPath '%~dp4' -Force"
) else if "%3"=="naked" (
    echo Replacing exe...
    copy /Y "%2" "%4"
)

echo Restarting application...
start "" "%4"

:: Self-delete
(goto) 2>nul & del "%~f0"
"""


def _download_file(url: str, dest_path: str) -> None:
    """下载文件到指定路径"""
    urllib.request.urlretrieve(url, dest_path)


def perform_update(install_mode: str, download_urls: dict) -> dict:
    """执行更新：下载新版本并启动更新脚本

    参数:
        install_mode: 安装模式 (setup / portable / naked / dev)
        download_urls: 下载链接字典 {"setup": ..., "portable": ..., "exe": ...}

    返回:
        {"ok": bool, "message": str}
    """
    # 根据安装模式选择下载哪个文件
    if install_mode == "dev":
        raise ValueError("开发模式不支持自动更新")

    if install_mode == "setup":
        url = download_urls.get("setup", "")
    elif install_mode == "portable":
        url = download_urls.get("portable", "")
    elif install_mode == "naked":
        url = download_urls.get("exe", "")
    else:
        raise ValueError(f"未知的安装模式: {install_mode}")

    if not url:
        return {"ok": False, "message": "下载失败: 未找到对应产物下载链接"}

    # 下载到临时目录
    update_dir = os.path.join(tempfile.gettempdir(), "skillshub_update")
    os.makedirs(update_dir, exist_ok=True)

    # 从 URL 中提取文件名
    filename = url.split("/")[-1] or "update_file"
    dest_path = os.path.join(update_dir, filename)

    try:
        _download_file(url, dest_path)
    except Exception as e:
        logger.exception("下载更新文件失败")
        return {"ok": False, "message": f"下载失败: {e}"}

    # 生成并启动更新脚本
    pid = str(os.getpid())
    app_exe_path = _get_app_exe_path()
    bat_content = _generate_updater_bat(pid, dest_path, install_mode, app_exe_path)

    bat_path = os.path.join(update_dir, "updater.bat")
    with open(bat_path, "w", encoding="ascii") as f:
        f.write(bat_content)

    # 启动更新脚本（detached），主程序随后退出
    DETACHED_PROCESS = 0x00000008
    CREATE_NO_WINDOW = 0x08000000
    subprocess.Popen(
        ["cmd.exe", "/c", bat_path],
        creationflags=DETACHED_PROCESS | CREATE_NO_WINDOW,
        close_fds=True,
    )

    return {"ok": True, "message": "更新已准备就绪，应用即将重启..."}
