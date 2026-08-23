# 09. 分阶段实施检查表

## 1. 用法

本文件是实施执行清单，不替代 `00`—`08` 的详细步骤。每个阶段完成后，必须填写实际命令、结果、commit hash 和未解决项；不能只勾选“完成”。

状态约定：

- `[ ]` 未开始；
- `[-]` 进行中；
- `[x]` 已完成且有证据；
- `[!]` 阻塞，必须记录原因和下一步。

## 2. Phase 0：基线与工具链

- [ ] 确认 `git branch --show-current` 输出为 `main-Rust`。
- [ ] 记录 `git status --short --branch`，单独保存实施前已有修改。
- [ ] 执行 `cd frontend; npm run check`，保存完整结果。
- [ ] 执行 `cd backend; python -m pytest -q`，保存完整结果。
- [ ] 执行 `rustc --version`、`cargo --version`、`node --version`、`npm --version`。
- [ ] 确认 Windows 构建所需的 WebView2、MSVC/Windows SDK、NSIS 状态；只记录实际检查结果。
- [ ] 生成并审阅 `generated/endpoint-inventory.md`、`dto-inventory.md`、`database-facts.md`、`frontend-call-sites.md`。
- [ ] 记录当前 `frontend/package-lock.json` 等既有修改，不得混入不相关 commit。

**通过证据**：基线命令输出、工具链版本、inventory 生成时间和当前 commit。

## 3. Phase 1：Tauri 最小壳

- [ ] 建立 `frontend/src-tauri/` 的最小 Tauri 工程。
- [ ] React dev server 可由 Tauri 窗口加载。
- [ ] 生产模式加载 `frontend/dist`，不启动 Python/FastAPI。
- [ ] 实现 `health_check` invoke，返回版本字段。
- [ ] 窗口标题、默认尺寸、最小尺寸与 Python 版本基线一致，除非有记录的变更理由。
- [ ] 单实例行为已验证。
- [ ] `cargo fmt --check`、`cargo check`、`cargo test` 已执行。

**通过证据**：启动截图或日志、Rust 测试结果、进程列表/端口检查结果。

## 4. Phase 2：公共错误、DTO、数据库与配置

- [ ] 冻结 `generated/command-map.md` 的 command 名称。
- [ ] 建立统一 `AppError` 和序列化错误结构。
- [ ] Rust DTO 与 `generated/dto-inventory.md` 逐项对照。
- [ ] 使用真实 Python 数据库副本执行 Rust 只读验证。
- [ ] 验证 12 张现有表、索引、外键、默认值和兼容迁移行为。
- [ ] 验证安装版与 Portable 数据目录规则。
- [ ] 数据库 repository 不依赖 HTTP/FastAPI。

**通过证据**：schema 对照表、旧库读写测试、数据目录测试和 commit hash。

## 5. Phase 3：文件系统与同步

- [ ] 先完成路径规范化和边界检查，再接入业务 service。
- [ ] 完成文件读写、目录枚举、hash 和受控删除。
- [ ] 在临时目录验证 symlink、junction、copy fallback。
- [ ] 验证 global/project scope 和 suite skill。
- [ ] 验证 unsync 不删除非托管文件。
- [ ] 验证失败时不写入伪成功的 `skill_targets` 记录。

**通过证据**：Windows 临时目录测试、Python/Rust hash 对照、失败场景日志。

## 6. Phase 4：领域服务

- [ ] 工具适配器、工具扫描和 cache 行为对照完成。
- [ ] Community Repo、Custom Repo、扫描和安装行为对照完成。
- [ ] 标签、排序、Onboarding、维护和新工具检测可用。
- [ ] 后台任务有启动、进度、取消、失败和清理路径。
- [ ] 更新检查和更新执行已用 mock 覆盖，不替换正在运行的 exe。

**通过证据**：领域 service 测试、临时 repo 测试、任务事件记录、更新 mock 结果。

## 7. Phase 5：前端 invoke 迁移

- [ ] `frontend/src/lib/api.ts` 的 HTTP transport 替换为 Tauri invoke transport。
- [ ] DTO 字段保持 `snake_case`。
- [ ] 组件不再直接拼接 `/api/` 或 command 字符串。
- [ ] GET query 参数迁移为结构化 invoke 参数。
- [ ] 数据库导出、打开目录、选择目录使用 Tauri 能力或 Rust command。
- [ ] 主要页面逐项完成 smoke test。

**通过证据**：前端检查结果、command 调用测试、主要页面操作记录。

## 8. Phase 6：发布与最终清理

- [ ] Windows 生产构建生成 exe。
- [ ] 生成 Portable ZIP，包含 `portable.flag` 和 `data/` 目录。
- [ ] 生成 NSIS 安装包并验证安装/卸载/覆盖安装。
- [ ] 验证升级不破坏数据库、Community Repo 和同步目标。
- [ ] CI 已包含 frontend、Rust、Windows bundle 和 artifact 检查。
- [ ] Rust 与 Python 的关键结果完成对照。
- [ ] `git grep` 确认生产代码不再依赖 HTTP、FastAPI、pywebview、PyInstaller。
- [ ] 由主 Agent 单独提交删除 Python 的 commit；删除前重新确认无引用。

**最终通过证据**：见 `08-integration-acceptance.md` 的验收结论格式。

## 9. 每个工作包的实施记录模板

```text
工作包：
负责人 Agent：
开始 commit：
修改文件：
未修改但依赖的文件：
实施步骤：
- ...

实际执行命令：
- ...

测试/构建结果：
- 通过：...
- 失败：...

提交 hash：
与其他 Agent 的接口：
未完成事项：
已知风险：
回滚方式：
```
