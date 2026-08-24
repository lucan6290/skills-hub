# Tasks 层 Agent 入口

本文件是 `tasks/` 的导航入口。Tasks 层提供进程内后台任务系统，用于长时间运行的操作（如批量同步、安装）。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

`TaskManager` 管理后台线程的提交、进度报告、取消和生命周期。任务在独立线程执行，通过 `TaskContext` 报告进度和日志。前端通过 `commands/tasks.rs` 的 command 查询和取消任务。

## 文件清单

| 文件 | 职责 |
|------|------|
| `mod.rs` | 全部实现：`TaskManager`、`TaskRecord`、`TaskContext`、`TaskFn`、`TaskStatus`、`TaskCancelled` |

## 核心类型

| 类型 | 说明 |
|------|------|
| `TaskManager` | 任务管理器，持有 `Arc<TaskManagerInner>`（`Mutex<HashMap<String, TaskRecord>>`） |
| `TaskRecord` | 任务记录：id、kind、status、progress、message、result、error、logs、cancel_requested、时间戳 |
| `TaskStatus` | `Pending` → `Running` → `Succeeded` / `Failed` / `Canceled`（`#[serde(rename_all = "lowercase")]`） |
| `TaskContext` | 传给任务函数的上下文：`set_progress`、`log`、`is_cancelled` / `check_cancelled` |
| `TaskFn` | `Box<dyn FnOnce(&TaskContext) -> Result<serde_json::Value, String> + Send>` |

## 硬规则

1. **任务提交通过** `TaskManager::submit(kind, task_fn)`，返回 `TaskRecord`
2. **任务在独立线程执行**（`std::thread::Builder::new().spawn()`），不阻塞调用方
3. **取消是协作式**：调用 `cancel(task_id)` 设置 `cancel_requested` 标志，任务函数需通过 `ctx.is_cancelled()` 或 `ctx.check_cancelled()` 主动检查
4. **日志上限 200 条**：超出时自动截断旧日志
5. **进度范围 0-100**：`set_progress` 自动 `clamp(0, 100)`
6. **任务函数签名**：`Box<dyn FnOnce(&TaskContext) -> Result<serde_json::Value, String> + Send>`
7. **前端交互** 通过 `commands/tasks.rs`（`get_task_list`、`get_task`、`cancel_task`）
8. **AppState 注入**：`TaskManager` 通过 `state.rs` 的 `AppState` 管理，command 通过 `state.task_manager` 访问

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 新增后台任务类型 | 本文件 + `mod.rs`（了解 TaskContext API）+ 对应 `commands/` 文件 |
| 修改任务生命周期 | 本文件 + `mod.rs` |
| 修改前端任务交互 | 本文件 + [../../commands/tasks.rs](../commands/tasks.rs) |
