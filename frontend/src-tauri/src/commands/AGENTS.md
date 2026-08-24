# Commands 层 Agent 入口

本文件是 `commands/` 的导航入口。command 层是前后端的通信边界——前端 `invokeCommand` 直接调用这里的 `#[tauri::command]` 函数。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

command 只做**参数解析和编排**：接收前端参数 → 调用 Repository / Service → 返回 DTO。禁止内联 SQL 逻辑（简单级联删除除外）。

## 文件清单

| 文件 | 功能领域 | 主要 command |
|------|---------|-------------|
| `health.rs` | 健康检查 | `health_check` |
| `skills.rs` | 技能管理 | `get_managed_skills`, `delete_managed_skill`, `update_skill_source_url`, `import_existing_skill`, `list_local_skills_cmd`, `install_local_selection` |
| `tags.rs` | 标签管理 | `get_tags`, `create_tag`, `rename_tag`, `delete_tag`, `get_skill_tags`, `set_skill_tags` |
| `sync.rs` | 同步到工具 | `sync_skill_to_tool`, `unsync_skill_from_tool`, `sync_suite_to_tool`, `unsync_suite_from_tool`, scope preferences, recent projects, `list_suite_sub_skills` |
| `files.rs` | 技能文件 | `list_skill_files`, `read_skill_file`, `write_skill_file` |
| `tools.rs` | 工具状态 | `get_tool_status`, `get_tool_skills`, adapter configs CRUD, `open_tool_skills_dir`, `skill_to_community_repo`, `clear_tool_skills` |
| `settings.rs` | 设置 | default sync tools, auto check update, community/custom repo path, `open_settings_folder`, `reset_general_settings` |
| `database.rs` | 数据库管理 | `db_overview`, `db_table_data`, `db_maintenance`, `db_reset`, `db_export`, `db_open_folder` |
| `onboarding.rs` | 引导 | `get_onboarding_plan` |
| `tasks.rs` | 后台任务 | `get_task_list`, `get_task`, `cancel_task` |
| `update.rs` | 自动更新 | `check_update`, `do_update` |
| `misc.rs` | 杂项 | `pick_folder`, `cancel_current_operation`, `reorder`, `open_new_window`, `create_new_window` |

## 硬规则

1. **必须标注** `#[tauri::command(rename_all = "snake_case")]`，确保前端参数名匹配
2. **必须为** `async fn`，通过 `State<'_, AppState>` 获取数据库和任务管理器
3. **返回类型统一为** `AppResult<T>`（即 `Result<T, AppError>`）
4. **新增 command 必须在** `lib.rs` 的 `invoke_handler`（`generate_handler!`）中注册，否则前端无法调用
5. **禁止内联 SQL**——数据库操作通过 `repositories/` 层
6. **DTO 定义** 放在 `contracts.rs`，不在 command 文件中定义返回结构体
7. **跨多个 Repository 的业务逻辑** 放到 `services/` 层，command 只做编排

## 模板

```rust
#[tauri::command(rename_all = "snake_case")]
pub async fn command_name(
    state: State<'_, AppState>,
    param: String,
    optional_param: Option<bool>,
) -> AppResult<ReturnType> {
    let repo = XxxRepository::new(&state.db);
    repo.some_method(&param)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(result)
}
```

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 新增 command | 本文件 + `lib.rs`（注册）+ `contracts.rs`（DTO）+ 对应 `repositories/` |
| 修改已有 command | 本文件 + 对应 command 文件 + `contracts.rs` |
| 前后端接口联调 | [../../../../docs/database-schema.md](../../../../docs/database-schema.md) + 前端 `docs/API_STANDARD.md` |
