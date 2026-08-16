import { memo, useMemo, useState } from 'react'
import { Folder, X } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { ManagedSkill } from '../types'

type ScopeSyncModalProps = {
  open: boolean
  loading: boolean
  skill: ManagedSkill | null
  scope: 'global' | 'project'
  projects: string[]
  recentProjects: string[]
  onRequestClose: () => void
  onScopeChange: (scope: 'global' | 'project', projects: string[]) => void
  onPickProject: () => Promise<string | undefined>
  t: TFunction
}

const ScopeSyncModal = ({
  open,
  loading,
  skill,
  scope,
  projects,
  recentProjects,
  onRequestClose,
  onScopeChange,
  onPickProject,
  t,
}: ScopeSyncModalProps) => {
  const [draftScope, setDraftScope] = useState<'global' | 'project'>(scope)
  const [draftProjects, setDraftProjects] = useState<string[]>(projects)

  const normalizedProjects = useMemo(
    () => Array.from(new Set(projects.filter(Boolean))),
    [projects],
  )
  const normalizedDraftProjects = useMemo(
    () => Array.from(new Set(draftProjects.filter(Boolean))),
    [draftProjects],
  )
  const projectListChanged =
    normalizedProjects.length !== normalizedDraftProjects.length ||
    normalizedProjects.some((item) => !normalizedDraftProjects.includes(item))
  const availableRecent = recentProjects.filter(
    (item) => !normalizedDraftProjects.includes(item),
  )
  const hasScopeChange = draftScope !== scope
  const requiresProject = draftScope === 'project' && normalizedDraftProjects.length === 0
  const addDraftProject = (projectPath: string) => {
    setDraftProjects((prev) => Array.from(new Set([...prev, projectPath].filter(Boolean))))
  }

  if (!open || !skill) return null

  return (
    <div className="modal-backdrop" onClick={loading ? undefined : onRequestClose}>
      <div
        className="modal scope-modal"
        role="dialog"
        aria-modal="true"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="modal-header">
          <div className="modal-title">
            {t('projectSync.title')} · {skill.name}
          </div>
          <button
            className="modal-close"
            type="button"
            onClick={onRequestClose}
            disabled={loading}
            aria-label={t('close')}
          >
            ✕
          </button>
        </div>
        <div className="modal-body scope-modal-body">
          <div className="scope-help">{t('projectSync.help')}</div>
          <label className={`scope-choice${draftScope === 'global' ? ' active' : ''}`}>
            <input
              type="radio"
              checked={draftScope === 'global'}
              onChange={() => setDraftScope('global')}
              disabled={loading}
            />
            <span>
              <strong>{t('scope.global')}</strong>
              <small>{t('projectSync.globalDesc')}</small>
            </span>
          </label>
          <label className={`scope-choice${draftScope === 'project' ? ' active' : ''}`}>
            <input
              type="radio"
              checked={draftScope === 'project'}
              onChange={() => setDraftScope('project')}
              disabled={loading}
            />
            <span>
              <strong>{t('scope.project')}</strong>
              <small>{t('projectSync.projectDesc')}</small>
            </span>
          </label>

          {draftScope === 'project' ? (
            <div className="project-sync-panel">
              <div className="project-sync-heading">{t('projectSync.projectDirs')}</div>
              {normalizedDraftProjects.length > 0 ? (
                <div className="project-path-list">
                  {normalizedDraftProjects.map((project) => (
                    <div className="project-path-row" key={project}>
                      <Folder size={14} />
                      <span className="mono">{project}</span>
                      <button
                        type="button"
                        className="icon-btn"
                        onClick={() =>
                          setDraftProjects((prev) => prev.filter((item) => item !== project))
                        }
                        disabled={loading}
                        aria-label={t('remove')}
                      >
                        <X size={14} />
                      </button>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="project-empty">{t('projectSync.noProjects')}</div>
              )}
              {requiresProject ? (
                <div className="scope-inline-warning">
                  {t('projectSync.projectRequired')}
                </div>
              ) : null}
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => {
                  void onPickProject().then((projectPath) => {
                    if (projectPath) addDraftProject(projectPath)
                  })
                }}
                disabled={loading}
              >
                {t('projectSync.addProject')}
              </button>

              {availableRecent.length > 0 ? (
                <>
                  <div className="project-sync-heading">{t('projectSync.recentProjects')}</div>
                  <div className="recent-project-list">
                    {availableRecent.map((project) => (
                      <button
                        key={project}
                        type="button"
                        className="recent-project-row"
                        onClick={() => addDraftProject(project)}
                        disabled={loading}
                      >
                        <span className="mono">{project}</span>
                        <span>{t('projectSync.addRecent')}</span>
                      </button>
                    ))}
                  </div>
                </>
              ) : null}
            </div>
          ) : null}
        </div>
        <div className="modal-footer">
          <button
            className="btn btn-secondary"
            type="button"
            onClick={onRequestClose}
            disabled={loading}
          >
            {t('cancel')}
          </button>
          <button
            className="btn btn-primary"
            type="button"
            onClick={() => onScopeChange(draftScope, normalizedDraftProjects)}
            disabled={loading || (!hasScopeChange && !projectListChanged) || requiresProject}
          >
            {t('projectSync.applyScope')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default memo(ScopeSyncModal)
