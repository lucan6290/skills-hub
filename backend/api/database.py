"""数据库监控与管理 API"""
from __future__ import annotations

import logging
import os
import shutil
import sqlite3
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

from fastapi import APIRouter, Depends, HTTPException
from fastapi.responses import FileResponse

from models.schemas import MaintenanceRequest, ResetRequest, TableQueryRequest
from api.dependencies import get_skill_store
from core.db.store import SkillStore

router = APIRouter()

logger = logging.getLogger(__name__)


def _open_folder(path: Path) -> None:
    """跨平台打开文件所在文件夹"""
    folder = path.parent if path.is_file() else path
    if sys.platform == "win32":
        os.startfile(str(folder))  # type: ignore[attr-defined]
    elif sys.platform == "darwin":
        subprocess.Popen(["open", str(folder)])
    else:
        subprocess.Popen(["xdg-open", str(folder)])

# 允许查询的表白名单（避免暴露内部敏感信息，同时防止 SQL 注入）
ALLOWED_TABLES = {
    "skills",
    "skill_targets",
    "settings",
    "discovered_skills",
    "skill_tags",
    "skill_tag_links",
    "tool_scan_state",
    "tool_skill_cache",
    "tool_adapter_configs",
    "skill_scope_preference",
    "recent_projects",
    "skill_usage",
}

# 各表显示名称（中文）
TABLE_DISPLAY_NAMES = {
    "skills": "技能",
    "skill_targets": "同步目标",
    "settings": "设置项",
    "discovered_skills": "发现的技能",
    "skill_tags": "标签",
    "skill_tag_links": "标签关联",
    "tool_scan_state": "工具扫描状态",
    "tool_skill_cache": "工具技能缓存",
    "tool_adapter_configs": "工具适配器配置",
    "skill_scope_preference": "作用域偏好",
    "recent_projects": "最近项目",
    "skill_usage": "技能使用统计",
}


def _bytes_to_human(size: int) -> str:
    """将字节数转为人类可读格式"""
    for unit in ("B", "KB", "MB", "GB"):
        if size < 1024:
            return f"{size:.2f} {unit}" if unit != "B" else f"{size} B"
        size /= 1024
    return f"{size:.2f} TB"


