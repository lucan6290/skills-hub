import { memo } from 'react'
import { FileText, Layers, Monitor, Settings, Tag } from 'lucide-react'
import type { TFunction } from 'i18next'
import logoLight from '@/assets/logo.svg'
import logoDark from '@/assets/logo-dark.svg'

type HeaderProps = {
  language: string
  loading: boolean
  activeView: 'myskills' | 'detail' | 'settings' | 'tags' | 'tools' | 'prompts'
  activeSkillSource: 'custom' | 'community'
  skillCount: number
  customSkillCount: number
  communitySkillCount: number
  toolCount: number
  onToggleLanguage: () => void
  onOpenSettings: () => void
  onViewChange: (view: 'myskills' | 'tags' | 'tools' | 'prompts') => void
  onSkillSourceChange: (source: 'custom' | 'community') => void
  t: TFunction
}

const Header = ({
  language,
  activeView,
  activeSkillSource,
  skillCount,
  customSkillCount,
  communitySkillCount,
  toolCount,
  onToggleLanguage,
  onOpenSettings,
  onViewChange,
  onSkillSourceChange,
  t,
}: HeaderProps) => {
  return (
    <header className="skills-header">
      <div className="header-left">
        <div className="brand-area">
          <img className="brand-logo brand-logo-light" src={logoLight} alt="Skills Hub" width={36} height={36} />
          <img className="brand-logo brand-logo-dark" src={logoDark} alt="Skills Hub" width={36} height={36} />
          <div className="brand-text-wrap">
            <div className="brand-text" aria-label={t('appName')}>
              <span className="brand-word-main">{t('brand.skills')}</span>
              <span className="brand-word-accent">{t('brand.hub')}</span>
            </div>
          </div>
          <div className="header-stats">
            <span className="header-stat">
              <span className="header-stat-value">{skillCount}</span>
              <span className="header-stat-label">{t('skills')}</span>
            </span>
            <span className="header-stat-sep" />
            <span className="header-stat">
              <span className="header-stat-value">{toolCount}</span>
              <span className="header-stat-label">{t('toolsLabel')}</span>
            </span>
          </div>
        </div>
        <nav className="nav-tabs">
          <button
            className={`nav-tab${activeView === 'myskills' || activeView === 'detail' ? ' active' : ''}`}
            type="button"
            onClick={() => onViewChange('myskills')}
          >
            <Layers size={15} />
            {t('navMySkills')}
          </button>
          <button
            className={`nav-tab${activeView === 'tags' ? ' active' : ''}`}
            type="button"
            onClick={() => onViewChange('tags')}
          >
            <Tag size={15} />
            {t('navTags')}
          </button>
          <button
            className={`nav-tab${activeView === 'tools' ? ' active' : ''}`}
            type="button"
            onClick={() => onViewChange('tools')}
          >
            <Monitor size={15} />
            {t('navTools')}
          </button>
          <button
            className={`nav-tab${activeView === 'prompts' ? ' active' : ''}`}
            type="button"
            onClick={() => onViewChange('prompts')}
          >
            <FileText size={15} />
            {t('navPrompts')}
          </button>
        </nav>
        {(activeView === 'myskills' || activeView === 'detail') ? (
          <div className="source-tabs" role="tablist" aria-label={t('sourceTabs.label')}>
            <button
              className={`source-tab${activeSkillSource === 'custom' ? ' active' : ''}`}
              type="button"
              role="tab"
              aria-selected={activeSkillSource === 'custom'}
              onClick={() => onSkillSourceChange('custom')}
            >
              {t('sourceTabs.custom')}
              <span>{customSkillCount}</span>
            </button>
            <button
              className={`source-tab${activeSkillSource === 'community' ? ' active' : ''}`}
              type="button"
              role="tab"
              aria-selected={activeSkillSource === 'community'}
              onClick={() => onSkillSourceChange('community')}
            >
              {t('sourceTabs.community')}
              <span>{communitySkillCount}</span>
            </button>
          </div>
        ) : null}
      </div>
      <div className="header-actions">
        <button className="lang-btn" type="button" onClick={onToggleLanguage}>
          {language === 'en' ? t('languageShort.en') : t('languageShort.zh')}
        </button>
        <button className={`icon-btn${activeView === 'settings' ? ' active' : ''}`} type="button" onClick={onOpenSettings}>
          <Settings size={18} />
        </button>
      </div>
    </header>
  )
}

export default memo(Header)
