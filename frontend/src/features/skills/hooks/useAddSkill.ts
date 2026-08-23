import { useCallback, useState } from 'react'
import type { TFunction } from 'i18next'
import { invokeCommand } from '@/lib/api'
import { useApi } from '@/hooks/useApi'
import { pickFolder } from '@/lib/pickFolder'
import type {
  InstallResultDto,
  LocalSkillCandidate,
  ToolOption,
} from '../types'

interface UseAddSkillDeps {
  t: TFunction
  tools: ToolOption[]
  isInstalled: (id: string) => boolean
  uniqueToolIdsBySkillsDir: (toolIds: string[]) => string[]
  syncTargets: Record<string, boolean>
  setSyncTargets: React.Dispatch<React.SetStateAction<Record<string, boolean>>>
  loadManagedSkills: (refresh?: boolean, sourceType?: 'custom' | 'community') => Promise<void>
  loadTags: (sourceType?: 'custom' | 'community') => Promise<void>
  isSkillNameTaken: (name: string, sourceType?: string) => boolean
  showActionErrors: (errors: { title: string; message: string }[]) => void
  setError: (msg: string) => void
  loading: boolean
  setLoading: (v: boolean) => void
  setLoadingStartAt: (v: number | null) => void
  setActionMessage: (msg: string | null) => void
  setSuccessToastMessage: (msg: string) => void
}

/**
 * 添加技能流程 hook。
 * 从 App.tsx 提取 handleCreateLocal / handleInstallSelectedLocalCandidates 及相关 state。
 */
