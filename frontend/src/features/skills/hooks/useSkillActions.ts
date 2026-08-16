import { useCallback } from 'react'
import type { TFunction } from 'i18next'
import { skillService } from '@/services'
import type { ManagedSkill, SkillSource } from '@/features/skills'

interface UseSkillActionsParams {
  t: TFunction
  loadManagedSkills: () => Promise<void>
  loadTags: (source: SkillSource) => Promise<void>
  activeSkillSource: SkillSource
  setError: (msg: string) => void
  setSuccessToastMessage: (msg: string) => void
  setActionMessage: (msg: string | null) => void
  setSkillScopeState: (
    updater: (prev: Record<string, { scope: 'global' | 'project'; projects: string[] }>) =>
      Record<string, { scope: 'global' | 'project'; projects: string[] }>,
  ) => void
  pendingDeleteId: string | null
  setPendingDeleteId: (id: string | null) => void
  closeEditTags: () => void
  globalLoading: boolean
  setLoading: (v: boolean) => void
  setLoadingStartAt: (v: number | null) => void
}

/**
 * Skill 操作 hook。
 * 从 App.tsx 提取 handleDeleteManaged / handleSaveSkillTags / handleCloseDelete。
 */
export function useSkillActions(params: UseSkillActionsParams) {
  const {
    t,
    loadManagedSkills,
    loadTags,
    activeSkillSource,
    setError,
    setSuccessToastMessage,
    setActionMessage,
    setSkillScopeState,
    setPendingDeleteId,
    closeEditTags,
    globalLoading,
    setLoading,
    setLoadingStartAt,
  } = params

  const handleCloseDelete = useCallback(() => {
    if (!globalLoading) setPendingDeleteId(null)
  }, [globalLoading, setPendingDeleteId])

  const handleDeleteManaged = useCallback(
    async (skill: ManagedSkill) => {
      setLoading(true)
      setLoadingStartAt(Date.now())
      setActionMessage(t('actions.removing', { name: skill.name }))
      try {
        await skillService.deleteManagedSkill(skill.id)
        setActionMessage(t('status.skillRemoved'))
        setSuccessToastMessage(t('status.skillRemoved'))
        setActionMessage(null)
        setSkillScopeState((prev) => {
          const next = { ...prev }
          delete next[skill.id]
          return next
        })
        await loadManagedSkills()
        await loadTags(activeSkillSource)
        setPendingDeleteId(null)
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err))
      } finally {
        setPendingDeleteId(null)
        setLoadingStartAt(null)
      }
    },
    [
      setLoading,
      setLoadingStartAt,
      setActionMessage,
      t,
      setSkillScopeState,
      loadManagedSkills,
      loadTags,
      activeSkillSource,
      setPendingDeleteId,
      setSuccessToastMessage,
      setError,
    ],
  )

  const handleSaveSkillTags = useCallback(
    async (skill: ManagedSkill, tagIds: number[]) => {
      try {
        setLoading(true)
        setLoadingStartAt(Date.now())
        setActionMessage(t('actions.updatingTags', { name: skill.name }))
        await skillService.setSkillTags(skill.id, tagIds)
        await loadManagedSkills()
        await loadTags(activeSkillSource)
        closeEditTags()
        setSuccessToastMessage(t('tagsUpdated'))
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err))
      } finally {
        setLoading(false)
        setLoadingStartAt(null)
        setActionMessage(null)
      }
    },
    [
      setLoading,
      setLoadingStartAt,
      setActionMessage,
      t,
      loadManagedSkills,
      loadTags,
      activeSkillSource,
      closeEditTags,
      setSuccessToastMessage,
      setError,
    ],
  )

  return {
    handleDeleteManaged,
    handleSaveSkillTags,
    handleCloseDelete,
  }
}
