# Filesystem 文件系统层 Agent 入口

本文件是 `filesystem/` 的导航入口。Filesystem 模块提供平台无关的文件系统操作，将 OS 特定调用委托给 `platform` 模块或 `std::fs`。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

提供文件系统操作的统一入口：存在性检查、读写文件、目录复制、链接创建与移除、打开文件夹。业务逻辑通过本模块访问文件系统，不直接调用 `std::fs`。

## 文件清单

| 文件 | 职责 |
|------|------|
| `mod.rs` | 全部实现：`exists`、`is_file`、`is_dir`、`read_file`、`write_file`、`list_files`、`copy_directory`、`create_symlink`、`create_junction`（委托 `platform`）、`remove_link_or_directory`、`open_folder` |

## 核心函数

| 函数 | 说明 |
|------|------|
| `copy_directory` | 递归复制目录，跳过 `.git` / `.DS_Store` 等（通过 `IGNORE_NAMES`）和 symlink |
| `remove_link_or_directory` | 智能移除：链接/junction 只移除 reparse point 不跟随，目录用 `remove_dir_all`，文件用 `remove_file` |
| `write_file` | 写文件时自动创建父目录 |
| `open_folder` | 跨平台打开文件夹（Windows: `explorer`，macOS: `open`，Linux: `xdg-open`） |

## 硬规则

1. **业务代码通过本模块访问文件系统**，不直接调用 `std::fs`（路径安全、忽略列表等统一管理）
2. **`copy_directory` 跳过** `IGNORE_NAMES`（`.git`、`.DS_Store`、`Thumbs.db`、`.gitignore`）和 symlink，避免无限递归
3. **`remove_link_or_directory`** 对链接/junction 只移除 reparse point 本身，**不跟随到目标**——防止误删源文件
4. **`create_junction`** 委托给 `platform` 模块，非 Windows 返回错误
5. **`create_symlink`** 通过 `std::os::windows::fs::symlink_dir` / `std::os::unix::fs::symlink` 条件编译
6. **错误返回** `Result<T, String>`（非 `AppError`），由调用方转换为 `AppError::FileSystemError`

## 调用方

- `skills/sync_engine.rs`：同步时使用 `create_symlink` / `create_junction` / `copy_directory`
- `skills/files.rs`：文件读写
- `services/install.rs`：技能安装时复制文件
- `commands/misc.rs`：`open_folder`

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 修改文件操作 | 本文件 + `mod.rs` + [../../utils/mod.rs](../utils/mod.rs)（`IGNORE_NAMES`） |
| 修改链接移除逻辑 | 本文件 + `mod.rs` + [../../platform/mod.rs](../platform/mod.rs)（`is_link_or_junction`） |
| 修改目录复制 | 本文件 + `mod.rs` |
