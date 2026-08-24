# Skills 领域模块 Agent 入口

本文件是 `skills/` 的导航入口。Skills 模块包含技能同步引擎和文件操作的实际实现，以及三个指向 `services/` 的 re-export shim。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

技能领域的核心运行时逻辑：将技能同步到 AI 工具的 skills 目录（symlink → junction → copy 三级回退）、技能文件读写、技能安装/维护/引导的 re-export。

## 文件清单

| 文件 | 类型 | 职责 |
|------|------|------|
| `sync_engine.rs` | **实际实现** | 同步引擎：`SyncMode`（Auto/Symlink/Junction/Copy）、`SyncOutcome`、`sync_dir_hybrid`（三级回退）、`sync_dir_copy_with_overwrite` |
| `files.rs` | **实际实现** | 技能文件操作：`FileEntry`、`list_files`（SKILL.md 排序在前）、`read_file`、`write_file`（含 `MAX_FILE_SIZE` 限制） |
| `install.rs` | **re-export shim** | `pub use crate::services::install::*` — 实际实现在 `services/install.rs` |
| `maintenance.rs` | **re-export shim** | `pub use crate::services::maintenance::*` — 实际实现在 `services/maintenance.rs` |
| `onboarding.rs` | **re-export shim** | `pub use crate::services::onboarding::*` — 实际实现在 `services/onboarding.rs` |

## 硬规则

1. **`install.rs`、`maintenance.rs`、`onboarding.rs` 是 re-export shim**——修改业务逻辑请去 `services/` 目录，不要在这里加实现
2. **`sync_engine.rs` 和 `files.rs` 是实际实现**——同步引擎和文件操作的逻辑在这里
3. **同步三级回退**：`symlink` → `junction`（Windows）→ `copy`，通过 `platform` 模块检测能力
4. **文件操作限制**：读写文件不超过 `MAX_FILE_SIZE`（1MB），通过 `utils::IGNORE_NAMES` 过滤 `.git` 等
5. **路径安全**：通过 `utils::path_safety` 进行路径校验，防止目录穿越

## 同步模式说明

```
sync_dir_hybrid(source, target)
  ├── 尝试 symlink      (Unix 原生 / Windows 需要权限)
  ├── 尝试 junction      (Windows 专用，无需管理员权限)
  └── 回退 copy          (全量复制，无链接)
```

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 修改同步逻辑 | 本文件 + `sync_engine.rs` + [../../platform/mod.rs](../platform/mod.rs) + [../../filesystem/mod.rs](../filesystem/mod.rs) |
| 修改技能文件读写 | 本文件 + `files.rs` + [../../utils/mod.rs](../utils/mod.rs) |
| 修改安装/维护/引导逻辑 | 本文件 + [../../services/AGENTS.md](../services/AGENTS.md) + 对应 `services/` 文件 |
