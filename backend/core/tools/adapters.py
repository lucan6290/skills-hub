"""AI 工具适配器模块

定义 AI 工具的适配器，包括路径解析、安装检测、技能目录扫描等功能。

**数据库优先原则**: 工具的名称、路径、同步能力等配置统一存储在 SQLite 的
tool_adapter_configs 表中。运行时从数据库读取构建 ToolAdapter 对象。
默认值定义在 core.config.DEFAULT_TOOL_ADAPTERS，首次启动时写入数据库；
环境变量可在首次写入前覆盖默认值。用户在 UI 上的修改直接更新数据库，立即生效。
"""
from __future__ import annotations

import logging
import os
import sys
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)


# ── ToolId Enum ──────────────────────────────────────────
# 保留 ToolId 枚举用于 scan_tool_dir 中的 Codex 特殊逻辑以及 DetectedSkill 类型引用。
# 内置工具的实际配置来源是 config.DEFAULT_TOOL_ADAPTERS（而非此枚举）。


class ToolId(Enum):
    """内置 AI 工具标识符（用于代码内引用，配置来自数据库）"""
    Cursor = "cursor"
    ClaudeCode = "claude_code"
    Codex = "codex"
    OpenCode = "opencode"
    Antigravity = "antigravity"
    Amp = "amp"
    KimiCli = "kimi_cli"
    Augment = "augment"
    OpenClaw = "openclaw"
    Copaw = "copaw"
    Cline = "cline"
    CodeBuddy = "codebuddy"
    CommandCode = "command_code"
    Continue = "continue"
    Crush = "crush"
    Junie = "junie"
    IflowCli = "iflow_cli"
    KiroCli = "kiro_cli"
    Kode = "kode"
    McpJam = "mcpjam"
    MistralVibe = "mistral_vibe"
    Mux = "mux"
    OpenClaude = "openclaude"
    OpenHands = "openhands"
    Pi = "pi"
    Qoder = "qoder"
    QoderWork = "qoderwork"
    QwenCode = "qwen_code"
    Trae = "trae"
    TraeCn = "trae_cn"
    Zencoder = "zencoder"
    Neovate = "neovate"
    Pochi = "pochi"
    AdaL = "adal"
    KiloCode = "kilo_code"
    RooCode = "roo_code"
    Goose = "goose"
    GeminiCli = "gemini_cli"
    GithubCopilot = "github_copilot"
    Clawdbot = "clawdbot"
    Droid = "droid"
    Windsurf = "windsurf"
    Moltbot = "moltbot"
    HermesAgent = "hermes_agent"

    def as_key(self) -> str:
        return self.value


@dataclass(frozen=True)
class CustomToolId:
    key: str

    def as_key(self) -> str:
        return self.key


# ── Data Classes ─────────────────────────────────────────


@dataclass
class ToolAdapter:
    """工具适配器配置（从数据库记录构建，DB 中值即为生效值）"""
    id: ToolId | CustomToolId
    display_name: str
    relative_skills_dir: str  # 全局 skills 目录（相对 home 或绝对路径）
    relative_detect_dir: str  # 检测目录
    supports_symlink: bool = True
    supports_junction: bool = True
    force_copy: bool = False
    supports_project_scope_override: Optional[bool] = None
    project_relative_skills_dir: Optional[str] = None
    is_custom: bool = False


@dataclass
class DetectedSkill:
    """检测到的技能"""
    tool: ToolId | CustomToolId
    name: str
    path: str
    is_link: bool
    link_target: Optional[str] = None


# ── Built-in ToolId 查找 ────────────────────────────────

_BUILTIN_TOOL_IDS: dict[str, ToolId] = {t.value: t for t in ToolId}


def _make_tool_id(key: str, is_custom: bool) -> ToolId | CustomToolId:
    """根据 key 和 is_custom 返回 ToolId 枚举或 CustomToolId。"""
    if not is_custom:
        tid = _BUILTIN_TOOL_IDS.get(key)
        if tid is not None:
            return tid
    return CustomToolId(key=key)


# ── Default Adapters (from config) ──────────────────────


