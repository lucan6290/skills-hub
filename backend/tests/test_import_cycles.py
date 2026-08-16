"""循环依赖防护测试。

`core.skills.sync_engine` 与 `core.tools.adapters` 通过函数级延迟导入避免循环依赖：

- sync_engine.py 在函数体内 ``from core.tools.adapters import adapter_by_key``
- adapters.py 在函数体内 ``from core.skills.sync_engine import _is_junction``

本文件从两层防护：

1. 运行时：在干净的 Python 子进程中分别导入这两个模块，若有人把延迟导入提升到
   模块级，会触发 ImportError（循环导入）导致 returncode 非 0。
2. 静态：扫描源码，断言这两条 import 只允许出现在函数体（带缩进的行）中，
   防止未来把延迟导入提升到模块顶层。
"""
import re
import subprocess
import sys
from pathlib import Path

BACKEND_DIR = Path(__file__).resolve().parents[1]
CORE_DIR = BACKEND_DIR / "core"
SYNC_ENGINE_SRC = BACKEND_DIR / "core" / "skills" / "sync_engine.py"
ADAPTERS_SRC = BACKEND_DIR / "core" / "tools" / "adapters.py"


def _import_returncode(module: str) -> int:
    """在干净子进程中导入模块，返回进程退出码。

    以 backend 目录为工作目录（`python -c` 会把当前目录加入 sys.path），
    环境变量默认继承，从而保留 PYTHONPATH 等既有配置。
    """
    result = subprocess.run(
        [sys.executable, "-c", f"import {module}"],
        cwd=str(BACKEND_DIR),
        capture_output=True,
        text=True,
    )
    return result.returncode


def test_sync_engine_imports_without_cycle():
    """sync_engine 可在干净进程中独立导入，不触发循环导入。"""
    assert _import_returncode("core.skills.sync_engine") == 0


def test_adapters_imports_without_cycle():
    """adapters 可在干净进程中独立导入，不触发循环导入。"""
    assert _import_returncode("core.tools.adapters") == 0


def _module_level_import_present(source: Path, import_stmt: str) -> bool:
    """检查源码中是否存在模块顶层（缩进为 0）的指定 import 语句。

    判定规则：某行以 import_stmt 开头（即该行不以空格或制表符开头），
    即视为模块级 import。
    """
    for line in source.read_text(encoding="utf-8").splitlines():
        if line.startswith(import_stmt):
            return True
    return False


def test_sync_engine_keeps_adapters_import_lazy():
    """sync_engine 对 adapters 的依赖必须保持函数级延迟导入，防止循环依赖。"""
    assert not _module_level_import_present(
        SYNC_ENGINE_SRC, "from core.tools.adapters import"
    )


def test_adapters_keeps_sync_engine_import_lazy():
    """adapters 对 sync_engine 的依赖必须保持函数级延迟导入，防止循环依赖。"""
    assert not _module_level_import_present(
        ADAPTERS_SRC, "from core.skills.sync_engine import"
    )


_CORE_API_IMPORT_RE = re.compile(
    r"^(from\s+api(\.\S+)?\s+import|import\s+api(\.\S+)?(\s|,|$))"
)


def test_core_has_no_api_imports():
    """防止 core→api 反向依赖回归。

    遍历 core/ 目录下所有 .py 文件，断言不存在顶层（行首无缩进）的
    ``from api`` / ``from api.`` / ``import api`` 语句。core 层是纯业务
    逻辑层，不得依赖 api 层与 fastapi。
    """
    offenders = []
    for py_file in sorted(CORE_DIR.rglob("*.py")):
        lines = py_file.read_text(encoding="utf-8").splitlines()
        for lineno, line in enumerate(lines, 1):
            if _CORE_API_IMPORT_RE.match(line):
                rel = py_file.relative_to(BACKEND_DIR)
                offenders.append(f"{rel}:{lineno}: {line.strip()}")
    assert not offenders, (
        "core/ 下出现顶层 api 导入（core→api 反向依赖）:\n" + "\n".join(offenders)
    )
