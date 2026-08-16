# Skills Hub - 项目规范

> 中文 | [English](docs/AGENTS.en.md)

本文件是项目的导航入口（Agent 的第一张地图）。只保留每次任务都适用的全局规则；详细规范按任务类型从下方「任务路由」逐级加载。

> 面向人的项目介绍和开发指南见 [README.md](README.md)。

## 概述

Skills Hub 是一款跨平台桌面应用（React 19 + Python FastAPI），用于管理 AI Agent Skills 并将其同步到 44 款 AI 编程工具。核心理念："一次安装，处处同步。"

## 技术栈

- **前端**：React 19 + TypeScript 5.9（严格模式）+ Vite 7 + Tailwind CSS 4
- **后端**：Python 3.10+ + FastAPI + SQLite
- **国际化**：i18next（中英双语）
- **通知**：sonner（Toast 提示）
- **图标**：lucide-react

## 架构总览

### 目录结构（高层）

```
skills-hub/
├── frontend/          # React 前端 → 详见 frontend/AGENTS.md
├── backend/           # Python FastAPI 后端 → 详见 backend/AGENTS.md
├── docs/              # 跨前后端共享文档（数据库 schema 等）
├── scripts/           # 构建与版本脚本
├── .github/workflows/ # CI/CD 工作流（CI 检查 + Release 构建）
├── AGENTS.md          # 本文件（项目导航入口）
├── CLAUDE.md          # Claude Code 入口（指向本文件）
└── README.md          # 项目介绍
```

### 前后端通信

- 前端通过 HTTP（`fetch`，Vite 代理转发）调用 Python 后端
- API 适配器：`frontend/src/lib/api.ts`
- 调用模式：`apiCall('command_name', { param })` → POST 到 `/api/{command}`

### 错误处理

- 后端使用 `ErrorCode` 枚举（`backend/core/error_codes.py`）定义结构化错误码
- 统一返回 `ErrorResponse`（`{ ok, code, message, detail }`）格式
- 前端通过 try-catch 捕获，使用 sonner toast 展示

## 全局命名规范（跨端强制）

**强制遵守** [docs/naming-conventions.md](docs/naming-conventions.md)，核心原则：

- **跨端通信字段统一 `snake_case`**：前端 DTO 类型、API 调用参数、JSON 传输、后端 Pydantic 字段，全部使用 `snake_case`
- **前端内部状态使用 `camelCase`**：组件 Props 字段、useState 变量、函数名使用 `camelCase`
- **禁止** `toSnakeCase()` 转换、`Field(alias="camelCaseName")`、前后端字段名不一致

## 版本管理

- 单一版本源通过 `scripts/version.mjs` 管理，一键同步前后端
- 前端版本：`frontend/package.json`（Vite 构建时注入 `__APP_VERSION__`）
- 后端版本：`backend/core/version.py`（`__version__`）
- 发版流程见 [README.md](README.md#发布版本)

## 任务路由

收到任务后，先判断任务类型，然后读取对应的模块入口文件；模块入口会指引到更细粒度的专题文档。

| 涉及范围 | 必读入口 |
|---------|---------|
| 前端代码（组件/样式/API 调用/DTO） | [frontend/AGENTS.md](frontend/AGENTS.md) |
| 后端代码（API/数据库/业务逻辑/测试） | [backend/AGENTS.md](backend/AGENTS.md) |
| 数据库表结构（字段详情） | [docs/database-schema.md](docs/database-schema.md) |
| 仅文档修改 | 直接修改对应文档 |

## 开发流程

1. 启动后端：`cd backend && python main.py`
2. 启动前端：`cd frontend && npm run dev`
3. 前端：http://localhost:5173，后端：http://localhost:18921
4. 前端提交前检查：`cd frontend && npm run check`
5. 后端测试：`cd backend && python -m pytest`

## Git 工作流规则

1. **本地自动提交**：每当有文件改动，立即在本地执行 `git add` + `git commit`，确保每次变更都有记录。
2. **禁止推送远程**：所有提交仅保留在本地仓库，除非用户明确要求，否则绝不执行 `git push`。
3. **提交规范**：提交消息格式为 `类型: 简要描述`（中文描述），单次提交只包含一个功能/一个修复。
4. **默认仓库与分支**：默认远程仓库为 `origin`（`https://github.com/lucan6290/skills-hub.git`），默认分支为 `main`。
