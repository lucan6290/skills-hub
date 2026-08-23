# 03. 路径安全、文件系统与同步引擎

## 1. 目标

将 Skills Hub 最核心的本地文件操作从 Python 迁移到 Rust，保持 Windows 下的路径安全、symlink/junction/copy fallback 和 global/project scope 行为。

这是高风险工作包，必须先迁移路径安全和纯文件操作，再迁移业务同步编排。

## 2. 当前代码证据

权威实现：

- `backend/core/utils/path_safety.py`
- `backend/core/utils/content_hash.py`
- `backend/core/skills/files.py`
- `backend/core/skills/sync_engine.py`
- `backend/core/skills/sync_service.py`
- `backend/core/skills/source_paths.py`
- `backend/tests/test_path_safety.py`
- `backend/tests/test_sync_engine.py`
- `backend/tests/test_install_service.py`

当前行为基线：

```text
优先 symlink
失败时尝试 junction
仍失败时 copy
```

实际优先级、失败条件和返回值必须以 `sync_engine.py` 当前实现为准，不能只依据上面摘要实现。

## 3. 允许修改范围

```text
frontend/src-tauri/src/platform/**
frontend/src-tauri/src/filesystem/**
frontend/src-tauri/src/skills/files.rs
frontend/src-tauri/src/skills/sync_engine.rs
frontend/src-tauri/src/utils/**
```

不得修改：

- Tauri command 注册文件；
- 数据库 repository；
- 前端组件；
- 工具适配器配置；
- 正式用户目录。

## 4. 实施步骤

### 4.1 迁移路径安全

逐项复核并实现当前 Python 工具的行为：

- `safe_dir_name`
- `is_path_within`
- `require_path_within`
- `safe_child_path`
- `norm_path`
- `expand_home`

必须覆盖：

- 相对路径；
- 绝对路径；
- `~` 和 `~/`；
- Windows 大小写；
- `..` 穿越；
- 符号链接和 junction 边界；
- 用户输入的 project path；
- Community Repo 边界。

路径检查失败必须返回明确错误，不允许静默修正到另一个目录。

### 4.2 统一文件系统接口

建立平台无关接口，至少覆盖：

```text
exists
is_file
is_dir
read_file
write_file
list_files
copy_directory
create_symlink
create_junction
remove_link_or_directory
open_folder
```

Windows 实现单独放在平台模块中；不要把 Windows API 调用散落到业务 service。

### 4.3 迁移技能文件操作

保持当前 API 可见行为：

- 列出相对路径和文件大小；
- 读取指定技能文件；
- 写入指定技能文件；
- 防止 `file_path` 越界；
- 文件不存在返回结构化错误；
- 文件大小限制和异常行为与当前项目安全规范一致。

### 4.4 迁移目录 hash

从 `content_hash.py` 迁移目录哈希算法时，必须保持：

- 遍历顺序；
- 路径编码；
- 文件内容读取方式；
- 忽略项；
- 空目录行为。

先用固定临时目录建立 Python/Rust 输出对照测试，再接入安装和同步流程。

### 4.5 迁移同步引擎

按顺序实现：

1. 单目录同步；
2. symlink；
3. junction；
4. copy fallback；
5. overwrite 和同内容判断；
6. target 状态记录接口；
7. unsync；
8. retry copy；
9. global scope；
10. project scope；
11. suite skill 子路径同步。

数据库记录和文件操作必须分离：

- 先完成安全检查；
- 执行文件操作；
- 文件操作成功后再写 target 记录；
- 失败时保留可诊断错误，不产生伪成功记录。

### 4.6 保护非托管文件

unsync 和覆盖操作必须确认目标是否由 Skills Hub 创建或记录：

- 不删除普通用户目录；
- 不跟随未知符号链接删除目标内容；
- 不通过字符串拼接执行危险路径操作；
- 所有递归删除都要经过最终绝对路径校验。

## 5. 测试要求

使用临时目录，禁止修改真实工具目录。

必须测试：

- 正常 symlink；
- Windows junction；
- symlink 权限失败；
- junction 失败；
- copy fallback；
- 已存在目标；
- 目标为普通文件；
- 源目录不存在；
- source/target 路径越界；
- 中文、空格和长路径；
- global/project scope；
- unsync 不误删非托管内容；
- 复制中断后的状态；
- hash 与 Python 基线一致。

## 6. 验收标准

- Rust 测试覆盖当前 Python 同步引擎的关键行为。
- Windows 构建下真实创建并清理临时 symlink/junction 成功，若环境权限不足必须记录为环境限制而非伪造通过。
- 所有用户输入路径经过统一安全检查。
- 不出现跨层直接调用 HTTP 或 FastAPI 的代码。
