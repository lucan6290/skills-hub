# Backend Agent 入口

本文件是 `backend/` 的导航入口。只保留后端每次任务都适用的硬约束；详细规则按任务类型从下方「任务路由」逐级读取。

> 全局规则见根目录 [../AGENTS.md](../AGENTS.md)。

## 1. 每次任务都必须遵守

1. **全链路 snake_case**：Pydantic 字段、JSON key、API 参数、dict key 全部 snake_case，禁止 camelCase、禁止 `Field(alias=...)`
2. **分层不可逆**：`api/ → core/ → models/`，core/ 禁止导入 api/，models/ 禁止导入 api/ 或 core/
3. **不自动执行有副作用操作**：数据库写入、文件删除、外部服务调用需先确认
4. **禁止泄露敏感信息**：不读取/打印/提交 .env 内容和密钥
5. **路径安全**：所有接受用户输入路径的操作必须经过 `path_safety` 校验
6. **WIP=1**：一次只推进一个可独立验证的工作单元
7. **文档与代码冲突时以当前代码为准**，并报告冲突

## 2. 目录结构

```
backend/
├── main.py                  # FastAPI 入口（端口 18921），lifespan 自动扫描仓库
├── desktop.py               # pywebview 桌面窗口入口（自动托管后端）
├── build.py                 # PyInstaller 打包脚本（输出 SkillsHub.exe）
├── requirements.txt
├── api/                     # 路由层：参数校验、异常转换、DTO 组装
│   ├── skills/crud.py       # 技能增删改查
│   ├── skills/files.py      # /api/list_skill_files、read_skill_file
│   ├── skills/sync.py       # /api/sync_skill_to_tool、unsync
│   ├── tools/status.py      # /api/get_tool_status
│   ├── tools/tool_skills.py # 工具技能列表
│   ├── database.py          # 数据库管理 API
│   ├── dependencies.py      # 依赖注入
│   ├── health.py            # 健康检查（返回版本号）
│   ├── maintenance.py       # 同步健康 API
│   ├── onboarding.py        # /api/get_onboarding_plan
│   ├── reorder.py           # 排序 API
│   ├── settings.py          # /api/settings（社区仓库、缓存、Token）
│   ├── tags.py              # /api/tags CRUD
│   └── tasks.py             # 任务管理 API
├── core/                    # 业务逻辑层：纯业务规则，不依赖 FastAPI
│   ├── version.py           # 应用版本号（__version__，唯一版本源）
│   ├── config.py            # 应用配置
│   ├── error_codes.py       # 错误码枚举
│   ├── logging_config.py    # 集中式日志配置
│   ├── db/store.py          # 数据访问层：SQLite ORM（12 张表，自愈 DDL）
│   ├── repo/                # 仓库管理
│   │   ├── community.py     # 社区仓库路径管理
│   │   ├── community_migration.py # 仓库迁移
│   │   └── scanner.py       # 仓库扫描器
│   ├── skills/              # 技能操作
│   │   ├── files.py         # 技能文件操作
│   │   ├── install_service.py  # 安装编排服务
│   │   ├── installer.py     # 本地技能安装
│   │   ├── maintenance.py   # 健康检查与修复
│   │   ├── onboarding.py    # 已有技能扫描
│   │   ├── source_paths.py  # 来源路径解析
│   │   ├── sync_engine.py   # 符号链接/联接点/复制同步
│   │   └── sync_service.py  # 同步编排服务
│   ├── tasks/manager.py     # 后台任务管理器
│   ├── tools/               # 工具适配
│   │   ├── adapters.py      # 44 款 AI 工具的适配器配置
│   │   └── skill_cache.py   # 工具技能缓存
│   └── utils/               # 工具函数
│       ├── constants.py     # 共享常量
│       ├── content_hash.py  # SHA256 目录哈希
│       └── path_safety.py   # 路径安全工具
├── models/schemas.py        # DTO 层：Pydantic BaseModel（请求/响应模型）
└── tests/                   # 测试：pytest + 隔离 DB
```

## 3. 分层边界速查

```
api/     → 路由层：参数校验、异常转换、DTO 组装
core/    → 业务逻辑层：纯业务规则，不依赖 FastAPI
core/db/ → 数据访问层：Raw SQL + SQLite + Schema 自愈
models/  → DTO 层：Pydantic BaseModel（请求/响应模型），不导入 core/ 或 api/
tests/   → 测试：pytest + 隔离 DB
```

## 4. 编码规范

- **函数/方法**：snake_case
- **常量**：SCREAMING_SNAKE_CASE
- **文件操作**：使用 `pathlib.Path`
- **错误信息**：双语（中文为主）
- **版本号**：从 `core.version import __version__` 读取，禁止硬编码

## 5. 任务路由

涉及后端代码的任务，按任务类型读取对应专题文档：

| 任务类型 | 必读文档 |
|---------|---------|
| 新增/修改 API 端点 | [docs/API_STANDARD.md](docs/API_STANDARD.md) |
| 新增/修改数据库表或查询 | [docs/DATABASE_STANDARD.md](docs/DATABASE_STANDARD.md) |
| 新增/修改测试 | [docs/TESTING_STANDARD.md](docs/TESTING_STANDARD.md) |
| 理解后端整体结构 | [docs/PROJECT_STRUCTURE.md](docs/PROJECT_STRUCTURE.md) |
| 数据表字段详情 | [../docs/database-schema.md](../docs/database-schema.md) |
| 修改 backend/docs/ 下的文档 | [docs/AGENTS.md](docs/AGENTS.md) |
| 安全相关变更 | [docs/TESTING_STANDARD.md](docs/TESTING_STANDARD.md) § 安全规范 |

## 6. 常用命令

```bash
cd backend && python main.py              # 启动服务（端口 18921）
cd backend && python desktop.py           # 启动桌面窗口模式
cd backend && python build.py             # PyInstaller 打包（需先 npm run build）
cd backend && python -m pytest tests/ -v  # 运行测试
cd backend && python -m pytest tests/test_import_cycles.py -v  # 循环导入检查
```

## 7. 文档权威顺序

1. 用户明确指令
2. 根 `AGENTS.md` + 本文件硬约束
3. `backend/docs/` 下的专题规范
4. 当前代码实际行为
5. 其他说明性文档
