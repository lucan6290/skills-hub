# API 与数据传输规范

> 本文件描述前端 API 层架构、Tauri 命令调用约定、Service 层模式、错误处理机制和 DTO 管理规范。修改 API 调用或 DTO 类型后同步更新。

## 1. 架构总览

前端通过 Tauri IPC（`invoke`）调用 Rust 后端命令，不使用 HTTP 请求。数据流分四层：

```
组件 / Hook
    ↓
Service 层（services/）         ← 封装相关命令为对象方法
    或
命名 API 函数（lib/api.ts）      ← 直接调用 invokeCommand
    ↓
invokeCommand（lib/api.ts）      ← 唯一 transport，统一错误归一化
    ↓
Tauri invoke → Rust #[tauri::command]
```

### 1.1 两种调用方式

| 方式 | 位置 | 适用场景 | 示例 |
|------|------|---------|------|
| Service 对象 | `services/*.ts` | 可归组的 CRUD 操作（标签、技能） | `tagService.createTag(name)` |
| 命名 API 函数 | `lib/api.ts` | 独立命令（数据库、更新、代理等） | `fetchDbOverview()` |
| `useApi().invoke` | `hooks/useApi.ts` | Hook 内需稳定引用的临时调用 | `invoke('get_onboarding_plan')` |

三者最终都调用 `invokeCommand`，无 HTTP 层、无 REST client。

## 2. Transport 层：`invokeCommand`

**文件：** `src/lib/api.ts`

`invokeCommand` 是前端唯一的 Tauri IPC transport。所有 Rust 后端调用必须经过此函数。

```typescript
export async function invokeCommand<T = unknown>(
  command: string,
  params?: Record<string, unknown>,
): Promise<T>
```

### 2.1 错误归一化

`invokeCommand` 将 Rust 抛出的异常统一转为 JS `Error`：

| Rust 抛出类型 | 处理方式 |
|--------------|---------|
| `string` | `new Error(string)` |
| `Error` 实例 | 原样抛出 |
| 普通对象 | 提取 `.message` → `.detail` → `JSON.stringify(err)` |
| 其他 | `String(err)` |

### 2.2 规则

- **禁止**绕过 `invokeCommand` 直接调用 `@tauri-apps/api/core` 的 `invoke`
- **禁止**在 `invokeCommand` 之外新建第二个 transport 函数
- 所有参数直接使用 `snake_case` 传递，与 Rust `#[tauri::command]` 参数名完全一致，**禁止任何转换**

## 3. Service 层

**目录：** `src/services/`

Service 是将相关 Tauri 命令归组为对象方法的薄封装层。每个 service 是一个普通对象（非 class），方法内部直接调用 `invokeCommand`。

### 3.1 现有 Service

| Service | 文件 | 方法 | 对应 Tauri 命令 |
|---------|------|------|----------------|
| `tagService` | `tagService.ts` | `createTag(name)` | `create_tag` |
| | | `renameTag(tagId, name)` | `rename_tag` |
| | | `deleteTag(tagId)` | `delete_tag` |
| `skillService` | `skillService.ts` | `deleteManagedSkill(skillId)` | `delete_managed_skill` |
| | | `setSkillTags(skillId, tagIds)` | `set_skill_tags` |

通过 `services/index.ts` barrel 统一导出：

```typescript
import { tagService, skillService } from '@/services'
```

### 3.2 Service 层规则

- Service 方法参数使用 `camelCase`（JS 惯例），在调用 `invokeCommand` 时显式映射为 `snake_case`
- Service 不做错误处理——错误由 `invokeCommand` 归一化后，由调用方（hook）的 try-catch 处理
- **禁止**在 Service 中添加业务逻辑、状态管理或缓存
- 新增可归组的 CRUD 命令时，优先创建新 Service 对象而非散落在 `lib/api.ts` 中
- 独立命令（无法归组的）继续放在 `lib/api.ts` 中作为命名函数

### 3.3 camelCase → snake_case 映射示例

```typescript
// tagService.ts — JS 参数 camelCase，invokeCommand 参数 snake_case
renameTag(tagId: number, name: string) {
  return invokeCommand('rename_tag', { tag_id: tagId, name })
  //                                 ^^^^^^^^ 显式映射
}
```

## 4. 错误处理

