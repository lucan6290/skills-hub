"""Skills Hub 桌面入口 — pywebview 壳 + 单实例管理"""
import os
import subprocess
import sys
import threading
import time

import uvicorn
import webview

from core.config import API_HOST, API_PORT, resolve_data_dir

WINDOW_TITLE = "Skills Hub"
WINDOW_WIDTH = 1200
WINDOW_HEIGHT = 800
WINDOW_MIN_WIDTH = 900
WINDOW_MIN_HEIGHT = 600

# Windows: 隐藏 subprocess 控制台窗口
_CREATE_NO_WINDOW = 0x08000000 if sys.platform == "win32" else 0


def _run_hidden(args, **kwargs):
    """以无窗口方式运行子进程（Windows 下隐藏控制台）"""
    if sys.platform == "win32":
        kwargs.setdefault("creationflags", _CREATE_NO_WINDOW)
    kwargs.setdefault("shell", False)
    return subprocess.run(args, **kwargs)


def _check_output_hidden(args, **kwargs):
    """以无窗口方式运行子进程并捕获输出（Windows 下隐藏控制台）"""
    if sys.platform == "win32":
        kwargs.setdefault("creationflags", _CREATE_NO_WINDOW)
    kwargs.setdefault("shell", False)
    kwargs.setdefault("text", True)
    return subprocess.check_output(args, **kwargs)


def _find_pid_on_port(port: int):
    """Windows: 通过 netstat 查找占用端口的 PID"""
    try:
        output = _check_output_hidden(
            ["netstat", "-ano"],
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
    """检查进程名是否包含 skills_hub 或 skillshub"""
    try:
        output = _check_output_hidden(
            ["tasklist", "/FI", f"PID eq {pid}", "/FO", "CSV", "/NH"],
        )
        name = output.strip().strip('"').split('","')[0].lower()
        return "skillshub" in name or "skills_hub" in name
    except Exception:
        return False


def _kill_process(pid: int):
    """强制杀掉指定 PID 进程"""
    try:
        _run_hidden(["taskkill", "/F", "/PID", str(pid)], capture_output=True)
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
    webview.create_window(
        WINDOW_TITLE,
        url=f"http://{API_HOST}:{API_PORT}",
        width=WINDOW_WIDTH,
        height=WINDOW_HEIGHT,
        min_size=(WINDOW_MIN_WIDTH, WINDOW_MIN_HEIGHT),
    )
    webview.start()


if __name__ == "__main__":
    main()
