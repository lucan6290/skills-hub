"""Skills Hub Python 后端入口"""
import logging
import sys
from contextlib import asynccontextmanager
from pathlib import Path

import uvicorn
from fastapi import FastAPI, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from fastapi.staticfiles import StaticFiles

from core.config import API_HOST, API_PORT, IS_DEV_MODE
from core.logging_config import setup_logging
from core.error_codes import ErrorCode
from core.version import __version__
from models.schemas import ErrorResponse

setup_logging()

logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    from core.repo.scanner import sync_all_repo_registries
    from core.db.store import get_store
    result = sync_all_repo_registries(db_path=get_store().db_path)
    if result["registered"] or result["removed"]:
        logger.info("auto-synced skill repos: %s", result)
    yield


app = FastAPI(title="Skills Hub Backend", version=__version__, lifespan=lifespan)

# CORS: 仅开发模式启用（Vite dev server 跨域访问）
if IS_DEV_MODE:
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["http://localhost:5173", "tauri://localhost"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )


# 全局兜底异常处理：未捕获异常统一返回 500 + 结构化错误体
@app.exception_handler(Exception)
async def unhandled_exception_handler(request: Request, exc: Exception):
    logger.error(
        "Unhandled exception on %s %s",
        request.method,
        request.url.path,
        exc_info=True,
    )
    return JSONResponse(
        status_code=500,
        content=ErrorResponse(code=ErrorCode.INTERNAL_ERROR, message="internal error").model_dump(),
    )

# 自动发现并注册所有 API 路由
from api import register_all_routers
register_all_routers(app)


# 根路径健康检查：仅开发模式注册。生产模式下 / 交给 StaticFiles(html=True) 服务 index.html，
# 否则显式路由会先于 mount 命中，导致桌面窗口（打开根路径）看到的是这段 JSON 而非界面。
if IS_DEV_MODE:
    @app.get("/")
    async def root():
        return {"message": "Skills Hub Backend is running"}


@app.api_route("/api/cancel_current_operation", methods=["GET", "POST"])
async def cancel_current_operation():
    from core.tasks.manager import get_task_manager
    cancelled = get_task_manager().cancel_all_running()
    return {"ok": True, "cancelled": cancelled}


# 生产模式：托管前端静态文件
if not IS_DEV_MODE:
    static_dir = Path(__file__).parent / "static"
    if static_dir.is_dir():
        app.mount("/", StaticFiles(directory=str(static_dir), html=True), name="static")


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else API_PORT
    uvicorn.run(app, host=API_HOST, port=port, log_level="info", reload=IS_DEV_MODE)
