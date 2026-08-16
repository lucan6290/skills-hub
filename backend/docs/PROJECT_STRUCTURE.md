# 后端系统与目录地图

> **定位**：后端代码的结构总览与模块导航，帮助开发者快速定位代码位置和理解请求链路。
> **关系**：本文件是 `backend/docs/` 的入口级文档；API 细节见 [API_STANDARD.md](./API_STANDARD.md)，数据库细节见 [DATABASE_STANDARD.md](./DATABASE_STANDARD.md)，测试与安全见 [TESTING_STANDARD.md](./TESTING_STANDARD.md)。完整表结构见 [`docs/database-schema.md`](../../docs/database-schema.md)。

---

## 1. 系统职责概述

Skills Hub 后端是一个基于 FastAPI + SQLite 的本地服务，负责管理 AI Agent Skills 的生命周期：从社区仓库扫描、安装入库、同步到 44 款 AI 编程工具，到标签分类、使用统计和健康检查。所有数据存储在单个 SQLite 文件中，采用 Schema 自愈模式，无需迁移脚本。

---

## 2. 请求链路图

```
HTTP Request (localhost:18921)
  │
  ▼
main.py ─── 全局异常兜底 + CORS（仅开发模式）+ 日志初始化
  │
  ▼
api/          路由层
  │  · 参数校验（Pydantic BaseModel）
  │  · 依赖注入（Depends(get_skill_store)）
  │  · 异常 → HTTPException 转换
  │  · DTO 组装与返回
  │
  ▼
core/         业务逻辑层（纯 Python，不依赖 FastAPI）
  │  · core/skills/   安装、同步、文件操作
  │  · core/repo/     仓库扫描、社区路径管理
  │  · core/tools/    工具适配器、技能缓存
  │  · core/tasks/    后台任务管理器
  │  · core/utils/    路径安全、内容哈希、常量
  │
  ▼
core/db/      数据访问层
  │  · Raw SQL + sqlite3
  │  · threading.local 线程安全连接
  │  · Dataclass DTO（9 种记录类型）
  │  · Schema 自愈（_self_heal_schema）
  │
  ▼
models/       Pydantic DTO 定义（API 契约）
```

---

## 3. 顶层目录表

| 路径 | 职责 | 修改前读取 |
|------|------|-----------|
| `api/` | FastAPI 路由处理器、请求参数解析、异常转 HTTPException | [API_STANDARD.md](./API_STANDARD.md) |
| `api/skills/` | 技能 CRUD、文件操作、同步相关端点 | [API_STANDARD.md](./API_STANDARD.md) |
| `api/tools/` | 工具状态、工具技能列表、适配器配置端点 | [API_STANDARD.md](./API_STANDARD.md) |
| `core/` | 纯业务逻辑，可独立于 FastAPI 测试 | 本文件 §5 |
| `core/db/` | SQLite 数据访问层（store.py） | [DATABASE_STANDARD.md](./DATABASE_STANDARD.md) |
| `core/repo/` | 社区仓库路径管理、仓库扫描器 | 本文件 §5 |
| `core/skills/` | 技能安装、同步引擎、文件操作、维护 | 本文件 §5 |
| `core/tasks/` | 后台任务管理器 | 本文件 §5 |
| `core/tools/` | 44 AI 工具适配器、技能缓存 | 本文件 §5 |
| `core/utils/` | 路径安全、内容哈希、共享常量 | [TESTING_STANDARD.md](./TESTING_STANDARD.md) |
| `models/` | Pydantic BaseModel 定义（API 请求/响应契约） | [API_STANDARD.md](./API_STANDARD.md) |
| `tests/` | pytest 测试文件 | [TESTING_STANDARD.md](./TESTING_STANDARD.md) |

---

## 4. api/ 模块详解

### 顶层路由文件

