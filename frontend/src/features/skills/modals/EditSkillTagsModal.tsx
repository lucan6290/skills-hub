import { memo, useMemo, useState } from 'react'
import { Check, Search } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { ManagedSkill, TagWithCountDto } from '../types'

type EditSkillTagsModalProps = {
  open: boolean
  loading: boolean
  skill: ManagedSkill | null
  tags: TagWithCountDto[]
  onRequestClose: () => void
  onSave: (skill: ManagedSkill, tagIds: number[]) => void
  t: TFunction
}

const EditSkillTagsModal = ({
  open,
  loading,
  skill,
  tags,
  onRequestClose,
  onSave,
  t,
}: EditSkillTagsModalProps) => {
  const [query, setQuery] = useState('')
  const [selectedIds, setSelectedIds] = useState<number[]>(
    () => skill?.tags.map((tag) => tag.id) ?? [],
  )
  const selectedSet = useMemo(() => new Set(selectedIds), [selectedIds])
  const filteredTags = useMemo(() => {
    const normalized = query.trim().toLowerCase()
    if (!normalized) return tags
    return tags.filter((tag) => tag.name.toLowerCase().includes(normalized))
  }, [query, tags])

  if (!open || !skill) return null

  const toggleTag = (tagId: number) => {
    setSelectedIds((current) =>
      current.includes(tagId)
        ? current.filter((id) => id !== tagId)
        : [...current, tagId],
    )
  }

  return (
    <div className="modal-backdrop" onClick={loading ? undefined : onRequestClose}>
      <div className="modal edit-tags-modal" onClick={(event) => event.stopPropagation()}>
        <div className="modal-header">
          <div>
            <div className="modal-title">{t('editTagsTitle', { name: skill.name })}</div>
            <div className="modal-subtitle">{t('editTagsHelp')}</div>
          </div>
          <button className="modal-close" type="button" onClick={onRequestClose}>
            ×
          </button>
        </div>
        <div className="tag-filter-search edit-tags-search">
          <Search size={15} />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t('searchTags')}
          />
        </div>
        <div className="edit-tags-list">
          {filteredTags.length === 0 ? (
            <div className="empty">{t('tagsEmpty')}</div>
          ) : (
            filteredTags.map((tag) => {
              const selected = selectedSet.has(tag.id)
              return (
                <button
                  key={tag.id}
                  className={`tag-filter-option${selected ? ' selected' : ''}`}
                  type="button"
                  onClick={() => toggleTag(tag.id)}
                >
                  <span className="tag-check">{selected ? <Check size={14} /> : null}</span>
                  <span>{tag.name}</span>
                  <span className="tag-count">{tag.skill_count}</span>
                </button>
              )
            })
          )}
        </div>
        <div className="modal-actions">
          <button className="btn btn-secondary" type="button" onClick={onRequestClose}>
            {t('cancel')}
          </button>
          <button
            className="btn btn-primary"
            type="button"
            disabled={loading}
            onClick={() => onSave(skill, selectedIds)}
          >
            {t('done')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default memo(EditSkillTagsModal)
