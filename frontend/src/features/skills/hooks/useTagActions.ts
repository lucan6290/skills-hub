import { useCallback } from 'react'
import type { TFunction } from 'i18next'
import { tagService } from '@/services'
import type { SkillSource, TagWithCountDto } from '@/features/skills'

interface UseTagActionsParams {
  t: TFunction
  loadManagedSkills: () => Promise<void>
  loadTags: (source: SkillSource) => Promise<void>
  activeSkillSource: SkillSource
  setError: (msg: string) => void
  setSuccessToastMessage: (msg: string) => void
  setActionMessage: (msg: string | null) => void
  selectedTagIds: number[]
  setSelectedTagIds: (updater: number[] | ((prev: number[]) => number[])) => void
  pendingDeleteTag: TagWithCountDto | null
  setPendingDeleteTag: (tag: TagWithCountDto | null) => void
  globalLoading: boolean
  setLoading: (v: boolean) => void
  setLoadingStartAt: (v: number | null) => void
}

/**
 * Tag CRUD 操作 hook。
 * 从 App.tsx 提取 handleCreateTag / handleRenameTag / handleDeleteTag /
 * handleCloseDeleteTag / handleConfirmDeleteTag。
 */
export function useTagActions(params: UseTagActionsParams) {
  const {
    t,
    loadManagedSkills,
    loadTags,
    activeSkillSource,
    setError,
    setSuccessToastMessage,
    setActionMessage,
    setSelectedTagIds,
    pendingDeleteTag,
    setPendingDeleteTag,
    globalLoading,
    setLoading,
    setLoadingStartAt,
  } = params

  const handleCreateTag = useCallback(
    async (name: string) => {
      try {
        await tagService.createTag(name)
        await loadTags(activeSkillSource)
        setSuccessToastMessage(t('tagCreated'))
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err))
      }
    },
    [loadTags, activeSkillSource, setSuccessToastMessage, t, setError],
  )

  const handleRenameTag = useCallback(
    async (tagId: number, name: string) => {
      try {
        const renamed = await tagService.renameTag(tagId, name)
        setSelectedTagIds((current) =>
          current.includes(tagId)
            ? current.map((id) => (id === tagId ? renamed.id : id))
            : current,
        )
        await loadManagedSkills()
        await loadTags(activeSkillSource)
        setSuccessToastMessage(t('tagRenamed'))
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err))
      }
    },
    [
      setSelectedTagIds,
      loadManagedSkills,
      loadTags,
      activeSkillSource,
      setSuccessToastMessage,
      t,
      setError,
    ],
  )

  const handleDeleteTag = useCallback(
    (tag: TagWithCountDto) => {
      setPendingDeleteTag(tag)
    },
    [setPendingDeleteTag],
  )

  const handleCloseDeleteTag = useCallback(() => {
    if (!globalLoading) setPendingDeleteTag(null)
  }, [globalLoading, setPendingDeleteTag])

  const handleConfirmDeleteTag = useCallback(async () => {
    if (!pendingDeleteTag) return
    try {
      setLoading(true)
      setLoadingStartAt(Date.now())
      setActionMessage(
        t('actions.deletingTag', { name: pendingDeleteTag.name }),
      )
      await tagService.deleteTag(pendingDeleteTag.id)
      setSelectedTagIds((current) =>
        current.filter((id) => id !== pendingDeleteTag!.id),
      )
      await loadManagedSkills()
      await loadTags(activeSkillSource)
      setPendingDeleteTag(null)
      setSuccessToastMessage(t('tagDeleted'))
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
      setLoadingStartAt(null)
      setActionMessage(null)
    }
  }, [
    pendingDeleteTag,
    setLoading,
    setLoadingStartAt,
    setActionMessage,
    t,
    setSelectedTagIds,
    loadManagedSkills,
    loadTags,
    activeSkillSource,
    setPendingDeleteTag,
    setSuccessToastMessage,
    setError,
  ])

  return {
    handleCreateTag,
    handleRenameTag,
    handleDeleteTag,
    handleCloseDeleteTag,
    handleConfirmDeleteTag,
  }
}
