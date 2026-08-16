import { useCallback, useEffect, useState } from 'react'
import type { TFunction } from 'i18next'
import { parseErrorDetail } from '@/lib/errors'
import { useApi } from '@/hooks/useApi'
import type {
  OnboardingPlan,
  ToolOption,
} from '@/features/skills/types'

interface UseImportFlowDeps {
  t: TFunction
  tools: ToolOption[]
  installedToolIds: string[]
  isInstalled: (id: string) => boolean
  uniqueToolIdsBySkillsDir: (toolIds: string[]) => string[]
  sharedToolIdsByToolId: Record<string, string[]>
  toolLabelById: Record<string, string>
  loadManagedSkills: (refresh?: boolean) => Promise<void>
  isSkillNameTaken: (name: string) => boolean
  showActionErrors: (errors: { title: string; message: string }[]) => void
  setError: (msg: string) => void
  setActionMessage: (msg: string | null) => void
  setSuccessToastMessage: (msg: string) => void
}

/**
 * 导入/Onboarding 流程 hook。
 * 从 App.tsx 提取 handleImport / handleReviewImport / loadPlan /
 * handleToggleGroup / handleSelectVariant / handleSyncTargetChange 及相关 state。
 */
export function useImportFlow(deps: UseImportFlowDeps) {
  const {
    t,
    tools,
    isInstalled,
    uniqueToolIdsBySkillsDir,
    sharedToolIdsByToolId,
    toolLabelById,
    loadManagedSkills,
    showActionErrors,
    setError,
    setActionMessage,
    setSuccessToastMessage,
  } = deps

  const { get, post } = useApi()
  const [plan, setPlan] = useState<OnboardingPlan | null>(null)
  const [selected, setSelected] = useState<Record<string, boolean>>({})
  const [variantChoice, setVariantChoice] = useState<Record<string, string>>({})
  const [syncTargets, setSyncTargets] = useState<Record<string, boolean>>({})
  const [loading, setLoading] = useState(false)
  const [loadingStartAt, setLoadingStartAt] = useState<number | null>(null)

  const loadPlan = useCallback(async (showLoading = false) => {
    if (showLoading) setLoading(true)
    setLoadingStartAt(showLoading ? Date.now() : null)
    try {
      const result = await get<OnboardingPlan>('get_onboarding_plan')
      setPlan(result)
      const defaultSelected: Record<string, boolean> = {}
      const defaultChoice: Record<string, string> = {}
      result.groups.forEach((group) => {
        defaultSelected[group.name] = true
        const first = group.variants[0]
        if (first) {
          defaultChoice[group.name] = first.path
        }
      })
      setSelected(defaultSelected)
      setVariantChoice(defaultChoice)
      return result
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return null
    } finally {
      if (showLoading) setLoading(false)
      setLoadingStartAt(null)
    }
  }, [get, setError])

  // 初始加载 plan
  useEffect(() => {
    void loadPlan()
  }, [loadPlan])

  const handleToggleGroup = useCallback((groupName: string, checked: boolean) => {
    setSelected((prev) => ({
      ...prev,
      [groupName]: checked,
    }))
  }, [])

  const handleSelectVariant = useCallback((groupName: string, path: string) => {
    setVariantChoice((prev) => ({
      ...prev,
      [groupName]: path,
    }))
  }, [])

  const handleReviewImport = useCallback(async () => {
    if (plan) {
      return true // caller handles showing the modal
    }
    const result = await loadPlan(true)
    if (result) {
      return true
    }
    return false
  }, [loadPlan, plan])

  const handleSyncTargetChange = useCallback(
    (toolId: string, checked: boolean) => {
      const shared = sharedToolIdsByToolId[toolId] ?? [toolId]
      if (shared.length > 1) {
        const others = shared.filter((id) => id !== toolId)
        const otherLabels = others.map((id) => toolLabelById[id] ?? id).join(', ')
        const ok = window.confirm(
          t('sharedDirConfirm', {
            tool: toolLabelById[toolId] ?? toolId,
            others: otherLabels,
          }),
        )
        if (!ok) return
      }
      setSyncTargets((prev) => {
        const next = { ...prev }
        for (const id of shared) next[id] = checked
        return next
      })
    },
    [sharedToolIdsByToolId, t, toolLabelById],
  )

  const handleImport = useCallback(async () => {
    if (!plan) return
    if (!plan.groups.some((group) => selected[group.name])) {
      setError(t('errors.selectAtLeastOneSkill'))
      return
    }
    setLoading(true)
    setLoadingStartAt(Date.now())
    setActionMessage(null)
    try {
      const collectedErrors: { title: string; message: string }[] = []
      let successCount = 0
      for (const group of plan.groups) {
        if (!selected[group.name]) continue
        const chosenPath = variantChoice[group.name] ?? group.variants[0]?.path
        if (!chosenPath) continue
        const chosenVariant = group.variants.find((v) => v.path === chosenPath)
        const chosenVariantTool = chosenVariant?.tool ?? null
        const chosenFingerprint = chosenVariant?.fingerprint ?? null

        let installResult: {
          skill_id: string
          community_path: string
        }

        try {
          setActionMessage(t('actions.importExisting', { name: group.name }))
          installResult = await post<{
            skill_id: string
            community_path: string
          }>('import_existing_skill', {
            source_path: chosenPath,
            name: group.name,
            source_type: 'community',
          })
          successCount += 1
        } catch (err) {
          collectedErrors.push({
            title: t('errors.importFailedTitle', { name: group.name }),
            message: err instanceof Error ? err.message : String(err),
          })
          continue
        }

        const sourceToolIds = uniqueToolIdsBySkillsDir(
          group.variants
            .map((v) => v.tool)
            .filter((id) => isInstalled(id)),
        )
        const targets = sourceToolIds
          .map((id) => tools.find((t) => t.id === id))
          .filter(Boolean) as ToolOption[]
        for (const tool of targets) {
          setActionMessage(
            t('actions.syncing', { name: group.name, tool: tool.label }),
          )
          try {
            const sharedToolIds = sharedToolIdsByToolId[tool.id] ?? [tool.id]
            const hasSameContentVariant = Boolean(
              chosenFingerprint &&
                group.variants.some(
                  (variant) =>
                    sharedToolIds.includes(variant.tool) &&
                    variant.fingerprint === chosenFingerprint,
                ),
            )
            const overwrite = Boolean(
              (chosenVariantTool &&
                (chosenVariantTool === tool.id || sharedToolIds.includes(chosenVariantTool))) ||
                hasSameContentVariant,
            )
            await post('sync_skill_to_tool', {
              source_path: installResult.community_path,
              skill_id: installResult.skill_id,
              tool: tool.id,
              name: group.name,
              overwrite,
              overwrite_if_same_content: true,
            })
          } catch (err) {
            const raw = err instanceof Error ? err.message : String(err)
            const errDetail = parseErrorDetail(raw)
            if ('i18nKey' in errDetail && errDetail.i18nKey === 'errors.targetExists') {
              const targetPath = errDetail.params?.path ?? ''
              collectedErrors.push({
                title: t('errors.syncFailedTitle', {
                  name: group.name,
                  tool: tool.label,
                }),
                message: t('errors.syncTargetExistsMessage', {
                  path: targetPath,
                }),
              })
            } else {
              collectedErrors.push({
                title: t('errors.syncFailedTitle', {
                  name: group.name,
                  tool: tool.label,
                }),
                message: raw,
              })
            }
          }
        }
      }

      await loadManagedSkills()
      await loadPlan(true)
      if (collectedErrors.length > 0) {
        showActionErrors(collectedErrors)
      } else if (successCount > 0) {
        setSuccessToastMessage(t('status.importCompleted'))
      }
      return true
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return false
    } finally {
      setLoading(false)
      setLoadingStartAt(null)
    }
  }, [
    plan,
    selected,
    variantChoice,
    tools,
    isInstalled,
    uniqueToolIdsBySkillsDir,
    sharedToolIdsByToolId,
    loadManagedSkills,
    loadPlan,
    showActionErrors,
    post,
    setActionMessage,
    setError,
    setSuccessToastMessage,
    t,
  ])

  const handleCancelLoading = useCallback(() => {
    void post('cancel_current_operation').catch(() => {})
    setLoading(false)
    setLoadingStartAt(null)
    setActionMessage(null)
  }, [post, setActionMessage])

  return {
    plan,
    selected,
    variantChoice,
    syncTargets,
    loading,
    loadingStartAt,
    loadPlan,
    handleImport,
    handleReviewImport,
    handleToggleGroup,
    handleSelectVariant,
    handleSyncTargetChange,
    handleCancelLoading,
    setSyncTargets,
  }
}
