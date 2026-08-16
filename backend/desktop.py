"""Skills Hub 桌面入口 — pywebview 壳 + 单实例管理"""
import os
import subprocess
import sys
import threading
import time
from pathlib import Path

import uvicorn
import webview

from core.config import API_HOST, API_PORT, resolve_data_dir

WINDOW_TITLE = "Skills Hub"
WINDOW_WIDTH = 1200
WINDOW_HEIGHT = 800
WINDOW_MIN_WIDTH = 900
WINDOW_MIN_HEIGHT = 600


def _get_icon_path() -> str | None:
    """获取窗口图标路径（兼容开发和打包环境）"""
    if getattr(sys, "frozen", False):
        base = Path(sys.executable).parent
        icon = base / "icon.ico"
    else:
        icon = Path(__file__).resolve().parent / "icon.ico"
    return str(icon) if icon.is_file() else None


def _find_pid_on_port(port: int):
    """Windows: 通过 netstat 查找占用端口的 PID"""
    try:
        output = subprocess.check_output(
            f'netstat -ano | findstr :{port}', shell=True, text=True
        )
        for line in output.strip().split("\n"):
            parts = line.strip().split()
            if len(parts) >= 5 and parts[1].endswith(f":{port}"):
                if parts[3] == "LISTENING":
                    return int(parts[4])
    except subprocess.CalledProcessError:
        pass
    return None


def _is_skills_hub_process(pid: int):
    """检查进程名是否包含 skills_hub 或 python"""
    try:
        output = subprocess.check_output(
            f'tasklist /FI "PID eq {pid}" /FO CSV /NH', shell=True, text=True
        )
        name = output.strip().strip('"').split('","')[0].lower()
        return "skillshub" in name or "skills_hub" in name
    except Exception:
        return False


def _kill_process(pid: int):
    """强制杀掉指定 PID 进程"""
    try:
        subprocess.run(f"taskkill /F /PID {pid}", shell=True, capture_output=True)
    except Exception:
        pass


def ensure_port_available(port: int):
    """确保端口可用；如被 SkillsHub 占用则激活已有窗口退出，否则杀进程"""
    pid = _find_pid_on_port(port)
    if pid is None:
        return

    if _is_skills_hub_process(pid):
        # 已有实例运行中，激活窗口并退出
        try:
            import ctypes
            ctypes.windll.user32.ShowWindow(
                ctypes.windll.user32.FindWindowW(None, WINDOW_TITLE), 9
            )
        except Exception:
            pass
        sys.exit(0)

    # 其他进程占用，杀掉
    _kill_process(pid)
    deadline = time.time() + 5
    while time.time() < deadline:
        if _find_pid_on_port(port) is None:
            return
        time.sleep(0.5)
    print(f"Warning: port {port} still in use after killing process", file=sys.stderr)


def run_api():
    """启动 FastAPI（后台线程）"""
    from main import app
    uvicorn.run(app, host=API_HOST, port=API_PORT, log_level="warning")


def main():
    # 确保数据目录存在
    data_dir = resolve_data_dir()
    os.makedirs(data_dir, exist_ok=True)

    # 端口冲突处理
    ensure_port_available(API_PORT)

    # 启动 API 线程
    api_thread = threading.Thread(target=run_api, daemon=True)
    api_thread.start()

    # 等待 API 就绪
    deadline = time.time() + 10
    while time.time() < deadline:
        pid = _find_pid_on_port(API_PORT)
        if pid is not None:
            break
        time.sleep(0.3)
    else:
        print("Error: API failed to start", file=sys.stderr)
        sys.exit(1)

    # 创建窗口
    window_kwargs = dict(
        url=f"http://{API_HOST}:{API_PORT}",
        width=WINDOW_WIDTH,
        height=WINDOW_HEIGHT,
        min_size=(WINDOW_MIN_WIDTH, WINDOW_MIN_HEIGHT),
    )
    icon_path = _get_icon_path()
    if icon_path:
        window_kwargs["icon"] = icon_path
    webview.create_window(WINDOW_TITLE, **window_kwargs)
    webview.start()


if __name__ == "__main__":
    main()
