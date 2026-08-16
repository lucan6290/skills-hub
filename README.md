# Skills Hub

> 中文 | [English](docs/README.en.md)

一个跨平台桌面应用（React 19 + Python FastAPI），用于统一管理 AI Agent Skills，并把它们同步到多种 AI 编程工具的全局或项目级 skills 目录（优先 symlink/junction，失败回退 copy），实现 “Install once, sync everywhere”。

支持浏览器模式和独立桌面窗口模式，可打包为单文件 exe。

## 主要功能

- **Tags 标签页**：在独立页面中新建、重命名、删除自定义标签，并快速跳转到对应的 Skill 列表
- **标签筛选**：为 Skill 添加多个标签，并在 My Skills 中按标签筛选，包括查看 `无标签` Skill
- **全局 / 项目级同步**：Skill 可同步到全局目录，在所有项目中生效；也可限定到指定项目目录中生效
- **同步范围控制**：在全局和项目范围之间切换 Skill，管理项目目录，并按范围筛选 My Skills
- **技能详情页**：点击技能名称查看完整文件内容，支持 Markdown 渲染和代码语法高亮（40+ 语言）
- **统一视图**：查看 Hub 托管的 skills 总数、范围徽标及其在各工具的生效状态
- **Onboarding 迁移**：扫描已安装工具中的现有 skills，导入到 Community Repo 并同步
- **导入来源**：本地文件夹（支持多技能目录选择、`.claude/skills/` 目录）
- **新工具检测**：检测新安装的工具并提示同步托管的 skills

## 技术栈

- **前端**：React 19 + TypeScript 5.9（严格模式）+ Vite 7 + Tailwind CSS 4
- **后端**：Python 3.10+ + FastAPI + SQLite
- **HTTP 通信**：`fetch` → Python 后端（`localhost:18921`）
- **i18n**：i18next（中英双语）

## 开发

### 环境要求

- Node.js 18+（推荐 20+）
- Python 3.10+（含 pip）

### 浏览器模式

```bash
# 后端（终端 1）
cd backend
pip install -r requirements.txt
python main.py                 # FastAPI → http://localhost:18921

# 前端（终端 2）
cd frontend
npm install
npm run dev                    # Vite 开发服务器 → http://localhost:5173
```

### 桌面窗口模式

```bash
# 1. 构建前端
cd frontend
npm install
npm run build                  # 输出到 frontend/dist/

# 2. 启动桌面窗口（自动托管后端）
cd ../backend
pip install -r requirements.txt
python desktop.py              # pywebview 原生窗口，无需浏览器
```

> 桌面模式使用 `pywebview` 创建独立窗口，后端自动在后台启动，无需单独运行 `python main.py`。

### 打包为 exe

```bash
# 在 backend/ 目录下执行
python build.py                # 输出 SkillsHub.exe 到 dist/
```

> 打包前需先在 `frontend/` 执行 `npm run build`。`build.py` 会自动将前端静态文件打包进 exe。

### 质量检查（在 `frontend/` 下）

```bash
npm run lint            # ESLint
npm run build           # tsc + vite build
npm run check           # lint + build
```

### 后端测试

```bash
cd backend
python -m pytest        # 或：pytest
```

### 版本管理

项目版本号前后端统一管理，唯一需要手动维护的地方是通过脚本一键更新：

```bash
# 在项目根目录执行
node scripts/version.mjs check              # 校验前后端版本一致
node scripts/version.mjs set <x.y.z>        # 设置新版本号（同时更新前端 package.json 和后端 core/version.py）
```

版本号来源：
- 前端：`frontend/package.json` 中的 `version`（Vite 构建时自动注入到前端代码）
- 后端：`backend/core/version.py` 中的 `__version__`（FastAPI 和 health 接口使用）
- 两个文件通过 `scripts/version.mjs` 保证同步，请勿手动单独修改其中一个。

### 发布版本

**重要：必须先更新 CHANGELOG.md，再打 tag，否则 CI 无法自动提取 release notes。**

发布新版本：

```bash
# 1. 更新 CHANGELOG.md，在 "## [Unreleased]" 下方添加新版本条目
#    格式：## [0.x.x] - YYYY-MM-DD
#    分类：Added / Changed / Fixed / Technical

# 2. 更新版本号（同时更新 frontend/package.json 和 backend/core/version.py）
node scripts/version.mjs set 0.x.x

# 3. 校验前后端版本一致
node scripts/version.mjs check

# 4. 提交并打 tag
git add -A
git commit -m "chore: bump version to v0.x.x"
git tag -a v0.x.x -m "v0.x.x 版本发布"

# 5. 推送代码和 tag（推送 tag 会自动触发 GitHub Actions 构建 Release）
git push origin main
git push origin v0.x.x
```

