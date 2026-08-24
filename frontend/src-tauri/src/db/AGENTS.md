# Database 层 Agent 入口

本文件是 `db/` 的导航入口。Database 层封装 SQLite 连接管理、schema 自愈与迁移。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

提供线程安全的 SQLite 访问（`Mutex<Connection>`），统一 schema 管理（自愈 + 迁移 + 初始化），是所有 Repository 的底层依赖。

## 文件清单

| 文件 | 职责 |
|------|------|
| `connection.rs` | `Database` 结构体：`new`（文件数据库）、`new_in_memory`（内存数据库，用于测试）、`with_conn` / `with_conn_mut`（闭包访问 Connection）、`ensure_schema`（启动时调用）、`initialize_tool_adapter_configs`（初始化默认工具适配器）、`now_ms`（时间戳工具函数） |
| `schema.rs` | `ensure_schema`：schema 自愈入口。包含 `reset_incompatible_schema`（不兼容时重置）、`migrate_skill_targets_to_v4_if_old_shape`（迁移）、`self_heal_schema`（`CREATE TABLE IF NOT EXISTS`）、`initialize_sort_order_columns` / `initialize_sort_order_data`（排序列初始化） |

## 硬规则

1. **所有数据库连接通过** `Database` 结构体，禁止直接使用 `rusqlite::Connection`
2. **SQL 操作通过** `db.with_conn(|conn| { ... })` 或 `db.with_conn_mut()`，不直接暴露 `Connection`
3. **schema 变更必须通过** `schema.rs` 的 `ensure_schema` 流程（自愈 + 迁移），禁止手写 `ALTER TABLE` 散落在业务代码中
4. **建表使用** `CREATE TABLE IF NOT EXISTS`，不写 `DROP TABLE`（除非 `reset_incompatible_schema` 检测到不兼容）
5. **PRAGMA `foreign_keys = ON`** 在连接时设置
6. **测试使用** `Database::new_in_memory()`，不依赖文件系统
7. `with_conn` 和 `with_conn_mut` 当前实现一致（都通过 `Mutex` 锁），语义上区分只读与读写

## Schema 管理流程

```
ensure_schema(conn)
  ├── reset_incompatible_schema(conn)         # 检测不兼容 schema → DROP + 重建
  ├── migrate_skill_targets_to_v4_if_old_shape  # 旧版 skill_targets 迁移
  ├── self_heal_schema(conn)                   # CREATE TABLE IF NOT EXISTS（所有表）
  ├── initialize_sort_order_columns(conn)      # 排序列存在性检查
  └── initialize_sort_order_data(conn)         # 排序数据初始化
```

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 新增数据库表 | 本文件 + `schema.rs`（`self_heal_schema` 中加 `CREATE TABLE IF NOT EXISTS`）+ [../../../../docs/database-schema.md](../../../../docs/database-schema.md) |
| 修改表结构（加列） | 本文件 + `schema.rs`（`self_heal_schema` 中加 `ALTER TABLE ... ADD COLUMN`）+ 对应 `models/` + 对应 `repositories/` |
| 不兼容 schema 重置 | 本文件 + `schema.rs`（`has_development_incompatible_schema` + `reset_incompatible_schema`） |
