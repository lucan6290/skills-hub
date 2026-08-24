# Repository 层 Agent 入口

本文件是 `repositories/` 的导航入口。Repository 层封装所有 SQL 操作，每个数据库表对应一个 Repository。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

Repository 是唯一允许写 SQL 的层。command 层禁止直接写 SQL（简单级联删除除外）。每个 Repository 通过 `&Database` 借用连接，所有 SQL 通过 `db.with_conn()` / `db.with_conn_mut()` 执行。

## 文件清单

| 文件 | 对应数据表 | Repository | 主要方法 |
|------|-----------|------------|---------|
| `skills.rs` | `skills` | `SkillsRepository` | `list`, `get`, `upsert`, `delete`, `update_sort_order` |
| `tags.rs` | `skill_tags` | `TagsRepository` | `list`, `create`, `rename`, `delete`, `get_skill_tags`, `set_skill_tags` |
| `settings.rs` | `settings` | `SettingsRepository` | `get`, `set`, `get_all`, `delete` |
| `skill_targets.rs` | `skill_targets` | `SkillTargetsRepository` | `list_by_skill`, `upsert`, `delete`, `delete_by_skill` |
| `skill_usage.rs` | `skill_usage` | `SkillUsageRepository` | `get_by_skill`, `upsert`, `delete` |
| `scope_preferences.rs` | `skill_scope_preference` | `ScopePreferencesRepository` | `get`, `set`, `list_by_skill` |
| `recent_projects.rs` | `recent_projects` | `RecentProjectsRepository` | `list`, `save`, `delete` |
| `tool_adapter_configs.rs` | `tool_adapter_configs` | `ToolAdapterConfigsRepository` | `list_enabled`, `upsert`, `reset`, `get_by_key` |
| `tool_cache.rs` | `tool_skill_cache`, `tool_scan_state` | `ToolCacheRepository` | `get_tool_skills`, `upsert_entries`, `update_scan_state` |
| `maintenance.rs` | 多表（聚合查询） | `MaintenanceRepository` | `db_overview`, `table_data`, `maintenance`, `reset`, `export` |

## 硬规则

1. **每个表一个 Repository**，结构体名 `XxxRepository`，泛型生命周期 `<'a>` 持有 `&'a Database`
2. **初始化模式**：`XxxRepository::new(&state.db)`
3. **所有 SQL** 通过 `db.with_conn(|conn| { ... })` 或 `db.with_conn_mut()` 执行，不直接持有 `Connection`
4. **rusqlite 错误转换**：`.map_err(|e| AppError::DatabaseError(e.to_string()))?`
5. **新增 Repository 必须在** `mod.rs` 中声明模块并 `pub use` 导出
6. **返回类型** 统一 `AppResult<T>`
7. **禁止在 Repository 中写业务逻辑**——跨表编排放 `services/` 层

## 模板

```rust
use crate::db::Database;
use crate::error::AppResult;
use crate::models::XxxModel;

pub struct XxxRepository<'a> {
    db: &'a Database,
}

impl<'a> XxxRepository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn list(&self) -> AppResult<Vec<XxxModel>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT ... FROM xxx")?;
            let rows = stmt.query_map([], |row| { ... })?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
        }).map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}
```

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 新增 Repository | 本文件 + 对应 `models/` 文件 + `mod.rs`（导出）+ [../../db/schema.rs](../db/schema.rs)（建表） |
| 修改 SQL 查询 | 本文件 + 对应 Repository 文件 + [../../db/schema.rs](../db/schema.rs)（确认表结构） |
| 数据库表结构详情 | [../../../../docs/database-schema.md](../../../../docs/database-schema.md) |
