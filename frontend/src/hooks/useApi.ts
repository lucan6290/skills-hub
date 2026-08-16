import { useCallback } from 'react'
import { apiCall, apiGet } from '@/lib/api'

/**
 * 统一的 API 调用 hook。
 * 替代原来的 invokeTauri，由调用方显式选择 GET/POST 方法。
 */
export function useApi() {
  const get = useCallback(
    <T>(command: string, params?: Record<string, unknown>) =>
      apiGet<T>(command, params),
    [],
  )
  const post = useCallback(
    <T>(command: string, params?: Record<string, unknown>) =>
      apiCall<T>(command, params),
    [],
  )
  return { get, post }
}
