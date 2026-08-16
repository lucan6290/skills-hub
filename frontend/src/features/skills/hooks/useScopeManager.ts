import { useCallback, useEffect, useMemo, useState } from 'react'
import type { TFunction } from 'i18next'
import { parseErrorDetail } from '@/lib/errors'
import { saveScopePreference } from '@/lib/api'
import { useApi } from '@/hooks/useApi'
import type { ManagedSkill, ToolOption } from '../types'

type SkillScopeState = Record<
  string,
  {
    scope: 'global' | 'project'
    projects: string[]
  }
>

interface UseScopeManagerDeps {
  t: TFunction
  tools: ToolOption[]
  installedToolIds: string[]
  installedProjectToolIds: string[]
  toolSupportsProjectScope: (toolId: string) => boolean
  sharedToolIdsByToolId: Record<string, string[]>
  toolLabelById: Record<string, string>
  skillScopeState: SkillScopeState
  setSkillScopeState: React.Dispatch<React.SetStateAction<SkillScopeState>>
  managedSkills: ManagedSkill[]
  loadManagedSkills: (refresh?: boolean) => Promise<void>
  setError: (msg: string) => void
  setActionMessage: (msg: string | null) => void
  setSuccessToastMessage: (msg: string) => void
}

/**
 * Scope 管理 hook。
 * 从 App.tsx 提取 handleScopeChange / handleOpenScope / handleCloseScope /
 * handlePickProject / setSkillScopeAndProjects / runToggleToolForSkill /
 * handleToggleToolForSkill 及相关 state。
 */
