# 数据库表结构

Skills Hub 使用 **SQLite** 数据库，文件名为 `skills_hub.db`。数据库位置按平台：

- Windows: `%APPDATA%/skills-hub/skills_hub.db`
- macOS: `~/Library/Application Support/skills-hub/skills_hub.db`
- Linux: `~/.local/share/skills-hub/skills_hub.db`
- 便携模式: `<exe_dir>/data/skills_hub.db`

Schema 采用自愈模式（self-healing），应用启动时通过幂等 DDL 确保所有表和列存在，无版本号信任。

---

## 表总览

| 表名 | 用途 |
|---|---|
| [skills](#1-skills) | Skill 主记录 |
| [skill_targets](#2-skill_targets) | Skill 同步目标 |
| [skill_tags](#3-skill_tags) | 标签 |
| [skill_tag_links](#4-skill_tag_links) | 标签关联 |
| [settings](#5-settings) | 应用配置键值对 |
| [discovered_skills](#6-discovered_skills) | 已发现但未导入的 Skill（预留） |
| [tool_scan_state](#7-tool_scan_state) | 工具扫描状态缓存 |
| [tool_skill_cache](#8-tool_skill_cache) | 工具 Skill 条目缓存 |
| [tool_adapter_configs](#9-tool_adapter_configs) | 工具适配器配置 |
| [skill_scope_preference](#10-skill_scope_preference) | Skill 作用域偏好 |
| [recent_projects](#11-recent_projects) | 最近使用的项目 |
| [skill_usage](#12-skill_usage) | Skill 使用统计 |

---

## 1. skills

Skill 主记录表，存储每一个 skill 的完整元信息。

| 字段 | 类型 | 约束 | 描述 |
|---|---|---|---|
| `id` | TEXT | PRIMARY KEY | 唯一 UUID v4 标识符 |
| `name` | TEXT | NOT NULL | Skill 名称，来源于 SKILL.md 解析或目录名 |
| `description` | TEXT | NULL | Skill 描述，来源于 SKILL.md frontmatter |
| `source_type` | TEXT | NOT NULL | 来源类型：`"community"`（社区仓库安装）或 `"custom"`（用户自制） |
| `source_ref` | TEXT | NULL | Skill 来源的原始文件系统路径 |
| `source_subpath` | TEXT | NULL | 套件目录内的相对子路径（用于嵌套安装） |
| `source_revision` | TEXT | NULL | 版本控制标识（预留，如 git commit SHA） |
| `community_path` | TEXT | NOT NULL, UNIQUE | Skill 源文件在仓库内的唯一路径 |
| `content_hash` | TEXT | NULL | Skill 目录内容的哈希指纹，用于去重和变更检测 |
| `version` | TEXT | NULL | Skill 版本号 |
| `author` | TEXT | NULL | 作者名称 |
| `license` | TEXT | NULL | 许可证类型 |
| `category` | TEXT | NULL | 分类（如"代码生成"、"调试"、"文档"） |
| `homepage` | TEXT | NULL | 项目主页或源码仓库 URL |
| `frontmatter_extra` | TEXT | NULL | SKILL.md 除 name/description 外的所有 frontmatter 字段，JSON 对象格式 |
| `skill_file_count` | INTEGER | NULL | Skill 目录内文件数量（缓存值） |
| `skill_dir_size` | INTEGER | NULL | Skill 目录内文件总大小（字节，缓存值） |
| `created_at` | INTEGER | NOT NULL | 创建时间（Unix 毫秒时间戳） |
| `updated_at` | INTEGER | NOT NULL | 最后更新时间（Unix 毫秒时间戳） |
| `last_sync_at` | INTEGER | NULL | 最后一次同步到工具的时间（Unix 毫秒时间戳） |
| `last_seen_at` | INTEGER | NOT NULL | 仓库扫描器最后一次确认该 skill 存在的时间（Unix 毫秒时间戳） |
| `status` | TEXT | NOT NULL | 状态：`"active"`、`"missing"`、`"ok"` |

索引：`(name)`、`(updated_at)`

注：`version`、`author`、`license`、`category`、`homepage` 字段在安装时从 SKILL.md frontmatter 中提取；`skill_file_count`、`skill_dir_size` 在安装时自动计算。

---

## 2. skill_targets

记录每个 skill 在 AI 工具目录中的同步安装目标。一个 skill 可以同步到多个工具/作用域/项目。

| 字段 | 类型 | 约束 | 描述 |
|---|---|---|---|
| `id` | TEXT | PRIMARY KEY | 唯一目标记录标识符 |
| `skill_id` | TEXT | NOT NULL, FK→skills(id) ON DELETE CASCADE | 关联的 Skill ID |
| `tool` | TEXT | NOT NULL | 工具标识符（如 `"claude_code"`、`"cursor"` 等） |
| `scope` | TEXT | NOT NULL, DEFAULT 'global' | 作用域：`"global"`（全局）或 `"project"`（项目级） |
| `project_path` | TEXT | NULL | 当 scope 为 `"project"` 时的项目路径 |
| `target_path` | TEXT | NOT NULL | Skill 安装到的文件系统路径 |
| `mode` | TEXT | NOT NULL | 同步模式：`"symlink"`、`"junction"`、`"copy"` |
| `status` | TEXT | NOT NULL | 同步状态：`"ok"` 或 `"error"` |
| `last_error` | TEXT | NULL | 最后一次同步的错误信息 |
| `synced_at` | INTEGER | NULL | 最后一次同步完成时间（Unix 毫秒时间戳） |
| `target_content_hash` | TEXT | NULL | 目标目录内容哈希（用于检测 copy 模式下的过期拷贝） |
| `target_updated_at` | INTEGER | NULL | 目标目录最后更新时间（Unix 毫秒时间戳） |

唯一索引：`(skill_id, tool, scope, COALESCE(project_path, ''))`

---

## 3. skill_tags

用户创建的 Skill 分类标签。

| 字段 | 类型 | 约束 | 描述 |
|---|---|---|---|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | 数字标签 ID |
| `name` | TEXT | NOT NULL, UNIQUE, COLLATE NOCASE | 标签名称（不区分大小写，自动去除首尾空格） |
| `created_at` | INTEGER | NOT NULL | 创建时间（Unix 毫秒时间戳） |
| `updated_at` | INTEGER | NOT NULL | 最后更新时间（Unix 毫秒时间戳） |

---

## 4. skill_tag_links

Skills 与 Tags 的多对多关联表。

| 字段 | 类型 | 约束 | 描述 |
|---|---|---|---|
| `skill_id` | TEXT | NOT NULL, FK→skills(id) ON DELETE CASCADE | 关联的 Skill ID |
| `tag_id` | INTEGER | NOT NULL, FK→skill_tags(id) ON DELETE CASCADE | 关联的 Tag ID |
| `created_at` | INTEGER | NOT NULL | 关联建立时间（Unix 毫秒时间戳） |

主键：`(skill_id, tag_id)`

---

## 5. settings

应用级配置的通用键值存储。

| 字段 | 类型 | 约束 | 描述 |
|---|---|---|---|
| `key` | TEXT | PRIMARY KEY | 设置键名 |
| `value` | TEXT | NOT NULL | 设置值 |

已知键：

| 键名 | 值示例 | 用途 |
|---|---|---|
| `community_repo_path` | `/absolute/path/to/.skillshub` | 社区仓库存储目录 |
| `custom_repo_path` | `/absolute/path/to/.skills-hub-custom` | 自制 Skill 存储目录 |
| `default_sync_tools` | `["claude_code", "cursor"]` | 新建同步操作时的默认工具列表（JSON 数组） |

---

## 6. discovered_skills

在工具目录中发现但尚未导入的 Skill（Schema 已定义，当前代码未实际读写此表）。

| 字段 | 类型 | 约束 | 描述 |
|---|---|---|---|
| `id` | TEXT | PRIMARY KEY | 唯一标识符 |
| `tool` | TEXT | NOT NULL | 发现该 Skill 的工具标识符 |
| `found_path` | TEXT | NOT NULL | 发现的文件系统路径 |
| `name_guess` | TEXT | NULL | 根据路径推测的 Skill 名称 |
| `fingerprint` | TEXT | NULL | 目录内容指纹 |
| `found_at` | INTEGER | NOT NULL | 发现时间（Unix 毫秒时间戳） |
| `imported_skill_id` | TEXT | NULL, FK→skills(id) ON DELETE SET NULL | 导入后关联的 Skill ID |

---

## 7. tool_scan_state

每个 AI 工具的目录扫描缓存元数据，用于避免在 skills 目录未变化时重复扫描。

| 字段 | 类型 | 约束 | 描述 |
|---|---|---|---|
| `tool_key` | TEXT | PRIMARY KEY | 工具标识符（如 `"cursor"`、`"claude_code"`） |
| `tool_name` | TEXT | NOT NULL | 工具显示名称 |
| `installed` | INTEGER | NOT NULL | 工具是否已安装：0=未安装，1=已安装 |
| `skills_dir` | TEXT | NULL | 已扫描的 skills 目录路径 |
| `supports_project_scope` | INTEGER | NOT NULL, DEFAULT 1 | 工具是否支持项目范围同步：0=不支持，1=支持 |
| `dir_mtime_ns` | INTEGER | NULL | 扫描时 skills 目录的修改时间戳（纳秒） |
| `scanned_at` | INTEGER | NOT NULL | 扫描发生时间（Unix 毫秒时间戳） |
| `first_seen_at` | INTEGER | NULL | 工具首次被检测到的时间（Unix 毫秒时间戳） |

---

## 8. tool_skill_cache

工具 skills 目录中每个 skill 条目（文件/链接）的缓存版本，支持快速发现而无需文件系统扫描。

| 字段 | 类型 | 约束 | 描述 |
|---|---|---|---|
| `tool_key` | TEXT | NOT NULL, FK→tool_scan_state(tool_key) ON DELETE CASCADE | 工具标识符 |
| `skill_path` | TEXT | NOT NULL | Skill 条目的完整文件系统路径 |
| `name` | TEXT | NOT NULL | 从路径解析出的 Skill 名称 |
| `is_link` | INTEGER | NOT NULL | 是否为符号链接：0=否，1=是 |
| `link_target` | TEXT | NULL | 如果为链接，指向的目标路径 |
| `description` | TEXT | NULL | 缓存的 Skill 描述 |
| `in_community_repo` | INTEGER | NOT NULL, DEFAULT 0 | 是否位于社区仓库目录内：0=否，1=是 |
| `skill_mtime_ns` | INTEGER | NULL | 文件修改时间戳（纳秒） |
| `scanned_at` | INTEGER | NOT NULL | 缓存条目创建时间（Unix 毫秒时间戳） |

主键：`(tool_key, skill_path)`
索引：`(tool_key, name)`

---

## 9. tool_adapter_configs

支持 44 个 AI 工具的适配器配置，定义如何与各工具的 skills 目录交互。

| 字段 | 类型 | 约束 | 描述 |
|---|---|---|---|
| `tool_key` | TEXT | PRIMARY KEY | 工具标识符 |
| `display_name` | TEXT | NOT NULL | 面向用户显示的工具名称 |
| `skills_dir` | TEXT | NOT NULL | 相对于工具主目录的 skills 目录路径 |
| `detect_dir` | TEXT | NOT NULL | 用于检测工具是否已安装的路径模式 |
| `project_skills_dir` | TEXT | NULL | 项目范围内 skills 目录（相对于项目根目录） |
| `supports_symlink` | INTEGER | NOT NULL, DEFAULT 1 | 是否支持符号链接同步：0=不支持，1=支持 |
| `supports_junction` | INTEGER | NOT NULL, DEFAULT 1 | 是否支持 Windows Junction：0=不支持，1=支持 |
| `force_copy` | INTEGER | NOT NULL, DEFAULT 0 | 是否必须使用文件复制（不创建链接）：0=否，1=是 |
| `supports_project_scope` | INTEGER | NULL | 是否支持项目范围同步：0=不支持，1=支持，NULL=未知 |
| `is_custom` | INTEGER | NOT NULL, DEFAULT 0 | 是否为用户自定义工具（非内置）：0=内置，1=自定义 |
| `enabled` | INTEGER | NOT NULL, DEFAULT 1 | 是否启用：0=禁用，1=启用 |
| `updated_at` | INTEGER | NOT NULL | 最后更新时间（Unix 毫秒时间戳） |

---

## 10. skill_scope_preference

每个 Skill 的作用域偏好设置，控制下次同步时该 Skill 安装到哪个范围。

| 字段 | 类型 | 约束 | 描述 |
|---|---|---|---|
| `skill_id` | TEXT | PRIMARY KEY | Skill ID |
| `scope` | TEXT | NOT NULL, DEFAULT 'global' | 偏好作用域：`"global"` 或 `"project"` |
| `project_paths` | TEXT | NOT NULL, DEFAULT '[]' | 当 scope 为 `"project"` 时的项目路径（JSON 数组） |
| `updated_at` | INTEGER | NOT NULL | 最后更新时间（Unix 毫秒时间戳） |

---

## 11. recent_projects

最近使用项目的 LRU 列表，最多保留 8 个条目（超出时自动淘汰最旧记录）。

| 字段 | 类型 | 约束 | 描述 |
|---|---|---|---|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | 内部行标识符 |
| `project_path` | TEXT | NOT NULL, UNIQUE | 项目绝对路径 |
| `last_used_at` | INTEGER | NOT NULL | 最后访问时间（Unix 毫秒时间戳） |

---

## 12. skill_usage

记录每个 Skill 在各工具上的使用统计。

| 字段 | 类型 | 约束 | 描述 |
|---|---|---|---|
| `id` | INTEGER | PRIMARY KEY AUTOINCREMENT | 内部行标识符 |
| `skill_id` | TEXT | NOT NULL, FK→skills(id) ON DELETE CASCADE | 关联的 Skill ID |
| `tool` | TEXT | NOT NULL | 工具标识符 |
| `sync_count` | INTEGER | NOT NULL, DEFAULT 0 | 累计同步次数 |
| `last_synced_at` | INTEGER | NULL | 最后一次同步时间（Unix 毫秒时间戳） |
| `last_viewed_at` | INTEGER | NULL | 最后一次在 Hub 中查看时间（Unix 毫秒时间戳） |
| `view_count` | INTEGER | NOT NULL, DEFAULT 0 | 累计查看次数 |

唯一索引：`(skill_id, tool)`
