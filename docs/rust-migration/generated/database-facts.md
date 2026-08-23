# Generated: Database Facts

> 来源：`backend/core/db/store.py` 中的 DDL 字符串和 `docs/database-schema.md`。
> 生成日期：2026-08-23。这里只记录静态事实；迁移前仍需用真实旧数据库执行读写对照。

静态提取到 **12** 个 `CREATE TABLE IF NOT EXISTS` 表定义、**5** 个索引定义；另外源码包含排序列自愈和 `skill_targets` 旧形状迁移逻辑。

## 表定义

| 表名 | `store.py` 源行 |
|---|---:|
| `skills` | 1019 |
| `skill_targets` | 1036 |
| `settings` | 1053 |
| `discovered_skills` | 1058 |
| `skill_tags` | 1072 |
| `skill_tag_links` | 1080 |
| `tool_scan_state` | 1089 |
| `tool_skill_cache` | 1099 |
| `tool_adapter_configs` | 1116 |
| `skill_scope_preference` | 1131 |
| `recent_projects` | 1139 |
| `skill_usage` | 1145 |

## 索引定义

| 索引名 | `store.py` 源行 |
|---|---:|
| `idx_skill_targets_unique_scope` | 1050 |
| `idx_skills_name` | 1069 |
| `idx_skills_updated_at` | 1070 |
| `idx_tool_skill_cache_tool_name` | 1113 |
| `idx_skill_usage_skill_tool` | 1156 |

## 自愈/兼容性迁移逻辑

| 逻辑 | 实际行为 | `store.py` 源行 |
|---|---|---:|
| `_initialize_sort_order_columns` | 对 `skills`、`skill_tags`、`tool_adapter_configs` 尝试新增 `sort_order REAL NOT NULL DEFAULT 0`；已存在时忽略 `sqlite3.OperationalError`。 | 205 |
| `_migrate_skill_targets_to_v4_if_old_shape` | 当 `skill_targets` 存在且没有 `scope` 列时，创建新表、将旧记录按 `global` scope 转换后替换原表，并重建唯一索引。 | 1221 |
| `_self_heal_schema` / `_add_column_if_missing` | 按源码中的表和列检查并补齐缺失列；具体列清单必须以 `store.py` 当前实现和真实数据库 `PRAGMA table_info` 对照。 | 1012、1212 |

## 迁移前必须验证

- 使用 Python 版本生成的真实 `skills_hub.db` 做只读对照，不能只用全新空库。
- 核对表、列、索引、外键、默认值和自愈 DDL 的执行顺序。
- 验证数据库路径、Portable/安装版数据目录和现有 `docs/database-schema.md` 是否一致。
- Rust migration 不得修改现有 schema 语义；如果必须新增版本迁移，单独记录原因、SQL 和回滚方案。
