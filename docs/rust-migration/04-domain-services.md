# 04. 领域服务、工具适配、仓库和后台任务

## 1. 目标

将 Python `core/` 中除数据库、底层文件系统和同步引擎外的业务编排迁移到 Rust service 层，形成清晰的：

```text
Tauri command
    → service
        → repository / filesystem / tool adapter
```

本工作包不负责 command 注册和前端调用。

## 2. 当前代码证据

主要来源：

- `backend/core/config.py`
- `backend/core/tools/adapters.py`
- `backend/core/tools/skill_cache.py`
- `backend/core/repo/community.py`
- `backend/core/repo/community_migration.py`
- `backend/core/repo/scanner.py`
- `backend/core/skills/installer.py`
- `backend/core/skills/install_service.py`
- `backend/core/skills/onboarding.py`
- `backend/core/skills/maintenance.py`
- `backend/core/tasks/manager.py`
- `backend/core/update/checker.py`
- `backend/core/update/updater.py`

## 3. 允许修改范围

```text
frontend/src-tauri/src/services/**
frontend/src-tauri/src/tools/**
frontend/src-tauri/src/repo/**
frontend/src-tauri/src/skills/install.rs
frontend/src-tauri/src/skills/onboarding.rs
frontend/src-tauri/src/skills/maintenance.rs
frontend/src-tauri/src/tasks/**
frontend/src-tauri/src/update/**
```

## 4. 实施步骤

### 4.1 工具适配器

从当前代码实际提取每个内置工具的：

- `tool_key`
- `display_name`
- `skills_dir`
- `detect_dir`
- `project_skills_dir`
- `supports_symlink`
- `supports_junction`
- `force_copy`
- `supports_project_scope`
- custom/override 行为
- 排序行为

不得手工删减或重新设计 44 款工具配置。新增或修改工具必须有对应测试和来源说明。

适配器只负责配置和路径解析，不直接执行文件复制。

### 4.2 Tool skill cache

迁移：

- 工具是否安装检测；
- skills 目录扫描；
- cache 命中条件；
- cache 失效条件；
- link target 解析；
- community repo 判断。

缓存错误不能覆盖真实文件系统结果；缓存刷新必须有明确入口。

### 4.3 Community Repo 和扫描器

迁移：

- 默认 Community Repo；
- 自定义仓库；
- repository registry；
- 社区仓库扫描；
- 自定义仓库扫描；
- registry 自动同步；
- 仓库迁移。

涉及外部路径时，必须复用文件系统工作包的路径安全接口。

### 4.4 Skill 安装服务

保持当前安装行为：

- 本地目录导入；
- 单技能和多技能选择；
- 套件子路径；
- source type/ref/subpath；
- SKILL.md frontmatter 解析；
- content hash 去重；
- Community Repo 目标路径；
- duplicate 和冲突处理。

安装流程不能在文件复制成功前写入“已成功”的数据库状态。

### 4.5 Onboarding

迁移 `build_onboarding_plan` 及其调用路径：

1. 扫描已安装工具；
2. 排除已经管理的 target path；
3. 计算 fingerprint；
4. 识别 link 和 link target；
5. 按技能名称分组；
6. 标记冲突；
7. 生成前端可消费的计划 DTO。

### 4.6 Maintenance

迁移：

- sync health；
- repair；
- cache 清理；
- discovered skills 清理；
- integrity check；
- repository scan。

所有具有删除或覆盖性质的维护操作必须保留 dry-run 或明确确认机制。

### 4.7 后台任务

建立统一任务状态：

```text
task_id
status
progress
message
error
created_at
updated_at
```

任务事件固定为：

```text
task_started
task_progress
task_completed
task_failed
```

长时间扫描、导入、批量同步和更新任务不得阻塞 UI 线程。任务必须能处理：

- 正常完成；
- 失败；
- 用户取消；
- 应用退出；
- 部分完成后的状态清理。

### 4.8 更新功能

迁移当前更新检查和执行行为，但首版只以 Windows 为正式验收目标。

必须核对：

- 当前版本读取；
- GitHub release 响应解析；
- setup/portable/naked 模式；
- 临时下载文件；
- 等待主进程退出；
- 替换或安装；
- 重启；
- 更新失败恢复。

测试使用 mock 响应和临时文件，禁止测试中下载真实 release 或替换当前程序。

## 5. 验收标准

- 所有 service 不依赖 FastAPI。
- 所有工具配置与当前 Python 事实逐项一致。
- 安装、扫描、Onboarding、维护和任务服务可以被 command 层调用。
- service 不直接返回 HTTP status code。
- 长任务可以通过 Tauri event 向前端报告状态。
- 关键业务错误有 Rust 单元测试或集成测试。
