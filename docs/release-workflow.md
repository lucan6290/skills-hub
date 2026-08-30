# Release Workflow — Agent 发布工作流

> 本文件定义 Agent 执行 Skills Hub 版本发布的完整标准流程。Agent 收到发布指令后，必须严格按本文件顺序执行，不得跳步。
>
> **路径规则**：所有命令块在执行前确保当前目录为项目根目录（`e:\A-Code\skills-hub`）。使用 `pushd`/`popd` 切换子目录，避免 `cd` 链式依赖。
>
> **PowerShell 注意事项**：
> - 反引号 `` ` `` 续行符后面**不能有空格**，必须紧跟换行符

## 概述

发布流程分为 **6 个阶段**，每个阶段有明确的检查点和失败处理：

```
阶段 1: 前置检查 → 阶段 2: 确定版本号 → 阶段 3: 质量门禁
→ 阶段 4: 生成 CHANGELOG → 阶段 5: 更新版本号 → 阶段 6: 提交/打 tag/推送
```

> 推送后 CI 构建状态由用户在 GitHub 网页端自行监控，Agent 不通过 API 轮询（避免未认证 API 限流）。

---

## 发布前置：签名密钥（一次性初始化）

> 应用内"一键更新"依赖 Tauri updater 的签名机制，需要一对 minisign 密钥。这是**发布基础设施**，只需初始化一次，后续每次发版自动复用。**若从未初始化，应先完成本章节**。

### 为什么需要签名密钥

- 开发者用**私钥**在 CI 构建时给安装包和 `latest.json` 清单签名
- 应用内置**公钥**（`tauri.conf.json` 的 `plugins.updater.pubkey`），用户端下载更新包时用它验证签名
- 没有有效签名，用户点"更新"会报错（`Could not fetch a valid release JSON`）

### 一次性初始化步骤

**第 1 步：生成密钥对**

```powershell
# 生成 32 位随机密码（妥善保存，丢失后无法再签名更新包）
$KEY_PASSWORD = -join ((65..90) + (97..122) + (48..57) | Get-Random -Count 32 | ForEach-Object {[char]$_})
Write-Host "密钥密码（务必保存）：$KEY_PASSWORD"

# 生成密钥对（--ci 跳过交互提示）
pushd frontend
$env:CI = "true"
npx tauri signer generate -w "$env:USERPROFILE\.tauri\skills-hub.key" -p $KEY_PASSWORD
popd
```

产物：
- 私钥：`~/.tauri/skills-hub.key`（**保密，绝不提交到仓库**）
- 公钥：`~/.tauri/skills-hub.key.pub`

**第 2 步：把公钥写入 `tauri.conf.json`**

将 `plugins.updater.pubkey` 更新为 `skills-hub.key.pub` 文件的完整内容。

**第 3 步：配置 GitHub Secrets**

在 `https://github.com/lucan6290/skills-hub/settings/secrets/actions` 添加两个 Secret：

| Name | Value |
|------|-------|
| `TAURI_SIGNING_PRIVATE_KEY` | `skills-hub.key` 文件**完整内容**（含首尾注释行） |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 第 1 步生成的密码 |

**第 4 步：确认 `.gitignore` 已兜底**

确认 `.gitignore` 包含 `*.key` 和 `*.key.pub` 规则，防止私钥被误提交到开源仓库。

### fork 开发者注意事项

其他开发者 fork 本仓库后，`tauri.conf.json` 里是维护者的 pubkey，但他们没有对应的私钥。要启用自己仓库的自动更新，必须：
1. 用第 1 步生成**自己的**密钥对
2. 替换 `tauri.conf.json` 的 pubkey 为自己的公钥
3. 在自己的仓库 Secrets 里配置自己的私钥和密码

否则他们构建的应用自动更新会失败。

---

## 阶段 1：前置检查

在开始任何发布操作前，必须逐一通过以下检查。任一项失败则停止流程并报告。

### 1.1 确认工作目录

```powershell
# 必须在项目根目录
Get-Location  # 应输出包含 skills-hub 的路径
```

### 1.2 检查 git 状态

```powershell
git status --porcelain
git branch --show-current
git remote -v
```

检查项：
- 当前分支必须是 `main`
- 工作区必须干净（`git status --porcelain` 无输出）。若有未提交改动，询问用户是否先提交或 stash，**不得擅自处理用户的未提交改动**
- 远程仓库 `origin` 指向 `https://github.com/lucan6290/skills-hub.git`

### 1.3 检查网络/代理

```powershell
# 测试 GitHub 连通性（必须用 curl.exe，不能用 curl）
curl.exe -s -o NUL -w "%{http_code}" https://github.com
```

