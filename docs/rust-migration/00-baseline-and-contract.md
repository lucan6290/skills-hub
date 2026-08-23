# 00. 迁移基线与契约清单

## 1. 目的

建立 Rust 重构的事实基线，避免 Agent 根据 README、旧分支或记忆猜测当前行为。该工作包只负责盘点、验证和生成契约，不负责迁移业务实现。

## 2. 当前事实来源

必须以以下文件为优先事实来源：

- API 路由：`backend/api/**/*.py`
- 请求/响应 DTO：`backend/models/schemas.py`
- 错误码：`backend/core/error_codes.py`
- 数据库实现：`backend/core/db/store.py`
- 数据目录和内置工具配置：`backend/core/config.py`
- 同步实现：`backend/core/skills/sync_engine.py`、`sync_service.py`
- 工具适配器：`backend/core/tools/adapters.py`
- 前端调用入口：`frontend/src/lib/api.ts`、`frontend/src/services/`
- 当前构建：`backend/build.py`、`backend/desktop.py`、`.github/workflows/release.yml`

已有说明文档只能作为导航，不能覆盖当前代码事实：

- `README.md`
- `docs/database-schema.md`
- `backend/docs/*.md`
- `frontend/docs/*.md`

## 3. 实施步骤

### 3.1 固定 Git 基线

```powershell
git status --short --branch
git log --oneline --decorate -12
git branch --show-current
```

记录：

- 当前分支必须是 `main-Rust`。
- 实施前已有修改必须单独记录，不能被 Agent 提交覆盖。
- 当前已知的 `frontend/package-lock.json` 修改必须在后续 diff 中单独核对。

### 3.2 运行现有基线测试

```powershell
cd frontend
npm run check

cd ../backend
python -m pytest -q
```

如果命令失败，必须记录真实失败原因；不能把失败的基线写成通过。

### 3.3 生成 API 路由清单

从 `backend/api/**/*.py` 读取实际路由装饰器，记录：

- 文件
- HTTP method
- 原始路径
- 路径参数
- query 参数
- request model
- response model
- 错误码/异常映射
- 前端是否实际调用

当前已核实的路由模块包括：

| 模块 | 主要范围 |
|---|---|
| `api/skills/crud.py` | managed skills、导入、安装、删除、source URL |
| `api/skills/files.py` | skill 文件列出、读取、写入 |
| `api/skills/sync.py` | 单 skill、套件、scope、recent project |
| `api/tools/status.py` | 工具状态 |
| `api/tools/tool_skills.py` | 工具 skill、适配器配置、工具目录操作 |
| `api/tags.py` | 标签 CRUD 和 skill 标签关联 |
| `api/settings.py` | 仓库路径、默认工具、更新设置、目录选择 |
| `api/database.py` | 数据库概览、表查询、维护、导出、重置 |
| `api/onboarding.py` | Onboarding 计划 |
| `api/maintenance.py` | 同步健康和修复 |
| `api/tasks.py` | 后台任务 |
| `api/update.py` | 更新检查和执行 |
| `api/reorder.py` | 排序 |
| `api/health.py` | 健康检查 |

不得只依赖模块名称。完整 endpoint 表必须由当前代码实际提取后写入 `generated/endpoint-inventory.md`。当前生成结果包含 API router 和 `backend/main.py` 中的 app-level endpoint；`summary`、状态码、异常和前端使用情况仍需人工核对。

### 3.4 生成 DTO 清单

从 `backend/models/schemas.py` 记录每个 Pydantic model 的：

- 类名
- 字段
- 类型
- 默认值
- 可空性
- 校验约束
- 所属 command

重点模型包括：

- `ManagedSkillDto`
- `SkillTargetDto`
- `ToolInfo`
- `ToolSkillsResponse`
- `ToolAdapterConfigResponse`
- `SyncRequest`
- `SyncSuiteRequest`
- `InstallLocalRequest`
- `OnboardingPlan`
- `TaskStartResponse`
- `CheckUpdateResponse`
- `PerformUpdateResponse`
- 数据库管理相关 request/response model

### 3.5 生成 command 映射

Tauri command 按领域命名，不保留 `/api/` 前缀。命名规则：

- 集合查询：`list_*`
- 单项查询：`get_*`
- 创建：`create_*`
- 更新：`update_*` 或 `save_*`
- 删除：`delete_*`
- 扫描：`scan_*`
- 同步：`sync_*`、`unsync_*`
- 修复：`repair_*`
- 数据库操作：`get_database_*`、`run_database_*`

例如：

```text
/api/get_managed_skills  → list_managed_skills
/api/get_tool_skills     → list_tool_skills
/api/db/overview         → get_database_overview
/api/db/maintenance      → run_database_maintenance
```

以上示例只说明命名规则；完整映射必须以实际生成的 inventory 为准。

### 3.6 记录前端实际调用

检查：

```powershell
git grep -n "apiCall(" -- frontend/src
git grep -n "apiGet(" -- frontend/src
git grep -n "from '@/services'" -- frontend/src
```

将调用分为：

- 必须迁移的用户功能
- 仅开发或诊断用途
- 当前后端存在但前端未使用
- 需要在 Rust 中保留但可延后实现

## 4. 契约要求

### 4.1 Rust command 返回值

所有 command 统一采用：

```rust
Result<T, AppError>
```

错误序列化结构保持：

```json
{
  "ok": false,
  "code": "ERROR_CODE",
  "message": "用户可读错误",
  "detail": {}
}
```

### 4.2 字段命名

- Tauri 参数字段：`snake_case`
- Rust struct 字段：`snake_case`
- JSON 字段：`snake_case`
- React 内部变量和 Props：继续使用 `camelCase`
- 禁止增加全局 snake/camel 自动转换层

### 4.3 错误码

优先复用 `backend/core/error_codes.py` 当前错误码：

- `PROJECT_SCOPE_UNSUPPORTED`
- `TOOL_NOT_INSTALLED`
- `TOOL_NOT_WRITABLE`
- `TARGET_EXISTS`
- `SKILL_INVALID`
- `INTERNAL_ERROR`

如果 Rust 需要新错误码，必须在契约文档中说明触发条件、前端展示方式和测试用例。

## 5. 产物

```text
docs/rust-migration/generated/endpoint-inventory.md
docs/rust-migration/generated/dto-inventory.md
docs/rust-migration/generated/command-map.md
docs/rust-migration/generated/database-facts.md
docs/rust-migration/generated/frontend-call-sites.md
```

这些 inventory 是实施依据。若实现过程中发现当前代码与 inventory 不一致，必须先更新 inventory 并报告差异。

## 6. 验收标准

- 所有清单有当前源码证据。
- 没有凭空新增 endpoint 或 DTO。
- 基线测试结果已记录。
- command 命名和错误结构已经冻结。
- 其他工作包可以只依赖这些文档和共享接口，不再自行猜测。
