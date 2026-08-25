import { memo, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import {
  AlertCircle,
  ArrowUpDown,
  CheckCircle,
  ChevronRight,
  ExternalLink,
  FileText,
  Folder,
  FolderOpen,
  GripVertical,
  Link2,
  Monitor,
  Plus,
  RefreshCw,
  RotateCcw,
  Save,
  SlidersHorizontal,
  Trash2,
  X,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import { invokeCommand, reorder as apiReorder } from '@/lib/api'
import { pickFolder } from '@/lib/pickFolder'
import { promptService } from '@/services/promptService'
import type { PromptFileDto } from '@/features/prompts/types'
import { toast } from 'sonner'

type ToolSkillEntry = {
  name: string
  path: string
  is_link: boolean
  link_target: string | null
  description: string | null
  in_community_repo: boolean
}

type ToolSkillsResponse = {
  tool_key: string
  tool_name: string
  installed: boolean
  skills_dir: string | null
  supports_project_scope: boolean
  skills: ToolSkillEntry[]
  cached: boolean
  scanned_at: number | null
}

type ToolAdapterConfig = {
  tool_key: string
  display_name: string
  skills_dir: string
  detect_dir: string
  project_skills_dir: string | null
  default_skills_dir: string | null
  default_detect_dir: string | null
  supports_symlink: boolean
  supports_junction: boolean
  force_copy: boolean
  supports_project_scope: boolean
  is_custom: boolean
  has_override: boolean
  sort_order: number
}

type ToolsPageProps = {
  t: TFunction
}

const ToolsPage = ({ t }: ToolsPageProps) => {
  const [tools, setTools] = useState<ToolSkillsResponse[]>([])
  const [loading, setLoading] = useState(true)
  const [expandedTool, setExpandedTool] = useState<string | null>(null)
  const [syncing, setSyncing] = useState<string | null>(null)
  const [deleting, setDeleting] = useState<string | null>(null)
  const [clearingTool, setClearingTool] = useState<string | null>(null)
  const [refreshing, setRefreshing] = useState(false)
  const [adapterConfigs, setAdapterConfigs] = useState<ToolAdapterConfig[]>([])
  const [editingConfig, setEditingConfig] = useState<ToolAdapterConfig | null>(null)
  const [savingConfig, setSavingConfig] = useState(false)
  const [sortBy, setSortBy] = useState<'manual' | 'name'>('manual')
  const [dragIndex, setDragIndex] = useState<number | null>(null)
  const [overIndex, setOverIndex] = useState<number | null>(null)
  const dragIdRef = useRef<string | null>(null)

  // Prompt files state
  const [promptFiles, setPromptFiles] = useState<PromptFileDto[]>([])
  const [editingPromptId, setEditingPromptId] = useState<string | null>(null)
  const [editPromptContent, setEditPromptContent] = useState('')
  const [originalPromptContent, setOriginalPromptContent] = useState('')
  const [savingPrompt, setSavingPrompt] = useState(false)

  const loadPromptFiles = useCallback(async () => {
    try {
      const files = await promptService.getPromptFiles()
      setPromptFiles(files)
    } catch {
      setPromptFiles([])
    }
  }, [])

  const promptFilesByTool = useMemo(() => {
    const map = new Map<string, PromptFileDto[]>()
    for (const pf of promptFiles) {
      const existing = map.get(pf.tool)
      if (existing) {
        existing.push(pf)
      } else {
        map.set(pf.tool, [pf])
      }
    }
    return map
  }, [promptFiles])

  const loadAdapterConfigs = useCallback(async () => {
    try {
      const data = await invokeCommand<ToolAdapterConfig[]>('get_tool_adapter_configs')
      setAdapterConfigs(data)
    } catch {
      setAdapterConfigs([])
    }
  }, [])

  const loadTools = useCallback(async (refresh = false) => {
    if (refresh) {
      setRefreshing(true)
    } else {
      setLoading(true)
    }
    try {
      const data = await invokeCommand<ToolSkillsResponse[]>(
        'get_tool_skills',
        refresh ? { refresh: true } : undefined,
      )
      setTools(data)
      if (refresh) {
        toast.success(t('toolsPage.refreshSuccess'))
      }
    } catch {
      setTools([])
    } finally {
      setLoading(false)
      setRefreshing(false)
    }
  }, [t])

  useEffect(() => {
    void loadTools()
    void loadAdapterConfigs()
    void loadPromptFiles()
  }, [loadAdapterConfigs, loadPromptFiles, loadTools])

  const handleToggle = useCallback((key: string) => {
    setExpandedTool((prev) => (prev === key ? null : key))
  }, [])

  const handleSyncToCommunity = useCallback(async (skillPath: string, skillName: string) => {
    setSyncing(skillPath)
    try {
      const result = await invokeCommand<{ ok: boolean; name: string }>('skill_to_community_repo', {
        source_path: skillPath,
        name: skillName,
      })
      toast.success(t('toolsPage.syncedToHub', { name: result.name }))
      void loadTools()
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    } finally {
      setSyncing(null)
    }
  }, [loadTools, t])

  const handleDelete = useCallback(async (toolKey: string, skillPath: string, skillName: string, isLink: boolean) => {
    const msg = isLink
      ? t('toolsPage.confirmDeleteLink', { name: skillName })
      : t('toolsPage.confirmDelete', { name: skillName })
    if (!window.confirm(msg)) return
    setDeleting(skillPath)
    try {
      await invokeCommand('delete_tool_skill', { tool_key: toolKey, skill_path: skillPath })
      toast.success(t('toolsPage.deleted', { name: skillName }))
      void loadTools()
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    } finally {
      setDeleting(null)
    }
  }, [loadTools, t])

  const handleClearTool = useCallback(async (toolKey: string, toolName: string, count: number) => {
    if (!window.confirm(t('toolsPage.confirmClearTool', { name: toolName, count }))) return
    setClearingTool(toolKey)
    try {
      const result = await invokeCommand<{ ok: boolean; removed: number }>('clear_tool_skills', { tool_key: toolKey })
      toast.success(t('toolsPage.clearedTool', { name: toolName, count: result.removed }))
      void loadTools()
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    } finally {
      setClearingTool(null)
    }
  }, [loadTools, t])

  const handleOpenFolder = useCallback(async (toolKey: string, toolName: string) => {
    try {
      await invokeCommand<{ ok: boolean; path: string }>('open_tool_skills_dir', { tool_key: toolKey })
      toast.success(t('toolsPage.openedFolder', { name: toolName }))
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    }
  }, [t])

  const handleCreateCustomTool = useCallback(() => {
    setEditingConfig({
      tool_key: '',
      display_name: '',
      skills_dir: '',
      detect_dir: '',
      project_skills_dir: null,
      default_skills_dir: null,
      default_detect_dir: null,
      supports_symlink: true,
      supports_junction: true,
      force_copy: false,
      supports_project_scope: true,
      is_custom: true,
      has_override: false,
      sort_order: 0,
    })
  }, [])

  const handleSaveConfig = useCallback(async () => {
    if (!editingConfig) return
    setSavingConfig(true)
    try {
      await invokeCommand('save_tool_adapter_config', {
        tool_key: editingConfig.tool_key,
        display_name: editingConfig.display_name,
        skills_dir: editingConfig.skills_dir,
        detect_dir: editingConfig.detect_dir,
        project_skills_dir: editingConfig.project_skills_dir || undefined,
        supports_symlink: editingConfig.supports_symlink,
        supports_junction: editingConfig.supports_junction,
        force_copy: editingConfig.force_copy,
        supports_project_scope: editingConfig.supports_project_scope,
        is_custom: editingConfig.is_custom,
      })
      toast.success(t('toolsPage.configSaved'))
      setEditingConfig(null)
      await loadAdapterConfigs()
      await loadTools(true)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    } finally {
      setSavingConfig(false)
    }
  }, [editingConfig, loadAdapterConfigs, loadTools, t])

  const handleResetConfig = useCallback(async () => {
    if (!editingConfig) return
    const message = editingConfig.is_custom
      ? t('toolsPage.confirmDeleteCustomTool', { name: editingConfig.display_name })
      : t('toolsPage.confirmResetTool', { name: editingConfig.display_name })
    if (!window.confirm(message)) return
    setSavingConfig(true)
    try {
      await invokeCommand('reset_tool_adapter_config', { tool_key: editingConfig.tool_key })
      toast.success(editingConfig.is_custom ? t('toolsPage.customToolDeleted') : t('toolsPage.configReset'))
      setEditingConfig(null)
      await loadAdapterConfigs()
      await loadTools(true)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    } finally {
      setSavingConfig(false)
    }
  }, [editingConfig, loadAdapterConfigs, loadTools, t])

  const handlePickField = useCallback(async (field: 'detect_dir' | 'skills_dir' | 'project_skills_dir') => {
    const path = await pickFolder(t('browse'))
    if (!path) return
    setEditingConfig((prev) => {
      if (!prev) return prev
      if (field === 'project_skills_dir') return { ...prev, project_skills_dir: path }
      return { ...prev, [field]: path }
    })
  }, [t])

  // Prompt file handlers
  const handleSelectPromptFile = useCallback(async (pf: PromptFileDto) => {
    if (editingPromptId === pf.id) {
      setEditingPromptId(null)
      setEditPromptContent('')
      setOriginalPromptContent('')
      return
    }
    setEditingPromptId(pf.id)
    if (!pf.exists_on_disk) {
      setEditPromptContent('')
      setOriginalPromptContent('')
      return
    }
    try {
      const content = await promptService.readPromptFile(pf.file_path)
      setEditPromptContent(content)
      setOriginalPromptContent(content)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
      setEditPromptContent('')
      setOriginalPromptContent('')
    }
  }, [editingPromptId])

  const handleSavePromptFile = useCallback(async (pf: PromptFileDto) => {
    setSavingPrompt(true)
    try {
      await promptService.writePromptFile(pf.file_path, editPromptContent)
      setOriginalPromptContent(editPromptContent)
      toast.success(t('toolsPage.promptSaved'))
      await loadPromptFiles()
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    } finally {
      setSavingPrompt(false)
    }
  }, [editPromptContent, loadPromptFiles, t])

  const handleDeletePromptFile = useCallback(async (pf: PromptFileDto) => {
    if (!window.confirm(t('toolsPage.promptDeleteConfirm', { name: pf.file_name }))) return
    try {
      await promptService.deletePromptFile(pf.id)
      setEditingPromptId(null)
      setEditPromptContent('')
      setOriginalPromptContent('')
      toast.success(t('toolsPage.promptDeleted'))
      await loadPromptFiles()
    } catch (err) {
      toast.error(err instanceof Error ? err.message : String(err))
    }
  }, [loadPromptFiles, t])

  const adapterConfigByKey = useMemo(
    () => new Map(adapterConfigs.map((config) => [config.tool_key, config])),
    [adapterConfigs],
  )

  const installedTools = useMemo(() => {
    const installed = tools.filter((t) => t.installed)
    if (sortBy === 'name') {
      return [...installed].sort((a, b) => a.tool_name.localeCompare(b.tool_name))
    }
    // manual mode: sort by adapter config sort_order, then by name
    return [...installed].sort((a, b) => {
      const cfgA = adapterConfigByKey.get(a.tool_key)
      const cfgB = adapterConfigByKey.get(b.tool_key)
      const orderA = cfgA?.sort_order ?? 0
      const orderB = cfgB?.sort_order ?? 0
      if (orderA !== orderB) return orderA - orderB
      return a.tool_name.localeCompare(b.tool_name)
    })
  }, [tools, sortBy, adapterConfigByKey])

  const notInstalledTools = useMemo(
    () => tools.filter((t) => !t.installed).sort((a, b) => a.tool_name.localeCompare(b.tool_name)),
    [tools],
  )

  const canDrag = sortBy === 'manual'

  const handleDragStart = useCallback(
    (index: number) => (e: React.DragEvent) => {
      if (!canDrag) return
      dragIdRef.current = installedTools[index].tool_key
      setDragIndex(index)
      e.dataTransfer.effectAllowed = 'move'
      e.dataTransfer.setData('text/plain', String(index))
    },
    [canDrag, installedTools],
  )

  const handleDragOver = useCallback(
    (index: number) => (e: React.DragEvent) => {
      if (!canDrag || dragIndex === null) return
      e.preventDefault()
      e.dataTransfer.dropEffect = 'move'
      if (overIndex !== index) setOverIndex(index)
    },
    [canDrag, dragIndex, overIndex],
  )

  const handleDrop = useCallback(
    (index: number) => async (e: React.DragEvent) => {
      if (!canDrag || dragIndex === null) return
      e.preventDefault()
      const from = dragIndex
      const to = index
      setDragIndex(null)
      setOverIndex(null)
      if (from === to) return

      const next = [...installedTools]
      const [moved] = next.splice(from, 1)
      next.splice(to, 0, moved)

      // 乐观更新：更新 adapterConfigs 中对应项的 sort_order
      const reorderItems = next.map((tool, i) => ({
        id: tool.tool_key,
        sort_order: (i + 1) * 10,
      }))

      setAdapterConfigs((prev) => {
        const updated = [...prev]
        const orderMap = new Map(reorderItems.map((item) => [item.id, item.sort_order]))
        return updated.map((cfg) =>
          orderMap.has(cfg.tool_key) ? { ...cfg, sort_order: orderMap.get(cfg.tool_key)! } : cfg,
        )
      })

      try {
        await apiReorder('tools', reorderItems)
      } catch (err) {
        toast.error(err instanceof Error ? err.message : String(err))
        // 失败时重新加载
        await loadAdapterConfigs()
      }
    },
    [canDrag, dragIndex, installedTools, loadAdapterConfigs],
  )

  const handleDragEnd = useCallback(() => {
    setDragIndex(null)
    setOverIndex(null)
    dragIdRef.current = null
  }, [])

  if (loading) {
    return <div className="tools-page"><div className="tools-loading">{t('toolsPage.loading')}</div></div>
  }

  return (
    <div className="tools-page">
      <div className="tools-header">
        <div>
          <h2>{t('toolsPage.title')}</h2>
          <div className="tools-subtitle">
            {t('toolsPage.subtitle', { installed: installedTools.length, total: tools.length })}
          </div>
        </div>
        <div className="tools-header-actions">
          <button className="btn btn-secondary sort-btn" type="button">
            {sortBy === 'manual' ? t('sortManual') : t('sortName')}
            <ArrowUpDown size={12} />
            <select
              aria-label={t('filterSort')}
              value={sortBy}
              onChange={(event) => setSortBy(event.target.value as 'manual' | 'name')}
            >
              <option value="manual">{t('sortManual')}</option>
              <option value="name">{t('sortName')}</option>
            </select>
          </button>
          <button
            className="btn btn-secondary tools-refresh-btn"
            type="button"
            onClick={handleCreateCustomTool}
          >
            <Plus size={15} />
            {t('toolsPage.addCustomTool')}
          </button>
          <button
            className="btn btn-secondary tools-refresh-btn"
            type="button"
            disabled={refreshing}
            onClick={() => void loadTools(true)}
          >
            <RefreshCw size={15} className={refreshing ? 'spinning' : ''} />
            {refreshing ? t('toolsPage.refreshing') : t('toolsPage.refresh')}
          </button>
        </div>
      </div>

      <div className="tools-list">
        {installedTools.map((tool, index) => (
          <div
            key={tool.tool_key}
            className={`tool-card${expandedTool === tool.tool_key ? ' expanded' : ''}${dragIndex === index ? ' dragging' : ''}${overIndex === index && dragIndex !== null && dragIndex !== index ? ' drag-over' : ''}`}
            draggable={canDrag}
            onDragStart={handleDragStart(index)}
            onDragOver={handleDragOver(index)}
            onDrop={(e) => void handleDrop(index)(e)}
            onDragEnd={handleDragEnd}
          >
            <div
              className="tool-card-header"
              role="button"
              tabIndex={0}
              onClick={() => handleToggle(tool.tool_key)}
              onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleToggle(tool.tool_key) }}
            >
              <div className="tool-card-info">
                {canDrag && (
                  <span className="tool-drag-handle" title={t('dragToReorder')}>
                    <GripVertical size={14} />
                  </span>
                )}
                <Monitor size={18} className="tool-icon" />
                <div>
                  <div className="tool-name">{tool.tool_name}</div>
                  <div className="tool-path">{tool.skills_dir}</div>
                </div>
              </div>
              <div className="tool-card-right">
                <span className="tool-skill-count">
                  {t('toolsPage.skillCount', { count: tool.skills.length })}
                </span>
                <button
                  className="btn-icon tool-open-btn"
                  type="button"
                  title={t('toolsPage.openFolder')}
                  onClick={(e) => {
                    e.stopPropagation()
                    void handleOpenFolder(tool.tool_key, tool.tool_name)
                  }}
                >
                  <FolderOpen size={14} />
                </button>
                <button
                  className="btn-icon tool-config-btn"
                  type="button"
                  title={t('toolsPage.editPaths')}
                  onClick={(e) => {
                    e.stopPropagation()
                    setEditingConfig(adapterConfigByKey.get(tool.tool_key) ?? null)
                  }}
                >
                  <SlidersHorizontal size={14} />
                </button>
                {tool.skills.length > 0 && (
                  <button
                    className="btn-icon tool-clear-btn"
                    type="button"
                    disabled={clearingTool === tool.tool_key}
                    title={t('toolsPage.clearAllSkills')}
                    onClick={(e) => {
                      e.stopPropagation()
                      void handleClearTool(tool.tool_key, tool.tool_name, tool.skills.length)
                    }}
                  >
                    <Trash2 size={14} />
                  </button>
                )}
                <ChevronRight
                  size={16}
                  className={`tool-chevron${expandedTool === tool.tool_key ? ' rotated' : ''}`}
                />
              </div>
            </div>

            {expandedTool === tool.tool_key && (
              <div className="tool-skills-list">
                {tool.skills.length === 0 ? (
                  <div className="tool-skills-empty">
                    {t('toolsPage.noSkills')}
                  </div>
                ) : (
                  tool.skills.map((skill) => (
                    <div key={skill.path} className="tool-skill-item">
                      <div className="tool-skill-left">
                        <div className="tool-skill-info">
                          {skill.is_link ? (
                            <Link2 size={14} className="skill-link-icon" />
                          ) : (
                            <Folder size={14} className="skill-folder-icon" />
                          )}
                          <div>
                            <div className="tool-skill-name">
                              {skill.name}
                              {skill.is_link && (
                                <span className="tool-skill-badge link">{t('toolsPage.symlink')}</span>
                              )}
                              {!skill.is_link && (
                                <span className="tool-skill-badge file">{t('toolsPage.original')}</span>
                              )}
                              {skill.in_community_repo && (
                                <span className="tool-skill-badge hub">{t('toolsPage.inHub')}</span>
                              )}
                            </div>
                            {skill.description && (
                              <div className="tool-skill-desc">{skill.description}</div>
                            )}
                            {skill.is_link && skill.link_target && (
                              <div className="tool-skill-link">
                                → {skill.link_target}
                              </div>
                            )}
                          </div>
                        </div>
                      </div>
                      <div className="tool-skill-actions">
                        {!skill.in_community_repo && (
                          <button
                            className="btn-icon sync-btn"
                            type="button"
                            disabled={syncing === skill.path}
                            title={t('toolsPage.syncToHub')}
                            onClick={() => handleSyncToCommunity(skill.path, skill.name)}
                          >
                            <ExternalLink size={14} />
                          </button>
                        )}
                        <button
                          className="btn-icon delete-btn"
                          type="button"
                          disabled={deleting === skill.path}
                          title={t('toolsPage.delete')}
                          onClick={() => handleDelete(tool.tool_key, skill.path, skill.name, skill.is_link)}
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </div>
                  ))
                )}

                {/* Prompt files section */}
                {(() => {
                  const toolPrompts = promptFilesByTool.get(tool.tool_key) ?? []
                  if (toolPrompts.length === 0) return null
                  return (
                    <div className="tool-prompts-section">
                      <div className="tool-prompts-divider">
                        <FileText size={13} />
                        <span>{t('toolsPage.promptFiles')}</span>
                        <span className="tool-prompts-count">{toolPrompts.length}</span>
                      </div>
                      {toolPrompts.map((pf) => {
                        const isEditing = editingPromptId === pf.id
                        const hasChanges = isEditing && editPromptContent !== originalPromptContent
                        return (
                          <div key={pf.id} className={`tool-prompt-item${isEditing ? ' active' : ''}${!pf.exists_on_disk ? ' missing' : ''}`}>
                            <div
                              className="tool-prompt-row"
                              role="button"
                              tabIndex={0}
                              onClick={() => void handleSelectPromptFile(pf)}
                              onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') void handleSelectPromptFile(pf) }}
                            >
                              <div className="tool-prompt-left">
                                <FileText size={13} className="tool-prompt-icon" />
                                <div className="tool-prompt-info">
                                  <div className="tool-prompt-name">
                                    {pf.file_name}
                                    <span className={`tool-prompt-scope ${pf.scope}`}>
                                      {pf.scope === 'global' ? t('toolsPage.promptGlobal') : t('toolsPage.promptProject')}
                                    </span>
                                  </div>
                                  <div className="tool-prompt-path">{pf.file_path}</div>
                                </div>
                              </div>
                              <div className="tool-prompt-right">
                                {pf.exists_on_disk ? (
                                  <CheckCircle size={12} className="tool-prompt-status exists" />
                                ) : (
                                  <AlertCircle size={12} className="tool-prompt-status missing" />
                                )}
                              </div>
                            </div>
                            {isEditing && (
                              <div className="tool-prompt-editor">
                                {!pf.exists_on_disk ? (
                                  <div className="tool-prompt-missing">
                                    <AlertCircle size={16} />
                                    <span>{t('toolsPage.promptFileMissing')}</span>
                                  </div>
                                ) : (
                                  <textarea
                                    className="tool-prompt-textarea"
                                    value={editPromptContent}
                                    onChange={(e) => setEditPromptContent(e.target.value)}
                                    spellCheck={false}
                                  />
                                )}
                                <div className="tool-prompt-actions">
                                  <button
                                    className="btn btn-primary btn-sm"
                                    type="button"
                                    disabled={savingPrompt || !hasChanges}
                                    onClick={() => void handleSavePromptFile(pf)}
                                  >
                                    <Save size={12} />
                                    {savingPrompt ? t('toolsPage.promptSaving') : t('toolsPage.promptSave')}
                                  </button>
                                  <button
                                    className="btn btn-secondary btn-sm tool-prompt-delete-btn"
                                    type="button"
                                    onClick={() => void handleDeletePromptFile(pf)}
                                  >
                                    <Trash2 size={12} />
                                    {t('toolsPage.promptDelete')}
                                  </button>
                                  {hasChanges && (
                                    <span className="tool-prompt-unsaved">{t('toolsPage.promptUnsaved')}</span>
                                  )}
                                </div>
                              </div>
                            )}
                          </div>
                        )
                      })}
                    </div>
                  )
                })()}
              </div>
            )}
          </div>
        ))}

        {notInstalledTools.length > 0 && (
          <div className="tools-not-installed">
            <div className="tools-not-installed-title">
              {t('toolsPage.notInstalled')} ({notInstalledTools.length})
            </div>
            <div className="tools-not-installed-grid">
              {notInstalledTools.map((tool) => (
                <div key={tool.tool_key} className="tool-badge">
                  {tool.tool_name}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>

      {editingConfig ? (
        <div className="modal-backdrop" role="presentation">
          <div className="modal tool-config-modal" role="dialog" aria-modal="true">
            <div className="modal-header">
              <div>
                <div className="modal-title">
                  {editingConfig.is_custom ? t('toolsPage.addCustomTool') : t('toolsPage.editPaths')}
                </div>
                <div className="modal-subtitle">{t('toolsPage.configHelp')}</div>
              </div>
              <button
                className="icon-btn"
                type="button"
                onClick={() => setEditingConfig(null)}
                aria-label={t('close')}
              >
                <X size={18} />
              </button>
            </div>

            <div className="tool-config-form">
              <label className="settings-field">
                <span>{t('toolsPage.toolKey')}</span>
                <input
                  className="settings-input"
                  value={editingConfig.tool_key}
                  disabled={!editingConfig.is_custom || savingConfig}
                  onChange={(event) =>
                    setEditingConfig((prev) => prev ? { ...prev, tool_key: event.target.value } : prev)
                  }
                />
              </label>
              <label className="settings-field">
                <span>{t('toolsPage.toolName')}</span>
                <input
                  className="settings-input"
                  value={editingConfig.display_name}
                  disabled={savingConfig}
                  onChange={(event) =>
                    setEditingConfig((prev) => prev ? { ...prev, display_name: event.target.value } : prev)
                  }
                />
              </label>
              <label className="settings-field tool-config-wide">
                <span>{t('toolsPage.detectDir')}</span>
                <div className="settings-input-row">
                  <input
                    className="settings-input"
                    value={editingConfig.detect_dir}
                    disabled={savingConfig}
                    onChange={(event) =>
                      setEditingConfig((prev) => prev ? { ...prev, detect_dir: event.target.value } : prev)
                    }
                  />
                  <button
                    className="btn btn-secondary settings-browse"
                    type="button"
                    disabled={savingConfig}
                    onClick={() => void handlePickField('detect_dir')}
                  >
                    {t('browse')}
                  </button>
                </div>
              </label>
              <label className="settings-field tool-config-wide">
                <span>{t('toolsPage.skillsDir')}</span>
                <div className="settings-input-row">
                  <input
                    className="settings-input"
                    value={editingConfig.skills_dir}
                    disabled={savingConfig}
                    onChange={(event) =>
                      setEditingConfig((prev) => prev ? { ...prev, skills_dir: event.target.value } : prev)
                    }
                  />
                  <button
                    className="btn btn-secondary settings-browse"
                    type="button"
                    disabled={savingConfig}
                    onClick={() => void handlePickField('skills_dir')}
                  >
                    {t('browse')}
                  </button>
                </div>
              </label>
              <label className="settings-field tool-config-wide">
                <span>{t('toolsPage.projectSkillsDir')}</span>
                <div className="settings-input-row">
                  <input
                    className="settings-input"
                    value={editingConfig.project_skills_dir ?? ''}
                    disabled={savingConfig}
                    placeholder={t('toolsPage.projectSkillsDirPlaceholder')}
                    onChange={(event) =>
                      setEditingConfig((prev) => prev ? { ...prev, project_skills_dir: event.target.value || null } : prev)
                    }
                  />
                  <button
                    className="btn btn-secondary settings-browse"
                    type="button"
                    disabled={savingConfig}
                    onClick={() => void handlePickField('project_skills_dir')}
                  >
                    {t('browse')}
                  </button>
                </div>
              </label>

              <div className="tool-config-toggles">
                <label>
                  <input
                    type="checkbox"
                    checked={editingConfig.supports_symlink}
                    disabled={savingConfig}
                    onChange={(event) =>
                      setEditingConfig((prev) => prev ? { ...prev, supports_symlink: event.target.checked } : prev)
                    }
                  />
                  {t('toolsPage.supportsSymlink')}
                </label>
                <label>
                  <input
                    type="checkbox"
                    checked={editingConfig.supports_junction}
                    disabled={savingConfig}
                    onChange={(event) =>
                      setEditingConfig((prev) => prev ? { ...prev, supports_junction: event.target.checked } : prev)
                    }
                  />
                  {t('toolsPage.supportsJunction')}
                </label>
                <label>
                  <input
                    type="checkbox"
                    checked={editingConfig.force_copy}
                    disabled={savingConfig}
                    onChange={(event) =>
                      setEditingConfig((prev) => prev ? { ...prev, force_copy: event.target.checked } : prev)
                    }
                  />
                  {t('toolsPage.forceCopy')}
                </label>
                <label>
                  <input
                    type="checkbox"
                    checked={editingConfig.supports_project_scope}
                    disabled={savingConfig}
                    onChange={(event) =>
                      setEditingConfig((prev) => prev ? { ...prev, supports_project_scope: event.target.checked } : prev)
                    }
                  />
                  {t('toolsPage.supportsProjectScope')}
                </label>
              </div>
            </div>

            <div className="modal-actions">
              {(editingConfig.has_override || editingConfig.is_custom) ? (
                <button
                  className="btn btn-secondary"
                  type="button"
                  disabled={savingConfig}
                  onClick={() => void handleResetConfig()}
                >
                  <RotateCcw size={15} />
                  {editingConfig.is_custom ? t('deleteAction') : t('toolsPage.restoreDefault')}
                </button>
              ) : null}
              <button
                className="btn btn-secondary"
                type="button"
                disabled={savingConfig}
                onClick={() => setEditingConfig(null)}
              >
                {t('cancel')}
              </button>
              <button
                className="btn btn-primary"
                type="button"
                disabled={savingConfig}
                onClick={() => void handleSaveConfig()}
              >
                {savingConfig ? t('saving') : t('save')}
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </div>
  )
}

export default memo(ToolsPage)