- 返回 200/301/302 → 正常，继续
- 返回 000 或超时 → 按 `AGENTS.md` 中的网络代理规范设置 `$env:HTTP_PROXY`/`$env:HTTPS_PROXY` 后重试

### 1.4 同步远程最新代码

```powershell
git pull origin main
```

### 1.5 获取最新 tags

```powershell
git fetch --tags
```

---

## 阶段 2：确定版本号

### 2.1 确认当前版本

```powershell
node scripts/version.mjs check
```

记录当前版本号，记为 `CURRENT_VERSION`（如 `0.2.0`）。

### 2.2 确认上一个 release tag

```powershell
git describe --tags --abbrev=0
```

记录输出，记为 `LAST_TAG`（如 `v0.2.0`）。

### 2.3 分析自上版本以来的变更规模

```powershell
git log "$LAST_TAG..HEAD" --oneline
```

统计：
- `feat:` / `功能:` 前缀的 commit 数量 → 如有，至少需要 MINOR 版本
- 包含 `BREAKING CHANGE` 或不兼容变更的 commit → 需要 MAJOR 版本
- 只有 `fix:` / `修正:` 前缀 → PATCH 版本

### 2.4 与用户确认新版本号

使用 `AskUserQuestion` 向用户确认版本号。根据 2.3 的分析提供推荐选项，**不得擅自决定版本号**。

确认后记为 `NEW_VERSION`（如 `0.3.0`），tag 名为 `TAG_NAME = "v$NEW_VERSION"`（如 `v0.3.0`）。

### 2.5 检查 tag 是否已存在

```powershell
# 检查本地 tag
git tag -l "$TAG_NAME"
# 检查远程 tag
git ls-remote --tags origin "$TAG_NAME"
```

如果本地或远程已存在同名 tag：
- 向用户报告，询问是否为重新发布
- 如果是重新发布（CI 上次失败后修复重发），按阶段 6.5 中的"CI 失败后重发"流程先删除旧 tag
- 如果不是重新发布，停止流程让用户决定版本号

---

## 阶段 3：质量门禁（完整构建验证）

**必须全部通过**才能继续发布。任一项失败需报告错误并等待用户指示。

> 所有命令从项目根目录出发，使用 `pushd`/`popd` 管理目录。

### 3.1 安装/确认前端依赖

```powershell
pushd frontend
npm ci
popd
```

- 如果 `npm ci` 失败（如 package-lock.json 与 package.json 不同步），执行 `npm install` 更新 lockfile，并告知用户 lockfile 有变更需纳入提交
- 否则继续

### 3.2 前端检查

```powershell
pushd frontend
npm run check
popd
```

（`npm run check` = `npm run lint` + `npm run build`）

- 如果 lint 有可自动修复的错误，可执行 `pushd frontend; npm run lint -- --fix; popd` 后重新 check
- 如果有 TypeScript 类型错误，必须报告用户，不得继续
- 此步骤会在 `frontend/dist/` 生成前端构建产物，Rust 编译需要引用

### 3.3 Rust 测试

```powershell
pushd frontend/src-tauri
cargo test
popd
```

- 所有测试必须通过
- 如果测试失败，报告失败详情，等待用户修复

### 3.4 Rust release 编译检查

```powershell
pushd frontend/src-tauri
cargo build --release
popd
```

- release 模式必须能成功编译
- 编译失败则报告错误并停止
- 此步骤只编译 Rust 代码，不执行 Tauri bundle（不需要 NSIS 工具链），因为 3.2 已生成 `dist/` 目录，`beforeBuildCommand` 不会重复执行

> 注意：`cargo build --release` 编译时间较长（5-15 分钟），属于正常现象。完整的 Tauri bundle 构建（NSIS/MSI 安装包）在 CI 环境执行，本地不需要。

---

## 阶段 4：生成 CHANGELOG

### 4.1 提取自上一个 tag 以来的 commits

```powershell
git log "${LAST_TAG}..HEAD" --pretty=format:"%h %s"
```

### 4.2 分类 commits

按以下规则将 commits 分类到 CHANGELOG 的四个部分：

| CHANGELOG 分类 | commit 前缀（不区分大小写） | 说明 |
|---|---|---|
| **Added** | `feat:`、`功能:`、`add:` | 新功能、新特性 |
| **Changed** | `refactor:`、`重构:`、`style:`、`样式:`、`perf:`、`优化:`、`change:` | 重构、样式改进、性能优化 |
| **Fixed** | `fix:`、`修正:`、`bug:`、`修复:` | Bug 修复 |
| **Technical** | `build:`、`ci:`、`docs:`、`test:`、`deps:`、`技术:`、`chore:` | 构建系统、CI、文档、测试、依赖、杂项 |

