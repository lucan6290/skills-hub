# 08. 集成、回归与移除 Python 的验收清单

## 1. 目标

定义 Rust 重构完成的最低验收条件。未满足本文件条件时，禁止删除 Python 后端、修改正式发布入口或宣称迁移完成。

## 2. 集成前检查

```powershell
git status --short --branch
git diff --stat
git log --oneline --decorate -12
```

确认：

- 当前仍在 `main-Rust`；
- 无意外修改 `main-python`；
- 实施前已有修改仍然存在；
- 每个 Agent 的 commit 和测试结果齐全；
- generated inventory 与当前代码没有未解释差异。

## 3. 构建检查

```powershell
cd frontend
npm run check

cd src-tauri
cargo fmt --check
cargo check
cargo test
```

必须在 Windows 环境执行一次生产构建，并记录：

- 构建命令；
- 输出文件；
- 文件大小；
- 启动结果；
- 退出结果；
- 是否产生 Python 进程；
- 是否监听 `18921`。

## 4. 功能回归

### 桌面

- Tauri 窗口启动；
- 单实例；
- 窗口尺寸和最小尺寸；
- 图标；
- 关闭和重启；
- 安装版和 Portable 版数据目录。

### 数据库

- 旧 `skills_hub.db` 原位读取；
- skills、tags、targets、settings、scope preferences 保持；
- schema 初始化幂等；
- 数据库维护和 reset 有确认机制；
- 数据库错误可显示。

### Skill

- 列表；
- 创建/导入；
- 删除；
- source URL；
- 标签；
- 文件列出、读取和写入；
- 套件子技能。

### 同步

- global scope；
- project scope；
- symlink；
- junction；
- copy fallback；
- overwrite；
- same-content；
- unsync；
- retry copy；
- 非托管文件保护。

### 工具和仓库

- 工具检测；
- 44 款内置适配器；
- 自定义工具配置；
- 工具 skills cache；
- Community Repo；
- Custom Repo；
- Onboarding；
- 新工具检测。

### 后台任务和更新

- 扫描进度；
- 批量同步进度；
- 任务取消；
- 任务失败；
- 更新检查；
- setup/portable/naked 更新模式；
- 更新失败不破坏当前安装。

## 5. HTTP/Python 清理检查

Rust 版本功能验收通过后执行：

```powershell
git grep -n "localhost:18921"
git grep -n "from fastapi"
git grep -n "import uvicorn"
git grep -n "pywebview"
git grep -n "PyInstaller"
git grep -n "fetch('/api"
git grep -n "apiCall(" -- frontend/src
git grep -n "apiGet(" -- frontend/src
```

允许命中的内容只能是迁移文档、历史说明或明确的测试基线；生产运行代码不得命中。

## 6. 删除 Python 前的必要步骤

1. 生成并保存 Rust 版本回归结果。
2. 保存现有 Python 版本基线结果。
3. 对比关键数据读取结果。
4. 对比关键文件同步结果。
5. 在临时目录测试安装和卸载。
6. 在临时用户数据目录测试升级。
7. 确认无 Python 运行时依赖。
8. 由主 Agent 单独提交 Python 删除 commit。

允许删除的内容包括：

```text
backend/main.py
backend/desktop.py
backend/build.py
backend/requirements.txt
backend/api/**
backend/core/**
backend/models/**
```

删除范围必须以实际引用搜索结果为准，不能按目录盲删。

## 7. 最终验收结论格式

```text
Rust/Tauri 重构状态：通过 / 未通过

已通过：
- ...

未通过：
- ...

仍保留的 Python 引用：
- ...

已验证发布产物：
- ...

已知风险：
- ...

结论：允许 / 不允许删除 Python 运行时
```
