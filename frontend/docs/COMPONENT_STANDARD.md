# 组件与状态规范

> 本文件描述前端组件模式、Hooks 架构、Context 使用约定和 i18n 规范。修改组件后同步更新。

## 1. 命名规范

| 层级 | 命名风格 | 示例 |
|------|---------|------|
| JSON 传输 / DTO 字段 | `snake_case` | `skill_id`, `source_type`, `created_at` |
| 组件 Props 字段 | `camelCase` | `sortBy`, `searchQuery`, `loadingStartAt` |
| useState 变量 | `camelCase` | `pendingDeleteId`, `actionMessage` |
| 函数名 | `camelCase` | `handleDeleteManaged`, `getSkillScope` |
| 组件文件名 | `PascalCase` | `SkillCard.tsx`, `FilterBar.tsx` |
| Props 类型名 | `ComponentNameProps` | `SkillCardProps`, `FilterBarProps` |
| CSS 类名 | `kebab-case` | `modal-backdrop`, `skill-card` |
| Hook 文件名 | `camelCase.ts` | `useSkills.ts`, `useTheme.ts` |
| Hook 函数名 | `use*` | `useSkills`, `useSkillFilter` |

### 禁止事项

1. **禁止**在 `api.ts` 中使用 `toSnakeCase()` 转换函数
2. **禁止** API 调用参数使用 camelCase（如 `skillId` 必须写成 `skill_id`）
3. **禁止** DTO 类型字段使用 camelCase（如 `sourceType` 必须写成 `source_type`）
4. **禁止**前端内部变量使用 snake_case（如 `loading_start_at` 应写成 `loadingStartAt`）

## 2. Context 使用约定

应用使用两个 Context，**仅管理全局关注点，不传递业务数据**：

### 2.1 AppStateContext（`src/context/AppStateContext.tsx`）

管理的全局状态：
- `language`：当前语言（`'zh'` | `'en'`），持久化到 `localStorage`
- `error`：错误对象，`useEffect` 监听后通过 sonner toast 显示
- `actionMessage`：操作进行中提示（显示在 LoadingOverlay 中）
- `successToastMessage`：成功消息，`useEffect` 监听后通过 toast 显示

提供的核心函数：
- `setError(message)`：设置错误（自动触发 toast）
- `setActionMessage(message)` / `setSuccessToastMessage(message)`
- `toggleLanguage()`：切换中/英文
- `formatErrorMessage(err)`：使用 `parseErrorDetail()` 解析后端错误为 i18n key
- `showActionErrors(result, fallbackKey)`：批量处理操作结果中的错误

### 2.2 ModalContext（`src/context/ModalContext.tsx`）

管理的状态：
- `activeView`：当前视图（`'myskills'` | `'detail'` | `'settings'` | `'tags'` | `'tools'`）
- `activeSkillSource`：技能来源（`'custom'` | `'community'`）
- `detailSkill`：当前查看详情的技能
- 弹窗状态：`infoModalSkill`、`showImportModal`、`showNewToolsModal`、`pendingDeleteId`、`tagEditorSkill`、`pendingDeleteTag`

### 2.3 禁止事项

- **禁止**使用 Context 传递业务数据（技能列表、工具状态等应通过 hooks + props 传递）
- **禁止**新增第三个 Context，除非有充分的全局关注点理由

## 3. Hooks 架构

### 3.1 分层依赖

hooks 在 `AppContent` 中按依赖顺序分层实例化：

```
Layer 1（基础数据）
  useScopeState() → useSkills() → useTheme()

Layer 1.5（派生 helper）
  getSkillScope() 等 useCallback 函数

Layer 2（功能 hooks）
  useSkillFilter() → useImportFlow() → useScopeManager() → useAddSkill()

Layer 3（action hooks，使用 service 层）
  useTagActions() → useSkillActions()
```

### 3.2 依赖传递规则

- Layer 2/3 hooks 接收 Layer 1 的状态和回调函数作为**函数参数**，不通过 Context
- 例：`useImportFlow` 接收 `useSkills` 返回的 `tools`、`isInstalled`、`loadManagedSkills` 等
- Layer 3 action hooks（`useTagActions`、`useSkillActions`）通过 `@/services` 调用 API，而非直接 `invokeCommand`
- hooks 返回值通过 props 向下传递给组件
- 所有导入统一使用 `@/` 路径别名

### 3.3 依赖注入：两种参数风格

Hook 接收依赖时使用两种风格，根据参数数量选择：

**方式 A — 位置参数**（参数 ≤ 4 时，用于 `useSkills`、`useSkillFilter`、`useTheme`）：

```typescript
export function useSkills(
  t: TFunction,
  setError: (msg: string) => void,
  setSuccessToastMessage: (msg: string) => void,
)
```

**方式 B — 参数对象**（参数 > 4 时，用于 `useImportFlow`、`useTagActions`、`useSkillActions`、`useScopeManager`、`useAddSkill`）：