如果 CI 构建失败需要重新推送同一个 tag（修复后）：

```bash
git checkout main
git pull origin main

# 确认当前提交已经包含 CI 修复；不要直接 rerun 旧的失败 workflow
git push origin main
git tag -d v0.x.x
git push origin :refs/tags/v0.x.x
git tag -a v0.x.x -m "v0.x.x 版本发布"
git push origin v0.x.x
```

推送 tag 后 GitHub Actions 会自动在 Windows runner 上构建 exe、ZIP、NSIS 安装包，并从 CHANGELOG.md 提取对应版本的内容作为 release notes，创建一个 draft Release。你可以在 GitHub Releases 页面确认内容后手动发布。

## 项目结构

```
skills-hub/
├── frontend/               # React 19 + Vite 前端
│   ├── src/
│   │   ├── lib/                    # api.ts、errors.ts、pickFolder.ts、utils.ts
│   │   ├── hooks/                  # 自定义 hooks（useApi、useSkills、useScopeState 等）
│   │   ├── context/                # React context（AppState、Modal）
│   │   ├── components/skills/     # 技能相关组件（Header、FilterBar、SkillCard、SkillsList 等）
│   │   └── i18n/                   # 中英文翻译
│   └── package.json
├── backend/                # Python FastAPI 后端
│   ├── main.py                     # FastAPI 入口（端口 18921）
│   ├── desktop.py                  # pywebview 桌面窗口入口
│   ├── build.py                    # PyInstaller 打包脚本
│   ├── api/                        # 路由处理器（skills/、tools/、tags、settings、onboarding）
│   ├── core/                       # 业务逻辑
│   │   ├── db/store.py             # SQLite ORM（12 张表）
│   │   ├── repo/                   # 双源头仓库（community、scanner、migration）
│   │   ├── skills/                 # 技能操作（sync_engine、installer、files、source_paths）
│   │   └── tools/                  # 工具适配器
│   └── models/                     # Pydantic DTO
├── docs/                   # 文档
├── scripts/                # 构建与版本脚本
└── README.md
```

完整的架构与编码规范见 [`AGENTS.md`](AGENTS.md)。

## 支持的 AI 编程工具

项目级 skills 目录相对于所选项目根目录。标记为 `N/A` 的工具没有已确认的项目级 skills 目录，仅支持全局同步。

