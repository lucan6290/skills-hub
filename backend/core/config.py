"""全局配置"""
import json
import os
import sys
from pathlib import Path
from typing import Any

# ── .env 文件加载 ──────────────────────────────────────
# 在项目根目录或 backend/ 目录下查找 .env 文件并加载环境变量。
# 使用 try-import 避免 python-dotenv 成为硬依赖（未安装时跳过）。
def _load_dotenv():
    try:
        from dotenv import load_dotenv
    except ImportError:
        return
    # 查找 .env 文件：优先 backend/.env，其次项目根目录 .env
    search_dirs = [Path(__file__).resolve().parent.parent, Path(__file__).resolve().parent.parent.parent]
    for d in search_dirs:
        env_file = d / ".env"
        if env_file.is_file():
            load_dotenv(env_file, override=False)
            break

_load_dotenv()

# 后端 API 端口，可通过环境变量覆盖
API_PORT = int(os.environ.get("SKILLS_HUB_PORT", "18921"))
API_HOST = "127.0.0.1"

# 开发模式标记
IS_DEV_MODE = os.environ.get("SKILLS_HUB_DEV", "").strip() in ("1", "true", "yes")

# Community Repo 默认名称
DEFAULT_COMMUNITY_REPO_NAME = ".skillshub"

# 数据库文件名
DB_FILE_NAME = "skills_hub.db"

# 旧版 app identifiers（用于数据库迁移）
LEGACY_APP_IDENTIFIERS = ["com.tauri.dev", "com.tauri.dev.skillshub"]


# ── 默认工具适配器配置 ──────────────────────────────────
# 所有内置 AI 工具的默认配置。格式: dict[tool_key] = {字段: 值}
# 数据库初始化时写入 tool_adapter_configs 表；环境变量可覆盖单个字段。
#
# 环境变量格式: SKILLS_HUB_TOOL_<KEY>_<FIELD>
#   例如: SKILLS_HUB_TOOL_TRAE_CN_SKILLS_DIR="D:/my-trae/skills"
#   布尔字段用 "1"/"true"/"yes" 表示 True，其余为 False。
#
# 也可通过 SKILLS_HUB_TOOL_OVERRIDES 传入 JSON 覆盖多个工具:
#   SKILLS_HUB_TOOL_OVERRIDES='{"trae_cn":{"skills_dir":"D:/x"},"cursor":{"force_copy":false}}'

