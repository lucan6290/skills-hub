import { memo, useEffect, useMemo, useRef, useState } from 'react'
import { ArrowUpDown, Check, ChevronDown, Globe, Monitor, Plus, RefreshCw, Search, Tags } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { TagWithCountDto, ToolOption } from '../types'

type FilterBarProps = {
  sortBy: 'manual' | 'updated' | 'name'
  searchQuery: string
  scopeFilter: 'all' | 'global' | 'project'
  toolFilter: string
  installedTools: ToolOption[]
  tags: TagWithCountDto[]
  selectedTagIds: number[]
  includeUntagged: boolean
  untaggedCount: number
  totalCount: number
  refreshing: boolean
  loading: boolean
  onSortChange: (value: 'manual' | 'updated' | 'name') => void
  onSearchChange: (value: string) => void
  onScopeFilterChange: (value: 'all' | 'global' | 'project') => void
  onToolFilterChange: (value: string) => void
  onRefresh: () => void
  onOpenAdd: () => void
  onToggleTag: (tagId: number) => void
  onToggleUntagged: () => void
  onClearTags: () => void
  onManageTags: () => void
  t: TFunction
}

const FilterBar = ({
  sortBy,
  searchQuery,
  scopeFilter,
  toolFilter,
  installedTools,
  tags,
  selectedTagIds,
  includeUntagged,
  untaggedCount,
  totalCount,
  refreshing,
  loading,
  onSortChange,
  onSearchChange,
  onScopeFilterChange,
  onToolFilterChange,
  onRefresh,
  onOpenAdd,
  onToggleTag,
  onToggleUntagged,
  onClearTags,
  onManageTags,
  t,
}: FilterBarProps) => {
  const [tagMenuOpen, setTagMenuOpen] = useState(false)
  const [tagQuery, setTagQuery] = useState('')
  const tagMenuRef = useRef<HTMLDivElement | null>(null)
  const scopeOptions: { value: 'all' | 'global' | 'project'; label: string }[] = [
    { value: 'all', label: t('scope.allLabel') },
    { value: 'global', label: t('scope.globalLabel') },
    { value: 'project', label: t('scope.projectLabel') },
  ]
  const selectedTagSet = useMemo(() => new Set(selectedTagIds), [selectedTagIds])
  const selectedCount = selectedTagIds.length + (includeUntagged ? 1 : 0)
  const filteredTags = useMemo(() => {
    const query = tagQuery.trim().toLowerCase()
    if (!query) return tags
    return tags.filter((tag) => tag.name.toLowerCase().includes(query))
  }, [tagQuery, tags])
  const selectedTool = installedTools.find((tool) => tool.id === toolFilter)
  const title = selectedTool
    ? t('toolFilter.skillsTitle', { tool: selectedTool.label })
    : t('allSkills')

  useEffect(() => {
    if (!tagMenuOpen) return
    const handlePointerDown = (event: MouseEvent) => {
      if (!tagMenuRef.current?.contains(event.target as Node)) {
        setTagMenuOpen(false)
      }
    }
    document.addEventListener('mousedown', handlePointerDown)
    return () => document.removeEventListener('mousedown', handlePointerDown)
  }, [tagMenuOpen])

  return (
    <div className="filter-bar">
      <div className="filter-title">
        {title}（{totalCount}）
      </div>
      <div className="filter-actions">
        <button
          className="btn btn-secondary refresh-btn"
          type="button"
          onClick={onRefresh}
          disabled={refreshing}
          title={t('refreshSkills')}
          aria-label={t('refreshSkills')}
        >
          <RefreshCw size={14} className={refreshing ? 'spin' : undefined} />
          {refreshing ? t('refreshing') : t('refresh')}
        </button>
        <button className="btn btn-secondary" type="button" onClick={onOpenAdd} disabled={loading}>
          <Plus size={14} />
          {t('newSkill')}
        </button>
        <button className="btn btn-secondary sort-btn tool-filter-btn" type="button">
          <Monitor size={14} />
          {selectedTool?.label ?? t('toolFilter.all')}
          <ChevronDown size={12} />
          <select
            aria-label={t('toolFilter.label')}
            value={toolFilter}
            onChange={(event) => onToolFilterChange(event.target.value)}
          >
            <option value="all">{t('toolFilter.all')}</option>
            {installedTools.map((tool) => (
              <option key={tool.id} value={tool.id}>
                {tool.label}
              </option>
            ))}
          </select>
        </button>
        <button className="btn btn-secondary sort-btn" type="button">
          <Globe size={14} />
          {scopeOptions.find((option) => option.value === scopeFilter)?.label ?? t('scope.allLabel')}
          <ChevronDown size={12} />
          <select
            aria-label={t('scope.filterLabel')}
            value={scopeFilter}
            onChange={(event) =>
              onScopeFilterChange(event.target.value as 'all' | 'global' | 'project')
            }
          >
            {scopeOptions.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </button>
        <button className="btn btn-secondary sort-btn" type="button">
          {sortBy === 'manual' ? t('sortManual') : sortBy === 'updated' ? t('sortUpdated') : t('sortName')}
          <ArrowUpDown size={12} />
          <select
            aria-label={t('filterSort')}
            value={sortBy}
            onChange={(event) => onSortChange(event.target.value as 'manual' | 'updated' | 'name')}
          >
            <option value="manual">{t('sortManual')}</option>
            <option value="updated">{t('sortUpdated')}</option>
            <option value="name">{t('sortName')}</option>
          </select>
        </button>
        <div className="tag-filter-wrap" ref={tagMenuRef}>
          <button
            className={`btn btn-secondary tag-filter-btn${selectedCount > 0 ? ' active' : ''}`}
            type="button"
            onClick={() => setTagMenuOpen((open) => !open)}
          >
            <Tags size={14} />
            {selectedCount > 0
              ? t('tagsSelected', { count: selectedCount })
              : t('tags')}
            <ChevronDown size={12} />
          </button>
          {tagMenuOpen ? (
            <div className="tag-filter-menu">
              <div className="tag-filter-head">
                <span>{t('tags')}</span>
                <span>{t('matchAny')}</span>
              </div>
              <div className="tag-filter-search">
                <Search size={15} />
                <input
                  value={tagQuery}
                  onChange={(event) => setTagQuery(event.target.value)}
                  placeholder={t('searchTags')}
                />
              </div>
              <div className="tag-filter-options">
                <button
                  className={`tag-filter-option${includeUntagged ? ' selected' : ''}`}
                  type="button"
                  onClick={onToggleUntagged}
                >
                  <span className="tag-check">{includeUntagged ? <Check size={14} /> : null}</span>
                  <span>{t('untagged')}</span>
                  <span className="tag-count">{untaggedCount}</span>
                </button>
                {filteredTags.map((tag) => {
                  const selected = selectedTagSet.has(tag.id)
                  return (
                    <button
                      key={tag.id}
                      className={`tag-filter-option${selected ? ' selected' : ''}`}
                      type="button"
                      onClick={() => onToggleTag(tag.id)}
                    >
                      <span className="tag-check">{selected ? <Check size={14} /> : null}</span>
                      <span>{tag.name}</span>
                      <span className="tag-count">{tag.skill_count}</span>
                    </button>
                  )
                })}
              </div>
              <div className="tag-filter-footer">
                <button type="button" onClick={onClearTags} disabled={selectedCount === 0}>
                  {t('clearAll')}
                </button>
                <button type="button" onClick={onManageTags}>
                  {t('manageTags')}
                </button>
              </div>
            </div>
          ) : null}
        </div>
        <div className="search-container">
          <Search size={16} className="search-icon-abs" />
          <input
            className="search-input"
            value={searchQuery}
            onChange={(event) => onSearchChange(event.target.value)}
            placeholder={t('searchPlaceholder')}
          />
        </div>
      </div>
    </div>
  )
}

export default memo(FilterBar)
