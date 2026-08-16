import { memo, useState, type MouseEvent } from 'react'
import { Copy, Folder, GripVertical, Tag, Trash2 } from 'lucide-react'
import { toast } from 'sonner'
import type { TFunction } from 'i18next'
import type { ManagedSkill, ToolOption } from '../types'

type SkillCardProps = {
  skill: ManagedSkill
  installedTools: ToolOption[]
  loading: boolean
  getSkillSourceLabel: (skill: ManagedSkill) => string
  formatRelative: (ms: number | null | undefined) => string
  onDelete: (skillId: string) => void
  onToggleTool: (skill: ManagedSkill, toolId: string) => void
  onOpenScope: (skill: ManagedSkill) => void
  onOpenDetail: (skill: ManagedSkill) => void
  onEditTags: (skill: ManagedSkill) => void
  getSkillScope: (skill: ManagedSkill) => 'global' | 'project'
  getSkillProjects: (skill: ManagedSkill) => string[]
  draggable?: boolean
  onDragStart?: (e: React.DragEvent) => void
  onDragOver?: (e: React.DragEvent) => void
  onDrop?: (e: React.DragEvent) => void
  onDragEnd?: () => void
  isDragging?: boolean
  isDragOver?: boolean
  t: TFunction
}

const MAX_VISIBLE_TOOLS = 4

const SkillCard = ({
  skill,
  installedTools,
  loading,
  getSkillSourceLabel,
  formatRelative,
  onDelete,
  onToggleTool,
  onOpenScope,
  onOpenDetail,
  onEditTags,
  getSkillScope,
  getSkillProjects,
  draggable = false,
  onDragStart,
  onDragOver,
  onDrop,
  onDragEnd,
  isDragging = false,
  isDragOver = false,
  t,
}: SkillCardProps) => {
  const iconNode = <Folder size={18} />
  const copyValue = (skill.source_ref || getSkillSourceLabel(skill)).trim()
  const skillScope = getSkillScope(skill)
  const projectCount = getSkillProjects(skill).length

  const handleCardClick = (event: MouseEvent<HTMLDivElement>) => {
    const target = event.target as HTMLElement
    if (target.closest('button, a, input, textarea, select, [role="button"]')) {
      return
    }
    onOpenDetail(skill)
  }

  const handleCopy = async () => {
    if (!copyValue) return
    try {
      await navigator.clipboard.writeText(copyValue)
      toast.success(t('copied'))
    } catch {
      toast.error(t('copyFailed'))
    }
  }

  const syncedTools: { tool: ToolOption; target: (typeof skill.targets)[0] }[] = []
  const unsyncedTools: ToolOption[] = []
  for (const tool of installedTools) {
    const target = skill.targets.find(
      (tgt) => tgt.tool === tool.id && (tgt.scope ?? 'global') === skillScope,
    )
    if (target) {
      syncedTools.push({ tool, target })
    } else {
      unsyncedTools.push(tool)
    }
  }

  const [expanded, setExpanded] = useState(false)
  const needsCollapse = syncedTools.length > MAX_VISIBLE_TOOLS
  const visibleSynced = expanded ? syncedTools : syncedTools.slice(0, MAX_VISIBLE_TOOLS)
  const remainingCount = syncedTools.length - MAX_VISIBLE_TOOLS

  return (
    <div
      className={`skill-card clickable-card${isDragging ? ' dragging' : ''}${isDragOver ? ' drag-over' : ''}`}
      onClick={handleCardClick}
      draggable={draggable}
      onDragStart={onDragStart}
      onDragOver={onDragOver}
      onDrop={onDrop}
      onDragEnd={onDragEnd}
    >
      {draggable ? (
        <div className="drag-handle" title={t('dragToReorder')}>
          <GripVertical size={16} />
        </div>
      ) : null}
      <div className="skill-icon">{iconNode}</div>
      <div className="skill-main">
        <div className="skill-header-row">
          <button
            type="button"
            className="skill-name clickable"
            onClick={() => onOpenDetail(skill)}
          >
            {skill.name}
          </button>
          {skill.tags.length > 0 ? (
            <div className="skill-tags-inline">
              {skill.tags.slice(0, 2).map((tag) => (
                <button
                  key={tag.id}
                  className="skill-tag-pill"
                  type="button"
                  onClick={() => onEditTags(skill)}
                >
                  #{tag.name}
                </button>
              ))}
              {skill.tags.length > 2 ? (
                <button
                  className="skill-tag-pill muted"
                  type="button"
                  onClick={() => onEditTags(skill)}
                >
                  +{skill.tags.length - 2}
                </button>
              ) : null}
            </div>
          ) : null}
          {skill.version ? (
            <span className="skill-version-badge">v{skill.version}</span>
          ) : null}
          {skill.category ? (
            <span className="skill-category-badge">{skill.category}</span>
          ) : null}
        </div>
        {skill.description ? (
          <div className="skill-desc">{skill.description}</div>
        ) : null}
        <div className="skill-meta-row">
          <button
            className="repo-pill copyable"
            type="button"
            title={t('copy')}
            onClick={() => void handleCopy()}
            disabled={!copyValue}
          >
            <span className="mono">{getSkillSourceLabel(skill)}</span>
            <span className="copy-icon" aria-hidden="true">
              <Copy size={11} />
            </span>
          </button>
          <span className="skill-source time">
            <span className="dot">·</span>
            {formatRelative(skill.updated_at)}
          </span>
          <button
            className={`scope-badge ${skillScope}`}
            type="button"
            onClick={() => onOpenScope(skill)}
          >
            {skillScope === 'project'
              ? t('scope.projectCount', { count: projectCount })
              : t('scope.globalBadge')}
          </button>
        </div>
        <div className="tool-matrix">
          {visibleSynced.map(({ tool, target }) => (
            <button
              key={`${skill.id}-${tool.id}`}
              type="button"
              className="tool-pill active"
              title={`${tool.label} (${target.mode ?? t('unknown')})`}
              onClick={() => void onToggleTool(skill, tool.id)}
            >
              <span className="status-badge" />
              {tool.label}
            </button>
          ))}
          {needsCollapse && !expanded ? (
            <button
              type="button"
              className="tool-pill more-badge"
              onClick={() => setExpanded(true)}
            >
              +{remainingCount}
            </button>
          ) : null}
          {unsyncedTools.map((tool) => (
              <button
                key={`${skill.id}-${tool.id}`}
                type="button"
                className="tool-pill inactive"
                title={tool.label}
                onClick={() => void onToggleTool(skill, tool.id)}
              >
                {tool.label}
              </button>
            ))}
        </div>
      </div>
      <div className="skill-actions-col">
        <button
          className={`card-btn tag-action${skill.tags.length > 0 ? ' has-tags' : ''}`}
          type="button"
          onClick={() => onEditTags(skill)}
          disabled={loading}
          aria-label={t('editTags')}
          title={t('editTags')}
        >
          <Tag size={15} />
        </button>
        <button
          className="card-btn danger-action"
          type="button"
          onClick={() => onDelete(skill.id)}
          disabled={loading}
          aria-label={t('remove')}
        >
          <Trash2 size={15} />
        </button>
      </div>
    </div>
  )
}

export default memo(SkillCard)
