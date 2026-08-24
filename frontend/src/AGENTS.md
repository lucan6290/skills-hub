# 前端源码 Agent 入口

本文件是 `src/` 的导航入口。仅补充 [../AGENTS.md](../AGENTS.md) 的硬约束，不放宽任何规则。

> **权威入口**：[../AGENTS.md](../AGENTS.md) 包含完整的编码规范、命名速查、工程规则和任务路由。
> **后端入口**：[../src-tauri/AGENTS.md](../src-tauri/AGENTS.md) 包含 Rust 后端架构和规范。

## 1. 目录速查

```
src/
├── main.tsx               # React 入口（StrictMode + ErrorBoundary + 双 CSS import）
├── App.tsx                # 根组件（AppContent 编排所有 hooks + lazy 视图/弹窗）
├── index.css              # CSS 变量 + Tailwind 基础 + reset
├── app/                   # 顶层错误边界
├── components/layout/     # 布局组件（Header、LoadingOverlay）
├── features/              # 按功能领域组织（feature-based）
│   ├── skills/            # components/ hooks/ modals/ types.ts index.ts
│   ├── tags/              # components/
│   ├── settings/          # components/ hooks/ index.ts
│   ├── tools/             # components/ modals/
│   ├── import-flow/       # components/ hooks/ index.ts
│   └── database/          # components/ index.ts
├── services/              # Service 层（tagService、skillService）封装 API 调用
├── context/               # AppStateContext + ModalContext
├── hooks/useApi.ts        # API 调用统一入口（useCallback 稳定化的 invokeCommand）
├── lib/                   # api.ts、errors.ts、pickFolder.ts、utils.ts
├── styles/                # 模块化样式（13 个 CSS 文件，由 index.css 聚合）
└── i18n/                  # index.ts + resources.ts
```

## 2. 按任务类型快速定位

| 要做什么 | 看哪里 |
|---------|--------|
| 新增/修改组件 | [../docs/COMPONENT_STANDARD.md](../docs/COMPONENT_STANDARD.md) |
| 新增/修改样式 | [../docs/THEME_STANDARD.md](../docs/THEME_STANDARD.md) |
| 新增/修改 API 调用 | [../docs/API_STANDARD.md](../docs/API_STANDARD.md) |
| 新增 Tauri command（后端） | [../src-tauri/AGENTS.md](../src-tauri/AGENTS.md) |
| 理解整体结构 | [../docs/PROJECT_STRUCTURE.md](../docs/PROJECT_STRUCTURE.md) |
| 新增 i18n 文本 | [../docs/COMPONENT_STANDARD.md](../docs/COMPONENT_STANDARD.md) §7 i18n 规范 |

## 3. 本目录关键约束

以下规则在 `src/` 内工作时常被违反，特此强调（完整规则见 [../AGENTS.md](../AGENTS.md) §3）：

1. **路径别名 `@/`**：所有导入使用 `@/` 指向 `src/`，禁止 `../../` 相对路径
2. **样式位置**：CSS 写在 `styles/` 目录下的模块文件中，禁止在组件目录创建独立 CSS 文件
3. **懒加载弹窗**：`React.lazy()` 导入 + 条件渲染（`{state ? <Modal /> : null}`），且不得通过 `index.ts` barrel 静态导出
4. **Service 层**：action hooks 通过 `@/services` 调用 API，而非直接 `invokeCommand`
5. **DTO 字段**：全部 `snake_case`（与后端 JSON 一致），前端内部变量用 `camelCase`
6. **i18n**：所有用户可见文本使用 `t('key')`，禁止硬编码
7. **useCallback**：返回给组件的函数必须用 `useCallback` 包装，依赖数组完整