_DEFAULT_TOOL_ADAPTERS_RAW: dict[str, dict[str, Any]] = {
    "cursor": {
        "display_name": "Cursor",
        "skills_dir": ".cursor/skills",
        "detect_dir": ".cursor",
        "project_skills_dir": ".agents/skills",
        "supports_symlink": False,
        "supports_junction": False,
        "force_copy": True,
        "supports_project_scope": True,
    },
    "claude_code": {
        "display_name": "Claude Code",
        "skills_dir": ".claude/skills",
        "detect_dir": ".claude",
        "project_skills_dir": ".claude/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "codex": {
        "display_name": "Codex",
        "skills_dir": ".codex/skills",
        "detect_dir": ".codex",
        "project_skills_dir": ".agents/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "opencode": {
        "display_name": "OpenCode",
        "skills_dir": ".config/opencode/skills",
        "detect_dir": ".config/opencode",
        "project_skills_dir": ".agents/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "antigravity": {
        "display_name": "Antigravity",
        "skills_dir": ".gemini/antigravity/skills",
        "detect_dir": ".gemini/antigravity",
        "project_skills_dir": ".agents/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "amp": {
        "display_name": "Amp",
        "skills_dir": ".config/agents/skills",
        "detect_dir": ".config/agents",
        "project_skills_dir": ".agents/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "kimi_cli": {
        "display_name": "Kimi Code CLI",
        "skills_dir": ".config/agents/skills",
        "detect_dir": ".config/agents",
        "project_skills_dir": ".agents/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "augment": {
        "display_name": "Augment",
        "skills_dir": ".augment/skills",
        "detect_dir": ".augment",
        "project_skills_dir": ".augment/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "openclaw": {
        "display_name": "OpenClaw",
        "skills_dir": ".openclaw/skills",
        "detect_dir": ".openclaw",
        "project_skills_dir": "skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "copaw": {
        "display_name": "Copaw",
        "skills_dir": ".copaw/skill_pool",
        "detect_dir": ".copaw",
        "project_skills_dir": ".copaw/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "cline": {
        "display_name": "Cline",
        "skills_dir": ".agents/skills",
        "detect_dir": ".agents",
        "project_skills_dir": ".agents/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "codebuddy": {
        "display_name": "CodeBuddy",
        "skills_dir": ".codebuddy/skills",
        "detect_dir": ".codebuddy",
        "project_skills_dir": ".codebuddy/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "command_code": {
        "display_name": "Command Code",
        "skills_dir": ".commandcode/skills",
        "detect_dir": ".commandcode",
        "project_skills_dir": ".commandcode/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "continue": {
        "display_name": "Continue",
        "skills_dir": ".continue/skills",
        "detect_dir": ".continue",
        "project_skills_dir": ".continue/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "crush": {
        "display_name": "Crush",
        "skills_dir": ".config/crush/skills",
        "detect_dir": ".config/crush",
        "project_skills_dir": ".crush/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "junie": {
        "display_name": "Junie",
        "skills_dir": ".junie/skills",
        "detect_dir": ".junie",
        "project_skills_dir": ".junie/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "iflow_cli": {
        "display_name": "iFlow CLI",
        "skills_dir": ".iflow/skills",
        "detect_dir": ".iflow",
        "project_skills_dir": ".iflow/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "kiro_cli": {
        "display_name": "Kiro CLI",
        "skills_dir": ".kiro/skills",
        "detect_dir": ".kiro",
        "project_skills_dir": ".kiro/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "kode": {
        "display_name": "Kode",
        "skills_dir": ".kode/skills",
        "detect_dir": ".kode",
        "project_skills_dir": ".kode/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "mcpjam": {
        "display_name": "MCPJam",
        "skills_dir": ".mcpjam/skills",
        "detect_dir": ".mcpjam",
        "project_skills_dir": ".mcpjam/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "mistral_vibe": {
        "display_name": "Mistral Vibe",
        "skills_dir": ".vibe/skills",
        "detect_dir": ".vibe",
        "project_skills_dir": ".vibe/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "mux": {
        "display_name": "Mux",
        "skills_dir": ".mux/skills",
        "detect_dir": ".mux",
        "project_skills_dir": ".mux/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "openclaude": {
        "display_name": "OpenClaude IDE",
        "skills_dir": ".openclaude/skills",
        "detect_dir": ".openclaude",
        "project_skills_dir": ".openclaude/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "openhands": {
        "display_name": "OpenHands",
        "skills_dir": ".openhands/skills",
        "detect_dir": ".openhands",
        "project_skills_dir": ".openhands/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "pi": {
        "display_name": "Pi",
        "skills_dir": ".pi/agent/skills",
        "detect_dir": ".pi",
        "project_skills_dir": ".pi/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "qoder": {
        "display_name": "Qoder",
        "skills_dir": ".qoder/skills",
        "detect_dir": ".qoder",
        "project_skills_dir": ".qoder/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "qoderwork": {
        "display_name": "QoderWork",
        "skills_dir": ".qoderwork/skills",
        "detect_dir": ".qoderwork",
        "project_skills_dir": ".qoder/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "qwen_code": {
        "display_name": "Qwen Code",
        "skills_dir": ".qwen/skills",
        "detect_dir": ".qwen",
        "project_skills_dir": ".qwen/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "trae": {
        "display_name": "Trae",
        "skills_dir": ".trae/skills",
        "detect_dir": ".trae",
        "project_skills_dir": ".trae/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "trae_cn": {
        "display_name": "Trae CN",
        "skills_dir": ".trae-cn/skills",
        "detect_dir": ".trae-cn",
        "project_skills_dir": ".trae/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "zencoder": {
        "display_name": "Zencoder",
        "skills_dir": ".zencoder/skills",
        "detect_dir": ".zencoder",
        "project_skills_dir": ".zencoder/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "neovate": {
        "display_name": "Neovate",
        "skills_dir": ".neovate/skills",
        "detect_dir": ".neovate",
        "project_skills_dir": ".neovate/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "pochi": {
        "display_name": "Pochi",
        "skills_dir": ".pochi/skills",
        "detect_dir": ".pochi",
        "project_skills_dir": ".pochi/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "adal": {
        "display_name": "AdaL",
        "skills_dir": ".adal/skills",
        "detect_dir": ".adal",
        "project_skills_dir": ".adal/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "kilo_code": {
        "display_name": "Kilo Code",
        "skills_dir": ".kilocode/skills",
        "detect_dir": ".kilocode",
        "project_skills_dir": ".kilocode/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "roo_code": {
        "display_name": "Roo Code",
        "skills_dir": ".roo/skills",
        "detect_dir": ".roo",
        "project_skills_dir": ".roo/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "goose": {
        "display_name": "Goose",
        "skills_dir": ".config/goose/skills",
        "detect_dir": ".config/goose",
        "project_skills_dir": ".goose/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "gemini_cli": {
        "display_name": "Gemini CLI",
        "skills_dir": ".gemini/skills",
        "detect_dir": ".gemini",
        "project_skills_dir": ".agents/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "github_copilot": {
        "display_name": "GitHub Copilot",
        "skills_dir": ".copilot/skills",
        "detect_dir": ".copilot",
        "project_skills_dir": ".agents/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "clawdbot": {
        "display_name": "Clawdbot",
        "skills_dir": ".clawdbot/skills",
        "detect_dir": ".clawdbot",
        "project_skills_dir": ".clawdbot/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "droid": {
        "display_name": "Droid",
        "skills_dir": ".factory/skills",
        "detect_dir": ".factory",
        "project_skills_dir": ".factory/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "windsurf": {
        "display_name": "Windsurf",
        "skills_dir": ".codeium/windsurf/skills",
        "detect_dir": ".codeium/windsurf",
        "project_skills_dir": ".windsurf/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "moltbot": {
        "display_name": "MoltBot",
        "skills_dir": ".moltbot/skills",
        "detect_dir": ".moltbot",
        "project_skills_dir": ".moltbot/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": True,
    },
    "hermes_agent": {
        "display_name": "Hermes Agent",
        "skills_dir": ".hermes/skills",
        "detect_dir": ".hermes",
        "project_skills_dir": ".hermes/skills",
        "supports_symlink": True,
        "supports_junction": True,
        "force_copy": False,
        "supports_project_scope": False,
    },
}