### 4.1 两层错误处理

前端错误处理分两层，职责严格分离：

| 层 | 位置 | 职责 | 触发时机 |
|----|------|------|---------|
| Transport 归一化 | `lib/api.ts` `invokeCommand` | 将 Rust 异常转为 JS `Error` | 每次 invoke 调用 |
| 语义解析 | `lib/errors.ts` `parseErrorDetail` | 将结构化错误码映射为 i18n key | UI 层（toast 前） |

### 4.2 Rust AppError 结构

Rust 后端通过 `AppError` 枚举（`src-tauri/src/error.rs`）定义结构化错误，序列化为 JSON：

```json
{
  "ok": false,
  "code": "NOT_FOUND",
  "message": "not found: skill xyz",
  "detail": null
}
```

Rust → JSON 的 `code` 映射：

| Rust 变体 | code |
|-----------|------|
| `Unexpected` | `INTERNAL_ERROR` |
| `NotFound` | `NOT_FOUND` |
| `InvalidInput` | `INVALID_INPUT` |
| `PathError` | `PATH_ERROR` |
| `DatabaseError` | `DATABASE_ERROR` |
| `FileSystemError` | `FILESYSTEM_ERROR` |
| `TaskError` | `TASK_ERROR` |
| `UpdateError` | `UPDATE_ERROR` |

### 4.3 语义错误解析：`parseErrorDetail`

**文件：** `src/lib/errors.ts`

`parseErrorDetail` 将后端错误对象解析为 i18n key + params 或 raw message：

```typescript
export function parseErrorDetail(
  detail: unknown,
): { i18nKey: string; params?: Record<string, string> } | { rawMessage: string }
```

错误码 → i18n key 映射：

| code | i18n key | params | 说明 |
|------|----------|--------|------|
| `CANCELLED` | `__silent` | — | 用户主动取消，不显示 toast |
| `TARGET_EXISTS` | `errors.targetExists` | `{ path }` | 目标路径已存在 |
| `TOOL_NOT_INSTALLED` | `errors.toolNotInstalled` | — | 工具未安装 |
| `TOOL_NOT_WRITABLE` | `errors.toolNotWritable` | `{ tool, path }` | 工具目录不可写 |
| `PROJECT_SCOPE_UNSUPPORTED` | `projectSync.unsupportedTool` | `{ tool }` | 工具不支持项目级 scope |
| `SKILL_INVALID` | `errors.skillInvalid` | `{ reason }` | 技能无效（缺少 SKILL.md 等） |
| 未知 code | — | — | 返回 `rawMessage = JSON.stringify(detail)` |
| 非对象 | — | — | 返回 `rawMessage = String(detail)` |

### 4.4 Hook 中的错误处理模式

Action hook 中统一的 try-catch 模式（详见 [COMPONENT_STANDARD.md](COMPONENT_STANDARD.md) § Hooks 异步操作模式）：

```typescript
const handleDelete = useCallback(async () => {
  try {
    setLoading(true)
    setLoadingStartAt(Date.now())
    setActionMessage(t('actions.deleting'))
    await skillService.deleteManagedSkill(skillId)
    await loadManagedSkills()
    setSuccessToastMessage(t('success'))
  } catch (err) {
    setError(err instanceof Error ? err.message : String(err))
  } finally {
    setLoading(false)
    setLoadingStartAt(null)
    setActionMessage(null)
  }
}, [/* deps */])
```

`setError` 来自 `AppStateContext`，内部调用 `parseErrorDetail` 解析错误码并显示 toast。

## 5. DTO 类型管理

### 5.1 放置位置

| 位置 | 内容 | 示例 |
|------|------|------|
| `src/features/skills/types.ts` | 核心业务 DTO | `ManagedSkill`、`TagDto`、`OnboardingPlan`、`ToolInfoDto` |
| `src/lib/api.ts` | API 函数紧密关联的 DTO | `DbOverview`、`DbTableData`、`CheckUpdateResult` |

### 5.2 核心类型速查

#### `ManagedSkill`（`features/skills/types.ts`）

中心业务实体，包含技能全部信息：

