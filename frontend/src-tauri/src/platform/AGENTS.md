# Platform 平台层 Agent 入口

本文件是 `platform/` 的导航入口。Platform 模块封装平台特定代码（Windows junction/symlink），将 OS API 调用与业务逻辑隔离。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

提供跨平台的链接操作抽象：`create_junction`、`is_junction`、`is_link_or_junction`。在非 Windows 平台上安全降级（返回错误或 false）。

## 文件清单

| 文件 | 职责 |
|------|------|
| `mod.rs` | 平台抽象入口：`create_junction`、`is_junction`、`is_link_or_junction`，通过 `#[cfg(windows)]` 条件编译分发 |
| `windows.rs` | Windows 实现：调用 Windows API 创建 junction 和检测 reparse point（`#[cfg(windows)]` 限定） |

## 硬规则

1. **平台特定代码用** `#[cfg(windows)]` / `#[cfg(not(windows))]` 条件编译，不使用运行时判断
2. **非 Windows 平台** 的 `create_junction` 返回 `Err`，`is_junction` 返回 `false`
3. **`is_link_or_junction`** 是最常用的检测函数：`path.is_symlink() || is_junction(path)`
4. **业务代码不直接调用 Windows API**——通过本模块的抽象函数访问
5. **新增平台特定功能** 在 `mod.rs` 定义公开 API + 在对应平台文件中实现

## 调用方

- `skills/sync_engine.rs`：同步时使用 `create_junction` 创建 Windows junction
- `tools/adapter.rs`：检测工具 skills 目录时使用 `is_link_or_junction` 判断链接
- `filesystem/mod.rs`：`remove_link_or_directory` 使用 `is_link_or_junction` 区分链接和目录

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 修改 junction 创建 | 本文件 + `mod.rs` + `windows.rs` |
| 新增平台特定功能 | 本文件 + `mod.rs`（定义 API）+ 对应平台文件（实现） |
| 修改链接检测 | 本文件 + `mod.rs` |
