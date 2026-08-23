# 07. 多 Agent 并行实施与集成协议

## 1. 目标

将 Rust 重构拆分成互不覆盖的工作包，使多个 Agent 可以并行实施，并由主 Agent 统一集成、验证和处理跨模块冲突。

## 2. Agent 工作包

| Agent | 负责内容 | 独占写入范围 |
|---|---|---|
| A | 基线、endpoint/DTO/schema/command 清单 | `docs/rust-migration/00*`、`generated/**` |
| B | Tauri 壳、共享入口、Windows 构建 | `frontend/src-tauri` 入口、Tauri 配置、构建配置 |
| C | SQLite、配置、models、repositories | `frontend/src-tauri/src/models/**`、`frontend/src-tauri/src/db/**`、`frontend/src-tauri/src/repositories/**`、`frontend/src-tauri/src/config.rs` |
| D | 路径、文件系统、hash、同步引擎 | `platform/**`、`filesystem/**`、`skills/files.rs`、`sync_engine.rs`、`utils/**` |
| E | 工具、仓库、安装、Onboarding、维护、任务、更新 | `services/**`、`tools/**`、`repo/**`、对应 skills/tasks/update 模块 |
| F | 前端 invoke、Service、hooks 适配 | `frontend/src/lib/**`、`services/**`、通信相关 hooks/features |
| 主 Agent | command 注册、最终集成、跨包修复、删除旧运行时 | 共享入口、集成提交和最终清理 |

如果当前实际目录与表中写入范围冲突，必须先更新本文件再修改代码。

## 3. 执行依赖

```text
A：基线和契约
        ↓
B：Tauri skeleton 和共享接口
        ↓
C、D、E、F 并行
        ↓
主 Agent 集成 command、事件和构建
        ↓
集成回归
        ↓
删除 Python 运行时
```

C、D、E、F 可以并行，但必须依赖：

- A 的 contract/inventory；
- B 的 Rust 工程和共享错误结构；
- 约定好的 trait 和 DTO；
- 不直接修改其他 Agent 的目录。

## 4. 共享接口所有权

Agent B 独占：

```text
frontend/src-tauri/src/main.rs
frontend/src-tauri/src/lib.rs
frontend/src-tauri/src/error.rs
frontend/src-tauri/src/state.rs
frontend/src-tauri/src/contracts.rs
frontend/src-tauri/src/commands/**
```

其他 Agent：

- 只实现自己的 service/repository/platform；
- 通过 `contracts.rs` 中定义的接口协作；
- 不在自己的分支中复制和修改 command 注册逻辑；
- 不直接把业务逻辑写入 `main.rs`。

## 5. 每个 Agent 的启动步骤

```powershell
git status --short --branch
git branch --show-current
```

然后：

1. 阅读根目录 `AGENTS.md`；
2. 阅读对应模块 `AGENTS.md`；
3. 阅读本目录对应实施文档；
4. 阅读 `00-baseline-and-contract.md` 及 generated inventory；
5. 确认写入范围没有和其他 Agent 重叠；
6. 先运行相关基线测试；
7. 再开始修改。

## 6. Agent 交付格式

每个 Agent 完成后必须报告：

```text
工作包：
修改文件：
提交 hash：
运行命令：
测试结果：
依赖的其他工作包：
未完成事项：
已知风险：
```

不得只回复“已完成”。

## 7. Git 规则

- 每个工作包使用独立分支或独立 forked workspace。
- 不使用共享工作区并行写入。
- 不执行 `git reset --hard`。
- 不覆盖工作开始前已有修改。
- 不执行 `git push`。
- 提交只包含当前工作包。
- 主 Agent 使用 cherry-pick 或明确合并方式集成。
- 冲突必须记录解决原因和验证结果。

## 8. 并行完成后的集成顺序

主 Agent 按以下顺序集成：

1. C：数据库和公共 model；
2. D：文件系统和同步；
3. E：领域 service；
4. F：前端 invoke；
5. B/主 Agent：command 注册、Tauri event 和构建；
6. 最后更新 CI、发布和文档。

每次集成后至少执行：

```powershell
cd frontend
npm run check

cd src-tauri
cargo fmt --check
cargo check
cargo test
```

## 9. 防止幻觉的交接要求

- 任何新增接口必须指出实现文件和调用方。
- 任何行为变化必须指出旧 Python 位置和 Rust 新位置。
- 任何“与旧版一致”的结论必须有测试或对照结果。
- 不能因为编译通过就声称业务迁移完成。
- 未验证的 Windows 权限、symlink、junction 和更新行为必须明确标为未验证。
