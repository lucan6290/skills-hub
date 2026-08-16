# 前端项目结构

> 本文件描述 `frontend/src/` 的实际代码结构。修改代码后同步更新。

## 1. 目录树

```
frontend/src/
├── main.tsx                             # React 入口（StrictMode + ErrorBoundary + createRoot）
├── App.tsx                              # 根组件（AppContent 编排：hooks 实例化 + 视图切换 + lazy 加载）
├── index.css                            # CSS 变量 + Tailwind 导入 + 全局 reset
├── vite-env.d.ts                        # Vite 环境类型声明（__APP_VERSION__）
│
├── app/
│   └── ErrorBoundary.tsx                # 顶层错误边界（包裹整个 App）
│
├── components/
│   └── layout/
│       ├── Header.tsx                   # 顶部导航栏（视图切换、语言切换、设置入口）
│       ├── LoadingOverlay.tsx           # 全局加载遮罩
│       └── index.ts                     # barrel 导出
│
├── context/
│   ├── AppStateContext.tsx              # 全局应用状态（语言、错误、toast）
│   └── ModalContext.tsx                 # 弹窗状态 + 视图切换状态
│
├── features/                            # 按功能领域组织（feature-based architecture）
│   ├── skills/
│   │   ├── index.ts                     # barrel：导出类型、组件、hooks（不导出 lazy 组件）
│   │   ├── types.ts                     # 核心 DTO 类型定义（ManagedSkill 等）
│   │   ├── components/
│   │   │   ├── FilterBar.tsx            # 过滤/搜索/排序工具栏
│   │   │   ├── SkillCard.tsx            # 单个技能卡片
│   │   │   ├── SkillDetailView.tsx      # 技能详情视图（懒加载）
│   │   │   └── SkillsList.tsx           # 技能列表（含拖拽排序）
│   │   ├── hooks/
│   │   │   ├── useSkills.ts             # 核心数据 hook（技能、标签、工具状态）
│   │   │   ├── useSkillFilter.ts        # 搜索/过滤/排序逻辑
│   │   │   ├── useScopeState.ts        # Scope 偏好持久化
│   │   │   ├── useScopeManager.ts      # Scope 管理 + 工具同步/取消同步
│   │   │   ├── useAddSkill.ts          # 添加技能流程
│   │   │   ├── useTagActions.ts         # 标签 CRUD 操作（使用 tagService）
│   │   │   └── useSkillActions.ts      # 技能删除/标签保存操作（使用 skillService）
│   │   └── modals/                     # 弹窗组件（全部懒加载）
│   │       ├── AddSkillModal.tsx
│   │       ├── DeleteModal.tsx
│   │       ├── EditSkillTagsModal.tsx
│   │       ├── ScopeSyncModal.tsx
│   │       ├── SharedDirModal.tsx
│   │       └── SkillInfoModal.tsx
│   │
│   ├── tags/
│   │   └── components/
│   │       └── TagsPage.tsx             # 标签管理页面（懒加载）
│   │
│   ├── settings/
│   │   ├── index.ts                     # barrel：导出 useTheme
│   │   ├── components/
│   │   │   └── SettingsPage.tsx        # 设置页面（懒加载）
│   │   └── hooks/
│   │       └── useTheme.ts             # 主题管理 + 存储路径管理
│   │
│   ├── tools/
│   │   ├── index.ts                     # barrel：导出 UpdatePanel
│   │   ├── components/
│   │   │   ├── ToolsPage.tsx           # 工具页面（懒加载）
│   │   │   └── UpdatePanel.tsx         # 更新检查面板
│   │   └── modals/
│   │       └── NewToolsModal.tsx       # 新工具弹窗（懒加载）
│   │
│   ├── import-flow/
│   │   ├── index.ts                     # barrel：导出 useImportFlow
│   │   ├── components/
│   │   │   ├── ImportModal.tsx         # 导入确认弹窗（懒加载）
│   │   │   └── LocalPickModal.tsx      # 本地技能选择弹窗（懒加载）
│   │   └── hooks/
│   │       └── useImportFlow.ts        # 导入/Onboarding 流程
│   │
│   └── database/
│       ├── index.ts                     # barrel
│       └── components/
│           └── DatabasePanel.tsx        # 数据库管理面板
│
├── services/                            # Service 层：封装 API 调用
│   ├── index.ts                         # barrel：导出所有 service
│   ├── tagService.ts                    # 标签 CRUD API
│   └── skillService.ts                  # 技能删除/标签设置 API
│
├── hooks/
│   └── useApi.ts                        # API 调用统一入口（包装 apiCall/apiGet）
│
├── lib/
│   ├── api.ts                           # HTTP 客户端 + 专用 API 函数 + 部分 DTO 接口
│   ├── errors.ts                        # 错误解析（parseErrorDetail）
│   ├── utils.ts                         # 工具函数（formatSize）
│   └── pickFolder.ts                    # 文件夹选择器（后端 API + prompt 回退）
│
├── styles/                              # 全局模块化样式（按功能模块拆分）
│   ├── index.css                        # 样式聚合入口（@import 所有模块）
│   └── ...                              # 见 AGENTS.md § 目录结构
│
└── i18n/
    ├── index.ts                         # i18next 初始化
    └── resources.ts                     # 中英文翻译资源
```