```typescript
export type ManagedSkill = {
  id: string
  name: string
  description?: string | null
  source_type: string          // 'custom' | 'community'
  source_ref?: string | null
  source_subpath?: string | null
  source_url?: string | null
  community_path: string
  created_at: number            // Unix timestamp (ms)
  updated_at: number
  last_sync_at?: number | null
  status: string
  tags: TagDto[]
  targets: {
    tool: string
    scope: 'global' | 'project' | string
    project_path?: string | null
    mode: string
    status: string
    target_path: string
    synced_at?: number | null
    suite_skill_id?: string | null
  }[]
  version?: string | null
  author?: string | null
  license?: string | null
  category?: string | null
  homepage?: string | null
  frontmatter_extra?: Record<string, string> | null
  skill_file_count?: number | null
  skill_dir_size?: number | null
  usage?: SkillUsage[] | null
  sort_order: number
  is_suite?: boolean
}
```

#### 其他核心类型

| 类型 | 位置 | 说明 |
|------|------|------|
| `TagDto` | `types.ts` + `api.ts` | 标签（id, name, sort_order） |
| `TagWithCountDto` | `types.ts` | 标签 + skill_count, updated_at |
| `OnboardingPlan` | `types.ts` | 导入计划（groups + variants） |
| `ToolInfoDto` | `types.ts` | 工具信息（key, label, installed, skills_dir） |
| `ToolStatusDto` | `types.ts` | 工具状态汇总（tools, installed, newly_installed） |
| `SkillFileEntry` | `types.ts` + `api.ts` | 技能文件条目（path, size） |
| `LocalSkillCandidate` | `types.ts` | 本地技能候选（name, subpath, valid, reason） |
| `InstallResultDto` | `types.ts` | 安装结果（skill_id, name, community_path） |
| `SkillUsage` | `types.ts` | 使用统计（sync_count, view_count, last_synced_at） |
| `SuiteSubSkill` | `types.ts` | 套件子技能（name, subpath, description） |

#### API 关联 DTO（`lib/api.ts`）

| 类型 | 说明 |
|------|------|
| `ScopePreferenceDto` | Scope 偏好（skill_id, scope, project_paths） |
| `ReorderItem` | 排序项（id, sort_order） |
| `DbOverview` | 数据库概览（路径、大小、表列表） |
| `DbTableInfo` | 表信息（table_name, row_count, size_bytes） |
| `DbTableData` | 表数据（columns, rows, total, page, page_size, total_pages） |
| `DbColumnInfo` | 列信息（cid, name, type, notnull, pk） |
| `DbMaintenanceResult` | 维护结果（ok, action, message, integrity_result） |
| `CheckUpdateResult` | 更新检查结果（current/latest version, download_urls） |
| `PerformUpdateResult` | 执行更新结果（ok, message） |

### 5.3 DTO 规则

- 所有 DTO 字段必须使用 `snake_case`，与后端 Rust struct 字段 / JSON 序列化完全一致
- **禁止**使用 `camelCase` 字段名 + `toSnakeCase()` 转换
- **禁止**在 `api.ts` 中使用 `toSnakeCase()` / `toCamelCase()` 转换函数
- 新增 DTO 时根据用途选择放置位置：核心业务 → `features/skills/types.ts`，API 关联 → `lib/api.ts`
- `TagDto` 和 `SkillFileEntry` 在两处都有定义（历史遗留），新增类型不应重复此模式

## 6. `useApi` Hook

**文件：** `src/hooks/useApi.ts`

```typescript
export function useApi() {
  const invoke = useCallback(
    <T>(command: string, params?: Record<string, unknown>) =>
      invokeCommand<T>(command, params),
    [],
  )
  return { invoke }
}
```

- 返回 `{ invoke }`——`invokeCommand` 的 `useCallback` 稳定引用包裹
- 无 loading/error 状态管理、无缓存、无重试——纯透传
- 用于 Hook 内需稳定回调引用的临时 Tauri 命令调用
- **不用于**已有命名函数或 Service 方法封装的命令（优先使用后者）

### 使用示例

```typescript
// useTheme.ts — 使用 useApi().invoke 调用多个不同命令
const { invoke } = useApi()
const path = await invoke<string>('get_community_repo_path')
const result = await invoke<{ new_path: string }>('set_community_repo_path', { path })
```

## 7. 辅助工具

### 7.1 `pickFolder`（`lib/pickFolder.ts`）

```typescript
export async function pickFolder(promptTitle: string): Promise<string | null>
```

