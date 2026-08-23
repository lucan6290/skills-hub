# Generated: Frontend Call Sites

> 来源：对 `frontend/src/**/*.{ts,tsx}` 静态扫描 `apiCall`、`apiGet` 的直接字符串参数。
> 生成日期：2026-08-23。动态模板字符串和间接封装必须在人工复核时补齐。

## `frontend/src/features/skills/hooks/useAddSkill.ts`

| API 函数 | command/path 字符串 | 源行 |
|---|---|---:|
| `apiGet` | `get_default_sync_tools` | 72 |
| `apiCall` | `save_default_sync_tools` | 232 |

## `frontend/src/features/skills/hooks/useSkills.ts`

| API 函数 | command/path 字符串 | 源行 |
|---|---|---:|
| `apiGet` | `get_managed_skills` | 44 |
| `apiGet` | `get_tool_skills` | 83 |
| `apiGet` | `get_tool_skills` | 110 |
| `apiGet` | `get_managed_skills` | 112 |
| `apiGet` | `get_tags` | 113 |
| `apiGet` | `get_tool_status` | 114 |

## `frontend/src/features/tools/components/ToolsPage.tsx`

| API 函数 | command/path 字符串 | 源行 |
|---|---|---:|
| `apiGet` | `get_tool_adapter_configs` | 82 |
| `apiGet` | `get_tool_skills` | 96 |
| `apiCall` | `skill_to_community_repo` | 124 |
| `apiCall` | `delete_tool_skill` | 144 |
| `apiCall` | `clear_tool_skills` | 158 |
| `apiCall` | `open_tool_skills_dir` | 170 |
| `apiCall` | `save_tool_adapter_config` | 200 |
| `apiCall` | `reset_tool_adapter_config` | 231 |

## `frontend/src/lib/api.ts`

| API 函数 | command/path 字符串 | 源行 |
|---|---|---:|
| `apiCall` | `reorder` | 90 |
| `apiGet` | `get_scope_preferences` | 94 |
| `apiCall` | `set_scope_preference` | 102 |
| `apiGet` | `get_skill_tags` | 107 |
| `apiGet` | `list_skill_files` | 112 |
| `apiGet` | `read_skill_file` | 117 |
| `apiCall` | `write_skill_file` | 122 |
| `apiCall` | `update_skill_source_url` | 127 |
| `apiGet` | `db/overview` | 183 |
| `apiCall` | `db/maintenance` | 200 |
| `apiCall` | `db/reset` | 204 |
| `apiCall` | `db/open_folder` | 212 |
| `apiGet` | `check_update` | 239 |
| `apiCall` | `perform_update` | 243 |
| `apiGet` | `get_auto_check_update` | 247 |
| `apiCall` | `set_auto_check_update` | 251 |

## `frontend/src/lib/pickFolder.ts`

| API 函数 | command/path 字符串 | 源行 |
|---|---|---:|
| `apiGet` | `pick_folder` | 9 |

## `frontend/src/services/skillService.ts`

| API 函数 | command/path 字符串 | 源行 |
|---|---|---:|
| `apiCall` | `delete_managed_skill` | 5 |
| `apiCall` | `set_skill_tags` | 9 |

## `frontend/src/services/tagService.ts`

| API 函数 | command/path 字符串 | 源行 |
|---|---|---:|
| `apiCall` | `create_tag` | 5 |
| `apiCall` | `rename_tag` | 9 |
| `apiCall` | `delete_tag` | 13 |

## 人工复核项

- `frontend/src/lib/api.ts` 中 `db/table/${table_name}`、`getDbExportUrl()` 等动态路径必须迁移为结构化 command 参数或受控下载接口。
- 检查所有 `fetch(`、`/api/`、`API_BASE` 和第三方封装，避免漏掉非 `apiCall/apiGet` 的 HTTP 调用。
- React 组件不得在迁移后直接写 command 字符串；统一经 Service/transport 层调用。
