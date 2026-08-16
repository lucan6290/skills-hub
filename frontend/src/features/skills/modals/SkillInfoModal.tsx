import { memo, useState } from 'react'
import {
  Check,
  Copy,
  ExternalLink,
  Folder,
  Layers,
  Pencil,
  Tag,
  X,
} from 'lucide-react'
import { toast } from 'sonner'
import type { TFunction } from 'i18next'
import type { ManagedSkill, ToolOption } from '../types'

type SkillInfoModalProps = {
  skill: ManagedSkill | null
  installedTools: ToolOption[]
  loading: boolean
  getSkillSourceLabel: (skill: ManagedSkill) => string
  formatRelative: (ms: number | null | undefined) => string
  onRequestClose: () => void
  onViewFiles: (skill: ManagedSkill) => void
  onDelete: (skillId: string) => void
  onEditTags: (skill: ManagedSkill) => void
  onOpenScope: (skill: ManagedSkill) => void
  getSkillScope: (skill: ManagedSkill) => 'global' | 'project'
  getSkillProjects: (skill: ManagedSkill) => string[]
  onUpdateSourceUrl: (skillId: string, url: string | null) => Promise<ManagedSkill>
  t: TFunction
}

const SkillInfoModal = ({
  skill,
  installedTools,
  loading,
  getSkillSourceLabel,
  formatRelative,
  onRequestClose,
  onViewFiles,
  onDelete,
  onEditTags,
  onOpenScope,
  getSkillScope,
  getSkillProjects,
  onUpdateSourceUrl,
  t,
}: SkillInfoModalProps) => {
  const [editingSourceUrl, setEditingSourceUrl] = useState(false)
  const [sourceUrlDraft, setSourceUrlDraft] = useState('')
  const [savingSourceUrl, setSavingSourceUrl] = useState(false)

  if (!skill) return null

  const sourceIcon = <Folder size={20} />
  const toolLabels = new Map(installedTools.map((tool) => [tool.id, tool.label]))
  const scope = getSkillScope(skill)
  const projects = getSkillProjects(skill)
  const sourceLabel = getSkillSourceLabel(skill)

  const copyText = async (value: string) => {
    if (!value) return
    try {
      await navigator.clipboard.writeText(value)
      toast.success(t('copied'))
    } catch {
      toast.error(t('copyFailed'))
    }
  }

  const startEditSourceUrl = () => {
    setSourceUrlDraft(skill.source_url ?? '')
    setEditingSourceUrl(true)
  }

  const cancelEditSourceUrl = () => {
    setEditingSourceUrl(false)
    setSourceUrlDraft('')
  }

  const saveSourceUrl = async () => {
    const trimmed = sourceUrlDraft.split('\n').map((l) => l.trim()).filter(Boolean).join('\n')
    setSavingSourceUrl(true)
    try {
      await onUpdateSourceUrl(skill.id, trimmed || null)
      toast.success(t('detail.sourceUrlSaved'))
      setEditingSourceUrl(false)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    } finally {
      setSavingSourceUrl(false)
    }
  }

  const toolName = (toolId: string) =>
    toolLabels.get(toolId) ?? t(`tools.${toolId}`, { defaultValue: toolId })

  const sourceUrlLines = skill.source_url
    ? skill.source_url.split('\n').filter(Boolean)
    : []

  return (
    <div className="modal-backdrop" onClick={onRequestClose}>
      <div
        className="modal skill-info-modal"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="skill-info-title"
      >
        <div className="skill-info-hero">
          <div className="skill-info-title-block">
            <div className="skill-info-icon">
              {sourceIcon}
            </div>
            <div>
              <div className="skill-info-kicker">{t('detail.infoTitle')}</div>
              <h2 id="skill-info-title">{skill.name}</h2>
            </div>
          </div>
          <button
            className="modal-icon-btn"
            type="button"
            onClick={onRequestClose}
            aria-label={t('close')}
          >
            <X size={20} />
          </button>
        </div>

        <div className="skill-info-body">
          <section className="skill-info-summary">
            <div className="skill-info-section-title">{t('detail.summary')}</div>
            <p>{skill.description || t('detail.noDescription')}</p>
          </section>

          <section className="skill-info-grid" aria-label={t('detail.metadata')}>
            <div className="skill-info-field span-2">
              <span>{t('pathLabel')}</span>
              <button
                className="skill-info-copy-line mono"
                type="button"
                onClick={() => void copyText(skill.community_path)}
                title={t('copy')}
              >
                <span>{skill.community_path}</span>
                <Copy size={13} />
              </button>
            </div>
            <div className="skill-info-field">
              <span>{t('sourceLabel')}</span>
              <strong>{skill.source_type}</strong>
            </div>
            <div className="skill-info-field">
              <span>{t('statusLabel')}</span>
              <strong>{skill.status}</strong>
            </div>
            <div className="skill-info-field span-2">
              <span className="skill-info-field-label-row">
                {t('detail.sourceRef')}
              </span>
              <button
                className="skill-info-copy-line mono"
                type="button"
                onClick={() => void copyText(sourceLabel)}
                title={t('copy')}
                disabled={!sourceLabel}
              >
                <span>{sourceLabel || t('notAvailable')}</span>
                {sourceLabel ? <Copy size={13} /> : null}
              </button>
            </div>
            {skill.source_subpath ? (
              <div className="skill-info-field span-2">
                <span>{t('detail.sourceSubpath')}</span>
                <strong>{skill.source_subpath}</strong>
              </div>
            ) : null}
            <div className="skill-info-field span-2">
              <span className="skill-info-field-label-row">
                {t('detail.sourceUrl')}
                {!editingSourceUrl ? (
                  <button className="text-btn" type="button" onClick={startEditSourceUrl}>
                    <Pencil size={12} />
                    {sourceUrlLines.length > 0 ? t('edit') : t('detail.addSourceUrl')}
                  </button>
                ) : null}
              </span>
              {editingSourceUrl ? (
                <div className="skill-info-url-edit">
                  <textarea
                    className="skill-info-url-textarea"
                    value={sourceUrlDraft}
                    onChange={(e) => setSourceUrlDraft(e.target.value)}
                    placeholder={t('detail.sourceUrlPlaceholder')}
                    autoFocus
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) void saveSourceUrl()
                      if (e.key === 'Escape') cancelEditSourceUrl()
                    }}
                    rows={3}
                  />
                  <div className="skill-info-url-edit-actions">
                    <button
                      className="icon-btn"
                      type="button"
                      onClick={() => void saveSourceUrl()}
                      disabled={savingSourceUrl}
                      title={t('save')}
                    >
                      <Check size={14} />
                    </button>
                    <button
                      className="icon-btn"
                      type="button"
                      onClick={cancelEditSourceUrl}
                      disabled={savingSourceUrl}
                      title={t('cancel')}
                    >
                      <X size={14} />
                    </button>
                  </div>
                </div>
              ) : sourceUrlLines.length > 0 ? (
                <div className="skill-info-url-list">
                  {sourceUrlLines.map((url, i) => {
                    const isLink = /^https?:\/\//i.test(url)
                    return isLink ? (
                      <a
                        key={i}
                        href={url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="skill-info-source-url"
                      >
                        {url}
                        <ExternalLink size={12} />
                      </a>
                    ) : (
                      <span key={i} className="mono skill-info-source-url-text">
                        {url}
                        <button
                          type="button"
                          className="icon-btn-mini"
                          onClick={() => void copyText(url)}
                          title={t('copy')}
                          aria-label={t('copy')}
                        >
                          <Copy size={12} />
                        </button>
                      </span>
                    )
                  })}
                </div>
              ) : (
                <span className="skill-info-muted">{t('detail.noSourceUrl')}</span>
              )}
            </div>
            {skill.version ? (
              <div className="skill-info-field">
                <span>{t('detail.version')}</span>
                <strong>{skill.version}</strong>
              </div>
            ) : null}
            {skill.author ? (
              <div className="skill-info-field">
                <span>{t('detail.author')}</span>
                <strong>{skill.author}</strong>
              </div>
            ) : null}
            {skill.license ? (
              <div className="skill-info-field">
                <span>{t('detail.license')}</span>
                <strong>{skill.license}</strong>
              </div>
            ) : null}
            {skill.category ? (
              <div className="skill-info-field">
                <span>{t('detail.category')}</span>
                <strong>{skill.category}</strong>
              </div>
            ) : null}
            {skill.homepage ? (
              <div className="skill-info-field span-2">
                <span>{t('detail.homepage')}</span>
                <strong>
                  <a href={skill.homepage} target="_blank" rel="noopener noreferrer">
                    {skill.homepage}
                  </a>
                </strong>
              </div>
            ) : null}
            <div className="skill-info-field">
              <span>{t('detail.createdAt')}</span>
              <strong>{formatRelative(skill.created_at)}</strong>
            </div>
            <div className="skill-info-field">
              <span>{t('updatedLabel')}</span>
              <strong>{formatRelative(skill.updated_at)}</strong>
            </div>
            <div className="skill-info-field">
              <span>{t('detail.scope')}</span>
              <strong>
                {scope === 'project'
                  ? t('scope.projectCount', { count: projects.length })
                  : t('scope.globalBadge')}
              </strong>
            </div>
            <div className="skill-info-field">
              <span>{t('detail.lastSynced')}</span>
              <strong>{formatRelative(skill.last_sync_at)}</strong>
            </div>
          </section>

          <section className="skill-info-section">
            <div className="skill-info-section-head">
              <div className="skill-info-section-title">
                <Tag size={15} />
                {t('tags')}
              </div>
              <button className="text-btn" type="button" onClick={() => onEditTags(skill)}>
                {t('editTags')}
              </button>
            </div>
            <div className="skill-info-tags">
              {skill.tags.length > 0 ? (
                skill.tags.map((tag) => (
                  <span key={tag.id} className="skill-tag-pill">
                    #{tag.name}
                  </span>
                ))
              ) : (
                <span className="skill-info-muted">{t('noTagsYet')}</span>
              )}
            </div>
          </section>

          <section className="skill-info-section">
            <div className="skill-info-section-head">
              <div className="skill-info-section-title">
                <Layers size={15} />
                {t('detail.syncTargets')}
              </div>
              <button className="text-btn" type="button" onClick={() => onOpenScope(skill)}>
                {t('projectSync.title')}
              </button>
            </div>
            {skill.targets.length > 0 ? (
              <div className="skill-info-targets">
                {skill.targets.map((target, index) => (
                  <div
                    className="skill-info-target"
                    key={`${target.tool}-${target.scope}-${target.target_path}-${index}`}
                  >
                    <div>
                      <strong>{toolName(target.tool)}</strong>
                      <span>
                        {target.scope === 'project'
                          ? t('scope.project')
                          : t('scope.global')}
                        {' · '}
                        {target.mode || t('unknown')}
                        {' · '}
                        {formatRelative(target.synced_at)}
                      </span>
                    </div>
                    <div className="skill-info-target-path mono">{target.target_path}</div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="skill-info-empty">{t('detail.noSyncTargets')}</div>
            )}
          </section>
        </div>

        <div className="modal-footer skill-info-footer">
          <button
            className="btn btn-secondary"
            type="button"
            onClick={() => onDelete(skill.id)}
            disabled={loading}
          >
            {t('remove')}
          </button>
          <div className="skill-info-footer-actions">
            <button
              className="btn btn-primary"
              type="button"
              onClick={() => onViewFiles(skill)}
            >
              <ExternalLink size={16} />
              {t('detail.viewFiles')}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

export default memo(SkillInfoModal)
