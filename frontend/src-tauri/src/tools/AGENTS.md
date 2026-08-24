# Tools 工具适配器 Agent 入口

本文件是 `tools/` 的导航入口。Tools 模块管理 44 款 AI 编程工具的适配器配置、路径解析、安装检测和 skills 目录扫描。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

为每款 AI 工具提供：工具是否安装的检测、skills 目录路径解析（全局 + 项目级）、skills 目录扫描、工具适配器配置的 DB 优先加载。是同步引擎与具体工具之间的适配层。

## 文件清单

| 文件 | 职责 |
|------|------|
| `adapter.rs` | `ToolAdapter` 结构体（44 款工具的路径/能力配置）、`default_adapters`（从 `config.rs` 构建）、`effective_tool_adapters`（DB 优先加载）、`adapter_by_key`、`is_tool_installed`、`resolve_default_path` / `resolve_project_path`、`scan_tool_dir`、`supports_project_scope`、`DetectedSkill` |
| `skill_cache.rs` | 工具 skills 缓存：`ToolSkillsResponse`、`ToolSkillEntryDto`、`refresh_tool_cache`（扫描工具目录并写入 `tool_skill_cache` 表）、`cached_tool_response`（DB 缓存优先，miss 时扫描）、`build_skill_entries` |

## 核心概念

| 概念 | 说明 |
|------|------|
| `ToolAdapter` | 运行时适配器：tool_key、display_name、relative_skills_dir、relative_detect_dir、supports_symlink/junction、force_copy、supports_project_scope_override |
| 适配器来源优先级 | DB（`tool_adapter_configs` 表）→ `config.rs` 默认值 |
| `DetectedSkill` | 扫描工具 skills 目录后发现的技能：tool、name、path、is_link、link_target |

## 硬规则

1. **适配器配置 DB 优先**：`effective_tool_adapters(db)` 先查 DB，DB 为空时回退到 `config.rs` 默认值
2. **工具列表定义在** `config.rs` 的 `default_tool_adapters()` 中，不在本目录新增工具定义
3. **路径解析** 通过 `platform` 模块处理跨平台差异（home 目录、路径分隔符）
4. **skills 目录扫描** 通过 `scan_tool_dir` 进行，结果写入 `tool_skill_cache` 表（通过 `ToolCacheRepository`）
5. **缓存策略**：`cached_tool_response` 先查 DB 缓存，miss 时调用 `refresh_tool_cache` 扫描磁盘
6. **mtime 检测**：通过 `skill_mtime_ns` 比较技能目录和 SKILL.md 的修改时间，决定是否需要刷新缓存

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 新增/修改工具适配器 | 本文件 + [../../config.rs](../config.rs)（`default_tool_adapters`）+ [../../repositories/tool_adapter_configs.rs](../repositories/tool_adapter_configs.rs) |
| 修改 skills 缓存逻辑 | 本文件 + `skill_cache.rs` + [../../repositories/tool_cache.rs](../repositories/tool_cache.rs) |
| 修改路径解析 | 本文件 + `adapter.rs` + [../../platform/mod.rs](../platform/mod.rs) |
| 修改安装检测 | 本文件 + `adapter.rs` + [../../config.rs](../config.rs) |
