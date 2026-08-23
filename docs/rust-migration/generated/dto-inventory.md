# Generated: DTO Inventory

> 来源：`backend/models/schemas.py` 的 AST 静态提取。类型、默认值和 `Field(...)` 表达式按源码记录；Rust/TypeScript 类型映射必须在实现时用编译器和序列化测试确认。
> 生成日期：2026-08-23。

共提取 **60** 个包含字段的 model。

## `HealthResponse`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:10`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `status` | `str` | `'ok'` | 11 |
| `version` | `str` | `Field(default='')` | 12 |

## `ErrorResponse`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:17`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `ok` | `bool` | `False` | 18 |
| `code` | `str` | `required` | 19 |
| `message` | `str` | `required` | 20 |
| `detail` | `Optional[dict]` | `None` | 21 |

## `ToolInfo`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:26`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `key` | `str` | `required` | 27 |
| `label` | `str` | `required` | 28 |
| `installed` | `bool` | `required` | 29 |
| `skills_dir` | `str` | `required` | 30 |
| `supports_project_scope` | `bool` | `required` | 31 |
| `supports_symlink` | `bool` | `True` | 32 |
| `supports_junction` | `bool` | `True` | 33 |
| `force_copy` | `bool` | `False` | 34 |

## `ToolStatusResponse`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:37`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `tools` | `list[ToolInfo]` | `required` | 38 |
| `installed` | `list[str]` | `required` | 39 |
| `newly_installed` | `list[str]` | `required` | 40 |

## `ToolSkillEntry`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:45`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `name` | `str` | `required` | 46 |
| `path` | `str` | `required` | 47 |
| `is_link` | `bool` | `required` | 48 |
| `link_target` | `Optional[str]` | `None` | 49 |
| `description` | `Optional[str]` | `None` | 50 |
| `in_community_repo` | `bool` | `False` | 51 |

## `ToolSkillsResponse`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:54`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `tool_key` | `str` | `required` | 55 |
| `tool_name` | `str` | `required` | 56 |
| `installed` | `bool` | `required` | 57 |
| `skills_dir` | `Optional[str]` | `None` | 58 |
| `supports_project_scope` | `bool` | `required` | 59 |
| `skills` | `list[ToolSkillEntry]` | `required` | 60 |
| `cached` | `bool` | `False` | 61 |
| `scanned_at` | `Optional[int]` | `None` | 62 |

## `ToolAdapterConfigResponse`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:65`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `tool_key` | `str` | `required` | 66 |
| `display_name` | `str` | `required` | 67 |
| `skills_dir` | `str` | `required` | 68 |
| `detect_dir` | `str` | `required` | 69 |
| `project_skills_dir` | `Optional[str]` | `None` | 70 |
| `default_skills_dir` | `Optional[str]` | `None` | 71 |
| `default_detect_dir` | `Optional[str]` | `None` | 72 |
| `supports_symlink` | `bool` | `required` | 73 |
| `supports_junction` | `bool` | `required` | 74 |
| `force_copy` | `bool` | `required` | 75 |
| `supports_project_scope` | `bool` | `required` | 76 |
| `is_custom` | `bool` | `required` | 77 |
| `has_override` | `bool` | `required` | 78 |
| `sort_order` | `float` | `0.0` | 79 |

## `SaveToolAdapterConfigRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:82`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `tool_key` | `str` | `required` | 83 |
| `display_name` | `str` | `required` | 84 |
| `skills_dir` | `str` | `required` | 85 |
| `detect_dir` | `str` | `required` | 86 |
| `project_skills_dir` | `Optional[str]` | `None` | 87 |
| `supports_symlink` | `bool` | `True` | 88 |
| `supports_junction` | `bool` | `True` | 89 |
| `force_copy` | `bool` | `False` | 90 |
| `supports_project_scope` | `bool` | `True` | 91 |
| `is_custom` | `bool` | `False` | 92 |

## `ResetToolAdapterConfigRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:95`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `tool_key` | `str` | `required` | 96 |

