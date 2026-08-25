import { memo, useCallback, useEffect, useMemo, useState } from 'react'
import type { TFunction } from 'i18next'
import {
  FileText,
  RefreshCw,
  Save,
  Trash2,
  AlertCircle,
  CheckCircle,
  ChevronRight,
  Monitor,
  FolderOpen,
  X,
} from 'lucide-react'
import { promptService } from '@/services/promptService'
import type { PromptFileDto } from '../types'

type PromptsPageProps = {
  t: TFunction
}

/** Map tool keys to human-readable display names */
const TOOL_DISPLAY_NAMES: Record<string, string> = {
  cursor: 'Cursor',
  claude_code: 'Claude Code',
  codex: 'Codex',
  opencode: 'OpenCode',
  antigravity: 'Antigravity',
  amp: 'Amp',
  kimi_cli: 'Kimi Code CLI',
  augment: 'Augment',
  openclaw: 'OpenClaw',
  copaw: 'Copaw',
  cline: 'Cline',
  codebuddy: 'CodeBuddy',
  command_code: 'Command Code',
  continue: 'Continue',
  crush: 'Crush',
  junie: 'Junie',
  iflow_cli: 'iFlow CLI',
  kiro_cli: 'Kiro CLI',
  kode: 'Kode',
  mcpjam: 'MCPJam',
  mistral_vibe: 'Mistral Vibe',
  mux: 'Mux',
  openclaude: 'OpenClaude IDE',
  openhands: 'OpenHands',
  pi: 'Pi',
  qoder: 'Qoder',
  qoderwork: 'QoderWork',
  qwen_code: 'Qwen Code',
  trae: 'Trae',
  trae_cn: 'Trae CN',
  zencoder: 'Zencoder',
  neovate: 'Neovate',
  pochi: 'Pochi',
  adal: 'AdaL',
  kilo_code: 'Kilo Code',
  roo_code: 'Roo Code',
  goose: 'Goose',
  gemini_cli: 'Gemini CLI',
  github_copilot: 'GitHub Copilot',
  clawdbot: 'Clawdbot',
  droid: 'Droid',
  windsurf: 'Windsurf',
  moltbot: 'MoltBot',
  hermes_agent: 'Hermes Agent',
}

function getToolDisplayName(key: string): string {
  return TOOL_DISPLAY_NAMES[key] ?? key.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
}

