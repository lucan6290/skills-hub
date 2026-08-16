import { memo, useCallback, useRef, useState } from 'react'
import { MessageCircle } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { ManagedSkill, OnboardingPlan, ToolOption } from '../types'
import SkillCard from './SkillCard'

type SkillsListProps = {
  plan: OnboardingPlan | null
  visibleSkills: ManagedSkill[]
  installedTools: ToolOption[]
  loading: boolean
  getSkillSourceLabel: (skill: ManagedSkill) => string
  formatRelative: (ms: number | null | undefined) => string
  onReviewImport: () => void
  onDeleteSkill: (skillId: string) => void
  onToggleTool: (skill: ManagedSkill, toolId: string) => void
  onOpenScope: (skill: ManagedSkill) => void
  onOpenDetail: (skill: ManagedSkill) => void
  onEditTags: (skill: ManagedSkill) => void
  getSkillScope: (skill: ManagedSkill) => 'global' | 'project'
  getSkillProjects: (skill: ManagedSkill) => string[]
  draggable?: boolean
  onReorder?: (items: { id: string; sort_order: number }[]) => Promise<void>
  t: TFunction
}

const SkillsList = ({
  plan,
  visibleSkills,
  installedTools,
  loading,
  getSkillSourceLabel,
  formatRelative,
  onReviewImport,
  onDeleteSkill,
  onToggleTool,
  onOpenScope,
  onOpenDetail,
  onEditTags,
  getSkillScope,
  getSkillProjects,
  draggable = false,
  onReorder,
  t,
}: SkillsListProps) => {
  const discoveredToolCount = plan
    ? new Set(plan.groups.flatMap((g) => g.variants.map((v) => v.tool))).size
    : 0

  const [dragIndex, setDragIndex] = useState<number | null>(null)
  const [overIndex, setOverIndex] = useState<number | null>(null)
  const dragIdRef = useRef<string | null>(null)

  const handleDragStart = useCallback(
    (index: number) => (e: React.DragEvent) => {
      if (!draggable) return
      dragIdRef.current = visibleSkills[index].id
      setDragIndex(index)
      e.dataTransfer.effectAllowed = 'move'
      e.dataTransfer.setData('text/plain', String(index))
    },
    [draggable, visibleSkills],
  )

  const handleDragOver = useCallback(
    (index: number) => (e: React.DragEvent) => {
      if (!draggable || dragIndex === null) return
      e.preventDefault()
      e.dataTransfer.dropEffect = 'move'
      if (overIndex !== index) setOverIndex(index)
    },
    [draggable, dragIndex, overIndex],
  )

  const handleDrop = useCallback(
    (index: number) => (e: React.DragEvent) => {
      if (!draggable || dragIndex === null || !onReorder) return
      e.preventDefault()
      e.stopPropagation()
      const from = dragIndex
      const to = index
      setDragIndex(null)
      setOverIndex(null)
      if (from === to) return

      const next = [...visibleSkills]
      const [moved] = next.splice(from, 1)
      next.splice(to, 0, moved)

      const reorderItems = next.map((s, i) => ({
        id: s.id,
        sort_order: (i + 1) * 10,
      }))
      void onReorder(reorderItems)
    },
    [draggable, dragIndex, onReorder, visibleSkills],
  )

  const handleDragEnd = useCallback(() => {
    setDragIndex(null)
    setOverIndex(null)
    dragIdRef.current = null
  }, [])
  return (
    <div className="skills-list">
      {plan && plan.total_skills_found > 0 ? (
        <div className="discovered-banner">
          <div className="banner-left">
            <div className="banner-icon">
              <MessageCircle size={18} />
            </div>
            <div className="banner-content">
              <div className="banner-title">{t('discoveredTitle')}</div>
              <div className="banner-subtitle">
                {t('discoveredCount', {
                  count: plan.total_skills_found,
                  tools: discoveredToolCount,
                })}
              </div>
            </div>
          </div>
          <button
            className="btn btn-warning"
            type="button"
            onClick={onReviewImport}
            disabled={loading}
          >
            {t('reviewImport')}
          </button>
        </div>
      ) : null}

      {visibleSkills.length === 0 ? (
        <div className="empty">{t('skillsEmpty')}</div>
      ) : (
        <>
          {visibleSkills.map((skill, index) => (
            <SkillCard
              key={skill.id}
              skill={skill}
              installedTools={installedTools}
              loading={loading}
              getSkillSourceLabel={getSkillSourceLabel}
              formatRelative={formatRelative}
              onDelete={onDeleteSkill}
              onToggleTool={onToggleTool}
              onOpenScope={onOpenScope}
              onOpenDetail={onOpenDetail}
              onEditTags={onEditTags}
              getSkillScope={getSkillScope}
              getSkillProjects={getSkillProjects}
              draggable={draggable}
              onDragStart={handleDragStart(index)}
              onDragOver={handleDragOver(index)}
              onDrop={handleDrop(index)}
              onDragEnd={handleDragEnd}
              isDragging={dragIndex === index}
              isDragOver={overIndex === index && dragIndex !== null && dragIndex !== index}
              t={t}
            />
          ))}
        </>
      )}
    </div>
  )
}

export default memo(SkillsList)
