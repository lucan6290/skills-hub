# 测试与安全规范

> **定位**：后端测试框架、隔离策略、安全防护和提交前检查清单的综合规范。
> **关系**：API 规范见 [API_STANDARD.md](./API_STANDARD.md)；数据库规范见 [DATABASE_STANDARD.md](./DATABASE_STANDARD.md)；系统结构见 [PROJECT_STRUCTURE.md](./PROJECT_STRUCTURE.md)。

---

## 1. 测试框架与运行命令

| 项目 | 说明 |
|------|------|
| 框架 | pytest ≥ 8.0 |
| 运行命令 | `cd backend && python -m pytest` |
| 测试目录 | `backend/tests/` |
| 配置文件 | `backend/tests/conftest.py`（全局 fixtures） |

---

## 2. Fixtures 说明

`conftest.py` 提供三个核心 fixture：

| Fixture | 用途 | 适用场景 |
|---------|------|---------|
| `client` | 基于真实 DB 的 `TestClient(app)` | 需要真实数据的集成测试 |
| `isolated_store` | 临时 SQLite + monkeypatch 替换全局单例 | 单元测试、数据层测试 |
| `isolated_client` | 基于隔离 store 的 `TestClient` | 端到端测试首选 |

### isolated_store 实现

```python
@pytest.fixture
def isolated_store(tmp_path, monkeypatch):
    store = SkillStore(str(tmp_path / "isolated_skills_hub.db"))
    store.ensure_schema()
    monkeypatch.setattr(store_module, "_store_instance", store)
    yield store
    store.close()
```

### 使用示例

```python
# 数据层单元测试
def test_upsert_skill(isolated_store):
    record = SkillRecord(id="test", name="Test Skill", ...)
    isolated_store.upsert_skill(record)
    result = isolated_store.get_skill("test")
    assert result.name == "Test Skill"

# API 端到端测试
def test_get_managed_skills(isolated_client):
    resp = isolated_client.post("/api/get_managed_skills")
    assert resp.status_code == 200
```

---

## 3. 隔离策略

### 数据库隔离

每个测试使用临时 SQLite 文件，通过 `monkeypatch` 替换 `_store_instance`：

```python
# isolated_store fixture 自动完成
# 测试结束后临时文件由 tmp_path 自动清理
```

### 文件系统隔离

使用 `tmp_path` fixture 创建临时目录：

```python
def test_sync_creates_symlink(isolated_store, tmp_path):
    skill_dir = tmp_path / "skills" / "my-skill"
    skill_dir.mkdir(parents=True)
    # 在临时目录中操作，不影响真实文件系统
```

### 环境变量隔离

使用 `monkeypatch.setenv` / `monkeypatch.delenv`：

```python
def test_config_with_env_override(monkeypatch):
    monkeypatch.setenv("SKILLS_HUB_PORT", "9999")
    # 重新导入或读取配置
    from core.config import API_PORT
    assert API_PORT == 9999
```

---

## 4. 命名约定

| 项目 | 规范 | 示例 |
|------|------|------|
| 测试文件 | `test_<module>.py` | `test_api_skills.py`, `test_sync_engine.py` |
| 测试函数 | `test_<描述>` | `test_create_tag_success`, `test_sync_fails_when_tool_not_installed` |
| Fixture | 描述性名词 | `isolated_store`, `sample_skill_record` |
| 测试类 | `Test<Module>` | `TestSkillSync`, `TestPathSafety` |

---

## 5. 循环导入防护测试

`tests/test_import_cycles.py` 提供三层防护：

### 运行时防护

在干净子进程中分别导入互依模块，验证不触发 `ImportError`。

### 静态扫描

扫描源码，断言交叉依赖的 import 只出现在函数体（带缩进的行）中：

```python
# ✅ 正确：函数体内延迟导入
def sync_skill_to_tool(...):
    from core.tools.adapters import adapter_by_key
    ...

# ❌ 错误：模块顶层导入（可能触发循环导入）
from core.tools.adapters import adapter_by_key
```

### 反向依赖防护

扫描 `core/` 目录，断言不存在顶层 `from api` / `import api` 语句。

> **规则**：每次新增跨模块依赖时，必须确认循环导入防护测试仍然通过。

---

## 6. 路径穿越防护

所有接受用户输入路径的端点**必须**进行路径安全检查。

### 工具函数表

