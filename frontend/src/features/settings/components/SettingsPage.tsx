import { memo, useEffect, useState } from 'react'
import {
  ArrowLeft,
  Sun,
  Moon,
  Monitor,
  FolderOpen,
  Folder,
  Database,
  Copy,
  ChevronDown,
  RotateCcw,
  X,
  Globe,
  Power,
  XCircle,
  Minimize2,
  Bell,
  FileText,
  RefreshCw,
  Palette,
  HardDrive,
  Shield,
  Settings2,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import { toast } from 'sonner'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart'
import DatabasePanel from '@/features/database/components/DatabasePanel'
import UpdatePanel from '@/features/settings/components/UpdatePanel'
import {
  getProxyUrl, setProxyUrl,
  getCloseBehavior, setCloseBehavior,
  getShowTrayIcon, setShowTrayIcon,
  getLogLevel, setLogLevel,
  getAutoRefreshOnStartup, setAutoRefreshOnStartup,
} from '@/lib/api'
import logoLight from '@/assets/logo.svg'
import logoDark from '@/assets/logo-dark.svg'

type SettingsTab = 'general' | 'database' | 'about'

type SettingsPageProps = {
  language: string
  storagePath: string
  customRepoPath: string
  themePreference: 'system' | 'light' | 'dark'
  onBack: () => void
  onPickStoragePath: () => void
  onPickCustomRepoPath: () => void
  onOpenFolder: (path: string) => void
  onResetDefaults: () => Promise<void>
  onSetLanguage: (lang: string) => void
  onThemeChange: (nextTheme: 'system' | 'light' | 'dark') => void
  t: TFunction
}

const DEFAULT_LANGUAGE = 'zh'
const DEFAULT_THEME: 'system' | 'light' | 'dark' = 'system'

const TABS: { key: SettingsTab; labelKey: string }[] = [
  { key: 'general', labelKey: 'settings.tabGeneral' },
  { key: 'database', labelKey: 'settings.tabDatabase' },
  { key: 'about', labelKey: 'settings.tabAbout' },
]

const LANG_OPTIONS = [
  { key: 'zh', labelKey: 'settings.langZhHans' },
  { key: 'en', labelKey: 'settings.langEn' },
] as const

const THEME_OPTIONS = [
  { key: 'light', labelKey: 'settings.themeLight', Icon: Sun },
  { key: 'dark', labelKey: 'settings.themeDark', Icon: Moon },
  { key: 'system', labelKey: 'settings.themeSystem', Icon: Monitor },
] as const

const CLOSE_BEHAVIOR_OPTIONS = [
  { key: 'minimize_to_tray', labelKey: 'settings.closeMinimizeToTray', Icon: Minimize2 },
  { key: 'quit', labelKey: 'settings.closeQuit', Icon: XCircle },
] as const

const LOG_LEVEL_OPTIONS = [
  { key: 'debug', labelKey: 'settings.logDebug' },
  { key: 'info', labelKey: 'settings.logInfo' },
  { key: 'warn', labelKey: 'settings.logWarn' },
  { key: 'error', labelKey: 'settings.logError' },
] as const

type PathFieldProps = {
  label: string
  path: string
  copyLabel: string
  openLabel: string
  changeLabel: string
  onCopy: (path: string) => void
  onOpen: (path: string) => void
  onChange: () => void
}

const PathField = ({ label, path, copyLabel, openLabel, changeLabel, onCopy, onOpen, onChange }: PathFieldProps) => (
  <div className="settings-v2-path-group">
    <label className="settings-v2-path-label">{label}</label>
    <div className="settings-v2-path-row">
      <code className="settings-v2-path" title={path}>
        {path}
      </code>
      <button type="button" className="settings-v2-path-btn" onClick={() => onCopy(path)} title={copyLabel}>
        <Copy size={12} />
      </button>
      <button type="button" className="settings-v2-path-btn" onClick={() => onOpen(path)} title={openLabel}>
        <FolderOpen size={12} />
      </button>
      <button type="button" className="settings-v2-path-btn settings-v2-path-btn--text" onClick={onChange}>
        <Folder size={12} />
        {changeLabel}
      </button>
    </div>
  </div>
)

const SettingsPage = ({
  language,
  storagePath,
  customRepoPath,
  themePreference,
  onBack,
  onPickStoragePath,
  onPickCustomRepoPath,
  onOpenFolder,
  onResetDefaults,
  onSetLanguage,
  onThemeChange,
  t,
}: SettingsPageProps) => {
  const [activeTab, setActiveTab] = useState<SettingsTab>('general')
  const [showCustomRepo, setShowCustomRepo] = useState(false)
  const [showDetails, setShowDetails] = useState(false)
  const [showResetConfirm, setShowResetConfirm] = useState(false)
  const [proxyUrl, setProxyUrlState] = useState('')
  const [proxySaving, setProxySaving] = useState(false)
  const [autostartEnabled, setAutostartEnabled] = useState(false)
  const [autostartLoaded, setAutostartLoaded] = useState(false)
  const [autostartSaving, setAutostartSaving] = useState(false)
  const [closeBehavior, setCloseBehaviorState] = useState('minimize_to_tray')
  const [trayIconEnabled, setTrayIconEnabled] = useState(true)
  const [logLevel, setLogLevelState] = useState('info')
  const [autoRefresh, setAutoRefresh] = useState(false)

  useEffect(() => {
    getProxyUrl().then(setProxyUrlState).catch(() => {})
    getCloseBehavior().then(setCloseBehaviorState).catch(() => {})
    getShowTrayIcon().then(setTrayIconEnabled).catch(() => {})
    getLogLevel().then(setLogLevelState).catch(() => {})
    getAutoRefreshOnStartup().then(setAutoRefresh).catch(() => {})
  }, [])

  useEffect(() => {
    isEnabled()
      .then((enabled) => {
        setAutostartEnabled(enabled)
        setAutostartLoaded(true)
      })
      .catch(() => setAutostartLoaded(true))
  }, [])

  const handleAutostartToggle = async () => {
    if (!autostartLoaded || autostartSaving) return
    const next = !autostartEnabled
    setAutostartEnabled(next)
    setAutostartSaving(true)
    try {
      if (next) {
        await enable()
      } else {
        await disable()
      }
    } catch {
      setAutostartEnabled(!next)
      toast.error(t('settings.saveFailed'))
    } finally {
      setAutostartSaving(false)
    }
  }

  const handleCloseBehaviorChange = async (behavior: string) => {
    setCloseBehaviorState(behavior)
    try {
      await setCloseBehavior(behavior)
    } catch {
      setCloseBehaviorState(closeBehavior)
      toast.error(t('settings.saveFailed'))
    }
  }

  const handleTrayIconToggle = async () => {
    const next = !trayIconEnabled
    setTrayIconEnabled(next)
    try {
      await setShowTrayIcon(next)
    } catch {
      setTrayIconEnabled(!next)
      toast.error(t('settings.saveFailed'))
    }
  }

  const handleLogLevelChange = async (level: string) => {
    setLogLevelState(level)
    try {
      await setLogLevel(level)
    } catch {
      setLogLevelState(logLevel)
      toast.error(t('settings.saveFailed'))
    }
  }

  const handleAutoRefreshToggle = async () => {
    const next = !autoRefresh
    setAutoRefresh(next)
    try {
      await setAutoRefreshOnStartup(next)
    } catch {
      setAutoRefresh(!next)
      toast.error(t('settings.saveFailed'))
    }
  }

  const handleSaveProxy = async () => {
    setProxySaving(true)
    try {
      await setProxyUrl(proxyUrl.trim())
      toast.success(t('settings.saved'))
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    } finally {
      setProxySaving(false)
    }
  }

  const handleCopyPath = async (path: string) => {
    try {
      await navigator.clipboard.writeText(path)
      toast.success(t('copied'))
    } catch {
      toast.error(t('copyFailed'))
    }
  }

  const handleConfirmReset = async () => {
    setShowResetConfirm(false)
    onSetLanguage(DEFAULT_LANGUAGE)
    onThemeChange(DEFAULT_THEME)
    await onResetDefaults()
    toast.success(t('settings.saved'))
  }

  return (
    <div className="settings-page settings-page-v2">
      {/* Header with back button */}
      <div className="settings-v2-header">
        <button className="settings-back-btn" type="button" onClick={onBack} aria-label={t('back')}>
          <ArrowLeft size={20} />
        </button>
        <h2 className="settings-v2-title">{t('settings.title')}</h2>
      </div>

      {/* Top Tab Bar (CC Switch style segmented) */}
      <div className="settings-v2-tabs">
        {TABS.map((tab) => (
          <button
            key={tab.key}
            type="button"
            className={`settings-v2-tab ${activeTab === tab.key ? 'active' : ''}`}
            onClick={() => setActiveTab(tab.key)}
          >
            {t(tab.labelKey)}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div className="settings-v2-content">
        {activeTab === 'general' && (
          <div className="settings-v2-sections">
            {/* ── Group: Appearance ── */}
            <div className="settings-v2-group-label">
              <Palette size={13} />
              {t('settings.groupAppearance')}
            </div>

            {/* Language Section */}
            <section className="settings-v2-card">
              <div className="settings-v2-card-header">
                <div className="settings-v2-card-icon settings-v2-card-icon--teal">
                  <Globe size={18} />
                </div>
                <div className="settings-v2-card-text">
                  <h3 className="settings-v2-section-title">{t('settings.interfaceLanguage')}</h3>
                  <p className="settings-v2-section-desc">{t('settings.languageDesc')}</p>
                </div>
              </div>
              <div className="settings-v2-seg-group">
                {LANG_OPTIONS.map((opt) => (
                  <button
                    key={opt.key}
                    type="button"
                    title={t(opt.labelKey)}
                    className={`settings-v2-seg-btn ${language === opt.key ? 'active' : ''}`}
                    onClick={() => onSetLanguage(opt.key)}
                  >
                    {t(opt.labelKey)}
                  </button>
                ))}
              </div>
              <p className="settings-v2-card-note">{t('settings.languageRestartHint')}</p>
            </section>

            {/* Theme Section */}
            <section className="settings-v2-card">
              <div className="settings-v2-card-header">
                <div className="settings-v2-card-icon settings-v2-card-icon--amber">
                  <Sun size={18} />
                </div>
                <div className="settings-v2-card-text">
                  <h3 className="settings-v2-section-title">{t('settings.appearanceTheme')}</h3>
                  <p className="settings-v2-section-desc">{t('settings.themeDesc')}</p>
                </div>
              </div>
              <div className="settings-v2-seg-group">
                {THEME_OPTIONS.map(({ key, labelKey, Icon }) => (
                  <button
                    key={key}
                    type="button"
                    title={t(labelKey)}
                    className={`settings-v2-seg-btn ${themePreference === key ? 'active' : ''}`}
                    onClick={() => onThemeChange(key)}
                  >
                    <Icon size={14} />
                    {t(labelKey)}
                  </button>
                ))}
              </div>
            </section>

            {/* ── Group: Data & Storage ── */}
            <div className="settings-v2-group-label">
              <HardDrive size={13} />
              {t('settings.groupDataStorage')}
            </div>

            {/* Storage Section */}
            <section className="settings-v2-card">
              <div className="settings-v2-card-header">
                <div className="settings-v2-card-icon settings-v2-card-icon--blue">
                  <Folder size={18} />
                </div>
                <div className="settings-v2-card-text">
                  <h3 className="settings-v2-section-title">{t('settings.skillsStorage')}</h3>
                  <p className="settings-v2-section-desc">{t('settings.storageDesc')}</p>
                </div>
              </div>

              <PathField
                label={t('settings.storageLocation')}
                path={storagePath}
                copyLabel={t('settings.copyPath')}
                openLabel={t('settings.openFolder')}
                changeLabel={t('settings.change')}
                onCopy={handleCopyPath}
                onOpen={onOpenFolder}
                onChange={onPickStoragePath}
              />

              <p className="settings-v2-card-note">
                {t('settings.storageSummary')}{' '}
                <button type="button" className="settings-v2-link" onClick={() => setShowDetails(true)}>
                  {t('settings.viewDetails')}
                </button>
              </p>

              {/* Advanced: custom repository */}
              <div className="settings-v2-advanced">
                <button
                  type="button"
                  className="settings-v2-advanced-toggle"
                  onClick={() => setShowCustomRepo((v) => !v)}
                  aria-expanded={showCustomRepo}
                >
                  <ChevronDown size={14} className={showCustomRepo ? 'open' : ''} />
                  {t('settings.storageAdvancedTitle')}
                </button>
                {showCustomRepo && (
                  <div className="settings-v2-advanced-body">
                    <p className="settings-v2-risk">{t('settings.storageRisk')}</p>
                    <PathField
                      label={t('settings.customRepoLocation')}
                      path={customRepoPath}
                      copyLabel={t('settings.copyPath')}
                      openLabel={t('settings.openFolder')}
                      changeLabel={t('settings.change')}
                      onCopy={handleCopyPath}
                      onOpen={onOpenFolder}
                      onChange={onPickCustomRepoPath}
                    />
                  </div>
                )}
              </div>
            </section>

            {/* ── Group: Network ── */}
            <div className="settings-v2-group-label">
              <Shield size={13} />
              {t('settings.groupNetwork')}
            </div>

            {/* Network Proxy Section */}
            <section className="settings-v2-card">
              <div className="settings-v2-card-header">
                <div className="settings-v2-card-icon settings-v2-card-icon--green">
                  <Globe size={18} />
                </div>
                <div className="settings-v2-card-text">
                  <h3 className="settings-v2-section-title">{t('settings.proxyTitle')}</h3>
                  <p className="settings-v2-section-desc">{t('settings.proxyDesc')}</p>
                </div>
              </div>
              <div className="settings-v2-proxy-row">
                <input
                  type="text"
                  className="settings-v2-proxy-input"
                  placeholder={t('settings.proxyPlaceholder')}
                  value={proxyUrl}
                  onChange={(e) => setProxyUrlState(e.target.value)}
                />
                <button
                  type="button"
                  className="btn btn-primary"
                  disabled={proxySaving}
                  onClick={handleSaveProxy}
                >
                  {proxySaving ? t('saving') : t('save')}
                </button>
              </div>
              <p className="settings-v2-card-note">{t('settings.proxyHint')}</p>
            </section>

            {/* ── Group: System ── */}
            <div className="settings-v2-group-label">
              <Settings2 size={13} />
              {t('settings.groupSystem')}
            </div>

            {/* Autostart + Close Behavior + Tray + Log + AutoRefresh — combined system card */}
            <section className="settings-v2-card">
              <div className="settings-v2-card-header">
                <div className="settings-v2-card-icon settings-v2-card-icon--slate">
                  <Power size={18} />
                </div>
                <div className="settings-v2-card-text">
                  <h3 className="settings-v2-section-title">{t('settings.autostartTitle')}</h3>
                  <p className="settings-v2-section-desc">{t('settings.autostartDesc')}</p>
                </div>
              </div>
              <div className="settings-v2-item">
                <div className="settings-v2-item-info">
                  <div className="settings-v2-item-title">{t('settings.autostartToggle')}</div>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={autostartEnabled}
                  className={`settings-toggle ${autostartEnabled ? 'checked' : ''}`}
                  onClick={handleAutostartToggle}
                  disabled={!autostartLoaded || autostartSaving}
                >
                  <span className="settings-toggle-knob" />
                </button>
              </div>
            </section>

            <section className="settings-v2-card">
              <div className="settings-v2-card-header">
                <div className="settings-v2-card-icon settings-v2-card-icon--rose">
                  <XCircle size={18} />
                </div>
                <div className="settings-v2-card-text">
                  <h3 className="settings-v2-section-title">{t('settings.closeBehaviorTitle')}</h3>
                  <p className="settings-v2-section-desc">{t('settings.closeBehaviorDesc')}</p>
                </div>
              </div>
              <div className="settings-v2-seg-group">
                {CLOSE_BEHAVIOR_OPTIONS.map(({ key, labelKey, Icon }) => (
                  <button
                    key={key}
                    type="button"
                    title={t(labelKey)}
                    className={`settings-v2-seg-btn ${closeBehavior === key ? 'active' : ''}`}
                    onClick={() => handleCloseBehaviorChange(key)}
                  >
                    <Icon size={14} />
                    {t(labelKey)}
                  </button>
                ))}
              </div>
            </section>

            <section className="settings-v2-card">
              <div className="settings-v2-card-header">
                <div className="settings-v2-card-icon settings-v2-card-icon--amber">
                  <Bell size={18} />
                </div>
                <div className="settings-v2-card-text">
                  <h3 className="settings-v2-section-title">{t('settings.trayIconTitle')}</h3>
                  <p className="settings-v2-section-desc">{t('settings.trayIconDesc')}</p>
                </div>
              </div>
              <div className="settings-v2-item">
                <div className="settings-v2-item-info">
                  <div className="settings-v2-item-title">{t('settings.showTrayIcon')}</div>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={trayIconEnabled}
                  className={`settings-toggle ${trayIconEnabled ? 'checked' : ''}`}
                  onClick={handleTrayIconToggle}
                >
                  <span className="settings-toggle-knob" />
                </button>
              </div>
            </section>

            <section className="settings-v2-card">
              <div className="settings-v2-card-header">
                <div className="settings-v2-card-icon settings-v2-card-icon--slate">
                  <FileText size={18} />
                </div>
                <div className="settings-v2-card-text">
                  <h3 className="settings-v2-section-title">{t('settings.logLevelTitle')}</h3>
                  <p className="settings-v2-section-desc">{t('settings.logLevelDesc')}</p>
                </div>
              </div>
              <div className="settings-v2-seg-group">
                {LOG_LEVEL_OPTIONS.map(({ key, labelKey }) => (
                  <button
                    key={key}
                    type="button"
                    title={t(labelKey)}
                    className={`settings-v2-seg-btn ${logLevel === key ? 'active' : ''}`}
                    onClick={() => handleLogLevelChange(key)}
                  >
                    {t(labelKey)}
                  </button>
                ))}
              </div>
            </section>

            <section className="settings-v2-card">
              <div className="settings-v2-card-header">
                <div className="settings-v2-card-icon settings-v2-card-icon--teal">
                  <RefreshCw size={18} />
                </div>
                <div className="settings-v2-card-text">
                  <h3 className="settings-v2-section-title">{t('settings.autoRefreshTitle')}</h3>
                  <p className="settings-v2-section-desc">{t('settings.autoRefreshDesc')}</p>
                </div>
              </div>
              <div className="settings-v2-item">
                <div className="settings-v2-item-info">
                  <div className="settings-v2-item-title">{t('settings.autoRefreshToggle')}</div>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={autoRefresh}
                  className={`settings-toggle ${autoRefresh ? 'checked' : ''}`}
                  onClick={handleAutoRefreshToggle}
                >
                  <span className="settings-toggle-knob" />
                </button>
              </div>
            </section>

            {/* Reset to defaults */}
            <div className="settings-v2-reset">
              <button type="button" className="btn btn-secondary" onClick={() => setShowResetConfirm(true)}>
                <RotateCcw size={14} />
                {t('settings.resetDefaults')}
              </button>
            </div>
          </div>
        )}

        {activeTab === 'database' && (
          <div className="settings-v2-sections">
            <DatabasePanel t={t} />
          </div>
        )}

        {activeTab === 'about' && (
          <div className="settings-v2-sections">
            {/* 关于 */}
            <section className="settings-v2-card settings-v2-about">
              <div className="settings-v2-about-logo-wrap">
                <img className="settings-v2-about-logo-icon settings-v2-about-logo-light" src={logoLight} alt="" width={64} height={64} />
                <img className="settings-v2-about-logo-icon settings-v2-about-logo-dark" src={logoDark} alt="" width={64} height={64} />
                <div className="settings-v2-about-logo">
                  <span className="settings-v2-about-brand-skills">Skills</span>
                  <span className="settings-v2-about-brand-hub">Hub</span>
                </div>
              </div>
              <p className="settings-v2-version">v{__APP_VERSION__}</p>
              <p className="settings-v2-about-desc">{t('settings.aboutDesc')}</p>
              <div className="settings-v2-about-links">
                <span className="settings-v2-about-item">
                  <Database size={14} />
                  SQLite
                </span>
              </div>
            </section>

            {/* 更新 */}
            <section className="settings-v2-card">
              <h3 className="settings-v2-section-title">{t('update.title')}</h3>
              <UpdatePanel t={t} />
            </section>
          </div>
        )}
      </div>

      {/* Storage details modal */}
      {showDetails && (
        <div className="modal-backdrop" onClick={() => setShowDetails(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <span className="modal-title">{t('settings.storageDetailsTitle')}</span>
              <button type="button" className="modal-close" onClick={() => setShowDetails(false)} aria-label={t('back')}>
                <X size={18} />
              </button>
            </div>
            <div className="modal-body">{t('settings.storageHint')}</div>
          </div>
        </div>
      )}

      {/* Reset confirm modal */}
      {showResetConfirm && (
        <div className="modal-backdrop" onClick={() => setShowResetConfirm(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <span className="modal-title">{t('settings.resetConfirmTitle')}</span>
              <button
                type="button"
                className="modal-close"
                onClick={() => setShowResetConfirm(false)}
                aria-label={t('back')}
              >
                <X size={18} />
              </button>
            </div>
            <div className="modal-body">{t('settings.resetConfirmDesc')}</div>
            <div className="modal-footer">
              <button type="button" className="btn btn-secondary" onClick={() => setShowResetConfirm(false)}>
                {t('cancel')}
              </button>
              <button type="button" className="btn btn-warning" onClick={handleConfirmReset}>
                {t('settings.resetDefaults')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default memo(SettingsPage)
