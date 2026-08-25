import { memo, useCallback, useEffect, useMemo, useState } from 'react'
import type { TFunction } from 'i18next'
import { FileText, RefreshCw, Save, Trash2, AlertCircle, CheckCircle } from 'lucide-react'
import { promptService } from '@/services/promptService'
import type { PromptFileDto } from '../types'

type PromptsPageProps = {
  t: TFunction
}

const PromptsPage = ({ t }: PromptsPageProps) => {
  const [promptFiles, setPromptFiles] = useState<PromptFileDto[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedFileId, setSelectedFileId] = useState<string | null>(null)
  const [editContent, setEditContent] = useState('')
  const [saving, setSaving] = useState(false)
  const [scanning, setScanning] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null)

  const loadPromptFiles = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const files = await promptService.getPromptFiles()
      setPromptFiles(files)
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
    try {
      await promptService.scanPromptFiles()
      await loadPromptFiles()
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
      setMessage(null)
      return
    }
    setSelectedFileId(file.id)
    setMessage(null)
    try {
      const content = await promptService.readPromptFile(file.file_path)
      setEditContent(content)
    } catch (err) {
      setMessage({ type: 'error', text: t('prompts.readError') })
      setEditContent('')
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
      setMessage({ type: 'success', text: t('prompts.deleted') })
      await loadPromptFiles()
    } catch (err) {
      setMessage({ type: 'error', text: t('prompts.deleteError') })
      console.error(err)
    }
  }, [loadPromptFiles, promptFiles, selectedFileId, t])

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

  const formatTimestamp = useCallback((ms: number) => {
    try {
      return new Date(ms).toLocaleString()
    } catch {
      return String(ms)
    }
  }, [])

  if (loading) {
    return (
      <div className="prompts-page">
        <div className="prompts-empty">{t('prompts.scanning')}</div>
      </div>
    )
  }

  return (
    <div className="prompts-page">
      <div className="prompts-header">
        <h2>{t('prompts.title')}</h2>
        <button
          className="btn btn-secondary prompts-scan-btn"
          type="button"
          disabled={scanning}
          onClick={() => void handleScan()}
        >
          <RefreshCw size={15} className={scanning ? 'spinning' : ''} />
          {scanning ? t('prompts.scanning') : t('prompts.scan')}
        </button>
      </div>

      {error && (
        <div className="prompts-error">
          <AlertCircle size={16} />
          <span>{error}</span>
        </div>
      )}

      {groupedFiles.length === 0 && !error && (
        <div className="prompts-empty">
          <FileText size={32} />
          <p>{t('prompts.noFiles')}</p>
        </div>
      )}

      {groupedFiles.map(([tool, files]) => (
        <div key={tool} className="prompts-group">
          <div className="prompts-group-title">{tool}</div>
          {files.map((file) => (
            <div
              key={file.id}
              className={`prompts-file-item${selectedFileId === file.id ? ' selected' : ''}`}
              role="button"
              tabIndex={0}
              onClick={() => void handleSelectFile(file)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') void handleSelectFile(file)
              }}
            >
              <FileText size={16} className="prompts-file-icon" />
              <div className="prompts-file-info">
                <div className="prompts-file-name">{file.file_name}</div>
                <div className="prompts-file-meta">
                  <span className={`prompts-scope-badge ${file.scope}`}>
                    {file.scope === 'global' ? t('prompts.global') : t('prompts.project')}
                  </span>
                  <span
                    className={`prompts-status-dot ${file.exists_on_disk ? 'exists' : 'missing'}`}
                    title={file.exists_on_disk ? t('prompts.exists') : t('prompts.missing')}
                  />
                  <span>{formatTimestamp(file.last_scanned_at)}</span>
                </div>
              </div>
            </div>
          ))}
        </div>
      ))}

      {selectedFile && (
        <div className="prompts-editor">
          <div className="prompts-editor-path">{selectedFile.file_path}</div>

          {message && (
            <div className={`prompts-message ${message.type}`}>
              {message.type === 'success' ? <CheckCircle size={14} /> : <AlertCircle size={14} />}
              <span>{message.text}</span>
            </div>
          )}

          <textarea
            value={editContent}
            onChange={(e) => setEditContent(e.target.value)}
            spellCheck={false}
          />

          <div className="prompts-editor-actions">
            <button
              className="btn btn-primary"
              type="button"
              disabled={saving}
              onClick={() => void handleSave()}
            >
              <Save size={14} />
              {saving ? t('prompts.saving') : t('prompts.save')}
            </button>
            <button
              className="btn btn-danger"
              type="button"
              onClick={() => void handleDelete()}
            >
              <Trash2 size={14} />
              {t('prompts.delete')}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

export default memo(PromptsPage)