def _apply_env_overrides(defaults: dict[str, dict[str, Any]]) -> dict[str, dict[str, Any]]:
    """应用环境变量覆盖到默认工具配置（返回深拷贝，不修改原字典）。"""
    import copy
    result = copy.deepcopy(defaults)

    # 1) 单字段覆盖: SKILLS_HUB_TOOL_<KEY>_<FIELD>
    bool_fields = {"supports_symlink", "supports_junction", "force_copy", "supports_project_scope"}
    str_fields = {"display_name", "skills_dir", "detect_dir", "project_skills_dir"}

    for env_key, env_val in os.environ.items():
        if not env_key.startswith("SKILLS_HUB_TOOL_"):
            continue
        suffix = env_key[len("SKILLS_HUB_TOOL_"):]
        # 跳过 JSON 覆盖
        if suffix == "OVERRIDES":
            continue
        # 找最后一个 _ 之前的部分作为 tool_key，之后作为 field
        parts = suffix.rsplit("_", 1)
        if len(parts) != 2:
            continue
        raw_key, field_lower = parts[0].lower(), parts[1].lower()
        # field 名映射: display_name, skills_dir, detect_dir, project_skills_dir, supports_symlink, ...
        field_map = {
            "display_name": "display_name", "name": "display_name",
            "skills_dir": "skills_dir",
            "detect_dir": "detect_dir",
            "project_skills_dir": "project_skills_dir",
            "supports_symlink": "supports_symlink", "symlink": "supports_symlink",
            "supports_junction": "supports_junction", "junction": "supports_junction",
            "force_copy": "force_copy", "copy": "force_copy",
            "supports_project_scope": "supports_project_scope", "project_scope": "supports_project_scope",
        }
        field = field_map.get(field_lower)
        if field is None:
            continue
        tool_key = raw_key
        if tool_key not in result:
            # 支持环境变量添加新工具的最小配置
            result[tool_key] = {
                "display_name": tool_key,
                "skills_dir": "",
                "detect_dir": "",
                "project_skills_dir": None,
                "supports_symlink": True,
                "supports_junction": True,
                "force_copy": False,
                "supports_project_scope": True,
            }
        val = env_val.strip()
        if field in bool_fields:
            result[tool_key][field] = val.lower() in ("1", "true", "yes")
        elif field in str_fields:
            result[tool_key][field] = val if val else None
        else:
            result[tool_key][field] = val

    # 2) JSON 批量覆盖: SKILLS_HUB_TOOL_OVERRIDES='{"key":{...},...}'
    json_env = os.environ.get("SKILLS_HUB_TOOL_OVERRIDES", "").strip()
    if json_env:
        try:
            overrides = json.loads(json_env)
            if isinstance(overrides, dict):
                for key, fields in overrides.items():
                    key = key.lower()
                    if not isinstance(fields, dict):
                        continue
                    if key not in result:
                        result[key] = {
                            "display_name": key,
                            "skills_dir": "",
                            "detect_dir": "",
                            "project_skills_dir": None,
                            "supports_symlink": True,
                            "supports_junction": True,
                            "force_copy": False,
                            "supports_project_scope": True,
                        }
                    for fk, fv in fields.items():
                        if fk in result[key]:
                            result[key][fk] = fv
        except (json.JSONDecodeError, TypeError):
            pass

    return result


