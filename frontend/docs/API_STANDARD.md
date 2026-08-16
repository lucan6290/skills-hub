# API 调用规范

> 本文件描述前端 HTTP 客户端、API 调用模式、错误处理和 DTO 约定。修改 API 相关代码后同步更新。

## 1. HTTP 客户端

### 1.1 核心函数（`src/lib/api.ts`）

两个核心函数，所有 API 调用必须通过这两个函数：

```typescript
// POST 请求（主要使用）
export async function apiCall<T = unknown>(
  command: string,
  params?: Record<string, unknown>,
): Promise<T>
// → fetch(`${API_BASE}/api/${command}`, { method: 'POST', body: JSON.stringify(params) })

// GET 请求（用于查询类 API）
export async function apiGet<T = unknown>(
  command: string,
  params?: Record<string, unknown>,
): Promise<T>
// → fetch(`${API_BASE}/api/${command}?${query}`)
```

- `API_BASE` 为空字符串（`''`），依赖 Vite 开发代理转发
- 所有参数使用 `snake_case`，与后端字段名完全一致
- **禁止**使用 `toSnakeCase()` 转换函数

### 1.2 useApi Hook（`src/hooks/useApi.ts`）

```typescript
export function useApi() {
  const get = useCallback(
    <T>(command: string, params?: Record<string, unknown>) => apiGet<T>(command, params),
    [],
  )
  const post = useCallback(
    <T>(command: string, params?: Record<string, unknown>) => apiCall<T>(command, params),
    [],
  )
  return { get, post }
}
```

使用 `useCallback` 保持引用稳定。hooks 内部通过 `useApi()` 获取 `{ get, post }`。

## 2. Vite 代理配置

**文件**：`frontend/vite.config.ts`

```typescript
server: {
  port: 5173,
  strictPort: true,
  proxy: {
    '/api': {
      target: 'http://127.0.0.1:18921',
      changeOrigin: true,
    },
  },
},
```

开发环境下，所有 `/api/*` 请求由 Vite 代理转发到 Python 后端（`127.0.0.1:18921`）。

## 3. API 调用模式

### 3.1 POST 模式（写操作）

```typescript
// 通用模式
await post<ReturnType>('command_name', { param1: 'value', param2: 123 })

// 示例
await post<ManagedSkill>('update_skill_source_url', { skill_id, source_url })
await post<DbMaintenanceResult>('db/maintenance', { action })
```

### 3.2 GET 模式（查询操作）

```typescript
// 通用模式
const result = await get<ReturnType>('command_name', { param1: 'value' })

// 示例
const tags = await get<TagDto[]>('get_skill_tags', { skill_id })
const overview = await get<DbOverview>('db/overview')
```

### 3.3 路径参数

部分 API 使用路径参数而非 query 参数：

```typescript
// 数据库表数据查询
await get<DbTableData>(`db/table/${table_name}`, { page, page_size, ... })

// 技能文件内容
await get<string>('read_skill_file', { skill_id, file_path })
```

## 4. Service 层

### 4.1 概述

Service 层位于 `src/services/`，封装 API 调用为语义化方法，供 hooks 调用：

```typescript
// src/services/tagService.ts
import { apiCall } from '@/lib/api'

export const tagService = {
  createTag(name: string): Promise<void> {
    return apiCall('create_tag', { name })
  },
  renameTag(tagId: number, name: string): Promise<{ id: number; name: string }> {
    return apiCall<{ id: number; name: string }>('rename_tag', { tag_id: tagId, name })
  },
  deleteTag(tagId: number): Promise<void> {
    return apiCall('delete_tag', { tag_id: tagId })
  },
}
```

### 4.2 Service 使用规则

- hooks 通过 service 对象调用 API，而非直接 `apiCall`/`api.post()`
- Service 方法名使用 `camelCase`，以动词开头（`createTag`、`deleteManagedSkill`）
- 每个 service 对象聚焦一个领域（`tagService` 管理标签，`skillService` 管理技能）
- Service 不管理状态、不调用 `setState`，只返回 Promise
- 所有 service 通过 `src/services/index.ts` barrel 导出

### 4.3 现有 Service

| Service | 文件 | 方法 |
|---------|------|------|
| `tagService` | `services/tagService.ts` | `createTag`、`renameTag`、`deleteTag` |
| `skillService` | `services/skillService.ts` | `deleteManagedSkill`、`setSkillTags` |

## 5. 专用 API 函数

`lib/api.ts` 导出了多个专用函数，封装特定 API 调用：

