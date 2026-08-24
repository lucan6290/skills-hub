# Rust 后端 Agent 入口

本文件是 `src-tauri/` 的导航入口。只保留 Rust 后端每次任务都适用的硬约束；详细规则按任务类型从下方「任务路由」逐级读取。

> 全局规则见根目录 [../../AGENTS.md](../../AGENTS.md)，前端规范见 [../AGENTS.md](../AGENTS.md)。

## 1. 每次任务都必须遵守

1. 先确认任务范围和完成证据；默认一次只完成一个可验证的工作单元
2. 修改前检查 `git status`，保留开发者已有改动；禁止擅自覆盖、回滚、清理、提交或推送
3. 只改完成当前任务所需的文件。发现邻近问题时记录并报告，不顺手重构
4. 文档与实现冲突时，以当前代码和配置为运行事实，并报告冲突，不能静默猜测
5. 数据库 schema 变更必须通过 `db/schema.rs` 的 `ensure_schema` 流程（自愈 + 迁移），禁止手写 `ALTER TABLE` 散落在业务代码中
6. 新增 Tauri command 必须在 `lib.rs` 的 `invoke_handler` 中注册，否则前端无法调用

## 2. 架构总览

```
src-tauri/src/
├── main.rs              # 二进制入口（仅调用 lib::run()）
├── lib.rs               # 库入口：Tauri Builder、插件注册、invoke_handler、系统托盘、全局快捷键
├── state.rs             # AppState：Database + TaskManager（通过 .manage() 注入）
├── config.rs            # 配置常量：DB 文件名、默认工具适配器（44 款 AI 工具）
├── error.rs             # AppError 枚举（8 变体）+ 自定义 Serialize → {ok, code, message, detail}
├── contracts.rs         # Tauri command 返回的 DTO/响应结构体
│
├── commands/            # Tauri command 层（#[tauri::command]）
├── repositories/         # Repository 层（每表一个，封装 SQL）
├── services/             # Service 层（业务逻辑：install、maintenance、onboarding）
├── models/               # 数据模型（与 DB 表对应的 Rust struct）
├── db/                   # 数据库层（connection + schema）
├── tasks/                # 后台任务系统（TaskManager）
├── skills/               # 技能领域模块（sync_engine、files 为实际实现；install、maintenance、onboarding 为 re-export shim，指向 services/）
├── tools/                # 工具适配器（adapter、skill_cache）
├── repo/                 # 社区/自定义仓库扫描与注册
├── platform/             # 平台特定代码（windows）
├── filesystem/           # 文件系统操作
├── update/               # 自动更新
└── utils/                # 工具函数（content_hash、path_safety）
```

## 3. 分层架构

### 3.1 调用链

```
前端 invokeCommand('command_name', { param })
  → Tauri invoke_handler
    → commands/*.rs        # #[tauri::command(rename_all = "snake_case")] async fn
      → repositories/*.rs # XxxRepository::new(&state.db) → SQL 操作
        → db/connection.rs # Database::with_conn(|conn| { ... })
      → services/*.rs     # 业务逻辑（跨多个 repository）
    → 返回 contracts.rs 中的 DTO
  → AppResult<T>（Ok(data) 或 Err(AppError)）
```

### 3.2 Commands 层（`commands/`）

- 每个命令模块对应一个功能领域（skills、tags、sync、tools、settings、database、onboarding、tasks、update、misc、files、health）
- 所有 command 必须标注 `#[tauri::command(rename_all = "snake_case")]`，确保前端调用参数名匹配
- command 为 `async fn`，通过 `State<'_, AppState>` 获取数据库和任务管理器
- 返回类型统一为 `AppResult<T>`（即 `Result<T, AppError>`）
- command 只做参数解析和编排，不内联 SQL 逻辑

```rust
#[tauri::command(rename_all = "snake_case")]
pub async fn get_managed_skills(
    state: State<'_, AppState>,
    refresh: Option<bool>,
    source_type: Option<String>,
) -> AppResult<Vec<ManagedSkillDto>> { ... }
```

### 3.3 Repository 层（`repositories/`）

