# 10. 架构决策、风险与回滚方案

## 1. 文档目的

集中记录 Rust/Tauri 重构中已经选择的方向、尚未验证的事项和发生问题时的回滚路径，避免实施 Agent 把“目标设计”误写成“当前事实”。

状态说明：

- **已决定**：用户目标或本次迁移方案已经明确。
- **待验证**：必须通过源码、工具链、测试或真实 Windows 运行确认。
- **禁止**：未满足前置条件不得执行。

## 2. 已决定的架构方向

| 决策 | 结论 | 理由/边界 |
|---|---|---|
| 桌面容器 | Tauri 2 | 最终交付是桌面 exe；不再把本地 HTTP 服务作为运行时通信层。具体 crate 版本实施时锁定。 |
| 后端语言 | Rust | 负责 command、SQLite、文件系统、同步、工具适配、任务和更新。 |
| 前端 | 继续 React + TypeScript | 迁移重点是通信层和桌面能力，不在本任务中重写 UI。 |
| 前后端通信 | Tauri `invoke` 和事件 | command 参数/返回 DTO 跨边界统一使用 `snake_case`。 |
| 数据库 | 继续使用现有 SQLite 文件和 schema | 先原位兼容 `skills_hub.db`，禁止借重构之名重新设计表结构。 |
| 发布形态 | Windows exe、Portable ZIP、NSIS | 具体 bundle 配置和安装行为以实际 Tauri 构建产物为准。 |
| Python 生命周期 | 迁移阶段保留，验收通过后再删除 | Python 是行为基线，不允许在 Rust 尚未通过对照测试时删除。 |

## 3. 必须保留的行为边界

- 现有用户数据目录和 `skills_hub.db` 不得因为迁移而静默换位置。
- Portable 模式使用 exe 同目录的 `data/`；安装版规则以当前 `backend/core/config.py` 和真实运行结果为准。
- 用户输入的 source、target、project、repo 和 skill file path 必须经过统一路径安全检查。
- 同步目标优先级、symlink/junction/copy fallback、scope、suite skill 和 unsync 保护必须以 Python 实现和测试为准。
- 更新失败不能破坏当前 exe、数据库或 Community Repo。

## 4. 待验证风险清单

| 风险 | 影响 | 验证方式 | 通过条件 |
|---|---|---|---|
| Rust/Tauri 工具链缺失或版本不兼容 | 无法构建桌面包 | 运行 `rustc --version`、`cargo --version`、Tauri 构建 | 干净 Windows 环境成功构建 |
| WebView2/Windows SDK/MSVC 不完整 | exe 无法启动或 CI 失败 | 在目标 Windows 环境执行启动和 CI 构建 | 运行、退出、安装包均成功 |
| SQLite 旧库字段/迁移差异 | 数据丢失或读写失败 | Python 旧库副本 + `PRAGMA`/关键查询对照 | 表、列、记录和写入结果一致 |
| symlink/junction 权限差异 | 同步模式变化或误删 | Windows 临时目录和权限矩阵 | 失败时正确 fallback，未授权时有明确错误 |
| Tauri 权限配置过宽或不足 | 安全风险/功能不可用 | 审阅 capabilities 并执行最小权限 smoke test | 无全盘/任意 shell 权限，功能正常 |
| 动态 HTTP 调用漏迁 | 页面运行时功能缺失 | 扫描 `fetch`、`/api/`、`apiCall`、`apiGet` 和人工页面操作 | 生产代码无未迁移调用 |
| 后台任务生命周期不一致 | 卡死、无法取消或资源泄漏 | 启动/进度/取消/失败/退出测试 | 所有状态都有终态和清理 |
| 更新覆盖正在运行的 exe | 更新失败或应用损坏 | 仅使用 mock/辅助进程验证替换流程 | 当前 exe 始终有可恢复副本 |
| 多 Agent 共享目录写入冲突 | 文件覆盖、难以集成 | 检查每个 Agent 的独占写入范围和 diff | 无重叠写集，冲突有记录 |

## 5. 回滚策略

### 5.1 代码回滚

1. 每个 Agent 使用独立工作区/分支，只提交自己的工作包。
2. 主 Agent 集成前记录 `git status`、基线 commit 和各 Agent commit hash。
3. 集成失败时只回退对应工作包 commit；禁止 `git reset --hard` 覆盖用户已有修改。
4. Python 运行入口和旧 CI 在 Rust 验收完成前保留，可作为行为基线和临时回退入口。

### 5.2 数据回滚

- 所有数据库测试使用副本或临时目录，不直接操作用户正式数据库。
- 任何 schema 迁移前复制 `skills_hub.db` 并记录文件 hash。
- Rust 版本默认只执行已经验证过的兼容迁移；新迁移必须先 dry-run/备份，再提交。
- Portable/安装版升级失败时保留旧 exe 和用户数据目录，不能用空库覆盖旧库。

### 5.3 发布回滚

- 发布前保留上一版本 exe/NSIS/Portable artifact。
- 升级器在替换前确认下载文件完整、目标路径合法，并生成失败可恢复的备份。
- 安装/升级 smoke test 失败时停止发布，不删除旧版 Python 或 Rust 运行入口。

## 6. 变更记录要求

每次修改以下事项时，必须同步更新本文件或对应实施文档：

- Tauri/crate/toolchain 版本；
- command 名称、DTO 字段或错误结构；
- 数据目录、数据库 schema 或 migration；
- symlink/junction/copy 策略；
- 安装、Portable 或自动更新行为；
- Python 删除时机和验收结论。

所有新增结论必须注明：源码文件、命令输出、测试名称或发布产物路径中的至少一种证据。