const PromptsPage = ({ t }: PromptsPageProps) => {
  const [promptFiles, setPromptFiles] = useState<PromptFileDto[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedFileId, setSelectedFileId] = useState<string | null>(null)
  const [editContent, setEditContent] = useState('')
  const [originalContent, setOriginalContent] = useState('')
  const [saving, setSaving] = useState(false)
  const [scanning, setScanning] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null)
  const [expandedTools, setExpandedTools] = useState<Set<string>>(new Set())

  const loadPromptFiles = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const files = await promptService.getPromptFiles()
      setPromptFiles(files)
      // Auto-expand all groups on first load
      const tools = new Set(files.map((f) => f.tool))
      setExpandedTools(tools)
    } catch (err) {
      setError(t('prompts.loadError'))
      console.error(err)
    } finally {
      setLoading(false)
    }
  }, [t])

  useEffect(() => {
    void loadPromptFiles()
  }, [loadPromptFiles])

  const handleScan = useCallback(async () => {
    setScanning(true)
    setError(null)
    setMessage(null)
    try {
      await promptService.scanPromptFiles()
      await loadPromptFiles()
      setMessage({ type: 'success', text: t('prompts.scanDone') })
    } catch (err) {
      setError(t('prompts.scanError'))
      console.error(err)
    } finally {
      setScanning(false)
    }
  }, [loadPromptFiles, t])

  const handleSelectFile = useCallback(async (file: PromptFileDto) => {
    if (selectedFileId === file.id) {
      setSelectedFileId(null)
      setEditContent('')
      setOriginalContent('')
      setMessage(null)
      return
    }
    setSelectedFileId(file.id)
    setMessage(null)
    if (!file.exists_on_disk) {
      setEditContent('')
      setOriginalContent('')
      return
    }
    try {
      const content = await promptService.readPromptFile(file.file_path)
      setEditContent(content)
      setOriginalContent(content)
    } catch (err) {
      setMessage({ type: 'error', text: t('prompts.readError') })
      setEditContent('')
      setOriginalContent('')
      console.error(err)
    }
  }, [selectedFileId, t])

  const handleSave = useCallback(async () => {
    const selectedFile = promptFiles.find((f) => f.id === selectedFileId)
    if (!selectedFile) return
    setSaving(true)
    setMessage(null)
    try {
      await promptService.writePromptFile(selectedFile.file_path, editContent)
      setOriginalContent(editContent)
      setMessage({ type: 'success', text: t('prompts.saved') })
      await loadPromptFiles()
    } catch (err) {
      setMessage({ type: 'error', text: t('prompts.saveError') })
      console.error(err)
    } finally {
      setSaving(false)
    }
  }, [editContent, loadPromptFiles, promptFiles, selectedFileId, t])

  const handleDelete = useCallback(async () => {
    const selectedFile = promptFiles.find((f) => f.id === selectedFileId)
    if (!selectedFile) return
    if (!window.confirm(t('prompts.deleteConfirm'))) return
    setMessage(null)
    try {
      await promptService.deletePromptFile(selectedFile.id)
      setSelectedFileId(null)
      setEditContent('')
      setOriginalContent('')
      setMessage({ type: 'success', text: t('prompts.deleted') })
      await loadPromptFiles()
    } catch (err) {
      setMessage({ type: 'error', text: t('prompts.deleteError') })
      console.error(err)
    }
  }, [loadPromptFiles, promptFiles, selectedFileId, t])

  const handleCloseEditor = useCallback(() => {
    setSelectedFileId(null)
    setEditContent('')
    setOriginalContent('')
    setMessage(null)
  }, [])

  const toggleToolGroup = useCallback((tool: string) => {
    setExpandedTools((prev) => {
      const next = new Set(prev)
      if (next.has(tool)) {
        next.delete(tool)
      } else {
        next.add(tool)
      }
      return next
    })
  }, [])

  const groupedFiles = useMemo(() => {
    const groups = new Map<string, PromptFileDto[]>()
    for (const file of promptFiles) {
      const existing = groups.get(file.tool)
      if (existing) {
        existing.push(file)
      } else {
        groups.set(file.tool, [file])
      }
    }
    return Array.from(groups.entries()).sort(([a], [b]) => a.localeCompare(b))
  }, [promptFiles])

  const selectedFile = useMemo(
    () => promptFiles.find((f) => f.id === selectedFileId) ?? null,
    [promptFiles, selectedFileId],
  )

  const hasChanges = editContent !== originalContent

  const totalCount = promptFiles.length
  const existsCount = promptFiles.filter((f) => f.exists_on_disk).length

  const formatTimestamp = useCallback((ms: number) => {
    try {
      const d = new Date(ms)
      const now = new Date()
      const diffMs = now.getTime() - d.getTime()
      const diffMin = Math.floor(diffMs / 60000)
      if (diffMin < 1) return t('prompts.justNow')
      if (diffMin < 60) return t('prompts.minutesAgo', { count: diffMin })
      const diffHour = Math.floor(diffMin / 60)
      if (diffHour < 24) return t('prompts.hoursAgo', { count: diffHour })
      return d.toLocaleDateString()
    } catch {
      return String(ms)
    }
  }, [t])

  if (loading) {
    return (
      <div className="prompts-page">
        <div className="prompts-loading">{t('prompts.loading')}</div>
      </div>
    )
  }

  return (
    <div className="prompts-page">
      {/* Header card */}
      <div className="prompts-header">
        <div>
          <h2>{t('prompts.title')}</h2>
          <div className="prompts-subtitle">
            {t('prompts.subtitle', { total: totalCount, exists: existsCount })}
          </div>
        </div>
        <div className="prompts-header-actions">
          <button
            className="btn btn-secondary"
            type="button"
            disabled={scanning}
            onClick={() => void handleScan()}
          >
            <RefreshCw size={15} className={scanning ? 'spinning' : ''} />
            {scanning ? t('prompts.scanning') : t('prompts.scan')}
          </button>
        </div>
      </div>

      {/* Toast message */}
      {message && (
        <div className={`prompts-toast ${message.type}`}>
          {message.type === 'success' ? <CheckCircle size={14} /> : <AlertCircle size={14} />}
          <span>{message.text}</span>
          <button className="prompts-toast-close" type="button" onClick={() => setMessage(null)}>
            <X size={12} />
          </button>
        </div>
      )}

      {/* Error banner */}
      {error && (
        <div className="prompts-error">
          <AlertCircle size={16} />
          <span>{error}</span>
        </div>
      )}

      {/* Empty state */}
      {groupedFiles.length === 0 && !error && (
        <div className="prompts-empty">
          <div className="prompts-empty-icon">
            <FileText size={40} />
          </div>
          <p className="prompts-empty-title">{t('prompts.emptyTitle')}</p>
          <p className="prompts-empty-desc">{t('prompts.noFiles')}</p>
          <button
            className="btn btn-primary"
            type="button"
            onClick={() => void handleScan()}
          >
            <RefreshCw size={15} />
            {t('prompts.scan')}
          </button>
        </div>
      )}

      {/* Main content: list + editor side by side */}
      <div className={`prompts-content${selectedFile ? ' has-editor' : ''}`}>
        {/* File list */}
        <div className="prompts-list">
          {groupedFiles.map(([tool, files]) => {
            const isExpanded = expandedTools.has(tool)
            const toolExistsCount = files.filter((f) => f.exists_on_disk).length
            return (
              <div key={tool} className={`prompts-tool-card${isExpanded ? ' expanded' : ''}`}>
                <button
                  className="prompts-tool-header"
                  type="button"
                  onClick={() => toggleToolGroup(tool)}
                >
                  <div className="prompts-tool-info">
                    <div className="prompts-tool-icon">
                      <Monitor size={18} />
                    </div>
                    <div>
                      <div className="prompts-tool-name">{getToolDisplayName(tool)}</div>
                      <div className="prompts-tool-count">
                        {files.length} {t('prompts.files')} · {toolExistsCount} {t('prompts.existsLabel')}
                      </div>
                    </div>
                  </div>
                  <ChevronRight
                    size={16}
                    className={`prompts-chevron${isExpanded ? ' rotated' : ''}`}
                  />
                </button>

                {isExpanded && (
                  <div className="prompts-files-list">
                    {files.map((file) => (
                      <div
                        key={file.id}
                        className={`prompts-file-item${selectedFileId === file.id ? ' active' : ''}${!file.exists_on_disk ? ' missing' : ''}`}
                        role="button"
                        tabIndex={0}
                        onClick={() => void handleSelectFile(file)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter' || e.key === ' ') void handleSelectFile(file)
                        }}
                      >
                        <div className="prompts-file-left">
                          <FileText size={14} className="prompts-file-icon" />
                          <div className="prompts-file-info">
                            <div className="prompts-file-name">{file.file_name}</div>
                            <div className="prompts-file-path">{file.file_path}</div>
                          </div>
                        </div>
                        <div className="prompts-file-right">
                          <span className={`prompts-scope-badge ${file.scope}`}>
                            {file.scope === 'global' ? t('prompts.global') : t('prompts.project')}
                          </span>
                          <span
                            className={`prompts-status-indicator ${file.exists_on_disk ? 'exists' : 'missing'}`}
                            title={file.exists_on_disk ? t('prompts.exists') : t('prompts.missing')}
                          >
                            {file.exists_on_disk ? (
                              <CheckCircle size={12} />
                            ) : (
                              <AlertCircle size={12} />
                            )}
                          </span>
                          <span className="prompts-file-time">{formatTimestamp(file.last_scanned_at)}</span>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )
          })}
        </div>

        {/* Editor panel */}
        {selectedFile && (
          <div className="prompts-editor-panel">
            <div className="prompts-editor-header">
              <div className="prompts-editor-title-row">
                <FileText size={16} />
                <span className="prompts-editor-filename">{selectedFile.file_name}</span>
                <span className={`prompts-scope-badge ${selectedFile.scope}`}>
                  {selectedFile.scope === 'global' ? t('prompts.global') : t('prompts.project')}
                </span>
                {hasChanges && (
                  <span className="prompts-unsaved-badge">{t('prompts.unsaved')}</span>
                )}
              </div>
              <div className="prompts-editor-path-row">
                <FolderOpen size={12} />
                <span className="prompts-editor-path">{selectedFile.file_path}</span>
              </div>
              <div className="prompts-editor-actions">
                <button
                  className="btn btn-primary"
                  type="button"
                  disabled={saving || !hasChanges || !selectedFile.exists_on_disk}
                  onClick={() => void handleSave()}
                >
                  <Save size={14} />
                  {saving ? t('prompts.saving') : t('prompts.save')}
                </button>
                <button
                  className="btn btn-secondary prompts-delete-btn"
                  type="button"
                  onClick={() => void handleDelete()}
                >
                  <Trash2 size={14} />
                  {t('prompts.delete')}
                </button>
                <button
                  className="prompts-editor-close"
                  type="button"
                  onClick={handleCloseEditor}
                >
                  <X size={16} />
                </button>
              </div>
            </div>

            {!selectedFile.exists_on_disk ? (
              <div className="prompts-editor-missing">
                <AlertCircle size={24} />
                <p>{t('prompts.fileMissing')}</p>
                <p className="prompts-editor-missing-hint">{t('prompts.fileMissingHint')}</p>
              </div>
            ) : (
              <textarea
                className="prompts-editor-textarea"
                value={editContent}
                onChange={(e) => setEditContent(e.target.value)}
                spellCheck={false}
                placeholder={t('prompts.editorPlaceholder')}
              />
            )}
          </div>
        )}
      </div>
    </div>
  )
}

export default memo(PromptsPage)
