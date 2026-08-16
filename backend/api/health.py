"""健康检查端点"""
from fastapi import APIRouter

from core.version import __version__
from models.schemas import HealthResponse

router = APIRouter()


@router.get(
    "/api/health",
    response_model=HealthResponse,
    summary="健康检查",
    description="返回服务运行状态，用于前端探活或外部健康检查。",
    tags=["Health"],
)
async def health_check():
    """健康检查：返回服务运行状态。"""
    return HealthResponse(version=__version__)
