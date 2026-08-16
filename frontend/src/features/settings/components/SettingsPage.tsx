import { memo, useState } from 'react'
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
} from 'lucide-react'
import type { TFunction } from 'i18next'
import { toast } from 'sonner'
import DatabasePanel from '@/features/database/components/DatabasePanel'
import UpdatePanel from '@/features/settings/components/UpdatePanel'
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
            {/* Language Section */}
            <section className="settings-v2-card">
              <h3 className="settings-v2-section-title">{t('settings.interfaceLanguage')}</h3>
              <p className="settings-v2-section-desc">{t('settings.languageDesc')}</p>
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
              <h3 className="settings-v2-section-title">{t('settings.appearanceTheme')}</h3>
              <p className="settings-v2-section-desc">{t('settings.themeDesc')}</p>
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

            {/* Storage Section */}
            <section className="settings-v2-card">
              <h3 className="settings-v2-section-title">{t('settings.skillsStorage')}</h3>
              <p className="settings-v2-section-desc">{t('settings.storageDesc')}</p>

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
