import { useCallback, useMemo, useState } from 'react'
import type { ManagedSkill } from '../types'

/**
 * 技能搜索/过滤/排序 hook。
 * 从 App.tsx 提取 searchQuery / sortBy / scopeFilter / toolFilter / selectedTagIds /
 * includeUntagged / visibleSkills / untaggedCount 相关逻辑。
 */
export function useSkillFilter(
  managedSkills: ManagedSkill[],
  getSkillScope: (skill: ManagedSkill) => 'global' | 'project',
  toolSkillNamesByTool: Record<string, string[]>,
) {
  const [searchQuery, setSearchQuery] = useState('')
  const [sortBy, setSortBy] = useState<'manual' | 'updated' | 'name'>('manual')
  const [scopeFilter, setScopeFilter] = useState<'all' | 'global' | 'project'>('all')
  const [toolFilter, setToolFilter] = useState('all')
  const [selectedTagIds, setSelectedTagIds] = useState<number[]>([])
  const [includeUntagged, setIncludeUntagged] = useState(false)

  const visibleSkills = useMemo(() => {
    const query = searchQuery.trim().toLowerCase()
    const selectedTagSet = new Set(selectedTagIds)
    const hasTagFilter = selectedTagIds.length > 0 || includeUntagged
    const selectedToolSkillNames =
      toolFilter === 'all'
        ? null
        : new Set(toolSkillNamesByTool[toolFilter] ?? [])
    const filtered = managedSkills.filter((skill) => {
      if (
        toolFilter !== 'all' &&
        !skill.targets.some((target) => target.tool === toolFilter) &&
        !selectedToolSkillNames?.has(skill.name.toLowerCase())
      ) {
        return false
      }
      if (scopeFilter !== 'all' && getSkillScope(skill) !== scopeFilter) return false
      if (hasTagFilter) {
        const matchesSelectedTag = skill.tags.some((tag) => selectedTagSet.has(tag.id))
        const matchesUntagged = includeUntagged && skill.tags.length === 0
        if (!matchesSelectedTag && !matchesUntagged) return false
      }
      if (!query) return true
      return (
        skill.name.toLowerCase().includes(query) ||
        skill.community_path.toLowerCase().includes(query) ||
        skill.source_type.toLowerCase().includes(query) ||
        skill.tags.some((tag) => tag.name.toLowerCase().includes(query))
      )
    })
    const sorted = [...filtered].sort((a, b) => {
      if (sortBy === 'manual') return 0
      if (sortBy === 'name') {
        return a.name.localeCompare(b.name)
      }
      return (b.updated_at ?? 0) - (a.updated_at ?? 0)
    })
    return sorted
  }, [
    getSkillScope,
    includeUntagged,
    managedSkills,
    scopeFilter,
    searchQuery,
    selectedTagIds,
    sortBy,
    toolFilter,
    toolSkillNamesByTool,
  ])

  const untaggedCount = useMemo(
    () => managedSkills.filter((skill) => skill.tags.length === 0).length,
    [managedSkills],
  )

  const handleSortChange = useCallback((value: 'manual' | 'updated' | 'name') => {
    setSortBy(value)
  }, [])

  const handleSearchChange = useCallback((value: string) => {
    setSearchQuery(value)
  }, [])

  const handleScopeFilterChange = useCallback(
    (value: 'all' | 'global' | 'project') => {
      setScopeFilter(value)
    },
    [],
  )

  const handleToolFilterChange = useCallback((value: string) => {
    setToolFilter(value)
  }, [])

  const handleToggleTagFilter = useCallback((tagId: number) => {
    setSelectedTagIds((current) =>
      current.includes(tagId)
        ? current.filter((id) => id !== tagId)
        : [...current, tagId],
    )
  }, [])

  const handleToggleUntaggedFilter = useCallback(() => {
    setIncludeUntagged((current) => !current)
  }, [])

  const handleClearTagFilters = useCallback(() => {
    setSelectedTagIds([])
    setIncludeUntagged(false)
  }, [])

  return {
    searchQuery,
    sortBy,
    scopeFilter,
    toolFilter,
    selectedTagIds,
    includeUntagged,
    visibleSkills,
    untaggedCount,
    handleSortChange,
    handleSearchChange,
    handleScopeFilterChange,
    handleToolFilterChange,
    handleToggleTagFilter,
    handleToggleUntaggedFilter,
    handleClearTagFilters,
    setSelectedTagIds,
    setIncludeUntagged,
  }
}
