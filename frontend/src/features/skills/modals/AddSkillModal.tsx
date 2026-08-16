import { memo } from 'react'
import { Check } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { TagWithCountDto, ToolOption, ToolStatusDto } from '../types'

type AddSkillModalProps = {
  open: boolean
  loading: boolean
  canClose: boolean
  localPath: string
  localName: string
  sourceType: 'custom' | 'community'
  tags: TagWithCountDto[]
  selectedTagIds: number[]
  syncTargets: Record<string, boolean>
  installedTools: ToolOption[]
  toolStatus: ToolStatusDto | null
  onRequestClose: () => void
  onLocalPathChange: (value: string) => void
  onPickLocalPath: () => void
  onLocalNameChange: (value: string) => void
  onSourceTypeChange: (sourceType: 'custom' | 'community') => void
  onToggleTag: (tagId: number) => void
  onSyncTargetChange: (toolId: string, checked: boolean) => void
  onSubmit: () => void
  t: TFunction
}

const AddSkillModal = ({
  open,
  loading,
  canClose,
  localPath,
  localName,
  sourceType,
  tags,
  selectedTagIds,
  syncTargets,
  installedTools,
  toolStatus,
  onRequestClose,
  onLocalPathChange,
  onPickLocalPath,
  onLocalNameChange,
  onSourceTypeChange,
  onToggleTag,
  onSyncTargetChange,
  onSubmit,
  t,
}: AddSkillModalProps) => {
  if (!open) return null

  return (
    <div
      className="modal-backdrop"
      onClick={() => (canClose ? onRequestClose() : null)}
    >
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <div className="modal-title">{t('addSkillTitle')}</div>
          <button
            className="modal-close"
            type="button"
            onClick={onRequestClose}
            disabled={!canClose}
          >
            ×
          </button>
        </div>
        <div className="modal-body modal-form">
          <div className="form-group">
            <label className="label">{t('sourceTabs.label')}</label>
            <div className="source-choice-group">
              <button
                className={`source-choice${sourceType === 'custom' ? ' active' : ''}`}
                type="button"
                onClick={() => onSourceTypeChange('custom')}
                disabled={!canClose}
              >
                <strong>{t('sourceTabs.custom')}</strong>
                <span>{t('sourceTabs.customHint')}</span>
              </button>
              <button
                className={`source-choice${sourceType === 'community' ? ' active' : ''}`}
                type="button"
                onClick={() => onSourceTypeChange('community')}
                disabled={!canClose}
              >
                <strong>{t('sourceTabs.community')}</strong>
                <span>{t('sourceTabs.communityHint')}</span>
              </button>
            </div>
          </div>
          <div className="form-group">
            <label className="label">{t('localFolder')}</label>
            <div className="input-row">
              <input
                className="input"
                placeholder={t('localPathPlaceholder')}
                value={localPath}
                onChange={(event) => onLocalPathChange(event.target.value)}
              />
              <button
                className="btn btn-secondary input-action"
                type="button"
                onClick={onPickLocalPath}
                disabled={!canClose}
              >
                {t('browse')}
              </button>
            </div>
          </div>
          <div className="form-group">
            <label className="label">{t('optionalNamePlaceholder')}</label>
            <input
              className="input"
              placeholder={t('optionalNamePlaceholder')}
              value={localName}
              onChange={(event) => onLocalNameChange(event.target.value)}
            />
          </div>

          <div className="form-group">
            <label className="label">{t('addTags')}</label>
            {tags.length > 0 ? (
              <div className="add-tags-list">
                {tags.map((tag) => {
                  const selected = selectedTagIds.includes(tag.id)
                  return (
                    <button
                      key={tag.id}
                      className={`add-tag-pill${selected ? ' selected' : ''}`}
                      type="button"
                      onClick={() => onToggleTag(tag.id)}
                    >
                      <span className="add-tag-check">
                        {selected ? <Check size={12} /> : null}
                      </span>
                      <span>#{tag.name}</span>
                    </button>
                  )
                })}
              </div>
            ) : (
              <div className="helper-text">{t('noTagsYet')}</div>
            )}
          </div>

          <div className="form-group">
            <label className="label">{t('installToTools')}</label>
            {toolStatus ? (
              <div className="tool-matrix">
                {installedTools.map((tool) => (
                  <label
                    key={tool.id}
                    className={`tool-pill-toggle${
                      syncTargets[tool.id] ? ' active' : ''
                    }`}
                  >
                    <input
                      type="checkbox"
                      checked={Boolean(syncTargets[tool.id])}
                      onChange={(event) =>
                        onSyncTargetChange(tool.id, event.target.checked)
                      }
                    />
                    {syncTargets[tool.id] ? <span className="status-badge" /> : null}
                    {tool.label}
                  </label>
                ))}
              </div>
            ) : (
              <div className="helper-text">{t('detectingTools')}</div>
            )}
            <div className="helper-text">{t('syncAfterCreate')}</div>
          </div>
        </div>
        <div className="modal-footer">
          <button
            className="btn btn-secondary"
            type="button"
            onClick={onRequestClose}
            disabled={!canClose}
          >
            {t('cancel')}
          </button>
          <button
            className="btn btn-primary"
            type="button"
            onClick={onSubmit}
            disabled={loading}
          >
            {t('create')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default memo(AddSkillModal)