调用 Rust `pick_folder` 命令打开系统原生文件夹选择对话框。Rust 命令不可用时回退到 `window.prompt()` 文本输入。

### 7.2 `formatSize`（`lib/utils.ts`）

```typescript
export function formatSize(bytes: number): string
```

将字节数转为人类可读字符串（`B` / `KB` / `MB` / `GB`，保留一位小数）。

## 8. Tauri 命令完整清单

以下为 `lib.rs` 中 `invoke_handler` 注册的全部命令，按模块分组：

### Skills

| 命令 | 参数 | 返回 | 前端封装 |
|------|------|------|---------|
| `get_managed_skills` | `{ source_type?, tool?, tag_id? }` | `ManagedSkill[]` | `useSkills` hook 内直接 `invokeCommand` |
| `delete_managed_skill` | `{ skill_id }` | `void` | `skillService.deleteManagedSkill` |
| `update_skill_source_url` | `{ skill_id, source_url }` | `ManagedSkill` | `updateSkillSourceUrl()` |
| `import_existing_skill` | `{ source_path, name, source_type }` | `InstallResultDto` | `useImportFlow` hook 内 `invoke` |
| `list_local_skills_cmd` | `{ base_path }` | `LocalSkillCandidate[]` | `useAddSkill` hook 内 `invoke` |
| `install_local_selection` | `{ items }` | `InstallResultDto[]` | `useAddSkill` hook 内 `invoke` |

### Tags

| 命令 | 参数 | 返回 | 前端封装 |
|------|------|------|---------|
| `get_tags` | `{ source? }` | `TagWithCountDto[]` | `useSkills` hook 内 `invoke` |
| `create_tag` | `{ name }` | `void` | `tagService.createTag` |
| `rename_tag` | `{ tag_id, name }` | `{ id, name }` | `tagService.renameTag` |
| `delete_tag` | `{ tag_id }` | `void` | `tagService.deleteTag` |
| `get_skill_tags` | `{ skill_id }` | `TagDto[]` | `fetchSkillTags()` |
| `set_skill_tags` | `{ skill_id, tag_ids }` | `void` | `skillService.setSkillTags` |

### Sync

| 命令 | 参数 | 返回 | 前端封装 |
|------|------|------|---------|
| `sync_skill_to_tool` | `{ source_path, skill_id, tool, name, overwrite, overwrite_if_same_content }` | `void` | `useImportFlow` hook 内 `invoke` |
| `unsync_skill_from_tool` | `{ skill_id, tool }` | `void` | `useScopeManager` hook |
| `sync_suite_to_tool` | `{ ... }` | `void` | `useScopeManager` hook |
| `unsync_suite_from_tool` | `{ ... }` | `void` | `useScopeManager` hook |
| `get_scope_preferences` | — | `ScopePreferenceDto[]` | `fetchScopePreferences()` |
| `set_scope_preference` | `{ skill_id, scope, project_paths }` | `void` | `saveScopePreference()` |
| `get_recent_projects` | — | `string[]` | `useScopeManager` hook |
| `save_recent_project` | `{ path }` | `void` | `useScopeManager` hook |
| `list_suite_sub_skills` | `{ skill_id }` | `SuiteSubSkill[]` | `SkillDetailView` 组件 |

### Files

| 命令 | 参数 | 返回 | 前端封装 |
|------|------|------|---------|
| `list_skill_files` | `{ skill_id }` | `SkillFileEntry[]` | `fetchSkillFiles()` |
| `read_skill_file` | `{ skill_id, file_path }` | `string` | `fetchSkillFileContent()` |
| `write_skill_file` | `{ skill_id, file_path, content }` | `void` | `saveSkillFileContent()` |

### Tools

| 命令 | 参数 | 返回 | 前端封装 |
|------|------|------|---------|
| `get_tool_status` | — | `ToolStatusDto` | `useSkills` hook 内 `invoke` |
| `get_tool_skills` | — | `ToolSkillSnapshot[]` | `useSkills` hook 内 `invokeCommand` |
| `get_tool_adapter_configs` | — | `ToolAdapterConfig[]` | `ToolsPage` 组件 |
| `save_tool_adapter_config` | `{ ... }` | `void` | `ToolsPage` 组件 |
| `reset_tool_adapter_config` | `{ tool_key }` | `void` | `ToolsPage` 组件 |
| `delete_tool_skill` | `{ ... }` | `void` | `ToolsPage` 组件 |
| `open_tool_skills_dir` | `{ tool }` | `void` | `ToolsPage` 组件 |
| `skill_to_community_repo` | `{ ... }` | `void` | `ToolsPage` 组件 |
| `clear_tool_skills` | `{ tool }` | `void` | `ToolsPage` 组件 |

