import { memo, useCallback, useMemo, useRef, useState } from 'react'
import { ArrowUpDown, Eye, GripVertical, Hash, Pencil, Plus, Search, Tag, Trash2 } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { TagWithCountDto } from '@/features/skills/types'

type TagSortMode = 'manual' | 'name' | 'updated'

type TagsPageProps = {
  tags: TagWithCountDto[]
  untaggedCount: number
  loading: boolean
  formatRelative: (ms: number | null | undefined) => string
  onReviewUntagged: () => void
  onViewTag: (tagId: number) => void
  onCreateTag: (name: string) => void
  onRenameTag: (tagId: number, name: string) => void
  onDeleteTag: (tag: TagWithCountDto) => void
  onReorder?: (items: { id: string; sort_order: number }[]) => Promise<void>
  onSortChange?: (sort: TagSortMode) => void
  defaultSort?: TagSortMode
  t: TFunction
}

const TagsPage = ({
  tags,
  untaggedCount,
  loading,
  formatRelative,
  onReviewUntagged,
  onViewTag,
  onCreateTag,
  onRenameTag,
  onDeleteTag,
  onReorder,
  onSortChange,
  defaultSort = 'manual',
  t,
}: TagsPageProps) => {
  const [query, setQuery] = useState('')
  const [newTagName, setNewTagName] = useState('')
  const [sortBy, setSortBy] = useState<TagSortMode>(defaultSort)
  const [dragIndex, setDragIndex] = useState<number | null>(null)
  const [overIndex, setOverIndex] = useState<number | null>(null)
  const dragIdRef = useRef<string | null>(null)

  const handleSortChange = useCallback((value: TagSortMode) => {
    setSortBy(value)
    onSortChange?.(value)
  }, [onSortChange])

  const filteredTags = useMemo(() => {
    const normalized = query.trim().toLowerCase()
    let list = tags
    if (normalized) {
      list = tags.filter((tag) => tag.name.toLowerCase().includes(normalized))
    }
    if (sortBy === 'name') {
      list = [...list].sort((a, b) => a.name.localeCompare(b.name))
    } else if (sortBy === 'updated') {
      list = [...list].sort((a, b) => (b.updated_at ?? 0) - (a.updated_at ?? 0))
    }
    return list
  }, [query, tags, sortBy])

  const canDrag = sortBy === 'manual' && !!onReorder

  const handleDragStart = useCallback(
    (index: number) => (e: React.DragEvent) => {
      if (!onReorder) return
      dragIdRef.current = String(filteredTags[index].id)
      setDragIndex(index)
      e.dataTransfer.effectAllowed = 'move'
      e.dataTransfer.setData('text/plain', String(index))
    },
    [filteredTags, onReorder],
  )

  const handleDragOver = useCallback(
    (index: number) => (e: React.DragEvent) => {
      if (!onReorder || dragIndex === null) return
      e.preventDefault()
      e.dataTransfer.dropEffect = 'move'
      if (overIndex !== index) setOverIndex(index)
    },
    [dragIndex, onReorder, overIndex],
  )

  const handleDrop = useCallback(
    (index: number) => (e: React.DragEvent) => {
      if (!onReorder || dragIndex === null) return
      e.preventDefault()
      const from = dragIndex
      const to = index
      setDragIndex(null)
      setOverIndex(null)
      if (from === to) return

      const next = [...filteredTags]
      const [moved] = next.splice(from, 1)
      next.splice(to, 0, moved)

      const reorderItems = next.map((tag, i) => ({
        id: String(tag.id),
        sort_order: (i + 1) * 10,
      }))
      void onReorder(reorderItems)
    },
    [dragIndex, filteredTags, onReorder],
  )

  const handleDragEnd = useCallback(() => {
    setDragIndex(null)
    setOverIndex(null)
    dragIdRef.current = null
  }, [])

  const totalTaggedCount = useMemo(
    () => tags.reduce((sum, tag) => sum + tag.skill_count, 0),
    [tags],
  )

  const submitNewTag = () => {
    const name = newTagName.trim()
    if (!name) return
    onCreateTag(name)
    setNewTagName('')
  }

  return (
    <div className="tags-page">
      <div className="detail-header tags-hero">
        <div className="tags-hero-copy">
          <div className="tags-hero-title-row">
            <span className="tags-hero-icon">
              <Tag size={20} />
            </span>
            <div className="detail-skill-name">{t('tags')}</div>
          </div>
          <div className="tags-page-subtitle">{t('tagsHelp')}</div>
          <div className="tags-metrics">
            <span>
              <strong>{tags.length}</strong>
              {t('tags')}
            </span>
            <span>
              <strong>{totalTaggedCount}</strong>
              {t('skills')}
            </span>
            <span>
              <strong>{untaggedCount}</strong>
              {t('untagged')}
            </span>
          </div>
        </div>
      </div>

      <div className="tags-review-row">
        <div className="tags-review-left">
          <span className="tags-review-icon">
            <Tag size={17} />
          </span>
          <span>{t('untaggedSkillsCount', { count: untaggedCount })}</span>
        </div>
        <button
          className="btn btn-secondary"
          type="button"
          onClick={onReviewUntagged}
          disabled={untaggedCount === 0}
        >
          {t('review')}
        </button>
      </div>

      <div className="tags-toolbar">
        <div className="search-container tags-search">
          <Search size={16} className="search-icon-abs" />
          <input
            className="search-input"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t('searchTags')}
          />
        </div>
        <button className="btn btn-secondary sort-btn" type="button">
          {sortBy === 'manual' ? t('sortManual') : sortBy === 'updated' ? t('sortUpdated') : t('sortName')}
          <ArrowUpDown size={12} />
          <select
            aria-label={t('filterSort')}
            value={sortBy}
            onChange={(event) => handleSortChange(event.target.value as TagSortMode)}
          >
            <option value="manual">{t('sortManual')}</option>
            <option value="updated">{t('sortUpdated')}</option>
            <option value="name">{t('sortName')}</option>
          </select>
        </button>
        <div className="tags-new-row">
          <input
            className="search-input"
            value={newTagName}
            onChange={(event) => setNewTagName(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter') submitNewTag()
            }}
            placeholder={t('newTagPlaceholder')}
          />
          <button
            className="btn btn-primary"
            type="button"
            onClick={submitNewTag}
            disabled={loading || !newTagName.trim()}
          >
            <Plus size={14} />
            {t('newTag')}
          </button>
        </div>
      </div>

      <div className="tags-table">
        <div className="tags-table-row tags-table-head">
          <span className="tags-drag-head" />
          <span>{t('tagName')}</span>
          <span>{t('skills')}</span>
          <span>{t('lastUsed')}</span>
          <span>{t('actionsLabel')}</span>
        </div>
        {filteredTags.length === 0 ? (
          <div className="empty tags-empty">{t('tagsEmpty')}</div>
        ) : (
          filteredTags.map((tag, index) => (
            <div
              className={`tags-table-row${dragIndex === index ? ' dragging' : ''}${overIndex === index && dragIndex !== null && dragIndex !== index ? ' drag-over' : ''}`}
              key={tag.id}
              draggable={canDrag}
              onDragStart={handleDragStart(index)}
              onDragOver={handleDragOver(index)}
              onDrop={handleDrop(index)}
              onDragEnd={handleDragEnd}
            >
              {canDrag ? (
                <span className="tags-drag-handle" title={t('dragToReorder')}>
                  <GripVertical size={14} />
                </span>
              ) : (
                <span className="tags-drag-head" />
              )}
              <span className="tags-table-name">
                <span className="tag-token">
                  <Hash size={14} />
                  {tag.name}
                </span>
              </span>
              <span className="tags-count-pill">{tag.skill_count}</span>
              <span className="tags-last-used">{formatRelative(tag.updated_at)}</span>
              <span className="tags-table-actions">
                <button className="tags-action-btn" type="button" onClick={() => onViewTag(tag.id)}>
                  <Eye size={14} />
                  {t('view')}
                </button>
                <button
                  className="tags-action-btn"
                  type="button"
                  onClick={() => {
                    const nextName = window.prompt(t('renameTagPrompt'), tag.name)
                    if (nextName?.trim()) onRenameTag(tag.id, nextName)
                  }}
                >
                  <Pencil size={14} />
                  {t('rename')}
                </button>
                <button className="tags-action-btn danger" type="button" onClick={() => onDeleteTag(tag)}>
                  <Trash2 size={14} />
                  {t('deleteAction')}
                </button>
              </span>
            </div>
          ))
        )}
      </div>
    </div>
  )
}

export default memo(TagsPage)
