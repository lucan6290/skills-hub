# Repo 仓库扫描 Agent 入口

本文件是 `repo/` 的导航入口。Repo 模块管理社区仓库和自定义仓库的路径解析与扫描注册。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

解析社区仓库（`~/.skillshub`）和自定义仓库（`~/.skills-hub-custom`）的路径，扫描仓库中的技能目录，将发现的技能注册到数据库，并移除已不存在的技能记录。

## 文件清单

| 文件 | 职责 |
|------|------|
| `community.rs` | 路径解析：`resolve_community_repo_path`（DB 优先 → `~/.skillshub`）、`resolve_custom_repo_path`（DB 优先 → `~/.skills-hub-custom`）、`ensure_community_repo`（创建目录） |
| `scanner.rs` | 扫描注册：`is_skill_dir`（检测 `SKILL.md` 或 `.claude/skills/*/SKILL.md`）、`is_suite_dir`（套件检测）、`has_sub_skills`、`scan_and_register_community_repo`、`scan_and_register_custom_repo`、`sync_all_repo_registries`、`sync_community_repo_registry` / `sync_custom_repo_registry`、`normalize_source_type` |

## 路径解析优先级

```
community_repo_path:  DB setting → ~/.skillshub
custom_repo_path:     DB setting → ~/.skills-hub-custom
```

## 技能目录检测规则

| 检测项 | 判定条件 |
|--------|---------|
| `is_skill_dir` | 目录下有 `SKILL.md`，或 `.claude/skills/*/SKILL.md` 存在 |
| `is_suite_dir` | 目录下无 `SKILL.md`，但 ≥2 个子目录都是技能目录 |
| `has_sub_skills` | 目录下有任意子目录是技能目录 |

## 硬规则

1. **路径来源优先级**：DB `settings` 表 → 默认路径（home 目录下）
2. **扫描注册**：发现新技能 → upsert 到 `skills` 表；磁盘上不存在的技能 → 从 DB 删除
3. **`normalize_source_type`** 统一 source_type 值（`community` / `custom`），用于前端筛选
4. **文件统计**（`skill_file_count`、`skill_dir_size`）在扫描时计算并写入
5. **content_hash** 通过 `utils::content_hash::hash_dir` 计算，用于检测技能内容变更
6. **路径安全** 通过 `utils::path_safety` 校验

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 修改仓库路径解析 | 本文件 + `community.rs` + [../../repositories/settings.rs](../repositories/settings.rs) |
| 修改扫描注册逻辑 | 本文件 + `scanner.rs` + [../../repositories/skills.rs](../repositories/skills.rs) + [../../utils/content_hash.rs](../utils/content_hash.rs) |
| 新增仓库类型 | 本文件 + `community.rs`（路径）+ `scanner.rs`（扫描）+ `commands/sync.rs`（command） |