def default_tool_adapters() -> list[ToolAdapter]:
    """从 core.config.DEFAULT_TOOL_ADAPTERS 构建默认适配器列表。
    用于数据库不可用（如初始化早期）时的回退，以及测试。"""
    from core.config import DEFAULT_TOOL_ADAPTERS
    result: list[ToolAdapter] = []
    for key, cfg in DEFAULT_TOOL_ADAPTERS.items():
        result.append(ToolAdapter(
            id=_make_tool_id(key, is_custom=False),
            display_name=cfg["display_name"],
            relative_skills_dir=cfg["skills_dir"],
            relative_detect_dir=cfg["detect_dir"],
            supports_symlink=cfg.get("supports_symlink", True),
            supports_junction=cfg.get("supports_junction", True),
            force_copy=cfg.get("force_copy", False),
            supports_project_scope_override=cfg.get("supports_project_scope"),
            project_relative_skills_dir=cfg.get("project_skills_dir"),
            is_custom=False,
        ))
    return result


# ── DB-first: effective adapters from database ──────────


def _adapter_from_db_config(config) -> ToolAdapter:
    """从数据库 ToolAdapterConfigRecord 构建 ToolAdapter（DB 值即为权威值）。"""
    return ToolAdapter(
        id=_make_tool_id(config.tool_key, is_custom=config.is_custom),
        display_name=config.display_name,
        relative_skills_dir=config.skills_dir,
        relative_detect_dir=config.detect_dir,
        supports_symlink=config.supports_symlink,
        supports_junction=config.supports_junction,
        force_copy=config.force_copy,
        supports_project_scope_override=config.supports_project_scope,
        project_relative_skills_dir=config.project_skills_dir,
        is_custom=config.is_custom,
    )


def effective_tool_adapters() -> list[ToolAdapter]:
    """数据库优先：从 tool_adapter_configs 表读取所有启用的工具配置。
    如果数据库不可用（初始化早期/异常情况），回退到 config 中的默认值。"""
    try:
        from core.db.store import get_store
        configs = get_store().list_tool_adapter_configs()
        if configs:
            return [_adapter_from_db_config(c) for c in configs]
    except Exception:
        pass
    # 回退：使用 config 默认值
    return default_tool_adapters()


def _normalize_tool_key(key: str) -> str:
    cleaned = key.strip().lower().replace(" ", "_").replace("-", "_")
    return "".join(ch for ch in cleaned if ch.isalnum() or ch == "_")


def _normalize_scope(scope: str | None) -> str:
    """Normalize and validate scope value."""
    s = (scope or "global").lower()
    if s not in ("global", "project"):
        raise ValueError(f"invalid scope: {s}")
    return s


def _target_base_for_record(target) -> str:
    """Resolve base path for a SkillTargetRecord (reused across sync, crud, installer)."""
    adapter = adapter_by_key(target.tool)
    if not adapter:
        raise ValueError(f"unknown tool: {target.tool}")
    if _normalize_scope(target.scope) == "project":
        if not target.project_path:
            raise ValueError("project path missing for target")
        return resolve_project_path(adapter, target.project_path)
    return resolve_default_path(adapter)


def adapter_by_key(key: str) -> Optional[ToolAdapter]:
    """根据工具键查找适配器"""
    normalized = _normalize_tool_key(key)
    for adapter in effective_tool_adapters():
        if adapter.id.as_key() == normalized:
            return adapter
    return None


# ── Path Resolution ──────────────────────────────────────


def _home_dir() -> Path:
    """获取用户主目录（跨平台）"""
    return Path.home()


def resolve_default_path(adapter: ToolAdapter) -> str:
    """解析工具全局技能目录路径"""
    configured = Path(adapter.relative_skills_dir).expanduser()
    if configured.is_absolute():
        return str(configured)
    home = _home_dir()
    return str(home / configured)


def _resolve_detect_path(adapter: ToolAdapter) -> str:
    """解析工具检测目录路径"""
    configured = Path(adapter.relative_detect_dir).expanduser()
    if configured.is_absolute():
        return str(configured)
    home = _home_dir()
    return str(home / configured)


def _project_relative_skills_dir(adapter: ToolAdapter) -> str:
    """返回项目级 skills 目录（相对于项目根）。"""
    if adapter.project_relative_skills_dir:
        return adapter.project_relative_skills_dir
    return adapter.relative_skills_dir


def resolve_project_path(adapter: ToolAdapter, project_root: str) -> str:
    """解析工具在项目内的技能目录路径"""
    return str(Path(project_root) / _project_relative_skills_dir(adapter))