# 经过环境变量覆盖后的默认工具配置（模块加载时计算一次）
DEFAULT_TOOL_ADAPTERS: dict[str, dict[str, Any]] = _apply_env_overrides(_DEFAULT_TOOL_ADAPTERS_RAW)


def get_default_tool_config(tool_key: str) -> dict[str, Any] | None:
    """获取指定工具的默认配置（已应用环境变量覆盖）。返回 None 表示非内置工具。"""
    return DEFAULT_TOOL_ADAPTERS.get(tool_key.lower())


def get_builtin_tool_keys() -> list[str]:
    """返回所有内置工具 key 列表，保持定义顺序。"""
    return list(_DEFAULT_TOOL_ADAPTERS_RAW.keys())


def resolve_data_dir() -> str:
    """获取应用数据目录，自动检测便携版 vs 安装版"""
    exe_dir = _get_exe_dir()

    # 便携版：exe 同目录下存在 portable.flag 或 data/ 目录
    if exe_dir is not None:
        portable_flag = exe_dir / "portable.flag"
        portable_data = exe_dir / "data"
        if portable_flag.exists() or portable_data.is_dir():
            return str(portable_data)

    # 安装版：使用 %APPDATA%\SkillsHub
    if sys.platform == "win32":
        base = os.environ.get("APPDATA") or os.path.expanduser("~")
        return os.path.join(base, "skills-hub")
    elif sys.platform == "darwin":
        return os.path.expanduser("~/Library/Application Support/skills-hub")
    else:
        base = os.environ.get("XDG_DATA_HOME", os.path.expanduser("~/.local/share"))
        return os.path.join(base, "skills-hub")


def _get_exe_dir():
    """获取 exe 所在目录（仅在 PyInstaller 打包后有效）"""
    if getattr(sys, "frozen", False):
        return Path(sys.executable).parent
    return None


def default_db_path() -> str:
    """获取默认数据库路径"""
    data_dir = resolve_data_dir()
    os.makedirs(data_dir, exist_ok=True)
    return os.path.join(data_dir, DB_FILE_NAME)
