# Generated: Tauri Command Map

> 这是从当前 endpoint 函数名生成的**初始候选映射**，不是已经实现的 Rust 接口。
> 规则：优先保留后端函数语义；`db/*` 使用现有函数名；不把 HTTP method 作为 command 语义的一部分。
> 生成日期：2026-08-23。主 Agent 在 Rust `contracts.rs` 和 command 注册完成后必须冻结本表。

| # | 原始方法 | 原始路径 | 候选 command | 请求体 | 响应模型 | 迁移状态 |
|---:|---|---|---|---|---|---|
| 1 | GET | `/api/db/overview` | `db_overview` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 2 | GET | `/api/db/table/{table_name}` | `db_table_data` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 3 | GET | `/api/db/table/{table_name}/columns` | `db_table_columns` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 4 | POST | `/api/db/maintenance` | `db_maintenance` | `MaintenanceRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 5 | GET | `/api/db/export` | `db_export` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 6 | POST | `/api/db/open_folder` | `db_open_folder` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 7 | POST | `/api/db/reset` | `db_reset` | `ResetRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 8 | GET | `/api/health` | `health_check` | `—` | `HealthResponse` | 待实现/待冻结 |
| 9 | GET | `/api/sync_health` | `get_sync_health` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 10 | POST | `/api/sync_health/repair` | `repair_sync_health_api` | `RepairSyncHealthRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 11 | GET | `/api/get_onboarding_plan` | `get_onboarding_plan` | `—` | `OnboardingPlan` | 待实现/待冻结 |
| 12 | POST | `/api/reorder` | `reorder` | `ReorderRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 13 | GET | `/api/pick_folder` | `pick_folder` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 14 | POST | `/api/open_settings_folder` | `open_settings_folder` | `OpenSettingsFolderRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 15 | GET | `/api/get_default_sync_tools` | `get_default_sync_tools` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 16 | POST | `/api/save_default_sync_tools` | `save_default_sync_tools` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 17 | GET | `/api/get_auto_check_update` | `get_auto_check_update` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 18 | POST | `/api/set_auto_check_update` | `set_auto_check_update` | `SetAutoCheckUpdateRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 19 | GET | `/api/get_community_repo_path` | `get_community_repo_path` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 20 | POST | `/api/set_community_repo_path` | `set_community_repo_path` | `SetCommunityRepoPathRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 21 | GET | `/api/get_custom_repo_path` | `get_custom_repo_path` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 22 | POST | `/api/set_custom_repo_path` | `set_custom_repo_path` | `SetCustomRepoPathRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 23 | POST | `/api/scan_community_repo` | `scan_community_repo` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 24 | POST | `/api/scan_all_repos` | `scan_all_repos` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 25 | POST | `/api/reset_general_settings` | `reset_general_settings` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 26 | GET | `/api/get_managed_skills` | `get_managed_skills` | `—` | `list[ManagedSkillDto]` | 待实现/待冻结 |
| 27 | POST | `/api/delete_managed_skill` | `delete_managed_skill` | `DeleteManagedSkillRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 28 | POST | `/api/update_skill_source_url` | `update_skill_source_url` | `UpdateSourceUrlRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 29 | POST | `/api/import_existing_skill` | `import_existing_skill` | `ImportExistingRequest` | `InstallResultDto` | 待实现/待冻结 |
| 30 | POST | `/api/list_local_skills_cmd` | `list_local_skills_api` | `ListLocalSkillsRequest` | `list[LocalSkillCandidate]` | 待实现/待冻结 |
| 31 | POST | `/api/install_local` | `install_local` | `InstallLocalRequest` | `InstallResultDto` | 待实现/待冻结 |
| 32 | POST | `/api/install_local_selection` | `install_local_selection` | `InstallLocalSelectionRequest` | `InstallResultDto` | 待实现/待冻结 |
| 33 | POST | `/api/retry_copy_target` | `retry_copy_target_api` | `RetryCopyTargetRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 34 | GET | `/api/list_skill_files` | `list_skill_files` | `—` | `list[SkillFileEntry]` | 待实现/待冻结 |
| 35 | GET | `/api/read_skill_file` | `read_skill_file` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 36 | POST | `/api/write_skill_file` | `write_skill_file` | `WriteSkillFileRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 37 | POST | `/api/sync_skill_dir` | `sync_skill_dir` | `SyncDirRequest` | `SyncResultDto` | 待实现/待冻结 |
| 38 | POST | `/api/sync_skill_to_tool` | `sync_skill_to_tool` | `SyncRequest` | `SyncResultDto` | 待实现/待冻结 |
| 39 | POST | `/api/unsync_skill_from_tool` | `unsync_skill_from_tool` | `UnsyncRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 40 | POST | `/api/save_recent_project` | `save_recent_project` | `SaveRecentProjectRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 41 | GET | `/api/get_recent_projects` | `get_recent_projects` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 42 | GET | `/api/get_scope_preferences` | `get_scope_preferences` | `—` | `list[ScopePreferenceDto]` | 待实现/待冻结 |
| 43 | POST | `/api/set_scope_preference` | `set_scope_preference` | `SetScopePreferenceRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 44 | GET | `/api/list_suite_sub_skills` | `list_suite_sub_skills` | `—` | `list[SuiteSubSkillDto]` | 待实现/待冻结 |
| 45 | POST | `/api/sync_suite_to_tool` | `sync_suite_to_tool` | `SyncSuiteRequest` | `list[SyncResultDto]` | 待实现/待冻结 |
| 46 | POST | `/api/unsync_suite_from_tool` | `unsync_suite_from_tool` | `UnsyncSuiteRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 47 | GET | `/api/get_tags` | `get_tags` | `—` | `list[TagWithCountDto]` | 待实现/待冻结 |
| 48 | POST | `/api/create_tag` | `create_tag` | `CreateTagRequest` | `TagDto` | 待实现/待冻结 |
| 49 | POST | `/api/rename_tag` | `rename_tag` | `RenameTagRequest` | `TagDto` | 待实现/待冻结 |
| 50 | POST | `/api/delete_tag` | `delete_tag` | `DeleteTagRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 51 | GET | `/api/get_skill_tags` | `get_skill_tags` | `—` | `list[TagDto]` | 待实现/待冻结 |
| 52 | POST | `/api/set_skill_tags` | `set_skill_tags` | `SetSkillTagsRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 53 | GET | `/api/get_untagged_skill_ids` | `get_untagged_skill_ids` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 54 | GET | `/api/tasks` | `list_tasks` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 55 | GET | `/api/tasks/{task_id}` | `get_task` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 56 | POST | `/api/tasks/{task_id}/cancel` | `cancel_task` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 57 | POST | `/api/tasks/get_tool_skills` | `start_get_tool_skills` | `—` | `TaskStartResponse` | 待实现/待冻结 |
| 58 | POST | `/api/tasks/set_community_repo_path` | `start_set_community_repo_path` | `MigrateCommunityRepoTaskRequest` | `TaskStartResponse` | 待实现/待冻结 |
| 59 | GET | `/api/get_tool_status` | `get_tool_status` | `—` | `ToolStatusResponse` | 待实现/待冻结 |
| 60 | GET | `/api/get_tool_skills` | `get_tool_skills` | `—` | `list[ToolSkillsResponse]` | 待实现/待冻结 |
| 61 | GET | `/api/get_tool_adapter_configs` | `get_tool_adapter_configs` | `—` | `list[ToolAdapterConfigResponse]` | 待实现/待冻结 |
| 62 | POST | `/api/save_tool_adapter_config` | `save_tool_adapter_config` | `SaveToolAdapterConfigRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 63 | POST | `/api/reset_tool_adapter_config` | `reset_tool_adapter_config` | `ResetToolAdapterConfigRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 64 | GET | `/api/get_tool_skills/{tool_key}` | `get_tool_skills_detail` | `—` | `ToolSkillsResponse` | 待实现/待冻结 |
| 65 | POST | `/api/delete_tool_skill` | `delete_tool_skill` | `DeleteToolSkillRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 66 | POST | `/api/open_tool_skills_dir` | `open_tool_skills_dir` | `OpenToolFolderRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 67 | POST | `/api/skill_to_community_repo` | `skill_to_community_repo` | `SyncToCommunityRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 68 | POST | `/api/clear_tool_skills` | `clear_tool_skills` | `ClearToolSkillsRequest` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 69 | GET | `/api/check_update` | `check_update` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 70 | POST | `/api/perform_update` | `do_update` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 71 | EXCEPTION_HANDLER | `Exception` | `unhandled_exception_handler` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 72 | GET | `/` | `root` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |
| 73 | GET, POST | `/api/cancel_current_operation` | `cancel_current_operation` | `—` | `未声明 response_model（需核对实际返回值）` | 待实现/待冻结 |

## 当前前端已出现的 command 字符串

这些字符串由 `frontend/src` 静态扫描得到；迁移时优先保持兼容，除非在前端 Service 层统一替换并有回归测试。

- `apiGet` → `get_default_sync_tools`（`frontend/src/features/skills/hooks/useAddSkill.ts`）
- `apiCall` → `save_default_sync_tools`（`frontend/src/features/skills/hooks/useAddSkill.ts`）
- `apiGet` → `get_managed_skills`（`frontend/src/features/skills/hooks/useSkills.ts`）
- `apiGet` → `get_tool_skills`（`frontend/src/features/skills/hooks/useSkills.ts`）
- `apiGet` → `get_tool_skills`（`frontend/src/features/skills/hooks/useSkills.ts`）
- `apiGet` → `get_managed_skills`（`frontend/src/features/skills/hooks/useSkills.ts`）
- `apiGet` → `get_tags`（`frontend/src/features/skills/hooks/useSkills.ts`）
- `apiGet` → `get_tool_status`（`frontend/src/features/skills/hooks/useSkills.ts`）
- `apiGet` → `get_tool_adapter_configs`（`frontend/src/features/tools/components/ToolsPage.tsx`）
- `apiGet` → `get_tool_skills`（`frontend/src/features/tools/components/ToolsPage.tsx`）
- `apiCall` → `skill_to_community_repo`（`frontend/src/features/tools/components/ToolsPage.tsx`）
- `apiCall` → `delete_tool_skill`（`frontend/src/features/tools/components/ToolsPage.tsx`）
- `apiCall` → `clear_tool_skills`（`frontend/src/features/tools/components/ToolsPage.tsx`）
- `apiCall` → `open_tool_skills_dir`（`frontend/src/features/tools/components/ToolsPage.tsx`）
- `apiCall` → `save_tool_adapter_config`（`frontend/src/features/tools/components/ToolsPage.tsx`）
- `apiCall` → `reset_tool_adapter_config`（`frontend/src/features/tools/components/ToolsPage.tsx`）
- `apiCall` → `reorder`（`frontend/src/lib/api.ts`）
- `apiGet` → `get_scope_preferences`（`frontend/src/lib/api.ts`）
- `apiCall` → `set_scope_preference`（`frontend/src/lib/api.ts`）
- `apiGet` → `get_skill_tags`（`frontend/src/lib/api.ts`）
- `apiGet` → `list_skill_files`（`frontend/src/lib/api.ts`）
- `apiGet` → `read_skill_file`（`frontend/src/lib/api.ts`）
- `apiCall` → `write_skill_file`（`frontend/src/lib/api.ts`）
- `apiCall` → `update_skill_source_url`（`frontend/src/lib/api.ts`）
- `apiGet` → `db/overview`（`frontend/src/lib/api.ts`）
- `apiCall` → `db/maintenance`（`frontend/src/lib/api.ts`）
- `apiCall` → `db/reset`（`frontend/src/lib/api.ts`）
- `apiCall` → `db/open_folder`（`frontend/src/lib/api.ts`）
- `apiGet` → `check_update`（`frontend/src/lib/api.ts`）
- `apiCall` → `perform_update`（`frontend/src/lib/api.ts`）
- `apiGet` → `get_auto_check_update`（`frontend/src/lib/api.ts`）
- `apiCall` → `set_auto_check_update`（`frontend/src/lib/api.ts`）
- `apiGet` → `pick_folder`（`frontend/src/lib/pickFolder.ts`）
- `apiCall` → `delete_managed_skill`（`frontend/src/services/skillService.ts`）
- `apiCall` → `set_skill_tags`（`frontend/src/services/skillService.ts`）
- `apiCall` → `create_tag`（`frontend/src/services/tagService.ts`）
- `apiCall` → `rename_tag`（`frontend/src/services/tagService.ts`）
- `apiCall` → `delete_tag`（`frontend/src/services/tagService.ts`）

## 冻结前必须确认

- 每个候选 command 都有 Rust handler、DTO、错误返回和至少一个调用方或明确标注为后端兼容保留项。
- `db/table/{table_name}` 等路径参数必须改为结构化 invoke 参数，禁止通过字符串拼接绕过路径/表白名单校验。
- 完成冻结后删除“待实现/待冻结”状态，记录 Rust 文件和测试名称。