## `DeleteToolSkillRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:99`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `tool_key` | `str` | `required` | 100 |
| `skill_path` | `str` | `required` | 101 |

## `ClearToolSkillsRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:104`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `tool_key` | `str` | `required` | 105 |
| `dry_run` | `bool` | `False` | 106 |

## `SyncToCommunityRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:109`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `source_path` | `str` | `required` | 110 |
| `name` | `Optional[str]` | `None` | 111 |

## `OpenToolFolderRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:114`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `tool_key` | `str` | `required` | 115 |

## `SkillTargetDto`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:120`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `tool` | `str` | `required` | 121 |
| `scope` | `str` | `required` | 122 |
| `project_path` | `Optional[str]` | `None` | 123 |
| `mode` | `str` | `required` | 124 |
| `status` | `str` | `required` | 125 |
| `target_path` | `str` | `required` | 126 |
| `synced_at` | `Optional[int]` | `None` | 127 |
| `suite_skill_id` | `Optional[str]` | `None` | 128 |

## `TagDto`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:131`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `id` | `int` | `required` | 132 |
| `name` | `str` | `required` | 133 |
| `sort_order` | `float` | `0.0` | 134 |

## `TagWithCountDto`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:137`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `id` | `int` | `required` | 138 |
| `name` | `str` | `required` | 139 |
| `skill_count` | `int` | `required` | 140 |
| `updated_at` | `int` | `required` | 141 |
| `sort_order` | `float` | `0.0` | 142 |

## `SkillUsageDto`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:145`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `id` | `int` | `required` | 146 |
| `skill_id` | `str` | `required` | 147 |
| `tool` | `str` | `required` | 148 |
| `sync_count` | `int` | `required` | 149 |
| `last_synced_at` | `Optional[int]` | `None` | 150 |
| `last_viewed_at` | `Optional[int]` | `None` | 151 |
| `view_count` | `int` | `required` | 152 |

## `ManagedSkillDto`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:155`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `id` | `str` | `required` | 156 |
| `name` | `str` | `required` | 157 |
| `description` | `Optional[str]` | `None` | 158 |
| `frontmatter_extra` | `Optional[str]` | `None` | 159 |
| `version` | `Optional[str]` | `None` | 160 |
| `author` | `Optional[str]` | `None` | 161 |
| `license` | `Optional[str]` | `None` | 162 |
| `category` | `Optional[str]` | `None` | 163 |
| `homepage` | `Optional[str]` | `None` | 164 |
| `skill_file_count` | `Optional[int]` | `None` | 165 |
| `skill_dir_size` | `Optional[int]` | `None` | 166 |
| `source_type` | `str` | `required` | 167 |
| `source_ref` | `Optional[str]` | `None` | 168 |
| `source_subpath` | `Optional[str]` | `None` | 169 |
| `source_url` | `Optional[str]` | `None` | 170 |
| `community_path` | `str` | `required` | 171 |
| `created_at` | `int` | `required` | 172 |
| `updated_at` | `int` | `required` | 173 |
| `last_sync_at` | `Optional[int]` | `None` | 174 |
| `status` | `str` | `required` | 175 |
| `tags` | `list[TagDto]` | `[]` | 176 |
| `targets` | `list[SkillTargetDto]` | `[]` | 177 |
| `usage` | `list[SkillUsageDto]` | `[]` | 178 |
| `sort_order` | `float` | `0.0` | 179 |
| `is_suite` | `bool` | `False` | 180 |

## `CreateTagRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:185`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `name` | `str` | `required` | 186 |

## `RenameTagRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:189`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `tag_id` | `int` | `required` | 190 |
| `name` | `str` | `required` | 191 |

## `DeleteTagRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:194`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `tag_id` | `int` | `required` | 195 |

## `SetSkillTagsRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:198`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `skill_id` | `str` | `required` | 199 |
| `tag_ids` | `list[int]` | `required` | 200 |