| 函数 | 位置 | 用途 |
|------|------|------|
| `safe_dir_name(name, fallback)` | `core/utils/path_safety.py` | 过滤非法字符、Windows 保留名、长度截断 |
| `is_path_within(path, base)` | `core/utils/path_safety.py` | 词法级路径包含检查（不跟踪符号链接） |
| `require_path_within(path, base, label)` | `core/utils/path_safety.py` | 校验不通过则抛出 `ValueError` |
| `safe_child_path(base, child_name, label)` | `core/utils/path_safety.py` | 拼接子路径并校验安全性 |
| `norm_path(path)` | `core/utils/path_safety.py` | 规范化为绝对路径 + 大小写归一 |
| `expand_home(input_path)` | `core/utils/path_safety.py` | 展开 `~` / `~/` 为用户主目录 |

### 使用示例

```python
from core.utils.path_safety import require_path_within, safe_dir_name, is_path_within

# 强制校验路径在基目录内
safe_path = require_path_within(user_input_path, base_dir, label="skill path")

# 生成安全的目录名
dir_name = safe_dir_name(user_provided_name, fallback="skill")

# 检查路径包含关系
if is_path_within(candidate, root):
    ...
```

---

## 7. TOCTOU 防护

文件操作应尽量使用 fd 级操作，避免 check-then-act 竞态：

```python
# ✅ 推荐：fd 级操作
fd = os.open(path, os.O_RDONLY)
try:
    data = os.read(fd, MAX_SIZE)
finally:
    os.close(fd)

# ❌ 避免：check-then-act
if os.path.exists(path):       # 检查
    with open(path) as f:      # 使用时可能已被替换
        data = f.read()
```

---

## 8. 文件大小限制

读取用户上传或外部文件时，限制最大大小为 **1 MB**：

```python
MAX_FILE_SIZE = 1 * 1024 * 1024  # 1 MB
```

> 超限时应返回明确的错误信息，而非静默截断。

---

## 9. CORS 策略

CORS **仅在开发模式**（`IS_DEV_MODE=True`）下启用：

```python
if IS_DEV_MODE:
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["http://localhost:5173", "tauri://localhost"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )
```

| 模式 | CORS | 前端访问方式 |
|------|------|-------------|
| 开发模式 | 启用，限定 localhost:5173 | Vite dev server |
| 生产模式 | 不启用 | pywebview 内嵌或静态文件托管 |

---

## 10. 敏感信息保护

| 规则 | 说明 |
|------|------|
| ❌ Token/密码/API Key | 不得出现在日志、错误消息、API 响应中 |
| ❌ `.env` 文件 | 不得提交到版本控制 |
| ✅ 环境变量加载 | 通过 `python-dotenv` 加载，使用 try-import 避免硬依赖 |
| ✅ 路径脱敏 | 必要时使用相对路径或省略用户目录前缀 |
| ✅ 日志级别 | INFO 级别以下不暴露完整文件路径中的用户名部分 |

---

## 11. 提交前清单

### 通用清单

- [ ] 所有新增/修改的代码符合分层架构依赖方向
- [ ] 全链路字段名使用 `snake_case`，无 camelCase
- [ ] 无 `print()` 语句，使用 `logging` 模块
- [ ] 无硬编码路径、端口、密钥
- [ ] 用户可见文本考虑国际化需求
- [ ] `python -m pytest` 全部通过
- [ ] 循环导入防护测试通过

### API 专项清单

- [ ] 新路由文件定义了 `router = APIRouter()` 对象（自动发现）
- [ ] 端点命名遵循 `动词_名词` 约定
- [ ] 请求模型定义在 `models/schemas.py`
- [ ] 关键端点声明了 `response_model`
- [ ] 通过 `Depends(get_skill_store)` 注入 store
- [ ] 异常已转换为适当的 HTTP 状态码
- [ ] 破坏性操作支持 `dry_run` 参数

### 数据库专项清单

- [ ] 新增表/列仅在 `_self_heal_schema()` 中声明
- [ ] 写入操作使用 UPSERT 模式
- [ ] 多条写操作使用 `with conn:` 保证原子性
- [ ] 新增 Dataclass 记录类型放在 `core/db/store.py`
- [ ] SQL 参数化查询，无字符串拼接（防注入）
- [ ] 外键约束已声明（`FOREIGN KEY ... ON DELETE CASCADE`）