| 文件 | 端点 | 说明 |
|------|------|------|
| `health.py` | `GET /api/health` | 健康检查 |
| `database.py` | `GET /api/db/overview` | 数据库概览 |
| | `GET /api/db/table/{table_name}` | 表数据查询 |
| | `GET /api/db/table/{table_name}/columns` | 表列信息 |
| | `POST /api/db/maintenance` | 数据库维护 |
| | `GET /api/db/export` | 数据库导出 |
| | `POST /api/db/reset` | 数据库重置 |
| `maintenance.py` | `GET /api/sync_health` | 同步健康检查 |
| | `POST /api/sync_health/repair` | 同步健康修复 |
| `onboarding.py` | `GET /api/get_onboarding_plan` | 获取引导计划 |
| `reorder.py` | `POST /api/reorder` | 排序更新 |
| `settings.py` | `GET /api/pick_folder` | 选择文件夹 |
| | `GET /api/get_default_sync_tools` | 获取默认同步工具 |
| | `POST /api/save_default_sync_tools` | 保存默认同步工具 |
| | `GET /api/get_community_repo_path` | 获取社区仓库路径 |
| | `POST /api/set_community_repo_path` | 设置社区仓库路径 |
| | `GET /api/get_custom_repo_path` | 获取自制仓库路径 |
| | `POST /api/set_custom_repo_path` | 设置自制仓库路径 |
| | `POST /api/scan_community_repo` | 扫描社区仓库 |
| | `POST /api/scan_all_repos` | 扫描所有仓库 |
| `tags.py` | `GET /api/get_tags` | 获取标签列表 |
| | `POST /api/create_tag` | 创建标签 |
| | `POST /api/rename_tag` | 重命名标签 |
| | `POST /api/delete_tag` | 删除标签 |
| | `GET /api/get_skill_tags` | 获取技能标签 |
| | `POST /api/set_skill_tags` | 设置技能标签 |
| | `GET /api/get_untagged_skill_ids` | 获取未标签技能 ID |
| `tasks.py` | `GET /api/tasks` | 获取任务列表 |
| | `GET /api/tasks/{task_id}` | 获取任务详情 |
| | `POST /api/tasks/{task_id}/cancel` | 取消任务 |
| | `POST /api/tasks/get_tool_skills` | 异步获取工具技能 |
| | `POST /api/tasks/set_community_repo_path` | 异步设置社区仓库路径 |

### api/skills/ 子包

| 文件 | 端点 | 说明 |
|------|------|------|
| `crud.py` | `GET /api/get_managed_skills` | 获取已管理技能列表 |
| | `POST /api/delete_managed_skill` | 删除已管理技能 |
| | `POST /api/update_skill_source_url` | 更新技能来源 URL |
| | `POST /api/import_existing_skill` | 导入已有技能 |
| | `POST /api/list_local_skills_cmd` | 列出本地技能命令 |
| | `POST /api/install_local` | 安装本地技能 |
| | `POST /api/install_local_selection` | 批量安装本地技能 |
| | `POST /api/retry_copy_target` | 重试复制目标 |
| `files.py` | `GET /api/list_skill_files` | 列出技能文件 |
| | `GET /api/read_skill_file` | 读取技能文件内容 |
| | `POST /api/write_skill_file` | 写入技能文件 |
| `sync.py` | `POST /api/sync_skill_dir` | 同步技能目录 |
| | `POST /api/sync_skill_to_tool` | 同步技能到工具 |
| | `POST /api/unsync_skill_from_tool` | 取消同步 |
| | `POST /api/save_recent_project` | 保存最近项目 |
| | `GET /api/get_recent_projects` | 获取最近项目 |
| | `GET /api/get_scope_preferences` | 获取作用域偏好 |
| | `POST /api/set_scope_preference` | 设置作用域偏好 |

### api/tools/ 子包

| 文件 | 端点 | 说明 |
|------|------|------|
| `status.py` | `GET /api/get_tool_status` | 获取工具安装状态 |
| `tool_skills.py` | `GET /api/get_tool_skills` | 获取工具技能列表 |
| | `GET /api/get_tool_adapter_configs` | 获取适配器配置 |
| | `POST /api/save_tool_adapter_config` | 保存适配器配置 |
| | `POST /api/reset_tool_adapter_config` | 重置适配器配置 |
| | `GET /api/get_tool_skills/{tool_key}` | 获取指定工具技能 |
| | `POST /api/delete_tool_skill` | 删除工具技能 |
| | `POST /api/open_tool_skills_dir` | 打开工具技能目录 |
| | `POST /api/skill_to_community_repo` | 技能保存到社区仓库 |
| | `POST /api/clear_tool_skills` | 清除工具技能 |

---

## 5. core/ 模块详解

### core/db/ — 数据访问

| 文件 | 职责 |
|------|------|
| `store.py` | SQLite ORM 层：Schema 自愈、9 种 Dataclass DTO、CRUD 方法、全局单例 `get_store()` |

### core/repo/ — 仓库管理

| 文件 | 职责 |
|------|------|
| `community.py` | 社区仓库路径解析与管理（`resolve_community_repo_path`, `resolve_custom_repo_path`） |
| `community_migration.py` | 旧版仓库路径迁移逻辑 |
| `scanner.py` | 仓库扫描器：遍历社区/自制仓库，发现并注册新 Skill |

### core/skills/ — 技能操作

| 文件 | 职责 |
|------|------|
| `installer.py` | SKILL.md frontmatter 解析、Skill 目录识别 |
| `install_service.py` | 安装编排：统一构造 `SkillRecord`、去重、入库 |
| `sync_engine.py` | 底层同步引擎：符号链接/Junction/复制的实际文件系统操作 |
| `sync_service.py` | 同步编排：完整的同步业务流程（路径解析→适配器选择→引擎调用→记录更新） |
| `files.py` | 技能文件的读写操作 |
| `source_paths.py` | 来源路径解析与类型推断 |
| `maintenance.py` | 技能健康检查与修复 |
| `onboarding.py` | 已有技能扫描与导入引导 |

### core/tasks/ — 后台任务

