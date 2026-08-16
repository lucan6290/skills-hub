"""集中式日志配置：为后端提供统一的日志格式与级别。"""
from __future__ import annotations

import logging
import logging.config

# 幂等保护标志：避免重复配置
_CONFIGURED = False


def setup_logging() -> None:
    """初始化全局日志配置（幂等，重复调用不会重复配置）。"""
    global _CONFIGURED
    if _CONFIGURED:
        return

    logging.config.dictConfig(
        {
            "version": 1,
            "disable_existing_loggers": False,
            "formatters": {
                "default": {
                    "format": "%(asctime)s [%(levelname)s] %(name)s: %(message)s",
                },
            },
            "handlers": {
                "console": {
                    "class": "logging.StreamHandler",
                    "formatter": "default",
                },
            },
            "root": {
                "level": "INFO",
                "handlers": ["console"],
            },
        }
    )
    _CONFIGURED = True
