# Models 层 Agent 入口

本文件是 `models/` 的导航入口。Models 层定义与数据库表对应的 Rust struct。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

每个 model 文件对应一张数据库表，定义 Rust struct 用于 ORM 映射。model 是纯粹的数据结构，不包含业务逻辑。

## 文件清单

| 文件 | Struct | 对应表 |
|------|--------|-------|
| `skill.rs` | `Skill` | `skills` |
| `tag.rs` | `Tag`, `TagWithCount` | `skill_tags` |
| `skill_target.rs` | `SkillTarget` | `skill_targets` |
| `skill_usage.rs` | `SkillUsage` | `skill_usage` |
| `setting.rs` | `Setting` | `settings` |
| `scope_preference.rs` | `ScopePreference` | `skill_scope_preference` |
| `recent_project.rs` | `RecentProject` | `recent_projects` |
| `tool_adapter_config.rs` | `ToolAdapterConfig` | `tool_adapter_configs` |
| `tool_cache.rs` | `ToolSkillCache`, `ToolScanState` | `tool_skill_cache`, `tool_scan_state` |

## 硬规则

1. **derive**：`#[derive(Debug, Clone, Serialize)]`（需要反序列化的加 `Deserialize`，如 `Skill`）
2. **字段名使用 `snake_case`**（Rust 原生风格，与 DB 列名一致，与前端 DTO 字段一致）
3. **新增 model 必须在** `mod.rs` 中声明模块并 `pub use` 导出
4. **Optional 字段** 用 `Option<T>`，对应 DB 中的 `NULL`
5. **时间戳字段** 用 `i64`（毫秒级 Unix 时间戳），命名 `*_at`
6. **禁止在 model 中写业务逻辑**——model 是纯数据结构
7. **字段类型映射**：`TEXT` → `String` / `Option<String>`，`INTEGER` → `i64` / `Option<i64>`，`REAL` → `f64`

## 模板

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct XxxModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 新增 model | 本文件 + `mod.rs`（导出）+ 对应 `repositories/` 文件 + [../../db/schema.rs](../db/schema.rs)（建表） |
| 修改 model 字段 | 本文件 + 对应 model 文件 + 对应 `repositories/`（SQL 列）+ [../../../../docs/database-schema.md](../../../../docs/database-schema.md) |