过滤规则：
- 提交信息以 `release:` 开头的 commit 跳过不纳入（它们是版本提交本身）
- 合并语义相同的 commits（如多个 fix 同一个问题的 commit 合并为一条）
- 如果某个 commit 的描述不够清晰（如 `fix: 修复问题`），使用 `git show <hash>` 查看 diff 补充上下文

### 4.3 生成 CHANGELOG 条目

在 `CHANGELOG.md` 的 `## [Unreleased]` 行**下方**插入新版本条目。先获取当前日期：

```powershell
$TODAY = [System.TimeZoneInfo]::ConvertTimeBySystemTimeZoneId((Get-Date), "China Standard Time").ToString("yyyy-MM-dd")
```

条目格式：

```markdown
## [NEW_VERSION] - TODAY_DATE

### Added
- 中文描述（基于 commit message 润色，去除前缀）

### Changed
- 中文描述

### Fixed
- 中文描述

### Technical
- 中文描述
```

写作规则：
- 如果某分类没有 commits，该分类标题省略不写
- 将 commit message 中的前缀（`feat:`, `fix:`, `功能:` 等）去除
- 使用简洁的中文，面向用户而非开发者
- **不得包含内部实现细节**（如具体文件名、变量名、函数名），应描述用户可感知的变化
- 每条以 `- ` 开头，结尾不加标点

### 4.4 插入到 CHANGELOG.md

将生成的内容插入到 CHANGELOG.md 中 `## [Unreleased]` 行的下一行，即在 `## [Unreleased]` 和上一个版本条目之间。

插入后 CHANGELOG.md 的结构应为：

```markdown
# Changelog

...

## [Unreleased]

## [NEW_VERSION] - YYYY-MM-DD

### Added
...

## [OLD_VERSION] - YYYY-MM-DD
...
```

### 4.5 展示给用户审阅

生成 CHANGELOG 后，**必须展示完整内容给用户审阅**，等待用户确认或修改。用户确认后才继续。

---

## 阶段 5：更新版本号

### 5.1 执行版本更新脚本

```powershell
node scripts/version.mjs set $NEW_VERSION
```

预期输出应包含三行变更：
- `frontend/package.json: OLD_VERSION -> NEW_VERSION`
- `Cargo.toml: OLD_VERSION -> NEW_VERSION`
- `frontend/package-lock.json: (old) -> NEW_VERSION`

### 5.2 校验版本一致性

```powershell
node scripts/version.mjs check
```

必须输出 `Version OK (NEW_VERSION) — frontend, Rust backend & lockfile in sync`。

### 5.3 验证 CHANGELOG 可被 CI 正确提取

```powershell
node scripts/extract-changelog.mjs "v$NEW_VERSION"
```

- 必须输出以 `## vNEW_VERSION` 开头的内容（即刚才写的 CHANGELOG 条目）
- 如果退出码非 0 或输出为空，说明 CHANGELOG 格式不正确（如标题不匹配），修正后重新验证
- 这一步模拟 CI 的 release notes 提取逻辑，确保不会 fallback 到默认文案

---

## 阶段 6：提交、打 Tag、推送

### 6.1 检查变更文件

```powershell
git status
git diff --stat
```

确认变更文件属于以下范围：
- `CHANGELOG.md`（必须）
- `frontend/package.json`（必须）
- `frontend/package-lock.json`（必须）
- `frontend/src-tauri/Cargo.toml`（必须）
- `scripts/version.mjs`（如有版本脚本修复）
- `scripts/extract-changelog.mjs`（如有提取逻辑修复）
- `docs/` 下的发布流程文档（如 `docs/release-workflow.md`，本次发布新增/修改时才会出现）

如果有其他文件变更（如 3.1 中 `npm install` 更新了 lockfile 中其他字段、或 `eslint --fix` 修改了源文件、或 Tauri 自动生成的 `gen/schemas/` 文件），向用户报告并确认是否纳入。Tauri 自动生成的 schema 文件（`frontend/src-tauri/gen/schemas/*.json`）如果只是行尾格式变化（`git diff` 无实质内容差异），用 `git restore` 恢复后不纳入提交。

### 6.2 提交

```powershell
git add CHANGELOG.md frontend/package.json frontend/package-lock.json frontend/src-tauri/Cargo.toml scripts/version.mjs scripts/extract-changelog.mjs docs/
git commit -m "release: v${NEW_VERSION}"
```

提交消息格式：`release: vNEW_VERSION`（如 `release: v0.3.0`）。

### 6.3 打 annotated tag

再次确认 tag 不存在（阶段 2.5 已检查，但此处二次确认防止并发操作）：