def supports_project_scope(adapter: ToolAdapter) -> bool:
    """工具是否支持项目级别技能"""
    if adapter.supports_project_scope_override is not None:
        return adapter.supports_project_scope_override
    # hermes_agent 默认不支持（兼容旧逻辑）
    return adapter.id.as_key() != "hermes_agent"


def tool_sync_capabilities(adapter: ToolAdapter) -> dict[str, bool]:
    """Return normalized sync capabilities for API/UI and sync decisions."""
    return {
        "supports_symlink": adapter.supports_symlink,
        "supports_junction": adapter.supports_junction,
        "force_copy": adapter.force_copy,
        "supports_project_scope": supports_project_scope(adapter),
    }


# ── Installation Detection ───────────────────────────────


def is_tool_installed(adapter: ToolAdapter) -> bool:
    """检测工具是否已安装"""
    if not adapter.relative_detect_dir:
        return False
    detect_path = _resolve_detect_path(adapter)
    return os.path.exists(detect_path)


# ── Skills Dir Sharing ───────────────────────────────────


def adapters_sharing_skills_dir(adapter: ToolAdapter) -> list[ToolAdapter]:
    """查找共享同一全局技能目录的所有适配器"""
    return [
        a for a in effective_tool_adapters()
        if a.relative_skills_dir == adapter.relative_skills_dir
    ]


def adapters_sharing_project_skills_dir(adapter: ToolAdapter) -> list[ToolAdapter]:
    """查找共享同一项目技能目录的所有适配器"""
    relative = _project_relative_skills_dir(adapter)
    return [
        a for a in effective_tool_adapters()
        if _project_relative_skills_dir(a) == relative
    ]


# ── Skill Scanning ───────────────────────────────────────


def _detect_link(path: str) -> tuple[bool, Optional[str]]:
    """检测路径是否为符号链接或 Windows junction，对应 Rust detect_link"""
    def _clean(target: str) -> str:
        """剥离 Windows 扩展长度路径前缀 \\\\?\\ ，避免展示给用户。"""
        s = str(target)
        for prefix in ("\\\\?\\", "\\??\\"):
            if s.startswith(prefix):
                s = s[len(prefix):]
        return s

    p = Path(path)
    try:
        if p.is_symlink():
            target = os.readlink(path)
            return (True, _clean(target))
    except OSError:
        pass

    # Also try readlink in case is_symlink returns False but it's still a link
    try:
        target = os.readlink(path)
        return (True, _clean(target))
    except OSError:
        pass

    # Windows junction: is_symlink 返回 False, readlink 在 Python < 3.12 也会失败
    if sys.platform == "win32":
        try:
            from core.skills.sync_engine import _is_junction
            if _is_junction(p):
                # 尝试获取 junction 目标（Python 3.12+ 的 os.readlink 支持 junction）
                try:
                    target = os.readlink(path)
                    return (True, _clean(target))
                except OSError:
                    pass
                # 低版本 Python 无法读取目标，但仍标记为链接
                return (True, None)
        except Exception:
            pass

    return (False, None)


def scan_tool_dir(tool: ToolAdapter, dir_path: str) -> list[DetectedSkill]:
    """扫描目录查找技能，完全对应 Rust scan_tool_dir"""
    results: list[DetectedSkill] = []
    dir_p = Path(dir_path)

    if not dir_p.exists():
        return results

    ignore_hint = "Application Support/com.tauri.dev/skills"

    try:
        entries = sorted(dir_p.iterdir())
    except OSError:
        return results

    for entry in entries:
        path = str(entry)
        try:
            is_dir = entry.is_dir() or (entry.is_symlink() and Path(entry.resolve()).is_dir())
        except OSError:
            continue
        if not is_dir:
            continue

        name = entry.name

        # Codex 跳过 .system 目录
        if tool.id == ToolId.Codex and name == ".system":
            continue

        is_link, link_target = _detect_link(path)

        # 跳过包含 ignore_hint 的路径
        if ignore_hint in path.replace("\\", "/"):
            continue
        if link_target and ignore_hint in link_target.replace("\\", "/"):
            continue

        try:
            if not (entry / "SKILL.md").is_file():
                continue
        except OSError:
            continue

        results.append(DetectedSkill(
            tool=tool.id,
            name=name,
            path=path,
            is_link=is_link,
            link_target=link_target,
        ))

    return results