export function useScopeManager(deps: UseScopeManagerDeps) {
  const {
    t,
    tools,
    installedToolIds,
    installedProjectToolIds,
    toolSupportsProjectScope,
    sharedToolIdsByToolId,
    toolLabelById,
    skillScopeState,
    setSkillScopeState,
    managedSkills,
    loadManagedSkills,
    setError,
    setActionMessage,
    setSuccessToastMessage,
  } = deps

  const { get, post } = useApi()
  const [scopeModalSkill, setScopeModalSkill] = useState<ManagedSkill | null>(null)
  const [pendingSharedToggle, setPendingSharedToggle] = useState<{
    skill: ManagedSkill
    toolId: string
    affectedToolIds?: string[]
  } | null>(null)
  const [recentProjects, setRecentProjects] = useState<string[]>([])
  const [loading, setLoading] = useState(false)
  const [loadingStartAt, setLoadingStartAt] = useState<number | null>(null)

  // 加载最近项目
  useEffect(() => {
    get<string[]>('get_recent_projects')
      .then((projects) => setRecentProjects(projects))
      .catch(() => {})
  }, [get])

  // ─── Helpers ────────────────────────────────
  const getSkillScope = useCallback(
    (skill: ManagedSkill): 'global' | 'project' => {
      const hasGlobalTarget = skill.targets.some(
        (target) => (target.scope ?? 'global') === 'global',
      )
      const hasProjectTarget = skill.targets.some(
        (target) => (target.scope ?? 'global') === 'project',
      )
      if (hasGlobalTarget && !hasProjectTarget) return 'global'
      if (hasProjectTarget && !hasGlobalTarget) return 'project'
      const stored = skillScopeState[skill.id]?.scope
      if (stored === 'global' || stored === 'project') return stored
      return hasProjectTarget ? 'project' : 'global'
    },
    [skillScopeState],
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

  // ─── Scope Modal ────────────────────────────
  const handleOpenScope = useCallback((skill: ManagedSkill) => {
    setScopeModalSkill(skill)
  }, [])

  const handleCloseScope = useCallback(() => {
    if (!loading) setScopeModalSkill(null)
  }, [loading])

  const handlePickProject = useCallback(async () => {
    if (!scopeModalSkill) return undefined
    try {
      const path = prompt(t('projectSync.enterProjectPath'))
      if (!path) return undefined
      return path
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return undefined
    }
  }, [scopeModalSkill, setError, t])

  const setSkillScopeAndProjects = useCallback(
    (skillId: string, scope: 'global' | 'project', projects: string[]) => {
      const uniqueProjects = Array.from(new Set(projects.filter(Boolean)))
      setSkillScopeState((prev) => ({
        ...prev,
        [skillId]: {
          scope,
          projects: uniqueProjects,
        },
      }))
      saveScopePreference(skillId, scope, JSON.stringify(uniqueProjects)).catch(() => {})
    },
    [setSkillScopeState],
  )

  const handleScopeChange = useCallback(
    async (nextScope: 'global' | 'project', nextProjects: string[]) => {
      const skill = scopeModalSkill
      if (!skill || loading) return
      const projects = Array.from(new Set(nextProjects.filter(Boolean)))
      const hasStaleTargets = skill.targets.some(
        (target) =>
          (target.scope ?? 'global') !== nextScope ||
          (nextScope === 'project' &&
            (target.scope ?? 'global') === 'project' &&
            (!target.project_path || !projects.includes(target.project_path))),
      )
      const activeTargets = skill.targets.filter(
        (target) =>
          (target.scope ?? 'global') !== nextScope ||
          (nextScope === 'project' &&
            (target.scope ?? 'global') === 'project' &&
            (!target.project_path || !projects.includes(target.project_path))),
      )
      const existingProjects = getSkillProjects(skill)
      const projectsChanged =
        projects.length !== existingProjects.length ||
        projects.some((project) => !existingProjects.includes(project))
      if (getSkillScope(skill) === nextScope && !hasStaleTargets && !projectsChanged) {
        return
      }

      setLoading(true)
      setLoadingStartAt(Date.now())
      try {
        const seen = new Set<string>()
        for (const target of activeTargets) {
          const targetScope = target.scope ?? 'global'
          const key = `${target.tool}|${targetScope}|${target.project_path ?? ''}`
          if (seen.has(key)) continue
          seen.add(key)
          await post('unsync_skill_from_tool', {
            skill_id: skill.id,
            tool: target.tool,
            scope: targetScope,
            project_path: target.project_path ?? undefined,
          })
        }
        if (nextScope === 'project' && projects.length > 0) {
          for (const toolId of installedProjectToolIds) {
            for (const projectPath of projects) {
              await post('sync_skill_to_tool', {
                source_path: skill.community_path,
                skill_id: skill.id,
                tool: toolId,
                name: skill.name,
                overwrite_if_same_content: true,
                scope: 'project',
                project_path: projectPath,
              })
            }
          }
        } else if (nextScope === 'global') {
          for (const toolId of installedToolIds) {
            try {
              await post('sync_skill_to_tool', {
                source_path: skill.community_path,
                skill_id: skill.id,
                tool: toolId,
                name: skill.name,
                overwrite_if_same_content: true,
                scope: 'global',
              })
            } catch (err) {
              const raw = err instanceof Error ? err.message : String(err)
              const errDetail = parseErrorDetail(raw)
              if ('i18nKey' in errDetail && errDetail.i18nKey === 'errors.toolNotInstalled') continue
              throw err
            }
          }
        }
        await loadManagedSkills()
        if (nextScope === 'project') {
          for (const projectPath of projects) {
            const saved = await post<string[]>('save_recent_project', { project_path: projectPath })
            setRecentProjects(saved)
          }
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err))
        return
      } finally {
        setLoading(false)
        setLoadingStartAt(null)
      }

      setSkillScopeAndProjects(
        skill.id,
        nextScope,
        nextScope === 'project' ? projects : [],
      )
      setScopeModalSkill(null)
    },
    [
      getSkillProjects,
      getSkillScope,
      installedToolIds,
      installedProjectToolIds,
      loadManagedSkills,
      loading,
      post,
      scopeModalSkill,
      setError,
      setSkillScopeAndProjects,
    ],
  )

  // ─── Toggle Tool ────────────────────────────
  const runToggleToolForSkill = useCallback(
    async (skill: ManagedSkill, toolId: string) => {
      if (loading) return
      const toolLabel = tools.find((t) => t.id === toolId)?.label ?? toolId
      const skillScope = getSkillScope(skill)
      const projects = getSkillProjects(skill)
      if (skillScope === 'project') {
        if (!toolSupportsProjectScope(toolId)) {
          setError(t('projectSync.unsupportedTool', { tool: toolLabel }))
          return
        }
        if (projects.length === 0) {
          setError(t('projectSync.noProjectsForSync'))
          setScopeModalSkill(skill)
          return
        }
      }
      const matchingTargets = skill.targets.filter(
        (target) => target.tool === toolId && (target.scope ?? 'global') === skillScope,
      )
      const synced = matchingTargets.length > 0

      setLoading(true)
      setLoadingStartAt(Date.now())
      try {
        if (synced) {
          setActionMessage(
            t('actions.unsyncing', { name: skill.name, tool: toolLabel }),
          )
          if (skillScope === 'project') {
            const targetProjects = Array.from(
              new Set(
                matchingTargets
                  .map((target) => target.project_path)
                  .filter((path): path is string => Boolean(path)),
              ),
            )
            for (const projectPath of targetProjects) {
              await post('unsync_skill_from_tool', {
                skill_id: skill.id,
                tool: toolId,
                scope: 'project',
                project_path: projectPath,
              })
            }
          } else {
            await post('unsync_skill_from_tool', {
              skill_id: skill.id,
              tool: toolId,
              scope: 'global',
            })
          }
        } else {
          setActionMessage(
            t('actions.syncing', { name: skill.name, tool: toolLabel }),
          )
          if (skillScope === 'project') {
            for (const projectPath of projects) {
              await post('sync_skill_to_tool', {
                source_path: skill.community_path,
                skill_id: skill.id,
                tool: toolId,
                name: skill.name,
                overwrite_if_same_content: true,
                scope: 'project',
                project_path: projectPath,
              })
            }
          } else {
            await post('sync_skill_to_tool', {
              source_path: skill.community_path,
              skill_id: skill.id,
              tool: toolId,
              name: skill.name,
              overwrite_if_same_content: true,
              scope: 'global',
            })
          }
        }
        const statusText = synced
          ? t('status.syncDisabled')
          : t('status.syncEnabled')
        setActionMessage(statusText)
        setSuccessToastMessage(statusText)
        setActionMessage(null)
        await loadManagedSkills()
      } catch (err) {
        const raw = err instanceof Error ? err.message : String(err)
        const errDetail = parseErrorDetail(raw)
        if ('i18nKey' in errDetail && errDetail.i18nKey === 'errors.targetExists') {
          setError(t('errors.targetExistsDetail', { path: errDetail.params?.path ?? '' }))
        } else if ('i18nKey' in errDetail && errDetail.i18nKey === 'errors.toolNotInstalled') {
          setError(t('errors.toolNotInstalled'))
        } else if ('i18nKey' in errDetail && errDetail.i18nKey === 'errors.toolNotWritable') {
          setError(t('errors.toolNotWritable', { tool: errDetail.params?.tool ?? '', path: errDetail.params?.path ?? '' }))
        } else {
          setError(raw)
        }
      } finally {
        setLoading(false)
        setLoadingStartAt(null)
      }
    },
    [
      getSkillProjects,
      getSkillScope,
      loadManagedSkills,
      loading,
      post,
      setActionMessage,
      setError,
      setSuccessToastMessage,
      t,
      tools,
      toolSupportsProjectScope,
    ],
  )

  const handleToggleToolForSkill = useCallback(
    (skill: ManagedSkill, toolId: string) => {
      if (loading) return
      const skillScope = getSkillScope(skill)
      const currentTarget = skill.targets.find(
        (target) => target.tool === toolId && (target.scope ?? 'global') === skillScope,
      )
      const shared = currentTarget
        ? skill.targets
            .filter(
              (target) =>
                (target.scope ?? 'global') === skillScope &&
                target.target_path === currentTarget.target_path,
            )
            .map((target) => target.tool)
        : sharedToolIdsByToolId[toolId] ?? null
      if (shared && shared.length > 1) {
        setPendingSharedToggle({ skill, toolId, affectedToolIds: shared })
        return
      }
      void runToggleToolForSkill(skill, toolId)
    },
    [getSkillScope, loading, runToggleToolForSkill, sharedToolIdsByToolId],
  )

  const handleSharedCancel = useCallback(() => {
    if (loading) return
    setPendingSharedToggle(null)
  }, [loading])

  const handleSharedConfirm = useCallback(() => {
    if (!pendingSharedToggle) return
    const payload = pendingSharedToggle
    setPendingSharedToggle(null)
    void runToggleToolForSkill(payload.skill, payload.toolId)
  }, [pendingSharedToggle, runToggleToolForSkill])

  // ─── Derived ────────────────────────────────
  const pendingSharedLabels = useMemo(() => {
    if (!pendingSharedToggle) return null
    const toolId = pendingSharedToggle.toolId
    const shared = pendingSharedToggle.affectedToolIds ?? sharedToolIdsByToolId[toolId] ?? []
    const others = shared.filter((id) => id !== toolId)
    return {
      toolLabel: toolLabelById[toolId] ?? toolId,
      otherLabels: others.map((id) => toolLabelById[id] ?? id).join(', '),
    }
  }, [pendingSharedToggle, sharedToolIdsByToolId, toolLabelById])

  const currentScopeModalSkill = useMemo(() => {
    if (!scopeModalSkill) return null
    return managedSkills.find((skill) => skill.id === scopeModalSkill.id) ?? scopeModalSkill
  }, [managedSkills, scopeModalSkill])

  return {
    scopeModalSkill,
    pendingSharedToggle,
    recentProjects,
    loading,
    loadingStartAt,
    getSkillScope,
    getSkillProjects,
    handleOpenScope,
    handleCloseScope,
    handlePickProject,
    setSkillScopeAndProjects,
    handleScopeChange,
    runToggleToolForSkill,
    handleToggleToolForSkill,
    handleSharedCancel,
    handleSharedConfirm,
    pendingSharedLabels,
    currentScopeModalSkill,
  }
}