```powershell
if (git tag -l "$TAG_NAME") { throw "Tag $TAG_NAME already exists locally" }
git tag -a "$TAG_NAME" -m "v${NEW_VERSION} 版本发布"
```

验证 tag 指向正确的 commit：

```powershell
git log -1 --oneline "$TAG_NAME"
```

### 6.4 推送到远程

> **注意**：按 AGENTS.md 规则，`git push` 需要用户明确授权。在执行 push 前必须告知用户即将推送的内容（main 分支的 release commit + tag）并获得确认。

```powershell
git push origin main
git push origin "$TAG_NAME"
```

### 6.5 推送后指引

推送成功后，Agent 向用户报告以下信息，**不通过 API 轮询 CI 状态**（避免未认证请求 60 次/小时的限流问题）：

1. **CI 监控地址**：告知用户在浏览器打开以下页面查看构建进度：
   - Actions 页面：`https://github.com/lucan6290/skills-hub/actions`
   - Release CI 由 tag push 触发（`v*`），在 Windows runner 上构建，通常需要 10-25 分钟

2. **构建成功后操作**：
   - GitHub Releases 页面：`https://github.com/lucan6290/skills-hub/releases`
   - CI 创建的是 **Draft Release**，需要用户在 Releases 页面手动点 **"Publish release"** 才会正式发布
   - CI 构建产物包含：NSIS 安装包（`.exe`）和 MSI 安装包（`.msi`）

3. **签名密钥状态（需确认）**：应用内"一键更新"依赖签名密钥和 `latest.json` 清单。若已完成「发布前置：签名密钥」初始化，CI 会生成签名的 `latest.json` 并上传，自动更新可正常使用；若 GitHub Secrets（`TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`）未配置，CI 构建会跳过签名产物，用户点"更新"会失败（只能手动下载安装包升级）。

4. **CI 失败后重发**：如果构建失败，修复代码后执行以下命令重发 tag（**不要删除或 revert main 分支上的 release commit**，它包含正确的 CHANGELOG 和版本号）：
   ```powershell
   # 删除远程和本地旧 tag
   git push origin ":refs/tags/$TAG_NAME"
   git tag -d "$TAG_NAME"
   # 在最新 commit 上重新打 tag 推送
   git tag -a "$TAG_NAME" -m "v${NEW_VERSION} 版本发布"
   git push origin "$TAG_NAME"
   ```

---

## 失败回滚流程

### 阶段 6 之前（本地变更，未推送）

直接恢复本地修改的文件：

```powershell
git restore --worktree -- CHANGELOG.md frontend/package.json frontend/package-lock.json frontend/src-tauri/Cargo.toml scripts/version.mjs scripts/extract-changelog.mjs
# 新增的 docs 文件需要手动删除（git restore 不会删除未跟踪文件）
```

### 阶段 6 之后（已推送，发现严重问题需要撤回）

```powershell
# 1. 删除远程 tag（阻止更多用户收到更新）
git push origin ":refs/tags/$TAG_NAME"
# 2. 删除本地 tag
git tag -d "$TAG_NAME"
# 3. Revert release commit（在 main 分支上创建一个反向提交）
git revert HEAD --no-edit
git push origin main
```

回滚后必须通知用户，说明撤回原因。回滚后如果需要修复重发，在 revert 之后的新 commit 上重新执行阶段 4-6（注意 CHANGELOG.md 中需要重新添加新版本条目，因为 revert 会把之前添加的条目也撤销）。

---

## 参考文件

| 文件 | 用途 |
|------|------|
| `scripts/version.mjs` | 版本号同步脚本（package.json / Cargo.toml / package-lock.json） |
| `scripts/extract-changelog.mjs` | 从 CHANGELOG.md 提取指定版本的 release notes（CI 和阶段 5.3 使用） |
| `.github/workflows/release.yml` | Release CI 工作流（tag push 触发，Windows runner，产出 nsis .exe + msi .msi + latest.json） |
| `.github/workflows/ci.yml` | PR/主干 CI 检查（Ubuntu runner，lint + build） |
| `CHANGELOG.md` | 版本变更日志（Keep a Changelog 格式） |
| `frontend/package.json` | 前端版本号源（Vite `define.__APP_VERSION__` 注入） |
| `frontend/package-lock.json` | 依赖锁定文件（版本号必须与 package.json 同步） |
| `frontend/src-tauri/Cargo.toml` | Rust 后端版本号 |
| `frontend/src-tauri/tauri.conf.json` | Tauri 配置（version 指向 `../package.json`，`createUpdaterArtifacts` 开启签名产物，updater pubkey 已配置） |
| `.gitignore` | 忽略规则（含 `*.key` / `*.key.pub` 防止签名私钥泄露） |