## `SyncRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:205`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `source_path` | `str` | `required` | 206 |
| `skill_id` | `str` | `required` | 207 |
| `tool` | `str` | `required` | 208 |
| `name` | `str` | `required` | 209 |
| `overwrite` | `Optional[bool]` | `None` | 210 |
| `overwrite_if_same_content` | `Optional[bool]` | `None` | 211 |
| `scope` | `Optional[str]` | `None` | 212 |
| `project_path` | `Optional[str]` | `None` | 213 |

## `UnsyncRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:216`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `skill_id` | `str` | `required` | 217 |
| `tool` | `str` | `required` | 218 |
| `scope` | `Optional[str]` | `None` | 219 |
| `project_path` | `Optional[str]` | `None` | 220 |

## `SyncSuiteRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:223`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `suite_skill_id` | `str` | `required` | 224 |
| `tool` | `str` | `required` | 225 |
| `sub_skill_subpaths` | `list[str]` | `required` | 226 |
| `scope` | `Optional[str]` | `None` | 227 |
| `project_path` | `Optional[str]` | `None` | 228 |

## `UnsyncSuiteRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:231`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `suite_skill_id` | `str` | `required` | 232 |
| `tool` | `str` | `required` | 233 |
| `scope` | `Optional[str]` | `None` | 234 |
| `project_path` | `Optional[str]` | `None` | 235 |

## `SuiteSubSkillDto`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:238`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `name` | `str` | `required` | 239 |
| `subpath` | `str` | `required` | 240 |
| `description` | `Optional[str]` | `None` | 241 |

## `SyncDirRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:244`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `source_path` | `str` | `required` | 245 |
| `target_path` | `str` | `required` | 246 |

## `SyncResultDto`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:249`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `mode_used` | `str` | `required` | 250 |
| `target_path` | `str` | `required` | 251 |

## `SaveRecentProjectRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:256`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `project_path` | `str` | `required` | 257 |

## `ScopePreferenceDto`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:260`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `skill_id` | `str` | `required` | 261 |
| `scope` | `str` | `required` | 262 |
| `project_paths` | `str` | `required` | 263 |

## `SetScopePreferenceRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:266`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `skill_id` | `str` | `required` | 267 |
| `scope` | `str` | `required` | 268 |
| `project_paths` | `str` | `required` | 269 |

## `SetCommunityRepoPathRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:272`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `path` | `str` | `Field(..., min_length=1)` | 273 |
| `dry_run` | `bool` | `False` | 274 |

## `SetCustomRepoPathRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:277`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `path` | `str` | `Field(..., min_length=1)` | 278 |

## `OpenSettingsFolderRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:281`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `path` | `str` | `Field(..., min_length=1)` | 282 |

## `InstallResultDto`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:287`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `skill_id` | `str` | `required` | 288 |
| `name` | `str` | `required` | 289 |
| `community_path` | `str` | `required` | 290 |
| `content_hash` | `Optional[str]` | `None` | 291 |

## `LocalSkillCandidate`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:294`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `name` | `str` | `required` | 295 |
| `description` | `Optional[str]` | `None` | 296 |
| `subpath` | `str` | `required` | 297 |
| `valid` | `bool` | `required` | 298 |
| `reason` | `Optional[str]` | `None` | 299 |

## `InstallLocalRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:302`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `source_path` | `str` | `required` | 303 |
| `name` | `Optional[str]` | `None` | 304 |
| `source_type` | `str` | `'community'` | 305 |

## `InstallLocalSelectionRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:308`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `base_path` | `str` | `required` | 309 |
| `subpath` | `str` | `required` | 310 |
| `name` | `Optional[str]` | `None` | 311 |
| `source_type` | `str` | `'community'` | 312 |

## `ImportExistingRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:315`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `source_path` | `str` | `required` | 316 |
| `name` | `Optional[str]` | `None` | 317 |
| `source_type` | `str` | `'community'` | 318 |

## `DeleteManagedSkillRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:321`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `skill_id` | `str` | `required` | 322 |
| `dry_run` | `bool` | `False` | 323 |

## `UpdateSourceUrlRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:326`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `skill_id` | `str` | `required` | 327 |
| `source_url` | `Optional[str]` | `None` | 328 |

## `ListLocalSkillsRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:331`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `base_path` | `str` | `required` | 332 |

