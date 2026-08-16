/**
 * Skills Hub 前端 API 适配层
 * 替代 invokeTauri，通过 HTTP 调用 Python 后端
 *
 * 命名规范：所有参数直接使用 snake_case，与后端字段名完全一致，禁止转换。
 */
import type { ManagedSkill } from '@/features/skills'

const API_BASE = ''

export async function apiCall<T = unknown>(
  command: string,
  params?: Record<string, unknown>,
): Promise<T> {
  const body = Array.isArray(params)
    ? JSON.stringify(params)
    : params
      ? JSON.stringify(params)
      : undefined

  const res = await fetch(`${API_BASE}/api/${command}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body,
  })

  if (!res.ok) {
    const errBody = await res.json().catch(() => ({}))
    const message =
      errBody.detail || errBody.message || `API error ${res.status}`
    throw new Error(message)
  }

  return res.json() as Promise<T>
}

/** GET 请求变体 */
export async function apiGet<T = unknown>(
  command: string,
  params?: Record<string, unknown>,
): Promise<T> {
  const query = params
    ? `?${new URLSearchParams(
        Object.fromEntries(
          Object.entries(params)
            .filter(([, value]) => value !== undefined && value !== null)
            .map(([key, value]) => [key, String(value)]),
        ),
      )}`
    : ''
  const res = await fetch(`${API_BASE}/api/${command}${query}`)

  if (!res.ok) {
    const errBody = await res.json().catch(() => ({}))
    const message =
      errBody.detail || errBody.message || `API error ${res.status}`
    throw new Error(message)
  }

  return res.json() as Promise<T>
}

export interface ScopePreferenceDto {
  skill_id: string
  scope: string
  project_paths: string
}

export interface TagDto {
  id: number
  name: string
  sort_order: number
}

export interface SkillFileEntry {
  path: string
  size: number
}

export interface ReorderItem {
  id: string
  sort_order: number
}

/** 批量更新排序 */
export async function reorder(
  entity: 'skills' | 'tags' | 'tools',
  items: ReorderItem[],
): Promise<void> {
  await apiCall('reorder', { entity, items })
}

export async function fetchScopePreferences(): Promise<ScopePreferenceDto[]> {
  return apiGet<ScopePreferenceDto[]>('get_scope_preferences')
}

export async function saveScopePreference(
  skill_id: string,
  scope: string,
  project_paths: string,
): Promise<void> {
  await apiCall('set_scope_preference', { skill_id, scope, project_paths })
}

/** 获取 skill 的标签列表 */
export async function fetchSkillTags(skill_id: string): Promise<TagDto[]> {
  return apiGet<TagDto[]>('get_skill_tags', { skill_id })
}

/** 列出 skill 的文件 */
export async function fetchSkillFiles(skill_id: string): Promise<SkillFileEntry[]> {
  return apiGet<SkillFileEntry[]>('list_skill_files', { skill_id })
}

/** 读取 skill 的单个文件内容 */
export async function fetchSkillFileContent(skill_id: string, file_path: string): Promise<string> {
  return apiGet<string>('read_skill_file', { skill_id, file_path })
}

/** 保存 skill 的单个文件内容 */
export async function saveSkillFileContent(skill_id: string, file_path: string, content: string): Promise<void> {
  await apiCall('write_skill_file', { skill_id, file_path, content })
}

/** 更新 skill 的 source_url（来源地址，支持多行） */
export async function updateSkillSourceUrl(skill_id: string, source_url: string | null): Promise<ManagedSkill> {
  return apiCall<ManagedSkill>('update_skill_source_url', { skill_id, source_url })
}

// ── 数据库管理 ─────────────────────────────────────────

export interface DbTableInfo {
  table_name: string
  display_name: string
  row_count: number
  size_bytes: number
  size_human: string
}

export interface DbOverview {
  db_path: string
  file_size: number
  file_size_human: string
  last_modified: number
  sqlite_version: string
  page_size: number
  page_count: number
  freelist_count: number
  free_size: number
  free_size_human: string
  fragmentation_pct: number
  tables: DbTableInfo[]
}

export interface DbColumnInfo {
  cid: number
  name: string
  type: string
  notnull: boolean
  default: string | null
  pk: boolean
}

export interface DbTableData {
  table: string
  display_name: string
  columns: DbColumnInfo[]
  rows: Record<string, unknown>[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

export interface DbMaintenanceResult {
  ok: boolean
  action: string
  message: string
  integrity_result?: string
}

export async function fetchDbOverview(): Promise<DbOverview> {
  return apiGet<DbOverview>('db/overview')
}

export async function fetchDbTableData(
  table_name: string,
  params: {
    page?: number
    page_size?: number
    sort_col?: string | null
    sort_dir?: 'asc' | 'desc'
    filter_text?: string | null
  } = {},
): Promise<DbTableData> {
  return apiGet<DbTableData>(`db/table/${table_name}`, params)
}

export async function runDbMaintenance(action: string): Promise<DbMaintenanceResult> {
  return apiCall<DbMaintenanceResult>('db/maintenance', { action })
}

export async function resetDb(confirm_text: string): Promise<{ ok: boolean; message: string }> {
  return apiCall('db/reset', { confirm_text })
}

export function getDbExportUrl(): string {
  return `/api/db/export`
}

export async function openDbFolder(): Promise<{ ok: boolean; message: string }> {
  return apiCall('db/open_folder', {})
}

// ── 更新检查 ───────────────────────────────────────────

export interface CheckUpdateResult {
  current_version: string
  latest_version: string
  update_available: boolean
  install_mode: string
  release_url: string
  release_notes: string
  changelog_url: string
  download_urls: {
    setup: string
    portable: string
    exe: string
  }
  error?: string
}

export interface PerformUpdateResult {
  ok: boolean
  message: string
}

export async function checkUpdate(): Promise<CheckUpdateResult> {
  return apiGet<CheckUpdateResult>('check_update')
}

export async function performUpdate(): Promise<PerformUpdateResult> {
  return apiCall<PerformUpdateResult>('perform_update')
}

export async function getAutoCheckUpdate(): Promise<boolean> {
  return apiGet<boolean>('get_auto_check_update')
}

export async function setAutoCheckUpdate(enabled: boolean): Promise<void> {
  await apiCall('set_auto_check_update', { enabled })
}
