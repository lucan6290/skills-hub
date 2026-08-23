/**
 * Skills Hub 前端 API 适配层
 * 通过 Tauri invoke 调用 Rust 后端
 *
 * 命名规范：所有参数直接使用 snake_case，与后端字段名完全一致，禁止转换。
 */
import { invoke } from '@tauri-apps/api/core'
import type { ManagedSkill } from '@/features/skills'

/**
 * 统一 Tauri invoke transport。
 * 将 Rust AppError 转为前端 Error，保持与旧 HTTP 层一致的异常语义。
 */
export async function invokeCommand<T = unknown>(
  command: string,
  params?: Record<string, unknown>,
): Promise<T> {
  try {
    // Tauri invoke 的参数直接传递，不需要 JSON 序列化
    const result = await invoke<T>(command, params)
    return result
  } catch (err) {
    // Rust AppError 序列化为字符串或对象
    if (typeof err === 'string') {
      throw new Error(err)
    }
    if (err instanceof Error) {
      throw err
    }
    // 尝试从 Rust 错误对象中提取 message
    if (typeof err === 'object' && err !== null) {
      const obj = err as Record<string, unknown>
      const message = (obj.message ?? obj.detail ?? JSON.stringify(err)) as string
      throw new Error(message)
    }
    throw new Error(String(err))
  }
}

/**
 * @deprecated 使用 invokeCommand 替代。保留仅为兼容过渡期。
 */
export async function apiCall<T = unknown>(
  command: string,
  params?: Record<string, unknown>,
): Promise<T> {
  return invokeCommand<T>(command, params)
}

/**
 * @deprecated 使用 invokeCommand 替代。Tauri invoke 不区分 GET/POST。
 */
export async function apiGet<T = unknown>(
  command: string,
  params?: Record<string, unknown>,
): Promise<T> {
  return invokeCommand<T>(command, params)
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
  await invokeCommand('reorder', { entity, items })
}

export async function fetchScopePreferences(): Promise<ScopePreferenceDto[]> {
  return invokeCommand<ScopePreferenceDto[]>('get_scope_preferences')
}

export async function saveScopePreference(
  skill_id: string,
  scope: string,
  project_paths: string,
): Promise<void> {
  await invokeCommand('set_scope_preference', { skill_id, scope, project_paths })
}

/** 获取 skill 的标签列表 */
export async function fetchSkillTags(skill_id: string): Promise<TagDto[]> {
  return invokeCommand<TagDto[]>('get_skill_tags', { skill_id })
}

/** 列出 skill 的文件 */
export async function fetchSkillFiles(skill_id: string): Promise<SkillFileEntry[]> {
  return invokeCommand<SkillFileEntry[]>('list_skill_files', { skill_id })
}

/** 读取 skill 的单个文件内容 */
export async function fetchSkillFileContent(skill_id: string, file_path: string): Promise<string> {
  return invokeCommand<string>('read_skill_file', { skill_id, file_path })
}

/** 保存 skill 的单个文件内容 */
export async function saveSkillFileContent(skill_id: string, file_path: string, content: string): Promise<void> {
  await invokeCommand('write_skill_file', { skill_id, file_path, content })
}

/** 更新 skill 的 source_url（来源地址，支持多行） */
export async function updateSkillSourceUrl(skill_id: string, source_url: string | null): Promise<ManagedSkill> {
  return invokeCommand<ManagedSkill>('update_skill_source_url', { skill_id, source_url })
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
  return invokeCommand<DbOverview>('db_overview')
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
  return invokeCommand<DbTableData>('db_table_data', { table_name, ...params })
}

export async function runDbMaintenance(action: string): Promise<DbMaintenanceResult> {
  return invokeCommand<DbMaintenanceResult>('db_maintenance', { action })
}

export async function resetDb(confirm_text: string): Promise<{ ok: boolean; message: string }> {
  return invokeCommand('db_reset', { confirm_text })
}

/**
 * 导出数据库。Tauri 模式下通过 Rust command 触发文件保存对话框 + 复制。
 * 返回导出结果信息。
 */
export async function exportDb(): Promise<{ ok: boolean; message: string; path?: string }> {
  return invokeCommand('db_export')
}

export async function openDbFolder(): Promise<{ ok: boolean; message: string }> {
  return invokeCommand('db_open_folder')
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
  return invokeCommand<CheckUpdateResult>('check_update')
}

export async function performUpdate(): Promise<PerformUpdateResult> {
  return invokeCommand<PerformUpdateResult>('do_update')
}

export async function getAutoCheckUpdate(): Promise<boolean> {
  return invokeCommand<boolean>('get_auto_check_update')
}

export async function setAutoCheckUpdate(enabled: boolean): Promise<void> {
  await invokeCommand('set_auto_check_update', { enabled })
}

// ── 代理设置 ───────────────────────────────────────────

export async function getProxyUrl(): Promise<string> {
  return invokeCommand<string>('get_proxy_url')
}

export async function setProxyUrl(url: string): Promise<void> {
  await invokeCommand('set_proxy_url', { url })
}
