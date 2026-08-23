import { useCallback, useEffect, useMemo, useState } from 'react'
import type { TFunction } from 'i18next'
import { invokeCommand, reorder as apiReorder, updateSkillSourceUrl } from '@/lib/api'
import { useApi } from '@/hooks/useApi'
import type {
  ManagedSkill,
  TagWithCountDto,
  ToolOption,
  ToolStatusDto,
} from '../types'

type ToolSkillSnapshot = {
  tool_key: string
  installed: boolean
  skills: {
    name: string
  }[]
}

/**
 * 技能数据管理 hook。
 * 从 App.tsx 提取 loadManagedSkills / loadTags / loadToolSkills /
 * handleRefreshSkills / loadToolStatus 及相关 state。
 */
export function useSkills(
  t: TFunction,
  setError: (msg: string) => void,
  setSuccessToastMessage: (msg: string) => void,
) {
  const { get } = useApi()
  const [managedSkills, setManagedSkills] = useState<ManagedSkill[]>([])
  const [tags, setTags] = useState<TagWithCountDto[]>([])
  const [toolStatus, setToolStatus] = useState<ToolStatusDto | null>(null)
  const [toolSkillNamesByTool, setToolSkillNamesByTool] = useState<Record<string, string[]>>({})
  const [refreshingSkills, setRefreshingSkills] = useState(false)

  const loadManagedSkills = useCallback(async (refresh = false, sourceType?: 'custom' | 'community', sort?: string) => {
    try {
      const params = {
        ...(refresh ? { refresh: true } : {}),
        ...(sourceType ? { source_type: sourceType } : {}),
        ...(sort ? { sort } : {}),
      }
      const result = await invokeCommand<ManagedSkill[]>(
        'get_managed_skills',
        Object.keys(params).length > 0 ? params : undefined,
      )
      setManagedSkills(result)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }, [setError])

  const loadTags = useCallback(async (sourceType?: 'custom' | 'community', sort?: string) => {
    try {
      const params = {
        ...(sourceType ? { source_type: sourceType } : {}),
        ...(sort ? { sort } : {}),
      }
      const result = await get<TagWithCountDto[]>(
        'get_tags',
        Object.keys(params).length > 0 ? params : undefined,
      )
      setTags(result)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }, [get, setError])

  const applyToolSkillSnapshots = useCallback((result: ToolSkillSnapshot[]) => {
    const next: Record<string, string[]> = {}
    for (const tool of result) {
      if (!tool.installed) continue
      next[tool.tool_key] = Array.from(
        new Set(tool.skills.map((skill) => skill.name.toLowerCase())),
      )
    }
    setToolSkillNamesByTool(next)
  }, [])

  const loadToolSkills = useCallback(async (refresh = false) => {
    try {
      const result = await invokeCommand<ToolSkillSnapshot[]>(
        'get_tool_skills',
        refresh ? { refresh: true } : undefined,
      )
      applyToolSkillSnapshots(result)
    } catch (err) {
      console.warn(err)
    }
  }, [applyToolSkillSnapshots])

  const loadToolStatus = useCallback(async () => {
    try {
      const status = await get<ToolStatusDto>('get_tool_status')
      setToolStatus(status)
      if (status.newly_installed.length > 0) {
        return status
      }
    } catch (err) {
      console.warn(err)
    }
    return undefined
  }, [get])

  const handleRefreshSkills = useCallback(async (sourceType: 'custom' | 'community') => {
    if (refreshingSkills) return
    setRefreshingSkills(true)
    try {
      const toolSkillsPromise = invokeCommand<ToolSkillSnapshot[]>('get_tool_skills', { refresh: true })
      const [skills, tagResult, status, toolSkills] = await Promise.all([
        invokeCommand<ManagedSkill[]>('get_managed_skills', { refresh: true, source_type: sourceType }),
        invokeCommand<TagWithCountDto[]>('get_tags', { source_type: sourceType }),
        invokeCommand<ToolStatusDto>('get_tool_status'),
        toolSkillsPromise,
      ])
      setManagedSkills(skills)
      setTags(tagResult)
      setToolStatus(status)
      if (status.newly_installed.length > 0) {
        return status
      }
      if (toolSkills) {
        applyToolSkillSnapshots(toolSkills)
      }
      setSuccessToastMessage(t('refreshSuccess'))
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setRefreshingSkills(false)
    }
  }, [
    applyToolSkillSnapshots,
    refreshingSkills,
    setError,
    setSuccessToastMessage,
    t,
  ])

  // 初始加载
  useEffect(() => {
    loadManagedSkills()
    loadTags()
    loadToolSkills()
  }, [loadManagedSkills, loadTags, loadToolSkills])

  // 工具状态初始加载
  useEffect(() => {
    void loadToolStatus()
  }, [loadToolStatus])

  // 窗口聚焦时刷新工具状态
  useEffect(() => {
    if (typeof window === 'undefined') return undefined
    const handleFocus = () => {
      void loadToolStatus()
      void loadToolSkills()
    }
    window.addEventListener('focus', handleFocus)
    return () => window.removeEventListener('focus', handleFocus)
  }, [loadToolSkills, loadToolStatus])

  // ─── 派生数据 ────────────────────────────────
  const toolInfos = useMemo(() => toolStatus?.tools ?? [], [toolStatus])

  const tools: ToolOption[] = useMemo(() => {
    return toolInfos.map((info) => ({
      id: info.key,
      label: t(`tools.${info.key}`, { defaultValue: info.label }),
      supports_project_scope: info.supports_project_scope,
    }))
  }, [t, toolInfos])

  const toolLabelById = useMemo(() => {
    const out: Record<string, string> = {}
    for (const tool of tools) out[tool.id] = tool.label
    return out
  }, [tools])

  const sharedToolIdsByToolId = useMemo(() => {
    const byDir: Record<string, string[]> = {}
    for (const info of toolInfos) {
      const dir = info.skills_dir
      if (!byDir[dir]) byDir[dir] = []
      byDir[dir].push(info.key)
    }
    const out: Record<string, string[]> = {}
    for (const dir of Object.keys(byDir)) {
      const ids = byDir[dir]
      if (ids.length <= 1) continue
      for (const id of ids) out[id] = ids
    }
    return out
  }, [toolInfos])

  const uniqueToolIdsBySkillsDir = useCallback(
    (toolIds: string[]) => {
      const wanted = new Set(toolIds)
      const seen = new Set<string>()
      const out: string[] = []
      for (const tool of toolInfos) {
        if (!wanted.has(tool.key)) continue
        if (seen.has(tool.skills_dir)) continue
        seen.add(tool.skills_dir)
        out.push(tool.key)
      }
      return out
    },
    [toolInfos],
  )

  const installedToolIds = useMemo(
    () => toolStatus?.installed ?? [],
    [toolStatus],
  )

  const isInstalled = useCallback(
    (id: string) => installedToolIds.includes(id),
    [installedToolIds],
  )

  const installedTools = useMemo(
    () => tools.filter((tool) => installedToolIds.includes(tool.id)),
    [tools, installedToolIds],
  )

  const toolSupportsProjectScope = useCallback(
    (toolId: string) =>
      tools.find((tool) => tool.id === toolId)?.supports_project_scope ?? true,
    [tools],
  )

  const installedProjectToolIds = useMemo(
    () => installedToolIds.filter((toolId) => toolSupportsProjectScope(toolId)),
    [installedToolIds, toolSupportsProjectScope],
  )

  const newlyInstalledToolsText = useMemo(() => {
    if (!toolStatus || toolStatus.newly_installed.length === 0) return ''
    return toolStatus.newly_installed
      .map((id) => tools.find((t) => t.id === id)?.label ?? id)
      .join('、')
  }, [toolStatus, tools])

  // ─── 工具函数 ────────────────────────────────
  const getSkillSourceLabel = useCallback(
    (skill: ManagedSkill) => skill.source_ref || skill.community_path,
    [],
  )

  const formatRelative = useCallback(
    (ms: number | null | undefined) => {
      if (!ms) return t('relative.empty')
      const diff = Date.now() - ms
      if (diff < 0) return t('relative.empty')
      const minutes = Math.floor(diff / 60000)
      if (minutes < 1) return t('relative.justNow')
      if (minutes < 60) {
        return t('relative.minutesAgo', { minutes })
      }
      const hours = Math.floor(minutes / 60)
      if (hours < 24) {
        return t('relative.hoursAgo', { hours })
      }
      const days = Math.floor(hours / 24)
      return t('relative.daysAgo', { days })
    },
    [t],
  )

  const isSkillNameTaken = useCallback(
    (name: string, sourceType = 'community') =>
      managedSkills.some((skill) => {
        const normalizedSource = skill.source_type === 'custom' ? 'custom' : 'community'
        return normalizedSource === sourceType && skill.name.toLowerCase() === name.toLowerCase()
      }),
    [managedSkills],
  )

  const getSkillProjects = useCallback(
    (skill: ManagedSkill) => {
      const projects = new Set<string>()
      for (const target of skill.targets) {
        if ((target.scope ?? 'global') === 'project' && target.project_path) {
          projects.add(target.project_path)
        }
      }
      return Array.from(projects)
    },
    [],
  )

  const handleUpdateSourceUrl = useCallback(
    async (skillId: string, url: string | null): Promise<ManagedSkill> => {
      const updated = await updateSkillSourceUrl(skillId, url)
      setManagedSkills((prev) =>
        prev.map((s) => (s.id === skillId ? updated : s)),
      )
      return updated
    },
    [],
  )

  // ─── 批量排序 ────────────────────────────────
  const reorderSkills = useCallback(
    async (items: { id: string; sort_order: number }[]) => {
      // 乐观更新：先更新本地顺序
      const orderMap = new Map(items.map((item) => [item.id, item.sort_order]))
      setManagedSkills((prev) =>
        [...prev]
          .map((s) => ({ ...s, sort_order: orderMap.get(s.id) ?? s.sort_order }))
          .sort((a, b) => a.sort_order - b.sort_order),
      )
      try {
        await apiReorder('skills', items)
      } catch (err) {
        // 失败时重新加载
        setError(err instanceof Error ? err.message : String(err))
        await loadManagedSkills()
      }
    },
    [loadManagedSkills, setError],
  )

  const reorderTags = useCallback(
    async (items: { id: string; sort_order: number }[]) => {
      const orderMap = new Map(items.map((item) => [item.id, item.sort_order]))
      setTags((prev) =>
        [...prev]
          .map((t) => ({ ...t, sort_order: orderMap.get(String(t.id)) ?? t.sort_order }))
          .sort((a, b) => a.sort_order - b.sort_order),
      )
      try {
        await apiReorder('tags', items)
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err))
        await loadTags()
      }
    },
    [loadTags, setError],
  )

  return {
    // 原始数据
    managedSkills,
    tags,
    toolStatus,
    toolSkillNamesByTool,
    refreshingSkills,
    // 加载函数
    loadManagedSkills,
    loadTags,
    loadToolSkills,
    loadToolStatus,
    handleRefreshSkills,
    // 工具派生数据
    tools,
    toolLabelById,
    sharedToolIdsByToolId,
    uniqueToolIdsBySkillsDir,
    installedToolIds,
    isInstalled,
    installedTools,
    toolSupportsProjectScope,
    installedProjectToolIds,
    newlyInstalledToolsText,
    // 工具函数
    getSkillSourceLabel,
    formatRelative,
    isSkillNameTaken,
    getSkillProjects,
    handleUpdateSourceUrl,
    // 批量排序
    reorderSkills,
    reorderTags,
  }
}
