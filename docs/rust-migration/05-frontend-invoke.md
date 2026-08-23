# 05. 前端 Tauri invoke 与 Service 层迁移

## 1. 目标

将 React 前端从 HTTP `fetch('/api/...')` 迁移到 Tauri `invoke`，保持现有用户界面、hooks、service 语义和 DTO 字段不变，最终删除本地 HTTP transport。

## 2. 当前代码证据

- `frontend/src/lib/api.ts`：当前 `apiCall`、`apiGet` 和 DTO。
- `frontend/src/services/skillService.ts`：技能相关 service。
- `frontend/src/services/tagService.ts`：标签相关 service。
- `frontend/src/hooks/`：API 使用入口。
- `frontend/docs/API_STANDARD.md`：当前 HTTP 调用和 DTO 规范。
- `frontend/docs/PROJECT_STRUCTURE.md`：前端分层和调用约束。

当前行为：

```text
hooks → services → apiCall/apiGet → fetch → /api/... → FastAPI
```

目标行为：

```text
hooks → services → invoke transport → Tauri command → Rust service
```

## 3. 允许修改范围

```text
frontend/src/lib/**
frontend/src/services/**
frontend/src/hooks/**
frontend/src/features/**
frontend/src/components/**
```

不修改：

- Rust command 注册；
- Rust service；
- Python 后端；
- 与通信迁移无关的 UI 和 CSS。

## 4. 实施步骤

### 4.1 建立统一 invoke transport

通信层必须提供统一调用入口，例如：

```text
invoke_command<TResponse>(command, params)
```

具体函数名以当前代码结构和 Tauri API 实际安装结果为准，但 React 组件不能直接导入 Tauri API。

所有调用必须：

- 传递 `snake_case` 参数；
- 返回已声明的 DTO；
- 将 Rust `AppError` 转为前端统一 Error；
- 保留当前 toast 和 loading 行为。

### 4.2 迁移 service

Service 方法继续使用前端 `camelCase`：

```text
skillService.listManagedSkills()
tagService.createTag()
```

但跨边界 command 和 DTO 字段使用 `snake_case`：

```text
list_managed_skills
skill_id
project_path
source_url
```

Service 负责把前端语义方法映射到 command，不允许组件散落 command 字符串。

### 4.3 迁移 GET/POST 差异

HTTP 的 GET/POST 差异在 Tauri 中不再存在。迁移时按业务动作划分 command，不保留 HTTP method 判断。

路径参数必须转为 command 参数。例如数据库表查询不再拼接：

```text
/db/table/{table_name}
```

而是传递：

```json
{
  "table_name": "skills",
  "page": 1,
  "page_size": 50
}
```

最终字段名以 `00-baseline-and-contract.md` 生成的 command map 为准。

### 4.4 迁移任务事件

前端监听：

```text
task_started
task_progress
task_completed
task_failed
```

组件卸载时必须取消监听，避免重复订阅和过期状态覆盖。

### 4.5 清理 HTTP 依赖

完成 invoke 迁移后搜索：

```powershell
git grep -n "apiCall(" -- frontend/src
git grep -n "apiGet(" -- frontend/src
git grep -n "fetch('/api" -- frontend/src
git grep -n "localhost:18921" -- frontend/src
```

允许保留的内容必须只出现在迁移说明文档中，不能出现在生产业务代码中。

### 4.6 更新前端文档

迁移完成后同步更新：

- `frontend/docs/API_STANDARD.md`
- 必要时更新 `frontend/docs/PROJECT_STRUCTURE.md`

文档必须描述当前实际的 Tauri invoke 行为，不得继续把 HTTP 规范写成当前实现。

## 5. 测试要求

- Service command 参数测试；
- Rust 错误到前端 Error 的转换测试；
- loading/error/success 状态测试；
- 任务事件订阅和清理测试；
- 主要页面启动集成测试；
- `npm run lint`；
- `npm run build`；
- `npm run check`。

## 6. 验收标准

- React 代码不直接依赖 HTTP API。
- Service 是唯一业务调用入口。
- 组件可见行为不变。
- Tauri 桌面模式启动后主要页面均能访问 Rust command。
- 前端不再依赖 `localhost:18921`。