```typescript
interface UseTagActionsParams {
  t: TFunction
  loadManagedSkills: () => Promise<void>
  loadTags: (source: SkillSource) => Promise<void>
  activeSkillSource: SkillSource
  setError: (msg: string) => void
  setSuccessToastMessage: (msg: string) => void
  setActionMessage: (msg: string | null) => void
  selectedTagIds: number[]
  setSelectedTagIds: (updater: number[] | ((prev: number[]) => number[])) => void
  pendingDeleteTag: TagWithCountDto | null
  setPendingDeleteTag: (tag: TagWithCountDto | null) => void
  globalLoading: boolean
  setLoading: (v: boolean) => void
  setLoadingStartAt: (v: number | null) => void
}
export function useTagActions(params: UseTagActionsParams)
```

每个使用方式 B 的 hook 须定义对应的 `...Params` 或 `...Deps` 接口。

### 3.4 Service 层访问：两种方式

Hook 访问后端 API 时使用两种方式，均最终调用 `invokeCommand`：

1. **Service 对象**（`tagService`、`skillService`）：用于可归组的 CRUD 操作。Action hook（`useTagActions`、`useSkillActions`）优先使用此方式：

   ```typescript
   await tagService.createTag(name)
   await skillService.deleteManagedSkill(skillId)
   ```

2. **直接 `invokeCommand` / `useApi().invoke`**：用于数据加载 hook（`useSkills`、`useImportFlow`、`useTheme`）中调用多个不同命令：

   ```typescript
   const { invoke } = useApi()
   const plan = await invoke<OnboardingPlan>('get_onboarding_plan')
   ```

> Service 层和 API 层的详细规范见 [API_STANDARD.md](API_STANDARD.md)。

### 3.5 useCallback 规则

- 所有返回给组件的函数必须使用 `useCallback` 包装，保持引用稳定
- `useCallback` 的依赖数组必须完整列出所有外部依赖
- 纯 `setState` 包裹的 handler 使用空依赖数组：

  ```typescript
  const handleSortChange = useCallback((value: 'manual' | 'updated' | 'name') => {
    setSortBy(value)
  }, [])
  ```

## 4. Hooks 异步操作模式

### 4.1 异步 Action + Loading 状态（标准模式）

所有 mutating 操作（删除、创建、更新、同步等）使用统一的 try-catch-finally 模式：

```typescript
const handleConfirmDelete = useCallback(async () => {
  if (!pendingDeleteTag) return
  try {
    setLoading(true)
    setLoadingStartAt(Date.now())
    setActionMessage(t('actions.deletingTag', { name: pendingDeleteTag.name }))
    await tagService.deleteTag(pendingDeleteTag.id)
    // 变更后刷新数据
    await loadManagedSkills()
    await loadTags(activeSkillSource)
    setPendingDeleteTag(null)
    setSuccessToastMessage(t('tagDeleted'))
  } catch (err) {
    setError(err instanceof Error ? err.message : String(err))
  } finally {
    setLoading(false)
    setLoadingStartAt(null)
    setActionMessage(null)
  }
}, [/* 所有引用的外部依赖 */])
```

**模式要点：**
- `setLoading(true)` + `setLoadingStartAt(Date.now())` + `setActionMessage(...)` 在 try 顶部
- `setLoading(false)` + `setLoadingStartAt(null)` + `setActionMessage(null)` 在 finally 中
- 变更成功后调用 `loadManagedSkills()` / `loadTags()` 刷新数据（无缓存库，手动 re-fetch）
- 错误通过 `setError` 传递（内部调用 `parseErrorDetail` 解析并显示 toast）

### 4.2 乐观更新 + 回退（拖拽排序模式）

拖拽排序使用乐观更新——先更新本地状态，API 失败时回退为全量重载：

```typescript
const reorderSkills = useCallback(async (items: ReorderItem[]) => {
  // 乐观更新：先更新本地状态
  setManagedSkills(prev => /* 按新顺序重排 */)
  try {
    await apiReorder('skills', items)
  } catch {
    // 失败时全量重载
    await loadManagedSkills()
    setError(t('errors.reorderFailed'))
  }
}, [loadManagedSkills, setError])
```

### 4.3 Loading 状态线程传递

Loading 状态不在 Context 中集中管理。`AppContent` 持有本地 `loading` / `loadingStartAt`，向下传递给 action hook：

```
AppContent
  ├── const [loading, setLoading] = useState(false)
  ├── const [loadingStartAt, setLoadingStartAt] = useState<number | null>(null)
  │
  ├── useTagActions({ ..., setLoading, setLoadingStartAt })
  ├── useSkillActions({ ..., setLoading, setLoadingStartAt })
  │
  ├── useImportFlow(...)         ← 自有 loading / loadingStartAt
  ├── useScopeManager(...)      ← 自有 loading / loadingStartAt
  │
  └── 合并：
      const globalLoading = loading || importFlow.loading || scopeManager.loading
      const globalLoadingStartAt = loadingStartAt || importFlow.loadingStartAt || scopeManager.loadingStartAt
```

`globalLoading` 和 `globalLoadingStartAt` 传递给 `<LoadingOverlay>` 组件显示全局加载遮罩。

### 4.4 并行数据加载

多个独立数据源可并行加载（`Promise.all`）：

