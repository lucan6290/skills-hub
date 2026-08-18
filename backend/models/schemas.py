"""API 请求/响应的 Pydantic 模型"""
from __future__ import annotations

from typing import Optional
from pydantic import BaseModel, Field


# ── Health ──────────────────────────────────────────────

class HealthResponse(BaseModel):
    status: str = "ok"
    version: str = Field(default="")


# ── 统一错误响应 ────────────────────────────────────────

class ErrorResponse(BaseModel):
    ok: bool = False
    code: str
    message: str
    detail: Optional[dict] = None


# ── Tool Status ─────────────────────────────────────────

class ToolInfo(BaseModel):
    key: str
    label: str
    installed: bool
    skills_dir: str
    supports_project_scope: bool
    supports_symlink: bool = True
    supports_junction: bool = True
    force_copy: bool = False


class ToolStatusResponse(BaseModel):
    tools: list[ToolInfo]
    installed: list[str]
    newly_installed: list[str]


# ── Tool Skills ─────────────────────────────────────────

class ToolSkillEntry(BaseModel):
    name: str
    path: str
    is_link: bool
    link_target: Optional[str] = None
    description: Optional[str] = None
    in_community_repo: bool = False


class ToolSkillsResponse(BaseModel):
    tool_key: str
    tool_name: str
    installed: bool
    skills_dir: Optional[str] = None
    supports_project_scope: bool
    skills: list[ToolSkillEntry]
    cached: bool = False
    scanned_at: Optional[int] = None


class ToolAdapterConfigResponse(BaseModel):
    tool_key: str
    display_name: str
    skills_dir: str
    detect_dir: str
    project_skills_dir: Optional[str] = None
    default_skills_dir: Optional[str] = None
    default_detect_dir: Optional[str] = None
    supports_symlink: bool
    supports_junction: bool
    force_copy: bool
    supports_project_scope: bool
    is_custom: bool
    has_override: bool
    sort_order: float = 0.0


class SaveToolAdapterConfigRequest(BaseModel):
    tool_key: str
    display_name: str
    skills_dir: str
    detect_dir: str
    project_skills_dir: Optional[str] = None
    supports_symlink: bool = True
    supports_junction: bool = True
    force_copy: bool = False
    supports_project_scope: bool = True
    is_custom: bool = False


class ResetToolAdapterConfigRequest(BaseModel):
    tool_key: str


class DeleteToolSkillRequest(BaseModel):
    tool_key: str
    skill_path: str


class ClearToolSkillsRequest(BaseModel):
    tool_key: str
    dry_run: bool = False


class SyncToCommunityRequest(BaseModel):
    source_path: str
    name: Optional[str] = None


class OpenToolFolderRequest(BaseModel):
    tool_key: str


# ── Skills ──────────────────────────────────────────────

class SkillTargetDto(BaseModel):
    tool: str
    scope: str
    project_path: Optional[str] = None
    mode: str
    status: str
    target_path: str
    synced_at: Optional[int] = None


class TagDto(BaseModel):
    id: int
    name: str
    sort_order: float = 0.0


class TagWithCountDto(BaseModel):
    id: int
    name: str
    skill_count: int
    updated_at: int
    sort_order: float = 0.0


class SkillUsageDto(BaseModel):
    id: int
    skill_id: str
    tool: str
    sync_count: int
    last_synced_at: Optional[int] = None
    last_viewed_at: Optional[int] = None
    view_count: int


class ManagedSkillDto(BaseModel):
    id: str
    name: str
    description: Optional[str] = None
    frontmatter_extra: Optional[str] = None
    version: Optional[str] = None
    author: Optional[str] = None
    license: Optional[str] = None
    category: Optional[str] = None
    homepage: Optional[str] = None
    skill_file_count: Optional[int] = None
    skill_dir_size: Optional[int] = None
    source_type: str
    source_ref: Optional[str] = None
    source_subpath: Optional[str] = None
    source_url: Optional[str] = None
    community_path: str
    created_at: int
    updated_at: int
    last_sync_at: Optional[int] = None
    status: str
    tags: list[TagDto] = []
    targets: list[SkillTargetDto] = []
    usage: list[SkillUsageDto] = []
    sort_order: float = 0.0
    is_suite: bool = False


# ── Tags ────────────────────────────────────────────────

class CreateTagRequest(BaseModel):
    name: str


class RenameTagRequest(BaseModel):
    tag_id: int
    name: str


class DeleteTagRequest(BaseModel):
    tag_id: int


class SetSkillTagsRequest(BaseModel):
    skill_id: str
    tag_ids: list[int]


# ── Sync ────────────────────────────────────────────────

class SyncRequest(BaseModel):
    source_path: str
    skill_id: str
    tool: str
    name: str
    overwrite: Optional[bool] = None
    overwrite_if_same_content: Optional[bool] = None
    scope: Optional[str] = None
    project_path: Optional[str] = None


