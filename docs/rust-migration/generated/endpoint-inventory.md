# Generated: Endpoint Inventory

> 生成方式：对 `backend/api/**/*.py` 和 `backend/main.py` 进行 Python AST 静态提取。
> 生成日期：2026-08-23。此文件只记录源码中实际出现的装饰器；函数内部的业务行为和错误映射仍需实施 Agent 阅读源码与测试确认。

当前静态提取到 **73** 个 endpoint 定义。

| # | 方法 | 原始路径 | Python 函数 | 响应模型 | 请求体模型 | 源文件:行 |
|---:|---|---|---|---|---|---|
| 1 | GET | `/api/db/overview` | `db_overview` | `—` | `—` | `backend/api/database.py:130 |
| 2 | GET | `/api/db/table/{table_name}` | `db_table_data` | `—` | `—` | `backend/api/database.py:199 |
| 3 | GET | `/api/db/table/{table_name}/columns` | `db_table_columns` | `—` | `—` | `backend/api/database.py:287 |
| 4 | POST | `/api/db/maintenance` | `db_maintenance` | `—` | `MaintenanceRequest` | `backend/api/database.py:301 |
| 5 | GET | `/api/db/export` | `db_export` | `—` | `—` | `backend/api/database.py:349 |
| 6 | POST | `/api/db/open_folder` | `db_open_folder` | `—` | `—` | `backend/api/database.py:382 |
| 7 | POST | `/api/db/reset` | `db_reset` | `—` | `ResetRequest` | `backend/api/database.py:401 |
| 8 | GET | `/api/health` | `health_check` | `HealthResponse` | `—` | `backend/api/health.py:17 |
| 9 | GET | `/api/sync_health` | `get_sync_health` | `—` | `—` | `backend/api/maintenance.py:24 |
| 10 | POST | `/api/sync_health/repair` | `repair_sync_health_api` | `—` | `RepairSyncHealthRequest` | `backend/api/maintenance.py:39 |
| 11 | GET | `/api/get_onboarding_plan` | `get_onboarding_plan` | `OnboardingPlan` | `—` | `backend/api/onboarding.py:26 |
| 12 | POST | `/api/reorder` | `reorder` | `—` | `ReorderRequest` | `backend/api/reorder.py:21 |
| 13 | GET | `/api/pick_folder` | `pick_folder` | `—` | `—` | `backend/api/settings.py:51 |
| 14 | POST | `/api/open_settings_folder` | `open_settings_folder` | `—` | `OpenSettingsFolderRequest` | `backend/api/settings.py:87 |
| 15 | GET | `/api/get_default_sync_tools` | `get_default_sync_tools` | `—` | `—` | `backend/api/settings.py:108 |
| 16 | POST | `/api/save_default_sync_tools` | `save_default_sync_tools` | `—` | `—` | `backend/api/settings.py:123 |
| 17 | GET | `/api/get_auto_check_update` | `get_auto_check_update` | `—` | `—` | `backend/api/settings.py:138 |
| 18 | POST | `/api/set_auto_check_update` | `set_auto_check_update` | `—` | `SetAutoCheckUpdateRequest` | `backend/api/settings.py:150 |
| 19 | GET | `/api/get_community_repo_path` | `get_community_repo_path` | `—` | `—` | `backend/api/settings.py:162 |
| 20 | POST | `/api/set_community_repo_path` | `set_community_repo_path` | `—` | `SetCommunityRepoPathRequest` | `backend/api/settings.py:174 |
| 21 | GET | `/api/get_custom_repo_path` | `get_custom_repo_path` | `—` | `—` | `backend/api/settings.py:197 |
| 22 | POST | `/api/set_custom_repo_path` | `set_custom_repo_path` | `—` | `SetCustomRepoPathRequest` | `backend/api/settings.py:208 |
| 23 | POST | `/api/scan_community_repo` | `scan_community_repo` | `—` | `—` | `backend/api/settings.py:226 |
| 24 | POST | `/api/scan_all_repos` | `scan_all_repos` | `—` | `—` | `backend/api/settings.py:238 |
| 25 | POST | `/api/reset_general_settings` | `reset_general_settings` | `—` | `—` | `backend/api/settings.py:250 |
| 26 | GET | `/api/get_managed_skills` | `get_managed_skills` | `list[ManagedSkillDto]` | `—` | `backend/api/skills/crud.py:127 |
| 27 | POST | `/api/delete_managed_skill` | `delete_managed_skill` | `—` | `DeleteManagedSkillRequest` | `backend/api/skills/crud.py:150 |
| 28 | POST | `/api/update_skill_source_url` | `update_skill_source_url` | `—` | `UpdateSourceUrlRequest` | `backend/api/skills/crud.py:224 |
| 29 | POST | `/api/import_existing_skill` | `import_existing_skill` | `InstallResultDto` | `ImportExistingRequest` | `backend/api/skills/crud.py:245 |
| 30 | POST | `/api/list_local_skills_cmd` | `list_local_skills_api` | `list[LocalSkillCandidate]` | `ListLocalSkillsRequest` | `backend/api/skills/crud.py:269 |
| 31 | POST | `/api/install_local` | `install_local` | `InstallResultDto` | `InstallLocalRequest` | `backend/api/skills/crud.py:287 |
| 32 | POST | `/api/install_local_selection` | `install_local_selection` | `InstallResultDto` | `InstallLocalSelectionRequest` | `backend/api/skills/crud.py:304 |
| 33 | POST | `/api/retry_copy_target` | `retry_copy_target_api` | `—` | `RetryCopyTargetRequest` | `backend/api/skills/crud.py:343 |
| 34 | GET | `/api/list_skill_files` | `list_skill_files` | `list[SkillFileEntry]` | `—` | `backend/api/skills/files.py:37 |
| 35 | GET | `/api/read_skill_file` | `read_skill_file` | `—` | `—` | `backend/api/skills/files.py:57 |
| 36 | POST | `/api/write_skill_file` | `write_skill_file` | `—` | `WriteSkillFileRequest` | `backend/api/skills/files.py:77 |
| 37 | POST | `/api/sync_skill_dir` | `sync_skill_dir` | `SyncResultDto` | `SyncDirRequest` | `backend/api/skills/sync.py:56 |
| 38 | POST | `/api/sync_skill_to_tool` | `sync_skill_to_tool` | `SyncResultDto` | `SyncRequest` | `backend/api/skills/sync.py:77 |
| 39 | POST | `/api/unsync_skill_from_tool` | `unsync_skill_from_tool` | `—` | `UnsyncRequest` | `backend/api/skills/sync.py:91 |
| 40 | POST | `/api/save_recent_project` | `save_recent_project` | `—` | `SaveRecentProjectRequest` | `backend/api/skills/sync.py:153 |
| 41 | GET | `/api/get_recent_projects` | `get_recent_projects` | `—` | `—` | `backend/api/skills/sync.py:170 |
| 42 | GET | `/api/get_scope_preferences` | `get_scope_preferences` | `list[ScopePreferenceDto]` | `—` | `backend/api/skills/sync.py:182 |
| 43 | POST | `/api/set_scope_preference` | `set_scope_preference` | `—` | `SetScopePreferenceRequest` | `backend/api/skills/sync.py:201 |
| 44 | GET | `/api/list_suite_sub_skills` | `list_suite_sub_skills` | `list[SuiteSubSkillDto]` | `—` | `backend/api/skills/sync.py:214 |
| 45 | POST | `/api/sync_suite_to_tool` | `sync_suite_to_tool` | `list[SyncResultDto]` | `SyncSuiteRequest` | `backend/api/skills/sync.py:230 |
| 46 | POST | `/api/unsync_suite_from_tool` | `unsync_suite_from_tool` | `—` | `UnsyncSuiteRequest` | `backend/api/skills/sync.py:251 |
| 47 | GET | `/api/get_tags` | `get_tags` | `list[TagWithCountDto]` | `—` | `backend/api/tags.py:28 |
| 48 | POST | `/api/create_tag` | `create_tag` | `TagDto` | `CreateTagRequest` | `backend/api/tags.py:47 |
| 49 | POST | `/api/rename_tag` | `rename_tag` | `TagDto` | `RenameTagRequest` | `backend/api/tags.py:63 |
| 50 | POST | `/api/delete_tag` | `delete_tag` | `—` | `DeleteTagRequest` | `backend/api/tags.py:78 |
| 51 | GET | `/api/get_skill_tags` | `get_skill_tags` | `list[TagDto]` | `—` | `backend/api/tags.py:94 |
| 52 | POST | `/api/set_skill_tags` | `set_skill_tags` | `—` | `SetSkillTagsRequest` | `backend/api/tags.py:108 |
| 53 | GET | `/api/get_untagged_skill_ids` | `get_untagged_skill_ids` | `—` | `—` | `backend/api/tags.py:127 |
| 54 | GET | `/api/tasks` | `list_tasks` | `—` | `—` | `backend/api/tasks.py:18 |
| 55 | GET | `/api/tasks/{task_id}` | `get_task` | `—` | `—` | `backend/api/tasks.py:29 |
| 56 | POST | `/api/tasks/{task_id}/cancel` | `cancel_task` | `—` | `—` | `backend/api/tasks.py:43 |
| 57 | POST | `/api/tasks/get_tool_skills` | `start_get_tool_skills` | `TaskStartResponse` | `—` | `backend/api/tasks.py:57 |
| 58 | POST | `/api/tasks/set_community_repo_path` | `start_set_community_repo_path` | `TaskStartResponse` | `MigrateCommunityRepoTaskRequest` | `backend/api/tasks.py:78 |
| 59 | GET | `/api/get_tool_status` | `get_tool_status` | `ToolStatusResponse` | `—` | `backend/api/tools/status.py:27 |
| 60 | GET | `/api/get_tool_skills` | `get_tool_skills` | `list[ToolSkillsResponse]` | `—` | `backend/api/tools/tool_skills.py:138 |
| 61 | GET | `/api/get_tool_adapter_configs` | `get_tool_adapter_configs` | `list[ToolAdapterConfigResponse]` | `—` | `backend/api/tools/tool_skills.py:155 |
| 62 | POST | `/api/save_tool_adapter_config` | `save_tool_adapter_config` | `—` | `SaveToolAdapterConfigRequest` | `backend/api/tools/tool_skills.py:179 |
| 63 | POST | `/api/reset_tool_adapter_config` | `reset_tool_adapter_config` | `—` | `ResetToolAdapterConfigRequest` | `backend/api/tools/tool_skills.py:218 |
| 64 | GET | `/api/get_tool_skills/{tool_key}` | `get_tool_skills_detail` | `ToolSkillsResponse` | `—` | `backend/api/tools/tool_skills.py:240 |
| 65 | POST | `/api/delete_tool_skill` | `delete_tool_skill` | `—` | `DeleteToolSkillRequest` | `backend/api/tools/tool_skills.py:263 |
| 66 | POST | `/api/open_tool_skills_dir` | `open_tool_skills_dir` | `—` | `OpenToolFolderRequest` | `backend/api/tools/tool_skills.py:309 |
| 67 | POST | `/api/skill_to_community_repo` | `skill_to_community_repo` | `—` | `SyncToCommunityRequest` | `backend/api/tools/tool_skills.py:334 |
| 68 | POST | `/api/clear_tool_skills` | `clear_tool_skills` | `—` | `ClearToolSkillsRequest` | `backend/api/tools/tool_skills.py:405 |
| 69 | GET | `/api/check_update` | `check_update` | `—` | `—` | `backend/api/update.py:32 |
| 70 | POST | `/api/perform_update` | `do_update` | `—` | `—` | `backend/api/update.py:40 |
| 71 | EXCEPTION_HANDLER | `Exception` | `unhandled_exception_handler` | `—` | `—` | `backend/main.py:49 |
| 72 | GET | `/` | `root` | `—` | `—` | `backend/main.py:70 |
| 73 | GET, POST | `/api/cancel_current_operation` | `cancel_current_operation` | `—` | `—` | `backend/main.py:75 |

## 读取和更新规则

- 如果路由装饰器、函数签名或 response model 发生变化，重新生成本文件。
- `summary`、状态码、异常和前端使用情况不能仅凭本表推断；实施 Agent 必须在 `00-baseline-and-contract.md` 的步骤中补齐证据。
- 不得将本文件中的“原始路径”直接当作 Tauri command 名；command 以 `command-map.md` 冻结结果为准。
