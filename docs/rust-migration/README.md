# Rust/Tauri 重构实施文档索引

> 适用分支：`main-Rust`
>
> 目标：将 Skills Hub 从 `Python + FastAPI + pywebview + PyInstaller` 迁移为 Windows 优先的 `Tauri 2 + Rust + React + TypeScript 5.x` 桌面应用。
>
> 本目录是实施依据，不是当前代码规范的替代品。实施前仍必须遵守根目录、`frontend/` 和 `backend/` 下的 `AGENTS.md`。

## 文档状态说明

文档中的标记含义：

- **已核实**：已从当前仓库代码、配置或已有测试中确认。
- **实施要求**：迁移时必须实现的目标行为。
- **待验证**：当前仓库没有足够证据，实施 Agent 必须先通过代码检查、工具链检查或测试确认，不能直接猜测。
- **禁止**：明确不能执行的操作。

## 阅读顺序

1. [00-baseline-and-contract.md](./00-baseline-and-contract.md)：迁移基线、事实清单、command/DTO 契约。
2. [01-tauri-shell-and-build.md](./01-tauri-shell-and-build.md)：Tauri 工程、窗口、权限和构建。
3. [02-database-and-config.md](./02-database-and-config.md)：SQLite、数据目录和配置兼容。
4. [03-filesystem-and-sync.md](./03-filesystem-and-sync.md)：路径安全、跨平台文件操作和同步引擎。
5. [04-domain-services.md](./04-domain-services.md)：工具适配、仓库、安装、Onboarding、任务和更新。
6. [05-frontend-invoke.md](./05-frontend-invoke.md)：前端通信、Service 层和 Tauri invoke。
7. [06-release-and-ci.md](./06-release-and-ci.md)：Windows exe、Portable ZIP、NSIS 和 CI。
8. [07-multi-agent-execution.md](./07-multi-agent-execution.md)：多 Agent 并行边界、依赖、交接和集成。
9. [08-integration-acceptance.md](./08-integration-acceptance.md)：最终集成、回归测试和移除 Python 的条件。
10. [09-implementation-checklist.md](./09-implementation-checklist.md)：按 Phase 执行的实施检查表和 Agent 交付模板。
11. [10-decisions-risks-and-rollback.md](./10-decisions-risks-and-rollback.md)：架构决策、未验证风险和回滚方案。

## 已生成的源码事实清单

以下文件由当前仓库源码静态提取生成；源码或路由变化后必须重新生成并审阅：

- [generated/endpoint-inventory.md](./generated/endpoint-inventory.md)：endpoint 装饰器、函数、参数和响应声明。
- [generated/dto-inventory.md](./generated/dto-inventory.md)：Pydantic DTO 字段、类型和默认表达式。
- [generated/command-map.md](./generated/command-map.md)：候选 Tauri command 映射，冻结前仍需人工确认。
- [generated/database-facts.md](./generated/database-facts.md)：SQLite 表、索引和兼容迁移事实。
- [generated/frontend-call-sites.md](./generated/frontend-call-sites.md)：前端实际 `apiCall/apiGet` 调用位置。

## 当前已核实的迁移事实

| 事实 | 当前证据 |
|---|---|
| 分支目前没有 Rust/Tauri 工程 | 仓库不存在 `Cargo.toml`、`frontend/src-tauri/` |
| 前端通过 HTTP 调用后端 | `frontend/src/lib/api.ts` 中的 `apiCall`、`apiGet` |
| 桌面入口启动 FastAPI 和 pywebview | `backend/desktop.py` 的 `run_api`、`webview.create_window` |
| Python API 入口 | `backend/main.py` |
| SQLite 数据访问 | `backend/core/db/store.py` |
| 数据目录与默认工具配置 | `backend/core/config.py` |
| 工具适配器 | `backend/core/tools/adapters.py` |
| 同步核心 | `backend/core/skills/sync_engine.py`、`sync_service.py` |
| 当前 Windows 打包 | `backend/build.py`、`.github/workflows/release.yml` |
| 当前前端检查 | `frontend/package.json` 的 `npm run check` |
| 当前后端测试 | `backend/tests/`、`.github/workflows/ci.yml` |

## 全局实施约束

- 不修改 `main-python` 分支。
- 不覆盖实施开始前已存在的工作区修改。
- 不执行 `git push`。
- 不在未验证旧数据和同步行为前删除 Python 实现。
- 不把未确认的 crate 版本、Windows API 行为或数据库字段写成已实现事实。
- 不引入与当前任务无关的 UI、样式、数据库重构或功能重构。
- 所有跨 Tauri 边界的字段使用 `snake_case`。
- 所有用户输入路径都必须经过统一路径安全校验。