- 每个数据库表对应一个 Repository（`SkillsRepository`、`TagsRepository`、`SettingsRepository` 等）
- 初始化模式：`XxxRepository::new(&state.db)`，借用 Database 的生命周期
- 所有 SQL 通过 `db.with_conn(|conn| { ... })` 或 `db.with_conn_mut()` 执行
- `rusqlite::Error` 通过 `.map_err(|e| AppError::DatabaseError(e.to_string()))` 转换

```rust
pub struct SkillsRepository<'a> {
    db: &'a Database,
}

impl<'a> SkillsRepository<'a> {
    pub fn new(db: &'a Database) -> Self { Self { db } }
    pub fn list(&self, sort: &str) -> AppResult<Vec<Skill>> { ... }
}
```

### 3.4 Service 层（`services/`）

- 跨多个 Repository 的业务逻辑放在 Service 层
- 当前模块：`install`（技能安装）、`maintenance`（同步健康检查）、`onboarding`（引导计划生成）
- Service 函数接收 `&Database` 而非 `State`，便于测试时传入内存数据库

### 3.5 Models 层（`models/`）

- 与数据库表对应的 Rust struct，使用 `#[derive(Debug, Clone, Serialize)]`
- 字段名使用 `snake_case`（Rust 原生风格，与 DB 列名一致）
- 模型：`Skill`、`Tag`、`TagWithCount`、`SkillTarget`、`SkillUsage`、`Setting`、`ScopePreference`、`RecentProject`、`ToolAdapterConfig`、`ToolSkillCache`、`ToolScanState`

### 3.6 Contracts 层（`contracts.rs`）

- Tauri command 返回的 DTO/响应结构体，序列化为 JSON 传给前端
- 使用 `#[serde(flatten)]` 平铺 model 字段（如 `ManagedSkillDto` 平铺 `Skill` + 补充 tags/targets/usage）
- 通用响应：`OkResponse`、`OkPathResponse`、`OkRemovedResponse`、`PickFolderResult`

### 3.7 数据库层（`db/`）

- `Database` 封装 `Mutex<Connection>`，线程安全的 SQLite 访问
- `with_conn` / `with_conn_mut`：通过闭包获取 `&Connection`，统一错误转换
- `ensure_schema`：启动时自动执行 schema 自愈 + 迁移 + 工具适配器初始化
- `PRAGMA foreign_keys = ON` 在连接时设置
- 测试使用 `Database::new_in_memory()` 创建内存数据库

### 3.8 任务系统（`tasks/`）

- `TaskManager`：进程内后台任务管理，任务在独立线程执行
- `TaskContext`：提供 `set_progress`、`log`、`is_cancelled` / `check_cancelled` 方法
- `TaskFn`：`Box<dyn FnOnce(&TaskContext) -> Result<serde_json::Value, String> + Send>`
- 任务状态：Pending → Running → Succeeded / Failed / Canceled
- 前端通过 `get_task_list`、`get_task`、`cancel_task` 命令查询和取消

## 4. 错误处理

### AppError 枚举（`error.rs`）

| 变体 | code | 用途 |
|------|------|------|
| `Unexpected(String)` | `INTERNAL_ERROR` | 通用错误 |
| `NotFound(String)` | `NOT_FOUND` | 资源不存在 |
| `InvalidInput(String)` | `INVALID_INPUT` | 参数校验失败 |
| `PathError(String)` | `PATH_ERROR` | 路径错误 |
| `DatabaseError(String)` | `DATABASE_ERROR` | 数据库操作失败 |
| `FileSystemError(String)` | `FILESYSTEM_ERROR` | 文件系统操作失败 |
| `TaskError(String)` | `TASK_ERROR` | 后台任务错误 |
| `UpdateError(String)` | `UPDATE_ERROR` | 自动更新错误 |

序列化为 JSON：`{ "ok": false, "code": "...", "message": "...", "detail": null }`

前端通过 `parseErrorDetail` 解析 `code` 字段映射到 i18n key。

### 错误转换规则

```rust
// rusqlite 错误 → AppError::DatabaseError
.map_err(|e| AppError::DatabaseError(e.to_string()))?;

// 文件系统错误 → AppError::FileSystemError
.map_err(|e| AppError::FileSystemError(e))?;

// 业务校验 → AppError::InvalidInput
return Err(AppError::InvalidInput(format!("...")));

// 资源不存在 → AppError::NotFound
.ok_or_else(|| AppError::NotFound(format!("skill not found: {}", skill_id)))?;
```

