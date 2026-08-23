import { useCallback } from 'react'
import { invokeCommand } from '@/lib/api'

/**
 * 统一的 API 调用 hook。
 * 通过 Tauri invoke 调用 Rust 后端，get/post 语义统一为 invokeCommand。
 */
export function useApi() {
  const get = useCallback(
    <T>(command: string, params?: Record<string, unknown>) =>
      invokeCommand<T>(command, params),
    [],
  )
  const post = useCallback(
    <T>(command: string, params?: Record<string, unknown>) =>
      invokeCommand<T>(command, params),
    [],
  )
  return { get, post }
}
