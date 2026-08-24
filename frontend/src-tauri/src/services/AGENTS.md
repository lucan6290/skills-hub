# Service 层 Agent 入口

本文件是 `services/` 的导航入口。Service 层封装跨多个 Repository 的业务逻辑。

> 上级入口：[../AGENTS.md](../AGENTS.md)

## 职责

当一个操作需要**跨多个 Repository 协调**（如安装技能时同时写 skills 表、skill_targets 表、计算 content_hash、同步文件），逻辑放在 Service 层。Service 函数接收 `&Database` 而非 `State`，便于测试时传入内存数据库。

## 文件清单

| 文件 | 职责 | 关键函数 / 类型 |
|------|------|----------------|
| `install.rs` | 技能安装：扫描本地目录、解析 SKILL.md frontmatter、计算文件统计、写入数据库 | `install_local_skill`, `install_local_skill_from_selection`, `list_local_skills`, `parse_skill_md`, `upsert_skill_from_install`, `InstallResult`, `LocalSkillCandidate`, `SkillFrontmatter` |
| `maintenance.rs` | 同步健康检查：扫描已同步技能的链接/文件状态，修复断开的同步 | `scan_sync_health`, `repair_sync_health`, `SyncHealthReport`, `HealthIssue`, `RepairReport` |
| `onboarding.rs` | 引导计划：为新用户生成推荐的技能安装列表 | `build_onboarding_plan`, `OnboardingPlan`, `OnboardingGroup`, `OnboardingVariant` |

## 硬规则

1. **Service 函数接收 `&Database`** 而非 `State<'_, AppState>`，便于测试时传入 `Database::new_in_memory()`
2. **跨 Repository 编排** 是放这里的唯一理由——单表 CRUD 留在 Repository 层
3. **新增 Service 必须在** `mod.rs` 中声明模块并 `pub use` 导出
4. **返回类型** 使用 `AppResult<T>` 或 `Result<T, String>`（视调用方需求）
5. **文件系统操作** 通过 `crate::filesystem` 或 `crate::utils` 进行，不直接调用 `std::fs`（路径安全、忽略列表等由 utils 统一管理）
6. **`skills/` 目录下的** `install.rs`、`maintenance.rs`、`onboarding.rs` 是 re-export shim，指向本目录。实际实现只在这里。

## 与 skills/ 目录的关系

```
skills/install.rs      → pub use crate::services::install::*;
skills/maintenance.rs  → pub use crate::services::maintenance::*;
skills/onboarding.rs   → pub use crate::services::onboarding::*;
```

`skills/` 目录的这三个文件是**向后兼容的 re-export shim**，不含实际实现。修改业务逻辑只改本目录的文件。

## 任务路由

| 任务 | 必读文件 |
|------|---------|
| 修改技能安装逻辑 | 本文件 + `install.rs` + `repositories/skills.rs` + `utils/content_hash.rs` |
| 修改同步健康检查 | 本文件 + `maintenance.rs` + `tools/adapter.rs` + `platform/mod.rs` |
| 修改引导计划 | 本文件 + `onboarding.rs` + `repositories/skills.rs` |