## `RetryCopyTargetRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:335`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `skill_id` | `str` | `required` | 336 |
| `tool` | `str` | `required` | 337 |

## `SkillFileEntry`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:342`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `path` | `str` | `required` | 343 |
| `size` | `int` | `required` | 344 |

## `WriteSkillFileRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:347`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `skill_id` | `str` | `required` | 348 |
| `file_path` | `str` | `required` | 349 |
| `content` | `str` | `required` | 350 |

## `OnboardingVariant`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:355`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `tool` | `str` | `required` | 356 |
| `name` | `str` | `required` | 357 |
| `path` | `str` | `required` | 358 |
| `fingerprint` | `Optional[str]` | `None` | 359 |
| `is_link` | `bool` | `required` | 360 |
| `link_target` | `Optional[str]` | `None` | 361 |

## `OnboardingGroup`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:364`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `name` | `str` | `required` | 365 |
| `variants` | `list[OnboardingVariant]` | `required` | 366 |
| `has_conflict` | `bool` | `required` | 367 |

## `OnboardingPlan`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:370`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `total_tools_scanned` | `int` | `required` | 371 |
| `total_skills_found` | `int` | `required` | 372 |
| `groups` | `list[OnboardingGroup]` | `required` | 373 |

## `ReorderItem`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:378`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `id` | `str` | `required` | 379 |
| `sort_order` | `float` | `required` | 380 |

## `ReorderRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:383`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `entity` | `str` | `required` | 384 |
| `items` | `list[ReorderItem]` | `required` | 385 |

## `TaskStartResponse`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:390`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `task_id` | `str` | `required` | 391 |
| `status` | `str` | `required` | 392 |

## `MigrateCommunityRepoTaskRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:395`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `path` | `str` | `required` | 396 |
| `dry_run` | `bool` | `False` | 397 |

## `TableQueryRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:402`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `table` | `str` | `required` | 403 |
| `page` | `int` | `Field(1, ge=1)` | 404 |
| `page_size` | `int` | `Field(50, ge=1, le=500)` | 405 |
| `sort_col` | `Optional[str]` | `None` | 406 |
| `sort_dir` | `str` | `Field('asc', pattern='^(asc\|desc)$')` | 407 |
| `filter_text` | `Optional[str]` | `None` | 408 |

## `MaintenanceRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:411`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `action` | `str` | `Field(..., pattern='^(vacuum\|analyze\|clear_cache\|clear_discovered\|integrity_check)$')` | 412 |

## `ResetRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:415`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `confirm_text` | `str` | `Field(..., min_length=1)` | 416 |

## `RepairSyncHealthRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:421`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `dry_run` | `bool` | `True` | 422 |

## `CheckUpdateResponse`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:427`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `current_version` | `str` | `required` | 428 |
| `latest_version` | `str` | `required` | 429 |
| `update_available` | `bool` | `required` | 430 |
| `install_mode` | `str` | `required` | 431 |
| `release_url` | `str` | `required` | 432 |
| `release_notes` | `str` | `''` | 433 |
| `download_urls` | `dict` | `{}` | 434 |
| `changelog_url` | `str` | `''` | 435 |
| `error` | `Optional[str]` | `None` | 436 |

## `SetAutoCheckUpdateRequest`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:439`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `enabled` | `bool` | `required` | 440 |

## `PerformUpdateResponse`

- 基类：`BaseModel`
- 源位置：`backend/models/schemas.py:443`

| 字段 | Python 类型 | 默认/约束表达式 | 字段源行 |
|---|---|---|---:|
| `ok` | `bool` | `required` | 444 |
| `message` | `str` | `required` | 445 |

## 迁移约束

- 跨 Tauri 边界字段保持 `snake_case`。
- `Optional[...]`、`list[...]`、字典类型和 `Field(...)` 约束不能只按名称猜测；必须分别写 Rust serde DTO、TypeScript DTO 和边界测试。
- Pydantic model 的字段清单不代表所有 endpoint 的实际返回形状；以 endpoint 函数和现有前端类型共同核对。