| tool key | 显示名称 | 全局 skills 目录（相对 `~`） | 项目 skills 目录（相对项目） | 检测条件（相对 `~`） |
| --- | --- | --- | --- | --- |
| `cursor` | Cursor | `.cursor/skills` | `.agents/skills` | `.cursor` |
| `claude_code` | Claude Code | `.claude/skills` | `.claude/skills` | `.claude` |
| `codex` | Codex | `.codex/skills` | `.agents/skills` | `.codex` |
| `opencode` | OpenCode | `.config/opencode/skills` | `.agents/skills` | `.config/opencode` |
| `antigravity` | Antigravity | `.gemini/antigravity/skills` | `.agents/skills` | `.gemini/antigravity` |
| `amp` | Amp | `.config/agents/skills` | `.agents/skills` | `.config/agents` |
| `kimi_cli` | Kimi Code CLI | `.config/agents/skills` | `.agents/skills` | `.config/agents` |
| `augment` | Augment | `.augment/skills` | `.augment/skills` | `.augment` |
| `openclaw` | OpenClaw | `.openclaw/skills` | `skills` | `.openclaw` |
| `copaw` | Copaw | `.copaw/skill_pool` | `.copaw/skills` | `.copaw` |
| `cline` | Cline | `.agents/skills` | `.agents/skills` | `.agents` |
| `codebuddy` | CodeBuddy | `.codebuddy/skills` | `.codebuddy/skills` | `.codebuddy` |
| `command_code` | Command Code | `.commandcode/skills` | `.commandcode/skills` | `.commandcode` |
| `continue` | Continue | `.continue/skills` | `.continue/skills` | `.continue` |
| `crush` | Crush | `.config/crush/skills` | `.crush/skills` | `.config/crush` |
| `junie` | Junie | `.junie/skills` | `.junie/skills` | `.junie` |
| `iflow_cli` | iFlow CLI | `.iflow/skills` | `.iflow/skills` | `.iflow` |
| `kiro_cli` | Kiro CLI | `.kiro/skills` | `.kiro/skills` | `.kiro` |
| `kode` | Kode | `.kode/skills` | `.kode/skills` | `.kode` |
| `mcpjam` | MCPJam | `.mcpjam/skills` | `.mcpjam/skills` | `.mcpjam` |
| `mistral_vibe` | Mistral Vibe | `.vibe/skills` | `.vibe/skills` | `.vibe` |
| `mux` | Mux | `.mux/skills` | `.mux/skills` | `.mux` |
| `openclaude` | OpenClaude IDE | `.openclaude/skills` | `.openclaude/skills` | `.openclaude` |
| `openhands` | OpenHands | `.openhands/skills` | `.openhands/skills` | `.openhands` |
| `pi` | Pi | `.pi/agent/skills` | `.pi/skills` | `.pi` |
| `qoder` | Qoder | `.qoder/skills` | `.qoder/skills` | `.qoder` |
| `qoderwork` | QoderWork | `.qoderwork/skills` | `.qoderwork/skills` | `.qoderwork` |
| `qwen_code` | Qwen Code | `.qwen/skills` | `.qwen/skills` | `.qwen` |
| `trae` | Trae | `.trae/skills` | `.trae/skills` | `.trae` |
| `trae_cn` | Trae CN | `.trae-cn/skills` | `.trae/skills` | `.trae-cn` |
| `zencoder` | Zencoder | `.zencoder/skills` | `.zencoder/skills` | `.zencoder` |
| `neovate` | Neovate | `.neovate/skills` | `.neovate/skills` | `.neovate` |
| `pochi` | Pochi | `.pochi/skills` | `.pochi/skills` | `.pochi` |
| `adal` | AdaL | `.adal/skills` | `.adal/skills` | `.adal` |
| `kilo_code` | Kilo Code | `.kilocode/skills` | `.kilocode/skills` | `.kilocode` |
| `roo_code` | Roo Code | `.roo/skills` | `.roo/skills` | `.roo` |
| `goose` | Goose | `.config/goose/skills` | `.goose/skills` | `.config/goose` |
| `gemini_cli` | Gemini CLI | `.gemini/skills` | `.agents/skills` | `.gemini` |
| `github_copilot` | GitHub Copilot | `.copilot/skills` | `.agents/skills` | `.copilot` |
| `clawdbot` | Clawdbot | `.clawdbot/skills` | `.clawdbot/skills` | `.clawdbot` |
| `droid` | Droid | `.factory/skills` | `.factory/skills` | `.factory` |
| `windsurf` | Windsurf | `.codeium/windsurf/skills` | `.windsurf/skills` | `.codeium/windsurf` |
| `moltbot` | MoltBot | `.moltbot/skills` | `.moltbot/skills` | `.moltbot` |
| `hermes_agent` | Hermes Agent | `.hermes/skills` | N/A | `.hermes` |

## 贡献与安全

- 行为准则：[`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)
- 安全策略：[`SECURITY.md`](SECURITY.md)

## 常见问题

- **Skills 存储在哪里？** Community Repo 默认位于 `~/.skillshub`（可在设置中修改）。
- **标签有什么用？** 标签帮助你查找和组织 skills，不会改变 skill 同步的位置或哪些工具可以使用它。
- **什么是项目级同步？** Skill 仍然只在 Community Repo 中存储一次，但其同步目标是某个选定的项目目录，例如 `<project>/.agents/skills`、`<project>/.claude/skills` 或其他工具特定的项目 skills 路径。
- **为什么同步到 Cursor 总是使用 copy？** Cursor 目前不支持基于 symlink/junction 的 skills 目录，因此 Skills Hub 在同步到 Cursor 时强制使用目录复制。
- **为什么同步有时会回退到 copy？** Skills Hub 优先使用 symlink/junction，但在某些系统上（尤其是 Windows）symlink 可能受限，此时回退为目录复制。
- **`TARGET_EXISTS|...` 是什么意思？** 目标文件夹已存在且操作未覆盖它（默认非破坏性）。请删除已有文件夹或使用相应的覆盖流程重试。

## 开源协议

MIT License — 详见 [`LICENSE`](LICENSE)。
