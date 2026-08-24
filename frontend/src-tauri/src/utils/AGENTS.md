# Utils 工具函数 Agent 入口

本文件是 `utils/` 的导航入口。Utils 模块提供跨模块共享的工具函数和常量。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

提供：目录内容哈希（SHA256，用于检测技能变更）、路径安全（目录名清理、路径穿越防护、`~` 展开）。这些函数被 `services/`、`repo/`、`skills/`、`filesystem/` 等多个模块依赖。

## 文件清单

| 文件 | 职责 |
|------|------|
| `content_hash.rs` | `hash_dir`：递归扫描目录，对排序后的文件路径+内容计算 SHA256。跳过 symlink 和 `IGNORE_NAMES`。输出确定性哈希。 |
| `path_safety.rs` | `safe_dir_name` / `safe_dir_name_with_fallback`：从显示名生成安全目录名（过滤非法字符、Windows 保留名、截断）。`norm_path`：路径规范化。`is_path_within` / `require_path_within` / `safe_child_path`：路径穿越防护。`expand_home`：`~` 展开。 |

## 共享常量

| 常量 | 值 | 说明 |
|------|-----|------|
| `IGNORE_NAMES` | `[".git", ".DS_Store", "Thumbs.db", ".gitignore"]` | 复制/哈希/扫描时跳过的文件名 |
| `MAX_FILE_SIZE` | `1 * 1024 * 1024`（1 MB） | 技能文件读写的大小上限 |

## 硬规则

1. **目录名清理** `safe_dir_name`：替换 `<>:"/\|?*` 和控制字符为 `-`，处理 Windows 保留名（`CON`、`PRN`、`AUX` 等），截断到 120 字符
2. **路径穿越防护** `require_path_within` / `safe_child_path`：所有用户输入的路径必须校验不逃逸出 base 目录
3. **`~` 展开** `expand_home`：支持 `~`、`~/path`、`~\path`，跨平台获取 home 目录
4. **哈希确定性** `hash_dir`：路径排序（POSIX 分隔符），跳过 symlink 和 `IGNORE_NAMES`，相同内容产生相同哈希
5. **错误返回** `Result<T, String>`（非 `AppError`），由调用方转换
6. **Windows 大小写** `norm_path` / `is_path_within`：Windows 上路径小写化以支持大小写不敏感比较

## 依赖方

| 函数 | 依赖方 |
|------|--------|
| `hash_dir` | `services/install.rs`、`repo/scanner.rs` |
| `safe_dir_name` | `services/install.rs` |
| `require_path_within` / `safe_child_path` | `commands/files.rs`、`services/install.rs` |
| `expand_home` | `repo/community.rs`、`tools/adapter.rs` |
| `IGNORE_NAMES` | `filesystem/mod.rs`、`skills/files.rs`、`utils/content_hash.rs` |
| `MAX_FILE_SIZE` | `skills/files.rs` |

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 修改哈希算法 | 本文件 + `content_hash.rs` |
| 修改路径安全 | 本文件 + `path_safety.rs` |
| 修改忽略列表 | 本文件 + `mod.rs`（`IGNORE_NAMES`） |
| 修改文件大小限制 | 本文件 + `mod.rs`（`MAX_FILE_SIZE`） |
