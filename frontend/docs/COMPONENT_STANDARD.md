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
- Layer 3 action hooks（`useTagActions`、`useSkillActions`）通过 `@/services` 调用 API，而非直接 `apiCall`
- hooks 返回值通过 props 向下传递给组件
- 所有导入统一使用 `@/` 路径别名

### 3.3 useCallback 规则

- 所有返回给组件的函数必须使用 `useCallback` 包装，保持引用稳定
- `useCallback` 的依赖数组必须完整列出所有外部依赖

## 4. 组件模式

### 4.1 memo 优化

以下组件使用 `memo()` 包装，避免不必要的重渲染：
- `Header`、`FilterBar`、`SkillsList`、`SkillCard`

**新增组件时**：如果组件接收的 props 不频繁变化，且父组件重渲染频繁，应使用 `memo()` 包装。

### 4.2 弹窗模式

- **懒加载 + 条件渲染**：所有弹窗在 `App.tsx` 中通过 `React.lazy()` 导入，包裹在 `<Suspense fallback={null}>` 中，且仅当条件满足时才渲染（如 `{tagEditorSkill ? <EditSkillTagsModal /> : null}`）
- **非懒加载弹窗**：使用 `if (!open) return null` 提前返回
- **禁止**在弹窗组件内部管理自己的 visible 状态（由 `ModalContext` 统一管理）
- **懒加载组件不得通过 barrel `index.ts` 静态导出**（会导致 Vite 无法拆分 chunk）

### 4.3 TFunction 传递

`t: TFunction` 作为 props 传入所有组件，而非在组件内部调用 `useTranslation()`：

```tsx
// ✅ 正确：通过 props 接收 t
function SkillCard({ skill, t, ... }: SkillCardProps) { ... }

// ❌ 错误：在组件内部调用 useTranslation
function SkillCard({ skill, ... }: SkillCardProps) {
  const { t } = useTranslation()  // 禁止
}
```

## 5. DTO 类型管理

### 5.1 核心 DTO（`src/features/skills/types.ts`）

`ManagedSkill`、`OnboardingPlan`、`OnboardingGroup`、`OnboardingVariant`、`ToolOption`、`TagDto`、`TagWithCountDto`、`LocalSkillCandidate`、`InstallResultDto`、`ToolInfoDto`、`ToolStatusDto`、`SkillFileEntry`、`SkillUsage`

### 5.2 API 关联 DTO（`src/lib/api.ts`）

与 API 函数紧密关联的 DTO 定义在 `api.ts` 中：
- `ScopePreferenceDto`、`ReorderItem`
- `DbOverview`、`DbTableInfo`、`DbColumnInfo`、`DbTableData`、`DbMaintenanceResult`

### 5.3 规则

- 新增 DTO 类型时，根据其用途选择放置位置（核心业务 → `features/skills/types.ts`，API 关联 → `lib/api.ts`）
- 所有 DTO 字段必须使用 `snake_case`，与后端 JSON 字段完全一致
- **禁止**使用 `camelCase` 字段名 + `toSnakeCase()` 转换

## 6. i18n 规范

### 6.1 初始化（`src/i18n/index.ts`）

```typescript
i18n.use(initReactI18next).init({
  resources,
  lng: getStoredLanguage() ?? 'zh',    // 从 localStorage 读取，默认中文
  fallbackLng: 'en',
  interpolation: { escapeValue: false },
})
```

### 6.2 翻译资源结构（`src/i18n/resources.ts`）

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

### 6.3 规则

- **所有用户可见文本必须使用 i18n**：`t('namespace.key')`
- 新增功能时，必须同时添加中英文翻译
- 翻译 key 使用点号分隔的命名空间：`errors.targetExists`、`actions.removing`
- 错误消息通过 `errors.ts` 的 `parseErrorDetail` 映射到 `errors.*` 和 `projectSync.*` 命名空间
- **禁止**在组件中硬编码用户可见文本