## 5. 命名规范

| 层级 | 命名风格 | 示例 |
|------|---------|------|
| Tauri command 函数 | `snake_case` | `get_managed_skills`, `delete_managed_skill` |
| Command 参数 | `snake_case` | `skill_id`, `source_type`, `source_url` |
| Rust struct 字段 | `snake_case` | `community_path`, `sort_order`, `created_at` |
| Repository 结构体 | `PascalCase` | `SkillsRepository`, `TagsRepository` |
| Model 结构体 | `PascalCase` | `Skill`, `TagWithCount`, `ToolAdapterConfig` |
| DTO 结构体 | `PascalCase`（部分用 `Dto` 后缀） | `ManagedSkillDto`, `ToolStatusDto`, `DbOverview`, `OkResponse`, `ReorderItem` |
| 模块文件名 | `snake_case` | `skill_usage.rs`, `tool_adapter_config.rs` |
| 常量 | `SCREAMING_SNAKE_CASE` | `DB_FILE_NAME`, `LEGACY_APP_IDENTIFIERS` |

## 6. Tauri 插件

应用注册的 Tauri 插件（`lib.rs` → `run()`）：

| 插件 | 用途 |
|------|------|
| `tauri_plugin_single_instance` | 单实例 + Deep Link 转发 |
| `tauri_plugin_autostart` | 开机自启 |
| `tauri_plugin_deep_link` | `skillshub://` 协议处理 |
| `tauri_plugin_notification` | 系统通知 |
| `tauri_plugin_updater` | 自动更新 |
| `tauri_plugin_global_shortcut` | 全局快捷键 `Ctrl+Shift+Space` |

## 7. 常用命令

```bash
cd frontend/src-tauri && cargo test          # 运行所有测试
cd frontend/src-tauri && cargo test -- --nocapture  # 显示 println! 输出
cd frontend/src-tauri && cargo build         # 编译检查
cd frontend/src-tauri && cargo clippy        # Lint 检查
cd frontend && npm run tauri dev             # 启动开发模式（前后端联动）
```

## 8. 任务路由

| 任务类型 | 必读文件 |
|---------|---------|
| 新增/修改 Tauri command | `commands/*.rs` + `lib.rs`（注册） + `contracts.rs`（DTO） |
| 新增/修改数据库操作 | `repositories/*.rs` + `models/*.rs` |
| 数据库 schema 变更 | `db/schema.rs` + [../../docs/database-schema.md](../../docs/database-schema.md) |
| 新增后台任务 | `tasks/mod.rs` + 对应 command |
| 技能安装/同步逻辑 | `services/install.rs` + `skills/sync_engine.rs` |
| 工具适配器配置 | `config.rs` + `tools/adapter.rs` + `repositories/tool_adapter_configs.rs` |
| 前后端接口联调 | [../docs/API_STANDARD.md](../docs/API_STANDARD.md)（前端 API 规范） |

## 9. 工程规则

1. 不保留向后兼容。过时的实现直接删除；不增加兼容层、不写迁移代码（schema 除外，使用 `ensure_schema` 自愈机制）
2. 选择能满足当前需求的最简单实现。不要预防性抽象，不要多余的配置层
3. 新增 command 必须同步在 `lib.rs` 的 `invoke_handler` 中注册
4. 新增数据库表必须通过 `db/schema.rs` 的 `CREATE TABLE IF NOT EXISTS` 创建
5. 新增 model 必须在 `models/mod.rs` 中导出
6. 新增 repository 必须在 `repositories/mod.rs` 中导出
7. 所有数据库操作通过 Repository 层，禁止在 command 中直接写 SQL（简单级联删除除外）
8. 测试优先使用 `Database::new_in_memory()`，不依赖文件系统
9. 依赖安装、全量构建及提交均需开发者明确指令

## 10. 文档权威顺序

1. 用户明确指令
2. 根 `AGENTS.md` + `frontend/AGENTS.md` 硬约束
3. 本文件
4. 当前代码实际行为
5. [../../docs/database-schema.md](../../docs/database-schema.md)（数据库 schema 详情）