class UnsyncRequest(BaseModel):
    skill_id: str
    tool: str
    scope: Optional[str] = None
    project_path: Optional[str] = None


class SyncSuiteRequest(BaseModel):
    suite_skill_id: str
    tool: str
    sub_skill_subpaths: list[str]
    scope: Optional[str] = None
    project_path: Optional[str] = None


class UnsyncSuiteRequest(BaseModel):
    suite_skill_id: str
    tool: str
    scope: Optional[str] = None
    project_path: Optional[str] = None


class SuiteSubSkillDto(BaseModel):
    name: str
    subpath: str
    description: Optional[str] = None


class SyncDirRequest(BaseModel):
    source_path: str
    target_path: str


class SyncResultDto(BaseModel):
    mode_used: str
    target_path: str


# ── Settings ────────────────────────────────────────────

class SaveRecentProjectRequest(BaseModel):
    project_path: str


class ScopePreferenceDto(BaseModel):
    skill_id: str
    scope: str
    project_paths: str


class SetScopePreferenceRequest(BaseModel):
    skill_id: str
    scope: str
    project_paths: str


class SetCommunityRepoPathRequest(BaseModel):
    path: str = Field(..., min_length=1)
    dry_run: bool = False


class SetCustomRepoPathRequest(BaseModel):
    path: str = Field(..., min_length=1)


class OpenSettingsFolderRequest(BaseModel):
    path: str = Field(..., min_length=1)


# ── Install ─────────────────────────────────────────────

class InstallResultDto(BaseModel):
    skill_id: str
    name: str
    community_path: str
    content_hash: Optional[str] = None


class LocalSkillCandidate(BaseModel):
    name: str
    description: Optional[str] = None
    subpath: str
    valid: bool
    reason: Optional[str] = None


class InstallLocalRequest(BaseModel):
    source_path: str
    name: Optional[str] = None
    source_type: str = "community"


class InstallLocalSelectionRequest(BaseModel):
    base_path: str
    subpath: str
    name: Optional[str] = None
    source_type: str = "community"


class ImportExistingRequest(BaseModel):
    source_path: str
    name: Optional[str] = None
    source_type: str = "community"


class DeleteManagedSkillRequest(BaseModel):
    skill_id: str
    dry_run: bool = False


class UpdateSourceUrlRequest(BaseModel):
    skill_id: str
    source_url: Optional[str] = None


class ListLocalSkillsRequest(BaseModel):
    base_path: str


class RetryCopyTargetRequest(BaseModel):
    skill_id: str
    tool: str


# ── Skill files ─────────────────────────────────────────

class SkillFileEntry(BaseModel):
    path: str
    size: int


class WriteSkillFileRequest(BaseModel):
    skill_id: str
    file_path: str
    content: str


# ── Onboarding (Phase 3 预定义) ─────────────────────────

class OnboardingVariant(BaseModel):
    tool: str
    name: str
    path: str
    fingerprint: Optional[str] = None
    is_link: bool
    link_target: Optional[str] = None


class OnboardingGroup(BaseModel):
    name: str
    variants: list[OnboardingVariant]
    has_conflict: bool


class OnboardingPlan(BaseModel):
    total_tools_scanned: int
    total_skills_found: int
    groups: list[OnboardingGroup]


# ── Reorder (批量排序) ──────────────────────────────────

class ReorderItem(BaseModel):
    id: str
    sort_order: float


class ReorderRequest(BaseModel):
    entity: str
    items: list[ReorderItem]


# ── Tasks ───────────────────────────────────────────────

class TaskStartResponse(BaseModel):
    task_id: str
    status: str


class MigrateCommunityRepoTaskRequest(BaseModel):
    path: str
    dry_run: bool = False


# ── Database 管理 ───────────────────────────────────────

class TableQueryRequest(BaseModel):
    table: str
    page: int = Field(1, ge=1)
    page_size: int = Field(50, ge=1, le=500)
    sort_col: Optional[str] = None
    sort_dir: str = Field("asc", pattern="^(asc|desc)$")
    filter_text: Optional[str] = None


class MaintenanceRequest(BaseModel):
    action: str = Field(..., pattern="^(vacuum|analyze|clear_cache|clear_discovered|integrity_check)$")


class ResetRequest(BaseModel):
    confirm_text: str = Field(..., min_length=1)


# ── Sync Health ─────────────────────────────────────────

class RepairSyncHealthRequest(BaseModel):
    dry_run: bool = True


# ── Update ──────────────────────────────────────────────

class CheckUpdateResponse(BaseModel):
    current_version: str
    latest_version: str
    update_available: bool
    install_mode: str
    release_url: str
    release_notes: str = ""
    download_urls: dict = {}
    changelog_url: str = ""
    error: Optional[str] = None


class SetAutoCheckUpdateRequest(BaseModel):
    enabled: bool


class PerformUpdateResponse(BaseModel):
    ok: bool
    message: str
