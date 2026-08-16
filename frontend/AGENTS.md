# Frontend Agent 入口

本文件是 `frontend/` 的导航入口。只保留前端每次任务都适用的硬约束；详细规则按任务类型从下方「任务路由」逐级读取。

> 全局规则见根目录 [../AGENTS.md](../AGENTS.md)。

## 1. 每次任务都必须遵守

1. 先确认任务范围和完成证据；默认一次只完成一个可验证的工作单元
2. 修改前检查 `git status`，保留开发者已有改动；禁止擅自覆盖、回滚、清理、提交或推送
3. 仓库存在 `.codegraph/` 时，理解或定位代码必须先使用 `codegraph explore`；失败后再使用文本搜索或直接读文件
4. 只改完成当前任务所需的文件。发现邻近问题时记录并报告，不顺手重构
5. 不得读取、打印、复制或提交密钥、令牌、密码及 `.env` 内容。未经明确授权，不调用真实付费模型、云服务、生产接口或会写入外部系统的脚本
6. 验证按风险分级：默认只做与改动直接相关的静态检查（`npm run check`）；启动服务、发请求、运行未知脚本或访问外部服务前，先说明命令、前置条件、预期结果和风险，等待授权
7. 文档与实现冲突时，以当前代码和配置为运行事实，并报告冲突，不能静默猜测
8. 读取目标目录沿途更深层的 `AGENTS.md`；局部规则只能补充本文件，不能放宽本文件的硬约束

## 2. 目录结构

```
frontend/
├── src/
│   ├── main.tsx               # React 入口（StrictMode + ErrorBoundary）
│   ├── App.tsx                # 根组件（AppContent 编排：hooks + lazy 视图/弹窗）
│   ├── index.css              # CSS 变量 + Tailwind 基础 + reset + 全局样式入口
│   ├── vite-env.d.ts          # Vite 环境类型声明（__APP_VERSION__）
│   ├── app/
│   │   └── ErrorBoundary.tsx  # 顶层错误边界
│   ├── components/
│   │   └── layout/            # 布局组件（Header、LoadingOverlay）
│   ├── features/              # 按功能领域组织（feature-based）
│   │   ├── skills/            # 技能：components/ hooks/ modals/ types.ts index.ts
│   │   ├── tags/              # 标签：components/
│   │   ├── settings/          # 设置：components/ hooks/ index.ts
│   │   ├── tools/             # 工具：components/ modals/ index.ts
│   │   ├── import-flow/       # 导入流程：components/ hooks/ index.ts
│   │   └── database/          # 数据库：components/ index.ts
│   ├── services/              # Service 层：封装 API 调用（tagService、skillService）
│   ├── context/               # AppStateContext + ModalContext
│   ├── hooks/
│   │   └── useApi.ts          # API 调用统一入口
│   ├── lib/                   # api.ts、errors.ts、pickFolder.ts、utils.ts
│   ├── styles/                # 全局模块化样式（按功能模块拆分，见下）
│   └── i18n/                  # index.ts + resources.ts
├── package.json               # 前端依赖 + 版本号（Vite 构建时注入）
├── vite.config.ts             # Vite 配置（代理、路径别名 @/、chunk 分割、版本注入）
└── tsconfig.json
```

> 详细目录树见 [docs/PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md)。
> 路径别名：`@/` → `frontend/src/`，配置在 `tsconfig.app.json` + `vite.config.ts`。

### styles/ 模块

| 文件 | 职责 |
|------|------|
| `index.css` | 样式聚合入口（@import 所有模块） |
| `app.css` | App 外壳 + Header + 导航 + 筛选栏 |
| `buttons.css` | 按钮系统（btn-primary/secondary/danger/icon/card-btn） |
| `skill-card.css` | 技能卡片 + 拖拽 + 工具矩阵 + scope badge |
| `modal.css` | Modal 基础设施 + 通用弹窗样式 |
| `skill-detail-modal.css` | 技能详情/信息弹窗 |
| `modals.css` | 专用弹窗（删除/scope/加载/发现/导入/工具配置） |
| `explore.css` | Explore 全页（hero/搜索/卡片网格） |
| `skill-detail-view.css` | Skill 详情视图（文件树/Markdown/元数据） |
| `tags-page.css` | Tags 管理页（hero/表格/token） |
| `tools-page.css` | Tools 管理页（工具卡片/配置弹窗） |
| `settings.css` | 设置页 v2（CC Switch 风格 Tab/segmented） |
| `database.css` | 数据库面板（概览/表格/维护/详情弹窗） |
| `markdown.css` | Markdown 渲染样式 |

## 3. 编码规范

