# 数据库规范

> **定位**：后端数据访问层的设计约定，覆盖 Schema 管理、SQL 编写、连接管理和 Dataclass DTO。
> **关系**：完整表结构见 [`docs/database-schema.md`](../../docs/database-schema.md)；系统结构见 [PROJECT_STRUCTURE.md](./PROJECT_STRUCTURE.md)；API 规范见 [API_STANDARD.md](./API_STANDARD.md)。

---

## 1. 核心原则

| 原则 | 说明 |
|------|------|
| **纯 Raw SQL** | 不使用任何 ORM 框架（禁止 SQLAlchemy、Tortoise、Peewee） |
| **Schema 自愈** | 幂等 DDL，无版本号信任，启动时自动确保表和列存在 |
| **线程安全** | `threading.local()` 每线程独立连接 |
| **全局单例** | `get_store()` 双重检查锁，全应用共享一个 `SkillStore` 实例 |
| **参数化查询** | 所有 SQL 使用 `?` 占位符，禁止字符串拼接（防注入） |

---

## 2. Schema 自愈机制详解

所有表和列通过 `_self_heal_schema()` 确保物理存在。新增表或列只需在此函数中添加一条记录。

### 新增表

```python
# 在 _self_heal_schema() 中添加
conn.execute("""
    CREATE TABLE IF NOT EXISTS new_table (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        created_at INTEGER NOT NULL
    )
""")
```

### 新增列

```python
# 使用辅助函数
_add_column_if_missing(conn, "existing_table", "new_column", "TEXT NULL")
```

### 辅助函数实现

```python
def _add_column_if_missing(conn, table: str, column: str, col_def: str):
    """幂等地为已有表添加列（如果列不存在）"""
    cursor = conn.execute(f"PRAGMA table_info({table})")
    existing_columns = {row[1] for row in cursor.fetchall()}
    if column not in existing_columns:
        conn.execute(f"ALTER TABLE {table} ADD COLUMN {column} {col_def}")
```

> **唯一修改点**：添加表/列只改 `_self_heal_schema()` 一处，无需迁移脚本、无需 Alembic。

---

## 3. UPSERT 模式

写入操作统一使用 `INSERT ... ON CONFLICT DO UPDATE`：

```python
self._execute(
    """INSERT INTO skills (id, name, description, source_type, community_path,
                           content_hash, created_at, updated_at, last_seen_at, status)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
       ON CONFLICT(id) DO UPDATE SET
         name = excluded.name,
         description = excluded.description,
         content_hash = excluded.content_hash,
         updated_at = excluded.updated_at,
         last_seen_at = excluded.last_seen_at,
         status = excluded.status""",
    (record.id, record.name, record.description, record.source_type,
     record.community_path, record.content_hash, record.created_at,
     record.updated_at, record.last_seen_at, record.status),
)
```

### 要点

- 冲突键通常是主键或唯一索引
- `excluded` 引用待插入的值
- 仅更新需要变更的列，时间戳列（如 `created_at`）通常不更新

---

## 4. Dataclass DTO 清单

数据访问层使用 `@dataclass` 作为内部传输对象，定义在 `core/db/store.py`：

| Dataclass | 对应表 | 说明 |
|-----------|--------|------|
| `SkillRecord` | `skills` | Skill 主记录 |
| `SkillTargetRecord` | `skill_targets` | 同步目标记录 |
| `SkillUsageRecord` | `skill_usage` | 使用统计 |
| `TagRecord` | `skill_tags` | 标签 |
| `TagWithCountRecord` | `skill_tags` + JOIN | 带技能计数的标签 |
| `ToolScanStateRecord` | `tool_scan_state` | 工具扫描状态 |
| `ToolSkillCacheRecord` | `tool_skill_cache` | 工具技能缓存条目 |
| `ToolAdapterConfigRecord` | `tool_adapter_configs` | 工具适配器配置 |
| `ScopePreferenceRecord` | `skill_scope_preference` | 作用域偏好 |

### 使用规则

- Dataclass 仅在 `core/db/store.py` 中定义
- `api/` 层和 `core/` 业务层通过 Dataclass 与数据层交互
- API 对外暴露使用 Pydantic 模型（`models/schemas.py`），不直接暴露 Dataclass

