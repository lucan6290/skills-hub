import React, { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { parseErrorDetail } from '../lib/errors'

// ─── Types ────────────────────────────────────────────
type AppState = {
  language: string
  actionMessage: string | null
}

type AppStateActions = {
  setError: (msg: string) => void
  setActionMessage: (msg: string | null) => void
  setSuccessToastMessage: (msg: string) => void
  toggleLanguage: () => void
  setLanguage: (lang: string) => void
  formatErrorMessage: (raw: unknown) => string
  showActionErrors: (errors: { title: string; message: string }[]) => void
}

type AppStateContextValue = AppState & AppStateActions

const AppStateContext = createContext<AppStateContextValue | null>(null)

// ─── Provider ─────────────────────────────────────────
export function AppStateProvider({ children }: { children: React.ReactNode }) {
  const { t, i18n } = useTranslation()
  const language = i18n.resolvedLanguage ?? i18n.language ?? 'en'
  const languageStorageKey = 'skills-language'
  const [actionMessage, setActionMessage] = useState<string | null>(null)

  const toggleLanguage = useCallback(() => {
    void i18n.changeLanguage(language === 'en' ? 'zh' : 'en')
  }, [i18n, language])

  const setLanguage = useCallback((lang: string) => {
    void i18n.changeLanguage(lang)
  }, [i18n])

  // 持久化语言偏好
  useEffect(() => {
    if (typeof window === 'undefined') return
    if (language !== 'en' && language !== 'zh') return
    try {
      window.localStorage.setItem(languageStorageKey, language)
    } catch {
      // ignore storage failures
    }
  }, [language, languageStorageKey])

  const formatErrorMessage = useCallback(
    (raw: unknown): string => {
      const result = parseErrorDetail(raw)
      if ('i18nKey' in result) {
        if (result.i18nKey === '__silent') return ''
        return t(result.i18nKey, result.params as Record<string, string>) as string
      }
      return result.rawMessage
    },
    [t],
  )

  const showActionErrors = useCallback(
    (errors: { title: string; message: string }[]) => {
      if (errors.length === 0) return
      const head = errors[0]
      const more =
        errors.length > 1
          ? t('errors.moreCount', { count: errors.length - 1 })
          : ''
      toast.error(
        `${formatErrorMessage(`${head.title}\n${head.message}`)}${more}`,
        { duration: 3200 },
      )
    },
    [formatErrorMessage, t],
  )

  // 错误 toast：在 setError 调用时直接展示并清空 loading 状态，避免 effect 内 setState
  const setError = useCallback(
    (msg: string) => {
      const formatted = formatErrorMessage(msg)
      if (formatted) toast.error(formatted, { duration: 2600 })
      setActionMessage(null)
    },
    [formatErrorMessage],
  )

  // 成功 toast：在 setSuccessToastMessage 调用时直接展示
  const setSuccessToastMessage = useCallback((msg: string) => {
    toast.success(msg, { duration: 1800 })
  }, [])

  const value = useMemo<AppStateContextValue>(
    () => ({
      language,
      actionMessage,
      setError,
      setActionMessage,
      setSuccessToastMessage,
      toggleLanguage,
      setLanguage,
      formatErrorMessage,
      showActionErrors,
    }),
    [
      language,
      actionMessage,
      setError,
      setSuccessToastMessage,
      toggleLanguage,
      setLanguage,
      formatErrorMessage,
      showActionErrors,
    ],
  )

  return (
    <AppStateContext.Provider value={value}>
      {children}
    </AppStateContext.Provider>
  )
}

// ─── Hook ─────────────────────────────────────────────
export function useAppState() {
  const ctx = useContext(AppStateContext)
  if (!ctx) {
    throw new Error('useAppState must be used within AppStateProvider')
  }
  return ctx
}