| 文件 | 职责 |
|------|------|
| `manager.py` | 后台任务管理器：异步执行长时间运行的操作（仓库扫描、工具技能获取等） |

### core/tools/ — 工具适配

| 文件 | 职责 |
|------|------|
| `adapters.py` | 44 AI 工具的适配器：路径解析、安装检测、作用域支持判断 |
| `skill_cache.py` | 工具技能缓存：避免重复文件系统扫描 |

### core/utils/ — 工具函数

| 文件 | 职责 |
|------|------|
| `constants.py` | 共享常量定义 |
| `content_hash.py` | SHA256 目录内容哈希（用于变更检测） |
| `path_safety.py` | 路径安全工具：防穿越、安全目录名、路径规范化 |

### core/ 顶层文件

| 文件 | 职责 |
|------|------|
| `config.py` | 全局配置：端口、数据目录、44 工具默认适配器配置、环境变量覆盖 |
| `error_codes.py` | `ErrorCode` 枚举：结构化错误码定义 |
| `logging_config.py` | 集中式日志配置：`setup_logging()` |

---

## 6. 数据模型概览

系统使用 12 张 SQLite 表，完整字段定义见 [`docs/database-schema.md`](../../docs/database-schema.md)。

| 表名 | 用途 |
|------|------|
| `skills` | Skill 主记录（名称、描述、来源、哈希、状态等） |
| `skill_targets` | Skill 同步目标（每个工具/作用域/项目的安装记录） |
| `skill_tags` | 用户创建的分类标签 |
| `skill_tag_links` | Skill ↔ Tag 多对多关联 |
| `settings` | 应用级键值配置 |
| `discovered_skills` | 已发现但未导入的 Skill（预留） |
| `tool_scan_state` | 工具目录扫描缓存元数据 |
| `tool_skill_cache` | 工具 Skill 条目缓存 |
| `tool_adapter_configs` | 44 工具的适配器配置 |
| `skill_scope_preference` | Skill 作用域偏好（global/project） |
| `recent_projects` | 最近使用项目 LRU 列表（最多 8 条） |
| `skill_usage` | Skill 使用统计（同步次数、查看次数） |

---

## 7. 关键业务流程

### 技能安装流程

```
用户触发安装
  │
  ▼
api/skills/crud.py ─── install_local / install_local_selection
  │
  ▼
core/skills/install_service.py ─── build_skill_record()
  │  · 解析 SKILL.md frontmatter
  │  · 计算 content_hash
  │  · 构造 SkillRecord
  │
  ▼
core/db/store.py ─── upsert_skill()
  │  · INSERT ... ON CONFLICT DO UPDATE
  │
  ▼
返回 InstallResultDto
```

### 同步流程

```
用户触发同步
  │
  ▼
api/skills/sync.py ─── sync_skill_to_tool
  │
  ▼
core/skills/sync_service.py ─── sync_skill_to_tool()
  │  · resolve_skill_source_path()  ← 定位源文件
  │  · adapter_by_key()            ← 获取工具适配器
  │  · resolve_default_path()      ← 计算目标路径
  │  · supports_project_scope()    ← 检查作用域支持
  │
  ▼
core/skills/sync_engine.py ─── sync_dir_for_tool_with_overwrite()
  │  · symlink / junction / copy   ← 执行文件系统操作
  │
  ▼
core/db/store.py ─── upsert_skill_target()
  │  · 记录同步结果
  │
  ▼
返回 SyncResultDto
```

### 仓库扫描流程

```
定时/手动触发扫描
  │
  ▼
api/settings.py ─── scan_community_repo / scan_all_repos
  │
  ▼
core/repo/scanner.py ─── scan_and_register()
  │  · resolve_community_repo_path()  ← 确定扫描根目录
  │  · 遍历子目录
  │  · is_skill_dir()                ← 识别 Skill 目录
  │  · parse_skill_md()              ← 解析元信息
  │  · hash_dir()                    ← 计算内容哈希
  │
  ▼
core/skills/install_service.py ─── build_skill_record()
  │
  ▼
core/db/store.py ─── upsert_skill()
  │  · 新增或更新 Skill 记录
  │
  ▼
返回扫描结果（新增数、更新数）
```

---

## 8. 文档导航

| 文件 | 内容 | 适用场景 |
|------|------|---------|
| [AGENTS.md](./AGENTS.md) | 文档目录编辑规则 | 修改 backend/docs/ 下任何文件前 |
| [API_STANDARD.md](./API_STANDARD.md) | API 设计规范 | 新增/修改端点、请求响应模型 |
| [DATABASE_STANDARD.md](./DATABASE_STANDARD.md) | 数据库规范 | 新增表/列、编写 SQL、Dataclass DTO |
| [TESTING_STANDARD.md](./TESTING_STANDARD.md) | 测试与安全规范 | 编写测试、安全检查、提交前验证 |
| [`docs/database-schema.md`](../../docs/database-schema.md) | 12 张表的完整字段定义 | 查阅表结构 |