---

## 5. 连接管理

使用 `threading.local()` 实现每线程独立连接：

```python
class SkillStore:
    def __init__(self, db_path: str):
        self._db_path = db_path
        self._local = threading.local()

    def _get_conn(self) -> sqlite3.Connection:
        """获取当前线程的数据库连接（线程安全）"""
        if not hasattr(self._local, "conn") or self._local.conn is None:
            conn = sqlite3.connect(self._db_path)
            conn.execute("PRAGMA foreign_keys = ON")
            conn.row_factory = sqlite3.Row
            self._local.conn = conn
        return self._local.conn
```

### 要点

- 外键约束默认开启（`PRAGMA foreign_keys = ON`）
- `row_factory = sqlite3.Row` 支持按列名访问
- 连接在线程内复用，避免频繁打开/关闭

---

## 6. 事务约定

| 场景 | 方式 | 说明 |
|------|------|------|
| 单条写操作 | `_execute()` 内部自动 `commit()` | 简单写入无需显式事务 |
| 多条写操作需原子性 | `with conn:` 上下文管理器 | 任一步骤失败则全部回滚 |
| 读操作 | 无需显式事务 | SQLite 读不阻塞写 |

### 多条写操作示例

```python
def batch_update_skills(self, records: list[SkillRecord]):
    conn = self._get_conn()
    with conn:  # 自动 BEGIN/COMMIT/ROLLBACK
        for record in records:
            conn.execute(
                """INSERT INTO skills (...) VALUES (...)
                   ON CONFLICT(id) DO UPDATE SET ...""",
                params,
            )
```

---

## 7. 全局单例模式

使用双重检查锁确保线程安全的单例初始化：

```python
_store_instance: Optional["SkillStore"] = None
_init_lock = threading.Lock()

def get_store() -> "SkillStore":
    global _store_instance
    if _store_instance is None:
        with _init_lock:
            if _store_instance is None:
                db_path = default_db_path()
                migrate_legacy_db_if_needed(db_path)
                _store_instance = SkillStore(db_path)
                _store_instance.ensure_schema()
    return _store_instance
```

### 要点

- 首次调用时创建实例并执行 Schema 自愈
- 旧版数据库迁移在创建实例前执行
- 后续调用直接返回已有实例，无锁开销

---

## 8. 新增表/列的操作步骤

### 步骤 1：在 `_self_heal_schema()` 中添加 DDL

```python
# 新增表
conn.execute("""
    CREATE TABLE IF NOT EXISTS my_new_table (
        id TEXT PRIMARY KEY,
        skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
        value TEXT NOT NULL,
        created_at INTEGER NOT NULL
    )
""")

# 新增列（对已有表）
_add_column_if_missing(conn, "existing_table", "new_column", "TEXT NULL")
```

### 步骤 2：定义 Dataclass DTO

在 `core/db/store.py` 中添加：

```python
@dataclass
class MyNewRecord:
    id: str
    skill_id: str
    value: str
    created_at: int
```

### 步骤 3：实现 CRUD 方法

在 `SkillStore` 类中添加对应的查询/写入方法：

```python
def upsert_my_new_record(self, record: MyNewRecord):
    self._execute(
        """INSERT INTO my_new_table (id, skill_id, value, created_at)
           VALUES (?, ?, ?, ?)
           ON CONFLICT(id) DO UPDATE SET
             value = excluded.value""",
        (record.id, record.skill_id, record.value, record.created_at),
    )

def get_my_new_records(self, skill_id: str) -> list[MyNewRecord]:
    rows = self._fetch_all(
        "SELECT * FROM my_new_table WHERE skill_id = ?",
        (skill_id,),
    )
    return [MyNewRecord(**dict(row)) for row in rows]
```

### 步骤 4：编写测试

使用 `isolated_store` fixture 验证新表/列的行为：

```python
def test_upsert_my_new_record(isolated_store):
    record = MyNewRecord(id="test-id", skill_id="skill-1", value="hello", created_at=1000)
    isolated_store.upsert_my_new_record(record)
    results = isolated_store.get_my_new_records("skill-1")
    assert len(results) == 1
    assert results[0].value == "hello"
```

> **验证**：运行 `cd backend && python -m pytest` 确认所有测试通过。
