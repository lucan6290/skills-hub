export interface ApiErrorDetail {
  code: string
  tool?: string
  tool_key?: string
  path?: string
  reason?: string
}

/**
 * 将后端错误 detail 解析为 i18n key + params 或 raw message。
 * 格式：{ code, ... } JSON 对象
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

  return { rawMessage: String(detail ?? '') }
}
