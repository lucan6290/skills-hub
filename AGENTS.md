# Skills Hub - 项目规范

> 中文 | [English](docs/AGENTS.en.md)

本文件是项目的导航入口（Agent 的第一张地图）。只保留每次任务都适用的全局规则；详细规范按任务类型从下方「任务路由」逐级加载。

> 面向人的项目介绍和开发指南见 [README.md](README.md)。

## 概述

Skills Hub 是一款跨平台桌面应用（React 19 + Rust Tauri），用于管理 AI Agent Skills 并将其同步到 44 款 AI 编程工具。核心理念："一次安装，处处同步。"

## 技术栈

- **前端**：React 19 + TypeScript 5.9（严格模式）+ Vite 7 + Tailwind CSS 4
- **后端**：Rust + Tauri 2 + SQLite（rusqlite）
- **国际化**：i18next（中英双语）
- **通知**：sonner（Toast 提示）
- **图标**：lucide-react

## 架构总览

### 目录结构（高层）

```
skills-hub/
├── frontend/          # React 前端 → 详见 frontend/AGENTS.md
│   └── src-tauri/     # Rust Tauri 后端
├── docs/              # 共享文档（数据库 schema 等）
├── scripts/           # 构建与版本脚本
├── .github/workflows/ # CI/CD 工作流（CI 检查 + Release 构建）
├── AGENTS.md          # 本文件（项目导航入口）
├── CLAUDE.md          # Claude Code 入口（指向本文件）
└── README.md          # 项目介绍
```

### 前后端通信

- 前端通过 Tauri `invoke` 调用 Rust 后端命令
- API 适配器：`frontend/src/lib/api.ts`（`invokeCommand` 封装）
- 调用模式：`invokeCommand('command_name', { param })` → Rust `#[tauri::command]`

### 错误处理

- Rust 后端使用 `AppError` 枚举定义结构化错误
- 前端通过 try-catch 捕获，使用 sonner toast 展示

## 全局命名规范（跨端强制）

**强制遵守** [docs/naming-conventions.md](docs/naming-conventions.md)，核心原则：

- **跨端通信字段统一 `snake_case`**：前端 DTO 类型、Tauri command 参数、JSON 传输、Rust struct 字段，全部使用 `snake_case`
- **前端内部状态使用 `camelCase`**：组件 Props 字段、useState 变量、函数名使用 `camelCase`
- **禁止** `toSnakeCase()` 转换、前后端字段名不一致

## 版本管理

- 单一版本源通过 `scripts/version.mjs` 管理，一键同步前后端
- 前端版本：`frontend/package.json`（Vite 构建时注入 `__APP_VERSION__`）
- 后端版本：`frontend/src-tauri/Cargo.toml`（`version` 字段）
- 发版流程见 [README.md](README.md#发布版本)

## 任务路由

收到任务后，先判断任务类型，然后读取对应的模块入口文件；模块入口会指引到更细粒度的专题文档。

| 涉及范围 | 必读入口 |
|---------|---------|
| 前端代码（组件/样式/API 调用/DTO） | [frontend/AGENTS.md](frontend/AGENTS.md) |
| Rust 后端代码（commands/repositories/services） | [frontend/src-tauri/AGENTS.md](frontend/src-tauri/AGENTS.md) |
| 数据库表结构（字段详情） | [docs/database-schema.md](docs/database-schema.md) |
| 仅文档修改 | 直接修改对应文档 |

## 开发流程

1. 启动开发：`cd frontend && npm run tauri dev`
2. 前端：http://localhost:5173（Vite HMR），后端：Tauri 自动管理
3. 前端提交前检查：`cd frontend && npm run check`
4. Rust 测试：`cd frontend/src-tauri && cargo test`

## Git 工作流规则

1. **本地自动提交**：每当有文件改动，立即在本地执行 `git add` + `git commit`，确保每次变更都有记录。
2. **禁止推送远程**：所有提交仅保留在本地仓库，除非用户明确要求，否则绝不执行 `git push`。
3. **提交规范**：提交消息格式为 `类型: 简要描述`（中文描述），单次提交只包含一个功能/一个修复。
4. **默认仓库与分支**：默认远程仓库为 `origin`（`https://github.com/lucan6290/skills-hub.git`），默认分支为 `main`。

## 网络代理规范

### 适用场景

当项目执行过程中遇到以下网络相关异常时，**必须立即启用本地代理服务**，所有 Agent 在执行任务时严格遵守：

- 网络连接超时或无法连接（如 `Failed to connect to ... port 443`）
- API 访问受限（GitHub API、npm registry、crates.io 等）
- 资源获取失败（依赖安装、更新检查、远程仓库推送等）
- 任何涉及外部网络请求的操作失败

### 代理配置

- **代理地址**：`127.0.0.1`
- **代理端口**：`7890`
- **代理协议**：HTTP/HTTPS（兼容 SOCKS5）

### 配置步骤

1. **Git 代理**（用于 `git push`、`git fetch` 等远程操作）：

   ```bash
   git config --global http.proxy http://127.0.0.1:7890
   git config --global https.proxy http://127.0.0.1:7890
   ```

2. **环境变量代理**（用于 cargo、npm、curl 等命令行工具）：

   ```bash
   # PowerShell（当前会话）
   $env:HTTP_PROXY = "http://127.0.0.1:7890"
   $env:HTTPS_PROXY = "http://127.0.0.1:7890"
   ```

3. **npm 代理**（用于 `npm install`、`npx` 等）：

   ```bash
   npm config set proxy http://127.0.0.1:7890
   npm config set https-proxy http://127.0.0.1:7890
   ```

4. **cargo 代理**（用于 `cargo build`、`cargo update` 等）：

   在 `~/.cargo/config.toml` 中添加：

   ```toml
   [http]
   proxy = "http://127.0.0.1:7890"
   ```

### 验证方法

配置完成后，执行以下命令验证代理是否生效：

```bash
# 验证 Git 代理
git config --global --get http.proxy

# 验证环境变量
echo $env:HTTPS_PROXY

# 验证网络连通性
curl -x http://127.0.0.1:7890 -s -o NUL -w "%{http_code}" https://github.com
# 返回 200 或 301 表示代理可用
```

### 异常处理流程

1. **首次失败**：遇到网络异常时，首先按上述步骤配置代理，然后重试操作
2. **代理已配置但仍失败**：确认本地代理服务（如 Clash、v2ray 等）正在运行且监听 7890 端口
3. **代理服务未运行**：提示用户启动代理服务后再重试
4. **代理服务无法启动**：停止操作，向用户报告网络问题，等待用户解决后继续
5. **所有尝试均失败**：记录错误详情，向用户报告，不得绕过代理直接进行无代理的网络操作