def _estimate_table_sizes(conn: sqlite3.Connection, page_size: int) -> dict[str, int]:
    """当 dbstat 不可用时，通过列 payload 长度估算各表占用字节数。"""
    sizes: dict[str, int] = {}
    usable = page_size - 100  # 每页可用 payload（扣除页头/槽指针/预留空间）
    for table in ALLOWED_TABLES:
        try:
            cols = [r[1] for r in conn.execute(f"PRAGMA table_info({table})").fetchall()]
            count = conn.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
            if count == 0:
                sizes[table] = 0
                continue
            # 数据量较大时跳过全表 SUM（避免 O(n) 开销），给一个保守估计
            if count > 50000:
                sizes[table] = count * 200  # 粗估每行 200 字节
                continue
            sum_expr = " + ".join([f"ifnull(length({c}),0)" for c in cols])
            total_payload: int = conn.execute(
                f"SELECT COALESCE(SUM({sum_expr}),0) FROM {table}"
            ).fetchone()[0]
            # 每行约 8-12 字节头开销，向上取整到整页
            raw = total_payload + count * 10
            pages = max(1, (raw + usable - 1) // usable)
            # 加上索引估算：约为数据大小的 30%（保守）
            index_pages = max(0, int(pages * 0.3))
            sizes[table] = (pages + index_pages) * page_size
        except sqlite3.OperationalError:
            sizes[table] = 0
    return sizes


def _get_table_columns(conn: sqlite3.Connection, table: str) -> list[dict[str, Any]]:
    """获取表的列信息"""
    cursor = conn.execute(f"PRAGMA table_info({table})")
    columns = []
    for row in cursor.fetchall():
        columns.append({
            "cid": row[0],
            "name": row[1],
            "type": row[2],
            "notnull": bool(row[3]),
            "default": row[4],
            "pk": bool(row[5]),
        })
    return columns


@router.get(
    "/api/db/overview",
    summary="获取数据库概览",
    description="返回数据库文件大小、SQLite 版本、各表白名单行数及碎片率等概览信息。",
    tags=["Database"],
)
async def db_overview(store: SkillStore = Depends(get_skill_store)):
    """获取数据库概览信息"""
    db_path = Path(store.db_path)

    if not db_path.exists():
        raise HTTPException(status_code=404, detail="数据库文件不存在")

    stat = db_path.stat()
    conn = store._get_conn()

    # SQLite 版本
    sqlite_version = conn.execute("SELECT sqlite_version()").fetchone()[0]

    # PRAGMA 信息
    page_size = conn.execute("PRAGMA page_size").fetchone()[0]
    page_count = conn.execute("PRAGMA page_count").fetchone()[0]
    freelist_count = conn.execute("PRAGMA freelist_count").fetchone()[0]
    total_size = page_size * page_count
    free_size = page_size * freelist_count
    fragmentation = (free_size / total_size * 100) if total_size > 0 else 0

    # 各表大小：优先使用 dbstat 虚拟表；不可用时通过 payload 长度估算
    table_sizes: dict[str, int] = {}
    try:
        for row in conn.execute(
            "SELECT name, SUM(pgsize) FROM dbstat GROUP BY name"
        ).fetchall():
            table_sizes[row[0]] = row[1] or 0
    except sqlite3.OperationalError:
        table_sizes = _estimate_table_sizes(conn, page_size)

    # 各表行数与大小
    tables_info = []
    for table in sorted(ALLOWED_TABLES):
        try:
            count = conn.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
        except sqlite3.OperationalError:
            count = 0
        size = table_sizes.get(table, 0)
        tables_info.append({
            "table_name": table,
            "display_name": TABLE_DISPLAY_NAMES.get(table, table),
            "row_count": count,
            "size_bytes": size,
            "size_human": _bytes_to_human(size),
        })

    return {
        "db_path": str(db_path),
        "file_size": stat.st_size,
        "file_size_human": _bytes_to_human(stat.st_size),
        "last_modified": int(stat.st_mtime * 1000),
        "sqlite_version": sqlite_version,
        "page_size": page_size,
        "page_count": page_count,
        "freelist_count": freelist_count,
        "free_size": free_size,
        "free_size_human": _bytes_to_human(free_size),
        "fragmentation_pct": round(fragmentation, 2),
        "tables": tables_info,
    }


@router.get(
    "/api/db/table/{table_name}",
    summary="分页查询表数据",
    description="按表名分页查询白名单表的数据，支持排序与文本过滤；表名非法时返回 400。",
    tags=["Database"],
)
async def db_table_data(
    table_name: str,
    page: int = 1,
    page_size: int = 50,
    sort_col: str | None = None,
    sort_dir: str = "asc",
    filter_text: str | None = None,
    store: SkillStore = Depends(get_skill_store),
):
    """分页查询表数据"""
    if table_name not in ALLOWED_TABLES:
        raise HTTPException(status_code=400, detail=f"不允许查询表: {table_name}")

    conn = store._get_conn()

    # 获取列信息
    columns = _get_table_columns(conn, table_name)
    col_names = [c["name"] for c in columns]

    # 验证排序列
    order_clause = ""
    if sort_col and sort_col in col_names:
        order_clause = f" ORDER BY {sort_col} {'ASC' if sort_dir == 'asc' else 'DESC'}"
    else:
        # 默认按主键或第一列排序
        pk_cols = [c["name"] for c in columns if c["pk"]]
        if pk_cols:
            order_clause = f" ORDER BY {pk_cols[0]} ASC"
        else:
            order_clause = f" ORDER BY {col_names[0]} ASC"

    # 构建 WHERE 条件（简单文本搜索，在所有文本列中 LIKE）
    where_clause = ""
    params: tuple = ()
    if filter_text:
        text_cols = [c["name"] for c in columns if c["type"] and "TEXT" in c["type"].upper()]
        if text_cols:
            like_parts = [f"{col} LIKE ?" for col in text_cols]
            where_clause = " WHERE " + " OR ".join(like_parts)
            params = tuple(f"%{filter_text}%" for _ in text_cols)

    # 总数
    total = conn.execute(
        f"SELECT COUNT(*) FROM {table_name}{where_clause}", params
    ).fetchone()[0]

    # 分页查询
    offset = (page - 1) * page_size
    rows = conn.execute(
        f"SELECT * FROM {table_name}{where_clause}{order_clause} LIMIT ? OFFSET ?",
        params + (page_size, offset),
    ).fetchall()

    # 将 Row 对象转为 dict
    data = []
    for row in rows:
        row_dict = {}
        for col in col_names:
            val = row[col]
            # 尝试解析 JSON 字段以便前端友好显示
            if isinstance(val, str) and val.startswith(("{","[")):
                try:
                    import json
                    row_dict[col] = json.loads(val)
                except (json.JSONDecodeError, ValueError):
                    row_dict[col] = val
            else:
                row_dict[col] = val
        data.append(row_dict)

    return {
        "table": table_name,
        "display_name": TABLE_DISPLAY_NAMES.get(table_name, table_name),
        "columns": columns,
        "rows": data,
        "total": total,
        "page": page,
        "page_size": page_size,
        "total_pages": (total + page_size - 1) // page_size,
    }


@router.get(
    "/api/db/table/{table_name}/columns",
    summary="获取表列结构",
    description="返回白名单表的列结构信息；表名非法时返回 400。",
    tags=["Database"],
)
async def db_table_columns(table_name: str, store: SkillStore = Depends(get_skill_store)):
    """获取表的列结构"""
    if table_name not in ALLOWED_TABLES:
        raise HTTPException(status_code=400, detail=f"不允许查询表: {table_name}")
    conn = store._get_conn()
    return {"table": table_name, "columns": _get_table_columns(conn, table_name)}


@router.post(
    "/api/db/maintenance",
    summary="执行数据库维护操作",
    description="支持 vacuum、analyze、clear_cache、clear_discovered、integrity_check 等维护动作；未知操作返回 400。",
    tags=["Database"],
)
async def db_maintenance(req: MaintenanceRequest, store: SkillStore = Depends(get_skill_store)):
    """执行数据库维护操作"""
    conn = store._get_conn()
    action = req.action

    try:
        if action == "vacuum":
            conn.execute("VACUUM")
            return {"ok": True, "action": action, "message": "VACUUM 完成，数据库已压缩"}

        elif action == "analyze":
            conn.execute("ANALYZE")
            return {"ok": True, "action": action, "message": "ANALYZE 完成，查询统计已更新"}

        elif action == "clear_cache":
            before = conn.execute("SELECT COUNT(*) FROM tool_skill_cache").fetchone()[0]
            conn.execute("DELETE FROM tool_skill_cache")
            conn.commit()
            return {"ok": True, "action": action, "message": f"已清空工具技能缓存，删除 {before} 条记录"}

        elif action == "clear_discovered":
            before = conn.execute("SELECT COUNT(*) FROM discovered_skills").fetchone()[0]
            conn.execute("DELETE FROM discovered_skills")
            conn.commit()
            return {"ok": True, "action": action, "message": f"已清空发现记录，删除 {before} 条记录"}

        elif action == "integrity_check":
            result = conn.execute("PRAGMA integrity_check").fetchone()[0]
            ok = result == "ok"
            return {
                "ok": ok,
                "action": action,
                "message": "数据库完整性检查通过" if ok else f"完整性检查发现问题: {result}",
                "integrity_result": result,
            }

    except sqlite3.Error as e:
        raise HTTPException(status_code=400, detail=f"操作失败: {e}")

    raise HTTPException(status_code=400, detail=f"未知操作: {action}")


@router.get(
    "/api/db/export",
    summary="导出数据库文件",
    description="下载数据库备份文件；数据库文件不存在时返回 404。",
    tags=["Database"],
)
async def db_export(store: SkillStore = Depends(get_skill_store)):
    """导出数据库文件（下载备份）"""
    db_path = Path(store.db_path)

    if not db_path.exists():
        raise HTTPException(status_code=404, detail="数据库文件不存在")

    # 使用临时副本，避免读取时锁定
    tmp_path = db_path.parent / f"skills_hub_backup_{int(time.time())}.db"
    try:
        # 确保 WAL 数据写入主文件
        conn = store._get_conn()
        conn.execute("PRAGMA wal_checkpoint(FULL)")
        shutil.copy2(db_path, tmp_path)
        return FileResponse(
            str(tmp_path),
            filename=f"skills_hub_backup_{time.strftime('%Y%m%d_%H%M%S')}.db",
            media_type="application/octet-stream",
            background=lambda: tmp_path.unlink(missing_ok=True) if tmp_path.exists() else None,
        )
    except Exception:
        if tmp_path.exists():
            tmp_path.unlink(missing_ok=True)
        logger.error("db_export failed: db_path=%s", db_path, exc_info=True)
        raise HTTPException(status_code=500, detail="导出失败，请稍后重试")


@router.post(
    "/api/db/open_folder",
    summary="打开数据库所在文件夹",
    description="在系统文件管理器中打开数据库文件所在目录；数据库文件不存在时返回 404。",
    tags=["Database"],
)
async def db_open_folder(store: SkillStore = Depends(get_skill_store)):
    """打开数据库所在文件夹"""
    db_path = Path(store.db_path)
    if not db_path.exists():
        raise HTTPException(status_code=404, detail="数据库文件不存在")
    try:
        _open_folder(db_path)
        return {"ok": True, "message": "已打开数据库所在文件夹"}
    except Exception:
        logger.error("db_open_folder failed: db_path=%s", db_path, exc_info=True)
        raise HTTPException(status_code=500, detail="打开文件夹失败，请稍后重试")


@router.post(
    "/api/db/reset",
    summary="重置数据库",
    description="危险操作：删除所有表并重建 schema，需在请求中提供确认文字 RESET，否则返回 400。",
    tags=["Database"],
)
async def db_reset(req: ResetRequest, store: SkillStore = Depends(get_skill_store)):
    """重置数据库（危险操作，需确认文字）——通过 DROP 所有表后自愈 schema"""
    if req.confirm_text.strip() != "RESET":
        raise HTTPException(status_code=400, detail='确认文字错误，请输入 "RESET"')

    conn = store._get_conn()

    # 获取所有用户表（排除 sqlite_ 系统表）
    tables = [
        row[0] for row in conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        ).fetchall()
    ]

    # 关闭外键检查以避免 DROP 顺序问题
    conn.execute("PRAGMA foreign_keys = OFF")
    for table in tables:
        conn.execute(f"DROP TABLE IF EXISTS {table}")
    conn.commit()
    conn.execute("PRAGMA foreign_keys = ON")

    # VACUUM 回收空间
    conn.execute("VACUUM")

    # 自愈 schema 重建所有表
    store.ensure_schema()

    return {"ok": True, "message": "数据库已重置，所有数据已清除"}
