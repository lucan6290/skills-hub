# API 设计规范

> **定位**：后端 API 层的设计约定与实施模板，覆盖路由声明、请求响应模型、错误处理和依赖注入。
> **关系**：系统结构见 [PROJECT_STRUCTURE.md](./PROJECT_STRUCTURE.md)；测试与安全规范见 [TESTING_STANDARD.md](./TESTING_STANDARD.md)；错误码定义在 `core/error_codes.py`。

---

## 1. 路由自动发现机制

新路由文件**无需手动注册**。`api/__init__.py` 通过 `pkgutil.walk_packages` 递归扫描 `api/` 包下所有子模块，自动注册包含 `router` 对象的模块：

```python
# api/__init__.py
def register_all_routers(app):
    """自动发现 api/ 包下所有模块的 router 对象并注册到 FastAPI app"""
    for info in pkgutil.walk_packages(__path__, "api."):
        mod = importlib.import_module(info.name)
        if hasattr(mod, "router"):
            app.include_router(mod.router)
```

**新增路由步骤**：在 `api/` 或其子目录下创建 `.py` 文件，定义 `router = APIRouter()`，即可自动生效。

---

## 2. 端点命名约定表

| 操作 | 命名模式 | 示例 |
|------|---------|------|
| 查询 | `get_<noun>` | `get_managed_skills`, `get_tags`, `get_tool_status` |
| 创建 | `create_<noun>` | `create_tag` |
| 更新 | `update_<noun>` / `set_<noun>` | `update_skill_source_url`, `set_skill_tags` |
| 删除 | `delete_<noun>` | `delete_tag`, `delete_managed_skill` |
| 动作 | `<verb>_<noun>_to_<target>` | `sync_skill_to_tool`, `unsync_skill_from_tool` |
| 列表 | `list_<noun>` | `list_skill_files`, `list_local_skills_cmd` |
| 保存 | `save_<noun>` | `save_recent_project`, `save_default_sync_tools` |
| 扫描 | `scan_<noun>` | `scan_community_repo`, `scan_all_repos` |
| 重置 | `reset_<noun>` | `reset_tool_adapter_config` |

> **规则**：全链路使用 `snake_case`，禁止 camelCase。端点名 = URL 路径最后一段 = Python 函数名。

---

## 3. 路由声明模板

```python
from fastapi import APIRouter, Depends, HTTPException
from models.schemas import SomeRequest, SomeResponse
from api.dependencies import get_skill_store
from core.db.store import SkillStore

router = APIRouter()

@router.post(
    "/api/some_endpoint",
    response_model=SomeResponse,       # 关键端点必须声明
    summary="中文简述",
    description="详细描述（可选）。",
    tags=["ModuleName"],
)
async def some_endpoint(
    req: SomeRequest,
    store: SkillStore = Depends(get_skill_store),
):
    try:
        result = store.some_method(req.field)
        return SomeResponse(...)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
```

### 要点

- 所有端点使用 `POST` 方法（除健康检查等纯查询用 `GET`）
- URL 路径以 `/api/` 开头
- 关键端点**必须**声明 `response_model`
- 通过 `Depends(get_skill_store)` 注入 store，不要直接调用 `get_store()`

---

## 4. 请求/响应模型规范

### 定义位置

- **集中定义**在 `models/schemas.py`
- 禁止在 `api/` 或 `core/` 中内联定义 Pydantic 模型

### 命名后缀

| 类型 | 后缀 | 示例 |
|------|------|------|
| 请求模型 | `*Request` | `SyncRequest`, `DeleteManagedSkillRequest` |
| 响应模型 | `*Response` / `*Dto` | `SyncResultDto`, `InstallResultDto` |

### 字段命名

- 全部使用 `snake_case`
- 禁止 `Field(alias=...)`
- 前端 DTO 字段名 = 后端 Pydantic 字段名 = JSON key

---

## 5. 统一错误响应

### ErrorResponse 模型

```python
class ErrorResponse(BaseModel):
    ok: bool = False
    code: str          # ErrorCode 枚举值
    message: str       # 人类可读描述
    detail: Optional[dict] = None  # 附加上下文
```

### ErrorCode 枚举

所有结构化错误码定义在 `core/error_codes.py`：

```python
class ErrorCode(str, Enum):
    PROJECT_SCOPE_UNSUPPORTED = "PROJECT_SCOPE_UNSUPPORTED"
    TOOL_NOT_INSTALLED = "TOOL_NOT_INSTALLED"
    TOOL_NOT_WRITABLE = "TOOL_NOT_WRITABLE"
    TARGET_EXISTS = "TARGET_EXISTS"
    SKILL_INVALID = "SKILL_INVALID"
    INTERNAL_ERROR = "INTERNAL_ERROR"
```

> **规则**：新增错误码时必须在此枚举中添加，禁止使用魔法字符串。