## 2. 路径别名

`@/` 指向 `frontend/src/`，在 `tsconfig.app.json` 和 `vite.config.ts` 中配置：

```typescript
// 导入示例
import { ManagedSkill } from '@/features/skills'
import { tagService } from '@/services'
import { ErrorBoundary } from '@/app/ErrorBoundary'
```

**禁止**使用相对路径跨层引用（如 `../../components/...`），统一使用 `@/` 别名。

## 3. 组件树与数据流

### 3.1 组件树

```
main.tsx
└── <ErrorBoundary>
    └── <App>                                # AppStateProvider → ModalProvider → AppContent
        ├── <Toaster>                         # sonner toast 通知
        ├── <LoadingOverlay>                  # 全局加载遮罩
        ├── <Header>                          # 顶部导航（视图切换、语言切换、设置入口）
        ├── <main>
        │   └── <Suspense fallback={<div className="view-loading" />}>
        │       ├── activeView='detail'   → <SkillDetailView>     (lazy)
        │       ├── activeView='myskills' → <FilterBar> + <SkillsList>
        │       ├── activeView='tags'     → <TagsPage>            (lazy)
        │       ├── activeView='settings' → <SettingsPage>        (lazy)
        │       └── activeView='tools'    → <ToolsPage>            (lazy)
        │
        └── 弹窗层（Suspense fallback={null}，条件渲染）
            ├── <SkillInfoModal>        (lazy, currentInfoModalSkill ?)
            ├── <AddSkillModal>         (lazy, showAddModal ?)
            ├── <EditSkillTagsModal>    (lazy, tagEditorSkill ?)
            ├── <ImportModal>           (lazy, showImportModal && plan ?)
            ├── <SharedDirModal>        (lazy, pendingSharedToggle ?)
            ├── <ScopeSyncModal>        (lazy, currentScopeModalSkill ?)
            ├── <NewToolsModal>         (lazy, showNewToolsModal ?)
            ├── <DeleteModal>           (lazy, pendingDeleteId ?)
            └── <LocalPickModal>        (lazy, showLocalPickModal ?)
```

### 3.2 分层数据流

`AppContent` 中 hooks 按依赖顺序分层实例化：

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

- Layer 2/3 hooks 接收 Layer 1 的状态和回调函数作为参数
- 所有 hooks 返回值通过 props 向下传递给组件
- Context 仅用于两个全局关注点（见下）

## 4. 状态管理

### 4.1 Context Providers

应用使用两个 Context，**仅管理全局关注点，不传递业务数据**：

| Context | 管理内容 | localStorage key |
|---------|---------|------------------|
| `AppStateContext` | 语言切换、错误处理、toast 消息、操作进行中提示 | `skills-language` |
| `ModalContext` | 视图切换（`activeView`）、弹窗状态、技能详情状态 | — |

### 4.2 核心 Hooks