| 函数 | API 端点 | 用途 |
|------|---------|------|
| `reorder(entity, items)` | `POST /api/reorder` | 批量更新排序（skills/tags/tools） |
| `fetchScopePreferences()` | `GET /api/get_scope_preferences` | 获取 Scope 偏好列表 |
| `saveScopePreference(...)` | `POST /api/set_scope_preference` | 保存 Scope 偏好 |
| `fetchSkillTags(skill_id)` | `GET /api/get_skill_tags` | 获取技能标签列表 |
| `fetchSkillFiles(skill_id)` | `GET /api/list_skill_files` | 列出技能文件 |
| `fetchSkillFileContent(...)` | `GET /api/read_skill_file` | 读取技能文件内容 |
| `saveSkillFileContent(...)` | `POST /api/write_skill_file` | 保存技能文件内容 |
| `updateSkillSourceUrl(...)` | `POST /api/update_skill_source_url` | 更新技能来源 URL |
| `fetchDbOverview()` | `GET /api/db/overview` | 数据库概览 |
| `fetchDbTableData(...)` | `GET /api/db/table/{name}` | 数据库表数据 |
| `runDbMaintenance(action)` | `POST /api/db/maintenance` | 数据库维护 |
| `resetDb(confirm_text)` | `POST /api/db/reset` | 重置数据库 |
| `getDbExportUrl()` | — | 返回导出 URL（`/api/db/export`） |

### 新增 API 函数规则

1. 优先使用 `apiCall`/`apiGet` 直接调用，而非新建包装函数
2. 如果同一 API 在多处调用，或参数复杂，才提取为专用函数
3. 专用函数必须放在 `lib/api.ts` 中
4. 函数名使用 `camelCase`，以动词开头（`fetch*`、`save*`、`update*`）

## 6. 错误处理

### 6.1 HTTP 层错误（`lib/api.ts`）

`apiCall`/`apiGet` 在 `!res.ok` 时：
1. 尝试解析响应 JSON
2. 提取 `errBody.detail || errBody.message || 'API error ${status}'`
3. 抛出 `Error(message)`

### 6.2 错误解析（`lib/errors.ts`）

`parseErrorDetail(detail)` 将后端错误转换为 i18n key + 参数，兼容两种格式：

**新格式（JSON 对象）**：
```json
{ "code": "TARGET_EXISTS", "path": "/path/to/skill" }
```
→ `{ i18nKey: 'errors.targetExists', params: { path } }`

**旧格式（管道符字符串）**：
```
TARGET_EXISTS|/path/to/skill
```
→ 同上

### 6.3 支持的错误码

| 错误码 | i18n key | 说明 |
|--------|---------|------|
| `CANCELLED` | — | 静默，不显示 toast |
| `TARGET_EXISTS` | `errors.targetExists` | 目标路径已存在 |
| `TOOL_NOT_INSTALLED` | `errors.toolNotInstalled` | 工具未安装 |
| `TOOL_NOT_WRITABLE` | `errors.toolNotWritable` | 工具目录不可写 |
| `PROJECT_SCOPE_UNSUPPORTED` | `projectSync.unsupportedTool` | 不支持项目级 Scope |
| `SKILL_INVALID` | `errors.skillInvalid` | 技能无效 |

### 6.4 错误展示

错误最终通过 `AppStateContext.formatErrorMessage()` 解析为 i18n key，通过 sonner toast 展示给用户。

### 6.5 错误处理模式

```typescript
// hooks 内部标准错误处理模式
try {
  await someApiCall()
  setSuccessToastMessage(t('status.operationSuccess'))
} catch (err) {
  setError(err instanceof Error ? err.message : String(err))
}
```

## 7. DTO 类型约定

- 所有 DTO 字段使用 `snake_case`
- 核心 DTO 定义在 `src/features/skills/types.ts`
- API 关联 DTO 定义在 `src/lib/api.ts`
- **禁止**使用 `camelCase` 字段名 + `Field(alias=...)` 模式
- **禁止**在 API 调用层做任何字段名转换

详细 DTO 类型清单见 [COMPONENT_STANDARD.md](COMPONENT_STANDARD.md) § DTO 类型管理。

## 8. 文件夹选择器

**文件**：`src/lib/pickFolder.ts`

通过后端 API `pick_folder` 打开系统原生文件夹选择对话框。后端不可用时回退到 `window.prompt()`。

```typescript
const path = await pickFolder(t('enterStoragePath'))
```
