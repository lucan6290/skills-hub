# 02. SQLite、数据目录与配置兼容

## 1. 目标

使用 `rusqlite` 在 Rust 中读取和写入现有 `skills_hub.db`，保持当前用户数据、数据目录和数据库行为兼容。

本工作包不重新设计 schema，不改变用户数据格式，不迁移文件同步逻辑。

## 2. 当前代码证据

权威实现：

- `backend/core/db/store.py`：SQLite 连接、schema、自愈 DDL、CRUD、legacy database 处理。
- `backend/core/config.py`：数据目录、数据库文件名、内置工具配置和环境覆盖。
- `docs/database-schema.md`：当前 12 张表的说明性文档。

当前数据库文件名：

```text
skills_hub.db
```

当前 schema 表清单以 `docs/database-schema.md` 和 `SkillStore.ensure_schema` 的实际 SQL 为准。文档与代码冲突时，以 `store.py` 为准并记录差异。

当前数据目录行为：

- Windows 安装版：`%APPDATA%/skills-hub/skills_hub.db`
- Portable：`<exe_dir>/data/skills_hub.db`
- macOS/Linux 路径虽然已有文档说明，但不属于首个 Windows 验收范围，仍需保留清晰的未实现/待验证标记。

## 3. 允许修改范围

```text
frontend/src-tauri/src/models/**
frontend/src-tauri/src/db/**
frontend/src-tauri/src/repositories/**
frontend/src-tauri/src/config.rs
```

禁止修改：

- `commands/**`
- `main.rs`、`lib.rs`
- 前端业务代码
- 现有正式数据库文件
- `docs/database-schema.md`，除非核实发现其与代码不一致并另行修正文档

## 4. 实施步骤

### 4.1 复核数据库事实

实施前重新检查：

```powershell
Select-String -Path backend/core/db/store.py -Pattern "CREATE TABLE|CREATE INDEX|ALTER TABLE"
```

并对照：

```text
docs/database-schema.md
backend/core/db/store.py
backend/skills_hub.db（只读检查，不修改）
```

必须记录：

- 表名
- 字段名、类型、默认值
- 主键和外键
- 唯一约束
- 初始化顺序
- 兼容旧字段的逻辑

### 4.2 设计 Rust 数据模型

Rust model 只表达当前 DTO/数据库真实字段，不提前增加抽象字段。

跨 Tauri 边界的模型必须可以通过 `serde` 序列化，字段保持 `snake_case`。

不要把 `rusqlite::Row` 暴露给 command 层。

### 4.3 建立数据库连接状态

推荐状态结构：

```text
AppState
  └── 数据库状态
        └── 受 Mutex 保护的连接或等价连接管理
```

要求：

- repository 操作短时间持有数据库锁；
- 事务完成后立即释放锁；
- 不在数据库锁内执行目录扫描、文件复制或外部进程；
- 读写错误统一转换为 `AppError`。

具体连接管理实现必须以 `rusqlite` 实际 trait 约束和编译结果为准，不得凭空假定线程安全属性。

### 4.4 迁移 schema 初始化

将 `SkillStore.ensure_schema` 的实际行为迁移为 Rust 初始化流程：

1. 创建数据目录；
2. 打开数据库；
3. 开启与当前行为一致的 SQLite 设置；
4. 执行幂等 DDL；
5. 执行必要的 legacy database 检测；
6. 初始化默认工具配置；
7. 提交事务。

禁止在 Rust 首版同时引入全新的 migration table 或重命名现有表，除非先完成兼容性设计和测试。

### 4.5 迁移 repository

按依赖顺序：

1. settings 和基础配置；
2. tags；
3. skills；
4. skill targets；
5. scope preferences；
6. recent projects；
7. tool adapter configs；
8. skill usage；
9. tool scan/cache；
10. database maintenance。

repository 返回领域 model 或 DTO，不返回 HTTP 状态码。

### 4.6 数据库管理能力

迁移当前数据库管理功能：

- overview
- table query
- columns
- maintenance
- export
- open folder
- reset

`reset` 属于破坏性操作，必须保留当前确认文本或等价的明确用户确认机制，不能由 command 默认执行。

## 5. 测试要求

所有测试使用临时数据库：

- 空数据库初始化；
- 已存在完整旧数据库；
- 旧版/legacy database；
- 缺少可选字段；
- 重复初始化；
- 事务回滚；
- 数据库损坏；
- 外键约束；
- reset 需要明确确认。

禁止测试直接修改仓库内的 `backend/skills_hub.db`。

## 6. 验收标准

- Rust 可以原位读取 Python 版本创建的数据库。
- skills、tags、targets、settings、scope preference 数据不丢失。
- 默认工具配置与 Python 当前配置一致。
- 数据库路径与 Portable/安装版规则一致。
- repository 不依赖 FastAPI。
- `cargo test` 和数据库对照测试通过。
