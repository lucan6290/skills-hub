import { useCallback } from 'react'
import { invokeCommand } from '@/lib/api'

/**
 * 统一的 API 调用 hook。
 * 通过 Tauri invoke 调用 Rust 后端。
 */
export function useApi() {
  const invoke = useCallback(
    <T>(command: string, params?: Record<string, unknown>) =>
      invokeCommand<T>(command, params),
    [],
  )
  return { invoke }
}
