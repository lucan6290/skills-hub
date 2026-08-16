export interface ApiErrorDetail {
  code: string
  tool?: string
  tool_key?: string
  path?: string
  reason?: string
}

/**
 * 将后端错误 detail 解析为 i18n key + params 或 raw message。
 * 兼容新旧两种格式：
 *   - 新：{ code, ... } JSON 对象
 *   - 旧："CODE|param1|param2" 管道符字符串
 */
export function parseErrorDetail(
  detail: unknown,
): { i18nKey: string; params?: Record<string, string> } | { rawMessage: string } {
  // 新格式：JSON 对象 with code
  if (typeof detail === 'object' && detail !== null && 'code' in detail) {
    const d = detail as ApiErrorDetail
    switch (d.code) {
      case 'CANCELLED':
        return { i18nKey: '__silent' }
      case 'TARGET_EXISTS':
        return { i18nKey: 'errors.targetExists', params: d.path ? { path: d.path } : undefined }
      case 'TOOL_NOT_INSTALLED':
        return { i18nKey: 'errors.toolNotInstalled' }
      case 'TOOL_NOT_WRITABLE':
        return {
          i18nKey: 'errors.toolNotWritable',
          params: { tool: d.tool ?? '', path: d.path ?? '' },
        }
      case 'PROJECT_SCOPE_UNSUPPORTED':
        return { i18nKey: 'projectSync.unsupportedTool', params: { tool: d.tool_key ?? '' } }
      case 'SKILL_INVALID':
        return { i18nKey: 'errors.skillInvalid', params: { reason: d.reason ?? '' } }
      default:
        return { rawMessage: JSON.stringify(d) }
    }
  }

  // Phase 1 过渡期兼容旧管道符格式，下一 PR 清理
  const raw = String(detail ?? '')

  if (raw.includes('CANCELLED|')) {
    return { i18nKey: '__silent' }
  }

  if (raw.includes('skill already exists in community repo')) {
    const pathMatch = raw.match(/community repo:\s*"?([^"]+)"?/)
    if (pathMatch) {
      const skillName = pathMatch[1].split('/').pop() ?? ''
      if (skillName) {
        return { i18nKey: 'errors.skillExistsInHubNamed', params: { name: skillName } }
      }
    }
    return { i18nKey: 'errors.skillExistsInHub' }
  }

  if (raw.startsWith('TARGET_EXISTS|')) {
    const path = raw.split('|')[1] ?? ''
    return { i18nKey: 'errors.targetExists', params: { path } }
  }

  if (raw.startsWith('TOOL_NOT_INSTALLED|')) {
    return { i18nKey: 'errors.toolNotInstalled' }
  }

  if (raw.startsWith('TOOL_NOT_WRITABLE|')) {
    const parts = raw.split('|')
    return { i18nKey: 'errors.toolNotWritable', params: { tool: parts[1] ?? '', path: parts[2] ?? '' } }
  }

  if (raw.startsWith('PROJECT_SCOPE_UNSUPPORTED|')) {
    const tool = raw.split('|')[1] ?? ''
    return { i18nKey: 'projectSync.unsupportedTool', params: { tool } }
  }

  if (raw.startsWith('SKILL_INVALID|')) {
    const reason = raw.split('|')[1] ?? ''
    return { i18nKey: 'errors.skillInvalid', params: { reason } }
  }

  return { rawMessage: raw }
}
