"""PyInstaller 打包配置 — 生成单文件 SkillsHub.exe"""
import subprocess
import sys
import shutil
from pathlib import Path


ROOT = Path(__file__).resolve().parent
STATIC_DIR = ROOT / "static"
FRONTEND_DIST_DIR = ROOT.parent / "frontend" / "dist"
DIST_DIR = ROOT.parent / "dist"
ICON_FILE = ROOT / "icon.ico"


def main():
    if not FRONTEND_DIST_DIR.is_dir():
        print("Error: frontend/dist directory not found. Run 'npm run build' in frontend/ first.")
        sys.exit(1)

    if STATIC_DIR.exists():
        shutil.rmtree(STATIC_DIR)
    shutil.copytree(FRONTEND_DIST_DIR, STATIC_DIR)

    DIST_DIR.mkdir(parents=True, exist_ok=True)

    hidden_imports = [
        "uvicorn.logging",
        "uvicorn.loops.auto",
        "uvicorn.protocols.http.auto",
        "fastapi",
        "pydantic",
        "pydantic.deprecated.decorator",
        "sqlite3",
        "webview",
        "core.config",
        "core.db",
        "core.db.store",
        "core.utils",
        "core.utils.content_hash",
        "core.utils.path_safety",
        "core.tasks",
        "core.tasks.manager",
        "core.error_codes",
        "core.skills",
        "core.skills.installer",
        "core.skills.sync_engine",
        "core.skills.files",
        "core.skills.source_paths",
        "core.skills.maintenance",
        "core.skills.onboarding",
        "core.tools",
        "core.tools.adapters",
        "core.repo",
        "core.repo.community",
        "core.repo.community_migration",
        "core.repo.scanner",
        "models",
        "models.schemas",
        "api",
        "api.health",
        "api.tags",
        "api.settings",
        "api.onboarding",
        "api.skills.crud",
        "api.skills.sync",
        "api.skills.files",
        "api.tools.status",
        "api.tools.tool_skills",
        "core.update",
        "core.update.checker",
        "core.update.updater",
        "api.update",
    ]

    add_data = []
    if STATIC_DIR.is_dir():
        add_data.append(f"{STATIC_DIR};static")

    args = [
        sys.executable, "-m", "PyInstaller",
        "--onefile",
        "--noconsole",
        f"--name=SkillsHub",
        f"--distpath={DIST_DIR}",
        str(ROOT / "desktop.py"),
    ]

    if ICON_FILE.is_file():
        args.append(f"--icon={ICON_FILE}")
    else:
        print(f"Warning: icon file not found at {ICON_FILE}, building without custom icon")

    for imp in hidden_imports:
        args.extend(["--hidden-import", imp])

    for data in add_data:
        args.extend(["--add-data", data])

    print(f"Running: {' '.join(args)}")
    subprocess.run(args, check=True)

    if ICON_FILE.is_file():
        shutil.copy2(ICON_FILE, DIST_DIR / "icon.ico")

    print(f"\nDone: {DIST_DIR / 'SkillsHub.exe'}")


if __name__ == "__main__":
    main()