```typescript
const handleRefreshSkills = useCallback(async () => {
  if (refreshingSkills) return  // 防止重复刷新
  setRefreshingSkills(true)
  try {
    await Promise.all([
      loadManagedSkills(),
      loadTags(activeSkillSource),
      loadToolSkills(),
      loadToolStatus(),
    ])
  } finally {
    setRefreshingSkills(false)
  }
}, [/* deps */])
```

## 5. 组件模式

### 5.1 memo 优化

以下组件使用 `memo()` 包装，避免不必要的重渲染：
- `Header`、`FilterBar`、`SkillsList`、`SkillCard`

**新增组件时**：如果组件接收的 props 不频繁变化，且父组件重渲染频繁，应使用 `memo()` 包装。

### 5.2 弹窗模式

- **懒加载 + 条件渲染**：所有弹窗在 `App.tsx` 中通过 `React.lazy()` 导入，包裹在 `<Suspense fallback={null}>` 中，且仅当条件满足时才渲染（如 `{tagEditorSkill ? <EditSkillTagsModal /> : null}`）
- **非懒加载弹窗**：使用 `if (!open) return null` 提前返回
- **禁止**在弹窗组件内部管理自己的 visible 状态（由 `ModalContext` 统一管理）
- **懒加载组件不得通过 barrel `index.ts` 静态导出**（会导致 Vite 无法拆分 chunk）

### 5.3 TFunction 传递

`t: TFunction` 作为 props 传入所有组件，而非在组件内部调用 `useTranslation()`：

```tsx
// ✅ 正确：通过 props 接收 t
function SkillCard({ skill, t, ... }: SkillCardProps) { ... }

// ❌ 错误：在组件内部调用 useTranslation
function SkillCard({ skill, ... }: SkillCardProps) {
  const { t } = useTranslation()  // 禁止
}
```

## 6. DTO 类型管理

### 6.1 核心 DTO（`src/features/skills/types.ts`）

`ManagedSkill`、`OnboardingPlan`、`OnboardingGroup`、`OnboardingVariant`、`ToolOption`、`TagDto`、`TagWithCountDto`、`LocalSkillCandidate`、`InstallResultDto`、`ToolInfoDto`、`ToolStatusDto`、`SkillFileEntry`、`SkillUsage`

### 6.2 API 关联 DTO（`src/lib/api.ts`）

与 API 函数紧密关联的 DTO 定义在 `api.ts` 中：
- `ScopePreferenceDto`、`ReorderItem`
- `DbOverview`、`DbTableInfo`、`DbColumnInfo`、`DbTableData`、`DbMaintenanceResult`

### 6.3 规则

- 新增 DTO 类型时，根据其用途选择放置位置（核心业务 → `features/skills/types.ts`，API 关联 → `lib/api.ts`）
- 所有 DTO 字段必须使用 `snake_case`，与后端 JSON 字段完全一致
- **禁止**使用 `camelCase` 字段名 + `toSnakeCase()` 转换

## 7. i18n 规范

### 7.1 初始化（`src/i18n/index.ts`）

```typescript
i18n.use(initReactI18next).init({
  resources,
  lng: getStoredLanguage() ?? 'zh',    // 从 localStorage 读取，默认中文
  fallbackLng: 'en',
  interpolation: { escapeValue: false },
})
```

### 7.2 翻译资源结构（`src/i18n/resources.ts`）

`en.translation` 和 `zh.translation` 两个语言包，嵌套命名空间：

| 命名空间 | 内容 |
|----------|------|
| `brand` | 品牌名称（`skills` / `hub`） |
| `languageShort` | 语言缩写（`en` / `zh`） |
| `languageOptions` | 语言选项（`English` / `中文`） |
| `sourceTabs` | 技能来源标签（自定义 / 社区） |
| `settings` | 设置页面文案 |
| `db` | 数据库面板文案 |
| `themeOptions` | 主题选项（`system` / `light` / `dark`） |
| `scope` | Scope 相关标签 |
| `projectSync` | 项目同步文案 |
| `toolFilter` | 工具过滤标签 |
| `toolsPage` | 工具页面文案 |
| `localSkillInvalid` | 本地技能无效提示（缺少 SKILL.md 等） |
| `errors` | 错误消息（`targetExists`、`toolNotInstalled` 等） |
| `actions` | 操作进行中消息 |
| `status` | 状态消息 |
| `relative` | 相对时间文案 |
| `delete` | 删除确认文案 |
| `layout` | 布局文案 |
| `detail` | 详情视图文案 |
| `tools` | 44 AI 工具翻译键值 |

> 除命名空间外，`translation` 根下还有扁平 key（如 `unknown`、`subtitle`、`navMySkills`、`newSkill`、`filterSort`、`allSkills` 等）。

### 7.3 规则

- **所有用户可见文本必须使用 i18n**：`t('namespace.key')`
- 新增功能时，必须同时添加中英文翻译
- 翻译 key 使用点号分隔的命名空间：`errors.targetExists`、`actions.removing`
- 错误消息通过 `errors.ts` 的 `parseErrorDetail` 映射到 `errors.*` 和 `projectSync.*` 命名空间
- **禁止**在组件中硬编码用户可见文本