### Settings

| 命令 | 参数 | 返回 | 前端封装 |
|------|------|------|---------|
| `get_default_sync_tools` | — | `string[]` | `SettingsPage` 组件 |
| `save_default_sync_tools` | `{ tools }` | `void` | `SettingsPage` 组件 |
| `get_auto_check_update` | — | `boolean` | `getAutoCheckUpdate()` |
| `set_auto_check_update` | `{ enabled }` | `void` | `setAutoCheckUpdate()` |
| `get_community_repo_path` | — | `string` | `useTheme` hook 内 `invoke` |
| `set_community_repo_path` | `{ path }` | `{ new_path }` | `useTheme` hook 内 `invoke` |
| `get_custom_repo_path` | — | `string` | `useTheme` hook 内 `invoke` |
| `set_custom_repo_path` | `{ path }` | `{ ok, path, empty? }` | `useTheme` hook 内 `invoke` |
| `open_settings_folder` | `{ path }` | `{ ok }` | `useTheme` hook 内 `invoke` |
| `reset_general_settings` | — | `{ ok, community_repo_path, custom_repo_path }` | `useTheme` hook 内 `invoke` |

### Database

| 命令 | 参数 | 返回 | 前端封装 |
|------|------|------|---------|
| `db_overview` | — | `DbOverview` | `fetchDbOverview()` |
| `db_table_data` | `{ table_name, page?, page_size?, sort_col?, sort_dir?, filter_text? }` | `DbTableData` | `fetchDbTableData()` |
| `db_maintenance` | `{ action }` | `DbMaintenanceResult` | `runDbMaintenance()` |
| `db_reset` | `{ confirm_text }` | `{ ok, message }` | `resetDb()` |
| `db_export` | — | `{ ok, message, path? }` | `exportDb()` |
| `db_open_folder` | — | `{ ok, message }` | `openDbFolder()` |

### Onboarding / Tasks / Update / Misc

| 命令 | 参数 | 返回 | 前端封装 |
|------|------|------|---------|
| `get_onboarding_plan` | — | `OnboardingPlan` | `useImportFlow` hook 内 `invoke` |
| `get_task_list` | — | `TaskRecord[]` | — |
| `get_task` | `{ task_id }` | `TaskRecord` | — |
| `cancel_task` | `{ task_id }` | `boolean` | — |
| `check_update` | — | `CheckUpdateResult` | `checkUpdate()` |
| `do_update` | — | `PerformUpdateResult` | `performUpdate()` |
| `pick_folder` | — | `{ path: string \| null }` | `pickFolder()` |
| `cancel_current_operation` | — | `void` | `useImportFlow` hook 内 `invoke` |
| `reorder` | `{ entity, items }` | `void` | `reorder()` |
| `open_new_window` | — | `void` | — |
| `get_proxy_url` | — | `string` | `getProxyUrl()` |
| `set_proxy_url` | `{ url }` | `void` | `setProxyUrl()` |
| `health_check` | — | `{ status, version }` | — |

## 9. 新增 API 调用流程

1. **确认 Rust 命令已注册**：检查 `src-tauri/src/lib.rs` 的 `invoke_handler` 中是否包含目标命令
2. **选择封装方式**：
   - 可归组到现有领域（标签、技能）→ 在对应 Service 中新增方法
   - 可归组到新领域 → 创建新 Service 文件 + 在 `services/index.ts` 导出
   - 独立命令 → 在 `lib/api.ts` 中新增命名函数
3. **定义 DTO 类型**：根据返回数据结构，在 `features/skills/types.ts`（核心业务）或 `lib/api.ts`（API 关联）中定义，字段使用 `snake_case`
4. **实现封装**：调用 `invokeCommand`，参数显式映射为 `snake_case`
5. **更新本文档**：在 §8 对应模块表中添加命令条目