| Hook | 职责 | 位置 |
|------|------|------|
| `useApi` | 包装 `apiCall`/`apiGet`，提供 `{ get, post }` | `hooks/` |
| `useTheme` | 主题偏好、系统主题监听、存储路径管理 | `features/settings/hooks/` |
| `useSkills` | 核心数据：技能列表、标签、工具状态、CRUD 操作、拖拽排序 | `features/skills/hooks/` |
| `useSkillFilter` | 搜索查询、排序方式、scope/tool/tag 过滤、`visibleSkills` 计算 | `features/skills/hooks/` |
| `useScopeState` | Scope 偏好持久化（后端 API + localStorage 备份） | `features/skills/hooks/` |
| `useScopeManager` | Scope 切换、工具同步/取消同步、共享目录确认流程 | `features/skills/hooks/` |
| `useImportFlow` | Onboarding 计划加载、批量导入流程 | `features/import-flow/hooks/` |
| `useAddSkill` | 添加技能流程（本地扫描、选择、安装、同步） | `features/skills/hooks/` |
| `useTagActions` | 标签 CRUD 操作（通过 `tagService`） | `features/skills/hooks/` |
| `useSkillActions` | 技能删除/标签保存操作（通过 `skillService`） | `features/skills/hooks/` |

### 4.3 localStorage 持久化

| key | 用途 | 管理者 |
|-----|------|--------|
| `skills-language` | 当前语言（`'zh'` / `'en'`） | `AppStateContext` |
| `skills-theme` | 主题偏好（`'system'` / `'light'` / `'dark'`） | `useTheme` |
| `skills-project-scope-state-v1` | Scope 偏好备份 | `useScopeState` |

## 5. 路由机制

应用**不使用 react-router**。`main.tsx` 直接渲染 `<App />`（包裹 `ErrorBoundary`），无路由包裹。

视图切换通过 `ModalContext` 中的 `activeView` 状态实现：

```typescript
type ActiveView = 'myskills' | 'detail' | 'settings' | 'tags' | 'tools'
```

在 `AppContent` 的 JSX 中通过条件渲染切换主视图。视图切换由 `modal.handleViewChange(view)` 触发，由 `Header` 组件的导航按钮调用。

## 6. 代码分割

使用 `React.lazy()` + `Suspense` 实现按需加载：

- **视图组件**（`SkillDetailView`、`TagsPage`、`SettingsPage`、`ToolsPage`）：在 `App.tsx` 中通过 `lazy(() => import('@/features/...'))` 加载，包裹在 `<Suspense fallback={<div className="view-loading" />}>` 中
- **弹窗组件**：同样使用 `lazy` 加载，包裹在 `<Suspense fallback={null}>` 中，且仅当条件满足时才渲染（如 `{state ? <Modal /> : null}`）

### Barrel 导出规则

**懒加载组件不得通过 barrel `index.ts` 静态导出**，否则 Vite 无法将其拆分为独立 chunk：

```typescript
// ❌ 错误：barrel 静态导出懒加载组件（导致 chunk 合并）
export { default as SkillInfoModal } from './modals/SkillInfoModal'

// ✅ 正确：barrel 只导出非懒加载的组件、hooks、类型
export { default as SkillsList } from './components/SkillsList'
export { useSkills } from './hooks/useSkills'
```

## 7. 关键设计决策

1. **Feature-Based Architecture**：按功能领域组织代码（`features/skills/`、`features/settings/` 等），每个 feature 自包含 components/hooks/modals
2. **Service 层**：API 调用通过 `services/` 封装，hooks 通过 service 调用而非直接 `api.post()`
3. **Barrel Exports**：每个 feature 的 `index.ts` 控制对外暴露 API；懒加载组件不进 barrel
4. **路径别名**：统一使用 `@/` 别名，禁止跨层相对路径引用
5. **ErrorBoundary**：在 `main.tsx` 顶层包裹，捕获渲染异常
6. **memo 优化**：`Header`、`FilterBar`、`SkillsList`、`SkillCard` 使用 `memo()` 包装
7. **TFunction 传递**：`t: TFunction` 作为 props 传入组件，而非在组件内部调用 `useTranslation()`
8. **弹窗条件渲染**：弹窗仅当需要时渲染（`{state ? <Modal /> : null}`），而非始终挂载靠 `open` 控制
9. **DTO 类型集中管理**：核心 DTO 在 `features/skills/types.ts`，与 API 紧密关联的 DTO 在 `lib/api.ts`
