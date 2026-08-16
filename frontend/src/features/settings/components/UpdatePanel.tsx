import { memo, useCallback, useEffect, useState } from 'react'
import {
  CheckCircle,
  AlertCircle,
  DownloadCloud,
  ExternalLink,
  FileText,
  Github,
  Loader2,
  RefreshCw,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import { toast } from 'sonner'
import {
  checkUpdate,
  performUpdate,
  getAutoCheckUpdate,
  setAutoCheckUpdate,
  type CheckUpdateResult,
} from '@/lib/api'

type UpdatePanelProps = {
  t: TFunction
}

type UpdateState = 'idle' | 'checking' | 'downloading' | 'done'

const UpdatePanel = ({ t }: UpdatePanelProps) => {
  const [result, setResult] = useState<CheckUpdateResult | null>(null)
  const [state, setState] = useState<UpdateState>('idle')
  const [checkFailed, setCheckFailed] = useState(false)
  const [autoCheck, setAutoCheck] = useState<boolean | null>(null)
  const [savingAutoCheck, setSavingAutoCheck] = useState(false)

  const doCheck = useCallback(async () => {
    setState('checking')
    setCheckFailed(false)
    try {
      const res = await checkUpdate()
      setResult(res)
    } catch {
      setCheckFailed(true)
    } finally {
      setState('idle')
    }
  }, [])

  // 加载「启动时自动检查更新」开关
  useEffect(() => {
    let cancelled = false
    getAutoCheckUpdate()
      .then((enabled) => {
        if (!cancelled) setAutoCheck(enabled)
      })
      .catch(() => {
        if (!cancelled) setAutoCheck(true)
      })
    return () => {
      cancelled = true
    }
  }, [])

  // 根据开关决定是否启动时自动检查
  useEffect(() => {
    if (autoCheck === true) {
      doCheck()
    }
  }, [autoCheck, doCheck])

  const handleAutoCheckToggle = useCallback(async () => {
    if (autoCheck === null || savingAutoCheck) return
    const next = !autoCheck
    setAutoCheck(next)
    setSavingAutoCheck(true)
    try {
      await setAutoCheckUpdate(next)
    } catch {
      setAutoCheck(!next)
      toast.error(t('update.saveFailed'))
    } finally {
      setSavingAutoCheck(false)
    }
  }, [autoCheck, savingAutoCheck, t])

  const handleUpdate = useCallback(async () => {
    if (!result?.update_available) return
    setState('downloading')
    try {
      const res = await performUpdate()
      if (res.ok) {
        setState('done')
        toast.success(res.message)
      } else {
        toast.error(res.message)
        setState('idle')
      }
    } catch (e) {
      toast.error(`${t('update.updateFailed')}: ${e instanceof Error ? e.message : String(e)}`)
      setState('idle')
    }
  }, [result, t])

  const isChecking = state === 'checking'
  const isDownloading = state === 'downloading'
  const isDone = state === 'done'
  const hasError = checkFailed || !!result?.error
  const errorMessage = checkFailed ? t('update.checkFailedHint') : (result?.error ?? '')

  return (
    <div className="settings-v2-update-list">
      {/* 当前版本 + 检查更新 */}
      <div className="settings-v2-item">
        <div className="settings-v2-item-info">
          <div className="settings-v2-item-title">
            <DownloadCloud size={16} color="var(--text-tertiary)" />
            {t('update.currentVersion')}
          </div>
          <div className="settings-v2-item-desc mono">
            v{result?.current_version ?? __APP_VERSION__}
          </div>
        </div>
        <button
          className="settings-v2-pill-btn"
          type="button"
          onClick={doCheck}
          disabled={isChecking || isDownloading}
          title={t('update.checkTooltip')}
        >
          {isChecking ? (
            <>
              <Loader2 size={16} className="animate-spin" />
              {t('update.checking')}
            </>
          ) : (
            <>
              <RefreshCw size={16} />
              {t('update.checkAgain')}
            </>
          )}
        </button>
      </div>

      {/* 更新状态 */}
      {hasError ? (
        <div className="settings-v2-item">
          <div className="settings-v2-item-info">
            <div className="settings-v2-item-title">
              <AlertCircle size={16} color="var(--status-warning)" />
              {t('update.checkFailed')}
            </div>
            <div className="settings-v2-item-desc" style={{ color: 'var(--status-warning)' }}>
              {errorMessage}
            </div>
          </div>
          <button
            className="settings-v2-pill-btn"
            type="button"
            onClick={doCheck}
            disabled={isChecking}
          >
            <RefreshCw size={16} />
            {t('update.retry')}
          </button>
        </div>
      ) : isChecking && !result ? (
        <div className="settings-v2-item">
          <div className="settings-v2-item-info">
            <div className="settings-v2-item-title">
              <Loader2 size={16} className="animate-spin" color="var(--text-tertiary)" />
              {t('update.checking')}
            </div>
          </div>
        </div>
      ) : !result ? (
        <div className="settings-v2-item">
          <div className="settings-v2-item-info">
            <div className="settings-v2-item-desc">{t('update.autoCheckOffHint')}</div>
          </div>
        </div>
      ) : result.update_available ? (
        <div className="settings-v2-item">
          <div className="settings-v2-item-info">
            <div className="settings-v2-item-title">
              <DownloadCloud size={16} color="var(--accent-primary)" />
              {t('update.updateAvailable')} v{result.latest_version}
            </div>
          </div>
          {isDone ? (
            <span className="settings-v2-item-desc">{t('update.restarting')}</span>
          ) : isDownloading ? (
            <span className="settings-v2-item-desc" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <Loader2 size={14} className="animate-spin" />
              {t('update.downloading')}
            </span>
          ) : (
            <button
              className="settings-v2-pill-btn"
              type="button"
              onClick={handleUpdate}
              disabled={isChecking}
            >
              <DownloadCloud size={16} />
              {t('update.updateNow')}
            </button>
          )}
        </div>
      ) : (
        <div className="settings-v2-item">
          <div className="settings-v2-item-info">
            <div className="settings-v2-item-title">
              <CheckCircle size={16} color="var(--status-success)" />
              {t('update.upToDate')}
            </div>
            <div className="settings-v2-item-desc">v{result.latest_version}</div>
          </div>
        </div>
      )}

      {/* 启动时自动检查更新 */}
      <div className="settings-v2-item">
        <div className="settings-v2-item-info">
          <div className="settings-v2-item-title">{t('update.autoCheck')}</div>
        </div>
        <button
          type="button"
          role="switch"
          aria-checked={autoCheck === true}
          className={`settings-toggle ${autoCheck === true ? 'checked' : ''}`}
          onClick={handleAutoCheckToggle}
          disabled={autoCheck === null || savingAutoCheck}
        >
          <span className="settings-toggle-knob" />
        </button>
      </div>

      {/* 更新内容摘要 */}
      {result?.release_notes && result.update_available && !hasError && (
        <div className="settings-v2-item settings-v2-item--column">
          <div className="settings-v2-item-info" style={{ width: '100%' }}>
            <div className="settings-v2-item-title">{t('update.releaseNotes')}</div>
            <div className="settings-v2-release-notes">{result.release_notes}</div>
          </div>
        </div>
      )}

      {/* 外部链接 — GitHub */}
      {result?.release_url && (() => {
        const repoMatch = result.release_url.match(/https:\/\/github\.com\/[^/]+\/[^/]+/)
        const repoUrl = repoMatch ? repoMatch[0] : result.release_url
        return (
          <div className="settings-v2-item">
            <div className="settings-v2-item-info">
              <div className="settings-v2-item-title">
                <Github size={16} color="var(--text-tertiary)" />
                GitHub
              </div>
            </div>
            <a
              href={repoUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="settings-v2-pill-btn"
              style={{ textDecoration: 'none' }}
              aria-label="GitHub"
            >
              <Github size={16} />
              <span>GitHub</span>
            </a>
          </div>
        )
      })()}

      {/* 外部链接 — 更新日志 */}
      {result?.release_url && (
        <div className="settings-v2-item">
          <div className="settings-v2-item-info">
            <div className="settings-v2-item-title">
              <FileText size={16} color="var(--text-tertiary)" />
              {t('update.changelog')}
            </div>
          </div>
          <a
            href={result.release_url}
            target="_blank"
            rel="noopener noreferrer"
            className="settings-v2-pill-btn"
            style={{ textDecoration: 'none' }}
          >
            <ExternalLink size={16} />
            <span>{t('update.changelog')}</span>
          </a>
        </div>
      )}
    </div>
  )
}

export default memo(UpdatePanel)