- **严格模式**：`noUnusedLocals` 和 `noUnusedParameters`
- **路径别名**：统一使用 `@/` 指向 `src/`，禁止跨层相对路径引用（`../../`）
- **组件文件**：PascalCase（`SkillCard.tsx`）
- **Props 类型**：`ComponentNameProps`（`SkillCardProps`）
- **CSS 类名**：kebab-case（`modal-backdrop`、`skill-card`）
- **弹窗条件渲染**：懒加载弹窗仅当需要时渲染（`{state ? <Modal /> : null}`），非懒加载弹窗 `if (!open) return null`
- **代码分割**：视图和弹窗组件使用 `React.lazy()` + `Suspense` 懒加载；**懒加载组件不得通过 barrel `index.ts` 静态导出**
- **用户可见文本**：必须使用 i18n（`t('key')`），禁止硬编码字符串
- **DTO 类型**：核心 DTO 放在 `@/features/skills/types`，API 关联 DTO 放在 `@/lib/api`
- **组件内部**：Props/state/函数名使用 camelCase
- **Service 层**：API 调用通过 `@/services` 中的 service 对象封装（如 `tagService.createTag()`），hooks 调用 service 而非直接 `api.post()`
- **Barrel 导出**：每个 feature 的 `index.ts` 只导出非懒加载的组件、hooks、类型
- **样式**：按功能模块拆分到 `src/styles/` 目录下，每个模块一个 CSS 文件，由 `src/styles/index.css` 统一 `@import` 聚合；禁止在组件目录下创建独立 CSS 文件。CSS 类名使用 kebab-case，前缀与模块名保持一致（如 `.skill-card` 对应 `skill-card.css`，`.settings-v2-*` 对应 `settings.css`）
- **样式原则**：① 同一选择器只定义一次（禁止后写覆盖模式）；② 新增组件样式添加到对应的模块 CSS 文件中；③ 公共/复用样式放入 `buttons.css` 等共享模块；④ 深色主题通过现有 CSS 变量系统自动适配，无需额外写 `[data-theme='dark']` 覆盖（除非确有必要）
- **主题**：通过 CSS 变量 + `[data-theme="dark"]` 实现，浅色模式为主要开发模式
- **版本号**：使用全局常量 `__APP_VERSION__`（Vite 构建时从 package.json 注入），禁止硬编码

## 4. 工程规则

1. 不保留向后兼容。过时的实现直接删除；不增加兼容层、不写迁移代码、不保留 fallback
2. 选择能满足当前需求的最简单实现。不要预防性抽象，不要多余的配置层
3. 组件保持模块化，关注点分离
4. 优先使用成熟且持续维护的库；没有明确理由，不自行重写
5. 先检查项目已有依赖能够提供什么能力，再考虑新增包或自行实现
6. 改动涉及异步加载、路由切换或 SSE 时，明确处理 loading、error、取消和过期响应，避免旧请求覆盖新状态
7. 新增依赖前，先查看根 `package.json` 和工作区包；依赖安装、全量构建、自动修复格式化及提交均需开发者明确指令

## 5. 任务路由

涉及前端代码的任务，按任务类型读取对应专题文档：

| 任务类型 | 必读文档 |
|---------|---------|
| 新增/修改组件或页面 | [docs/COMPONENT_STANDARD.md](docs/COMPONENT_STANDARD.md) |
| 新增/修改样式或主题 | [docs/THEME_STANDARD.md](docs/THEME_STANDARD.md) |
| 新增/修改 API 调用或 DTO | [docs/API_STANDARD.md](docs/API_STANDARD.md) |
| 理解前端整体结构 | [docs/PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md) |
| 新增/修改国际化文本 | [docs/COMPONENT_STANDARD.md](docs/COMPONENT_STANDARD.md) § i18n 规范 |
| 修改 frontend/docs/ 下的文档 | [docs/AGENTS.md](docs/AGENTS.md) |

## 6. 命名规范速查

| 层级 | 命名风格 | 示例 |
|------|---------|------|
| JSON 传输 / DTO 字段 | `snake_case` | `skill_id`, `source_type`, `created_at` |
| 组件 Props 字段 | `camelCase` | `sortBy`, `searchQuery`, `loadingStartAt` |
| useState 变量 | `camelCase` | `pendingDeleteId`, `actionMessage` |
| 函数名 | `camelCase` | `handleDeleteManaged`, `getSkillScope` |
| Service 对象 | `camelCase` | `tagService`, `skillService` |
| Service 方法 | `camelCase` | `createTag`, `deleteManagedSkill` |
| 组件文件名 | `PascalCase` | `SkillCard.tsx`, `FilterBar.tsx` |
| Props 类型 | `ComponentNameProps` | `SkillCardProps`, `FilterBarProps` |
| CSS 类名 | `kebab-case` | `modal-backdrop`, `skill-card` |

## 7. 常用命令

```bash
cd frontend && npm run dev    # 启动 Vite 开发服务器（端口 5173）
cd frontend && npm run check  # lint + build（提交前必跑）
cd frontend && npm run lint   # 仅 ESLint
cd frontend && npm run build  # tsc + vite 构建
```

## 8. 文档权威顺序

1. 用户明确指令
2. 根 `AGENTS.md` + 本文件硬约束
3. `frontend/docs/` 下的专题规范
4. 当前代码实际行为
5. 其他说明性文档
