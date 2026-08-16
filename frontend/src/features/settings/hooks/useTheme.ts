import { useCallback, useEffect, useState } from 'react'
import type { TFunction } from 'i18next'
import { toast } from 'sonner'
import { useApi } from '@/hooks/useApi'
import { pickFolder } from '@/lib/pickFolder'

const THEME_STORAGE_KEY = 'skills-theme'

/**
 * 主题、storage path 管理 hook。
 * 从 App.tsx 提取 themePreference / systemTheme / storagePath 相关逻辑。
 */
export function useTheme(
  t: TFunction,
  loadManagedSkills: (refresh?: boolean, sourceType?: 'custom' | 'community') => Promise<void>,
  setError: (msg: string) => void,
) {
  const { get, post } = useApi()
  const [themePreference, setThemePreference] = useState<'system' | 'light' | 'dark'>(() => {
    if (typeof window === 'undefined') return 'system'
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY)
    if (stored === 'light' || stored === 'dark' || stored === 'system') return stored
    return 'system'
  })
  const [systemTheme, setSystemTheme] = useState<'light' | 'dark'>('light')
  const [storagePath, setStoragePath] = useState<string>(t('notAvailable'))
  const [customRepoPath, setCustomRepoPath] = useState<string>(t('notAvailable'))

  // 监听系统主题变化
  useEffect(() => {
    if (typeof window === 'undefined') return
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    const update = () => {
      setSystemTheme(media.matches ? 'dark' : 'light')
    }
    update()
    if (media.addEventListener) {
      media.addEventListener('change', update)
    } else {
      media.addListener(update)
    }
    return () => {
      if (media.removeEventListener) {
        media.removeEventListener('change', update)
      } else {
        media.removeListener(update)
      }
    }
  }, [])

  // 应用主题到 document
  useEffect(() => {
    if (typeof document === 'undefined') return
    const resolvedTheme =
      themePreference === 'system' ? systemTheme : themePreference
    document.documentElement.dataset.theme = resolvedTheme
    document.documentElement.style.colorScheme = resolvedTheme
    try {
      window.localStorage.setItem(THEME_STORAGE_KEY, themePreference)
    } catch {
      // ignore storage failures
    }
  }, [systemTheme, themePreference])

  // 初始化 storage path
  useEffect(() => {
    get<string>('get_community_repo_path')
      .then((path) => setStoragePath(path))
      .catch((err) => {
        setError(err instanceof Error ? err.message : String(err))
      })
    get<string>('get_custom_repo_path')
      .then((path) => setCustomRepoPath(path))
      .catch((err) => {
        setError(err instanceof Error ? err.message : String(err))
      })
  }, [get, setError])

  const handleThemeChange = useCallback(
    (nextTheme: 'system' | 'light' | 'dark') => {
      setThemePreference(nextTheme)
    },
    [],
  )

  const handlePickStoragePath = useCallback(async () => {
    try {
      const path = await pickFolder(t('enterStoragePath'))
      if (!path) return
      const result = await post<{ new_path: string }>('set_community_repo_path', { path })
      setStoragePath(result.new_path)
      await loadManagedSkills()
      toast.success(t('settings.saved'))
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }, [loadManagedSkills, post, setError, t])

  const handlePickCustomRepoPath = useCallback(async () => {
    try {
      const path = await pickFolder(t('enterCustomRepoPath'))
      if (!path) return
      const result = await post<{ ok: boolean; path: string; empty?: boolean }>('set_custom_repo_path', { path })
      setCustomRepoPath(result.path)
      await loadManagedSkills(true, 'custom')
      toast.success(t('settings.saved'))
      if (result.empty) toast.info(t('settings.emptyDirHint'))
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }, [loadManagedSkills, post, setError, t])

  const handleOpenFolder = useCallback(async (path: string) => {
    try {
      await post<{ ok: boolean }>('open_settings_folder', { path })
      toast.success(t('settings.openedFolder'))
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }, [post, setError, t])

  const handleResetDefaults = useCallback(async () => {
    const result = await post<{ ok: boolean; community_repo_path: string; custom_repo_path: string }>(
      'reset_general_settings',
    )
    setStoragePath(result.community_repo_path)
    setCustomRepoPath(result.custom_repo_path)
  }, [post])

  return {
    themePreference,
    systemTheme,
    storagePath,
    customRepoPath,
    handleThemeChange,
    handlePickStoragePath,
    handlePickCustomRepoPath,
    handleOpenFolder,
    handleResetDefaults,
  }
}