export function useAddSkill(deps: UseAddSkillDeps) {
  const {
    t,
    tools,
    isInstalled,
    uniqueToolIdsBySkillsDir,
    syncTargets,
    setSyncTargets,
    loadManagedSkills,
    loadTags,
    isSkillNameTaken,
    showActionErrors,
    setError,
    loading,
    setLoading,
    setLoadingStartAt,
    setActionMessage,
    setSuccessToastMessage,
  } = deps

  const { post } = useApi()
  const [showAddModal, setShowAddModal] = useState(false)
  const [addModalTagIds, setAddModalTagIds] = useState<number[]>([])
  const [addSourceType, setAddSourceType] = useState<'custom' | 'community'>('custom')
  const [localPath, setLocalPath] = useState('')
  const [localName, setLocalName] = useState('')
  const [localCandidates, setLocalCandidates] = useState<LocalSkillCandidate[]>([])
  const [localCandidatesBasePath, setLocalCandidatesBasePath] = useState('')
  const [showLocalPickModal, setShowLocalPickModal] = useState(false)
  const [localCandidateSelected, setLocalCandidateSelected] = useState<
    Record<string, boolean>
  >({})

  const handleOpenAdd = useCallback((sourceType: 'custom' | 'community' = 'custom') => {
    setShowAddModal(true)
    setAddSourceType(sourceType)
    setAddModalTagIds([])
    invokeCommand<string[]>('get_default_sync_tools')
      .then((ids) => {
        if (ids.length > 0) {
          setSyncTargets((prev) => {
            const next = { ...prev }
            for (const id of ids) {
              next[id] = true
            }
            return next
          })
        }
      })
      .catch(() => {})
  }, [setSyncTargets])

  const handleCloseAdd = useCallback(() => {
    if (!loading) {
      setShowAddModal(false)
      setAddModalTagIds([])
    }
  }, [loading])

  const handleToggleAddModalTag = useCallback((tagId: number) => {
    setAddModalTagIds((current) =>
      current.includes(tagId)
        ? current.filter((id) => id !== tagId)
        : [...current, tagId],
    )
  }, [])

  const handlePickLocalPath = useCallback(async () => {
    try {
      const path = await pickFolder(t('enterLocalPath'))
      if (!path) return
      setLocalPath(path)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }, [setError, t])

  const handleToggleLocalCandidate = useCallback(
    (subpath: string, checked: boolean) => {
      setLocalCandidateSelected((prev) => ({
        ...prev,
        [subpath]: checked,
      }))
    },
    [],
  )

  const handleCloseLocalPick = useCallback(() => {
    if (!loading) setShowLocalPickModal(false)
  }, [loading])

  const handleCancelLocalPick = useCallback(() => {
    if (loading) return
    setShowLocalPickModal(false)
    setLocalCandidates([])
    setLocalCandidateSelected({})
    setLocalCandidatesBasePath('')
  }, [loading])

  const applySelectedAddModalTags = useCallback(
    async (skillId: string, skillName: string) => {
      if (addModalTagIds.length === 0) return
      try {
        await post('set_skill_tags', {
          skill_id: skillId,
          tag_ids: addModalTagIds,
        })
      } catch {
        // toast handled by setError
        setError(t('tagsApplyFailed', { name: skillName }))
      }
    },
    [addModalTagIds, post, setError, t],
  )

  const handleCreateLocal = useCallback(async () => {
    if (!localPath.trim()) {
      setError(t('errors.requireLocalPath'))
      return
    }
    setLoading(true)
    setLoadingStartAt(Date.now())
    setActionMessage(t('actions.creatingLocalSkill'))
    try {
      const basePath = localPath.trim()
      const candidates = await post<LocalSkillCandidate[]>(
        'list_local_skills_cmd',
        { base_path: basePath },
      )
      if (candidates.length === 0) {
        throw new Error(t('errors.noSkillsFoundLocal'))
      }
      if (candidates.length === 1 && candidates[0].valid) {
        const desiredName = localName.trim() || candidates[0].name
        if (isSkillNameTaken(desiredName, addSourceType)) {
          setError(t('errors.skillAlreadyExists', { name: desiredName }))
          return
        }
        const created = await post<InstallResultDto>(
          'install_local_selection',
          {
            base_path: basePath,
            subpath: candidates[0].subpath,
            name: localName.trim() || undefined,
            source_type: addSourceType,
          },
        )
        await applySelectedAddModalTags(created.skill_id, created.name)
        {
          const selectedInstalledIds = tools
            .filter((tool) => syncTargets[tool.id] && isInstalled(tool.id))
            .map((t) => t.id)
          const targets = uniqueToolIdsBySkillsDir(selectedInstalledIds)
            .map((id) => tools.find((t) => t.id === id))
            .filter(Boolean) as ToolOption[]
          {
            const collectedErrors: { title: string; message: string }[] = []
            for (let i = 0; i < targets.length; i++) {
              const tool = targets[i]
              setActionMessage(
                t('actions.syncStep', {
                  index: i + 1,
                  total: targets.length,
                  name: created.name,
                  tool: tool.label,
                }),
              )
              try {
                await post('sync_skill_to_tool', {
                  source_path: created.community_path,
                  skill_id: created.skill_id,
                  tool: tool.id,
                  name: created.name,
                  overwrite_if_same_content: true,
                })
              } catch (err) {
                const raw = err instanceof Error ? err.message : String(err)
                collectedErrors.push({
                  title: t('errors.syncFailedTitle', {
                    name: created.name,
                    tool: tool.label,
                  }),
                  message: raw,
                })
              }
            }
            if (collectedErrors.length > 0) showActionErrors(collectedErrors)
          }
        }
        setLocalPath('')
        setLocalName('')
        setActionMessage(t('status.localSkillCreated'))
        setSuccessToastMessage(t('status.localSkillCreated'))
        setActionMessage(null)
        setShowAddModal(false)
        // 保存默认同步工具选择
        const selectedKeys = Object.keys(syncTargets).filter((k) => syncTargets[k])
        invokeCommand('save_default_sync_tools', { tools: selectedKeys }).catch((err) => {
          console.warn('Failed to save default sync tools:', err)
        })
        await loadManagedSkills()
        await loadTags(addSourceType)
      } else {
        setLocalCandidatesBasePath(basePath)
        setLocalCandidates(candidates)
        setLocalCandidateSelected(
          Object.fromEntries(candidates.map((c) => [c.subpath, c.valid])),
        )
        setShowLocalPickModal(true)
        setActionMessage(null)
        setLoading(false)
        setLoadingStartAt(null)
        return
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
      setLoadingStartAt(null)
    }
  }, [
    addSourceType,
    applySelectedAddModalTags,
    isInstalled,
    isSkillNameTaken,
    loadManagedSkills,
    loadTags,
    localName,
    localPath,
    post,
    setActionMessage,
    setError,
    setLoading,
    setLoadingStartAt,
    setSuccessToastMessage,
    showActionErrors,
    syncTargets,
    t,
    tools,
    uniqueToolIdsBySkillsDir,
  ])

  const handleInstallSelectedLocalCandidates = useCallback(async () => {
    const selected = localCandidates.filter(
      (c) => c.valid && localCandidateSelected[c.subpath],
    )
    if (selected.length === 0) {
      setError(t('errors.selectAtLeastOneSkill'))
      return
    }
    if (selected.length > 1 && localName.trim()) {
      setError(t('errors.multiSelectNoCustomName'))
      return
    }
    if (selected.length > 1) {
      const seen = new Set<string>()
      const dup = selected.find((c) => {
        if (seen.has(c.name)) return true
        seen.add(c.name)
        return false
      })
      if (dup) {
        setError(t('errors.duplicateSelectedSkills', { name: dup.name }))
        return
      }
    }
    const desiredName =
      selected.length === 1 && localName.trim()
        ? localName.trim()
        : selected[0].name
    if (selected.length === 1 && isSkillNameTaken(desiredName, addSourceType)) {
      setError(t('errors.skillAlreadyExists', { name: desiredName }))
      return
    }
    const duplicated = selected.find((c) => isSkillNameTaken(c.name, addSourceType))
    if (selected.length > 1 && duplicated) {
      setError(t('errors.skillAlreadyExists', { name: duplicated.name }))
      return
    }

    setLoading(true)
    setLoadingStartAt(Date.now())
    try {
      const collectedErrors: { title: string; message: string }[] = []
      for (let i = 0; i < selected.length; i++) {
        const candidate = selected[i]
        setActionMessage(
          t('actions.importStep', {
            index: i + 1,
            total: selected.length,
            name: candidate.name,
          }),
        )
        try {
          const created = await post<InstallResultDto>(
            'install_local_selection',
            {
              base_path: localCandidatesBasePath,
              subpath: candidate.subpath,
              name: localName.trim() || undefined,
              source_type: addSourceType,
            },
          )
          await applySelectedAddModalTags(created.skill_id, created.name)
          {
            const selectedInstalledIds = tools
              .filter((tool) => syncTargets[tool.id] && isInstalled(tool.id))
              .map((t) => t.id)
            const targets = uniqueToolIdsBySkillsDir(selectedInstalledIds)
              .map((id) => tools.find((t) => t.id === id))
              .filter(Boolean) as ToolOption[]
            for (let ti = 0; ti < targets.length; ti++) {
              const tool = targets[ti]
              setActionMessage(
                t('actions.syncStep', {
                  index: ti + 1,
                  total: targets.length,
                  name: created.name,
                  tool: tool.label,
                }),
              )
              try {
                await post('sync_skill_to_tool', {
                  source_path: created.community_path,
                  skill_id: created.skill_id,
                  tool: tool.id,
                  name: created.name,
                  overwrite_if_same_content: true,
                })
              } catch (err) {
                const raw = err instanceof Error ? err.message : String(err)
                collectedErrors.push({
                  title: t('errors.syncFailedTitle', {
                    name: created.name,
                    tool: tool.label,
                  }),
                  message: raw,
                })
              }
            }
          }
        } catch (err) {
          const raw = err instanceof Error ? err.message : String(err)
          collectedErrors.push({
            title: t('errors.importFailedTitle', { name: candidate.name }),
            message: raw,
          })
        }
      }

      setShowLocalPickModal(false)
      setLocalCandidates([])
      setLocalCandidateSelected({})
      setLocalCandidatesBasePath('')
      setLocalPath('')
      setLocalName('')
      setActionMessage(t('status.selectedSkillsInstalled'))
      setSuccessToastMessage(t('status.selectedSkillsInstalled'))
      setActionMessage(null)
      setShowAddModal(false)
      await loadManagedSkills()
      await loadTags(addSourceType)
      if (collectedErrors.length > 0) showActionErrors(collectedErrors)
    } finally {
      setLoading(false)
      setLoadingStartAt(null)
    }
  }, [
    addSourceType,
    applySelectedAddModalTags,
    isInstalled,
    isSkillNameTaken,
    loadManagedSkills,
    loadTags,
    localCandidateSelected,
    localCandidates,
    localCandidatesBasePath,
    localName,
    post,
    setActionMessage,
    setError,
    setLoading,
    setLoadingStartAt,
    setSuccessToastMessage,
    showActionErrors,
    syncTargets,
    t,
    tools,
    uniqueToolIdsBySkillsDir,
  ])

  return {
    showAddModal,
    addModalTagIds,
    addSourceType,
    localPath,
    localName,
    localCandidates,
    localCandidatesBasePath,
    showLocalPickModal,
    localCandidateSelected,
    handleOpenAdd,
    handleCloseAdd,
    handleToggleAddModalTag,
    handlePickLocalPath,
    handleToggleLocalCandidate,
    handleCloseLocalPick,
    handleCancelLocalPick,
    handleCreateLocal,
    handleInstallSelectedLocalCandidates,
    setAddSourceType,
    setLocalPath,
    setLocalName,
  }
}