### 全局兜底异常处理

`main.py` 注册了未捕获异常的全局处理器：

```python
@app.exception_handler(Exception)
async def unhandled_exception_handler(request: Request, exc: Exception):
    logger.error("Unhandled exception on %s %s", request.method, request.url.path, exc_info=True)
    return JSONResponse(
        status_code=500,
        content=ErrorResponse(code=ErrorCode.INTERNAL_ERROR, message="internal error").model_dump(),
    )
```

> **禁止**向客户端暴露内部堆栈信息。兜底处理器仅返回通用错误消息。

---

## 6. HTTP 状态码约定表

| 状态码 | 含义 | 使用场景 |
|--------|------|---------|
| 200 | 成功 | 正常响应（含业务失败但请求本身合法的情况） |
| 400 | 业务错误 | 参数校验失败、资源冲突、业务规则违反 |
| 403 | 权限/模式限制 | 非开发模式访问受限端点 |
| 404 | 资源不存在 | 技能/标签/工具未找到 |
| 500 | 内部错误 | 未捕获异常（由全局兜底处理器返回） |

> **注意**：业务逻辑失败（如同步失败）但请求本身合法时，仍返回 200 + 业务级错误信息。仅在请求参数非法或系统异常时使用 4xx/5xx。

---

## 7. 依赖注入模式

通过 `Depends(get_skill_store)` 获取 `SkillStore` 实例：

```python
# api/dependencies.py
from core.db.store import SkillStore, get_store

def get_skill_store() -> SkillStore:
    return get_store()
```

### 使用规则

- ✅ 始终通过 `Depends(get_skill_store)` 注入
- ❌ 不要在路由函数中直接调用 `get_store()`
- ✅ 测试时可替换依赖实现

---

## 8. API 层异常转换模板

`api/` 层负责将 `core/` 抛出的异常转换为 HTTP 响应：

```python
try:
    result = some_core_function(...)
    return result
except ValueError as e:
    raise HTTPException(status_code=400, detail=str(e))
except PermissionError as e:
    raise HTTPException(status_code=403, detail=str(e))
except FileNotFoundError as e:
    raise HTTPException(status_code=404, detail=str(e))
```

### 自定义异常

`core/skills/sync_service.py` 定义了 `SkillSyncError`，携带 `status_code` 和 `detail`：

```python
class SkillSyncError(Exception):
    def __init__(self, status_code: int, detail):
        super().__init__(str(detail))
        self.status_code = status_code
        self.detail = detail
```

API 层捕获后直接转换：

```python
except SkillSyncError as e:
    raise HTTPException(status_code=e.status_code, detail=e.detail)
```

---

## 9. dry_run 预演模式

破坏性操作（删除、迁移等）**必须**支持 `dry_run` 参数：

```python
class DeleteManagedSkillRequest(BaseModel):
    skill_id: str
    dry_run: bool = False
```

### 行为约定

- `dry_run=True` 时仅返回预期影响，不执行实际变更
- 响应中包含 `would_affect` 字段描述将要执行的操作
- `dry_run=False`（默认）时正常执行

---

## 10. 新增 API 的实施步骤

### 步骤 1：定义请求/响应模型

在 `models/schemas.py` 中添加 Pydantic 模型：

```python
class NewFeatureRequest(BaseModel):
    field_a: str
    field_b: int = 0

class NewFeatureResponse(BaseModel):
    result: str
    count: int
```

### 步骤 2：实现业务逻辑

在 `core/` 对应子包中实现纯业务函数（不依赖 FastAPI）：

```python
# core/some_module/service.py
def do_new_feature(field_a: str, field_b: int) -> dict:
    # 纯业务逻辑
    ...
```

### 步骤 3：创建路由处理器

在 `api/` 对应子包中创建路由文件：

```python
# api/some_module/new_feature.py
from fastapi import APIRouter, Depends, HTTPException
from models.schemas import NewFeatureRequest, NewFeatureResponse
from api.dependencies import get_skill_store
from core.some_module.service import do_new_feature

router = APIRouter()

@router.post(
    "/api/new_feature",
    response_model=NewFeatureResponse,
    summary="新功能简述",
    tags=["SomeModule"],
)
async def new_feature(
    req: NewFeatureRequest,
    store = Depends(get_skill_store),
):
    try:
        result = do_new_feature(req.field_a, req.field_b)
        return NewFeatureResponse(**result)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
```

### 步骤 4：编写测试

在 `tests/` 中添加测试文件，使用 `isolated_client` fixture：

```python
def test_new_feature_success(isolated_client):
    resp = isolated_client.post("/api/new_feature", json={
        "field_a": "test",
        "field_b": 1,
    })
    assert resp.status_code == 200
    assert resp.json()["result"] == "expected"
```

> **验证**：运行 `cd backend && python -m pytest` 确认所有测试通过。
