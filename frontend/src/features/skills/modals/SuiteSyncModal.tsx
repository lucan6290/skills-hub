import { memo, useCallback, useMemo, useState } from 'react'
import type { TFunction } from 'i18next'
import type { SuiteSubSkill } from '../types'

type SuiteSyncModalProps = {
  open: boolean
  loading: boolean
  toolLabel: string
  subSkills: SuiteSubSkill[]
  loadingSubSkills: boolean
  onRequestClose: () => void
  onConfirm: (selectedSubpaths: string[]) => void
  t: TFunction
}

const SuiteSyncModal = ({
  open,
  loading,
  toolLabel,
  subSkills,
  loadingSubSkills,
  onRequestClose,
  onConfirm,
  t,
}: SuiteSyncModalProps) => {
  // 用 key 来重置选中状态：当 subSkills 引用变化时自动全选
  const subSkillsKey = useMemo(
    () => subSkills.map((s) => s.subpath).join(','),
    [subSkills],
  )
  const [selected, setSelected] = useState<Set<string>>(
    () => new Set(subSkills.map((s) => s.subpath)),
  )
  const [prevKey, setPrevKey] = useState(subSkillsKey)

  // 当 subSkills 列表变化时重置为全选（避免 useEffect + setState）
  if (subSkillsKey !== prevKey) {
    setPrevKey(subSkillsKey)
    setSelected(new Set(subSkills.map((s) => s.subpath)))
  }

  const toggleOne = useCallback((subpath: string) => {
    setSelected((prev) => {
      const next = new Set(prev)
      if (next.has(subpath)) {
        next.delete(subpath)
      } else {
        next.add(subpath)
      }
      return next
    })
  }, [])

  const selectAll = useCallback(() => {
    setSelected(new Set(subSkills.map((s) => s.subpath)))
  }, [subSkills])

  const deselectAll = useCallback(() => {
    setSelected(new Set())
  }, [])

  const handleConfirm = useCallback(() => {
    onConfirm(Array.from(selected))
  }, [onConfirm, selected])

  if (!open) return null

  const allSelected = subSkills.length > 0 && selected.size === subSkills.length
  const noneSelected = selected.size === 0

  return (
    <div className="modal-backdrop" onClick={onRequestClose}>
      <div
        className="modal modal-suite-sync"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
      >
        <div className="modal-header">
          <div className="modal-title">{t('suiteSync.title')}</div>
        </div>
        <div className="modal-body">
          <p className="suite-sync-desc">
            {t('suiteSync.description', { tool: toolLabel })}
          </p>

          {loadingSubSkills ? (
            <div className="suite-sync-loading">{t('suiteSync.loading')}</div>
          ) : subSkills.length === 0 ? (
            <div className="suite-sync-empty">{t('suiteSync.noSubSkills')}</div>
          ) : (
            <>
              <div className="suite-sync-actions">
                <button
                  className="btn btn-secondary btn-sm"
                  onClick={allSelected ? deselectAll : selectAll}
                  disabled={loading}
                >
                  {allSelected ? t('suiteSync.deselectAll') : t('suiteSync.selectAll')}
                </button>
              </div>
              <ul className="suite-sync-list">
                {subSkills.map((sub) => (
                  <li key={sub.subpath} className="suite-sync-item">
                    <label className="suite-sync-label">
                      <input
                        type="checkbox"
                        checked={selected.has(sub.subpath)}
                        onChange={() => toggleOne(sub.subpath)}
                        disabled={loading}
                      />
                      <span className="suite-sync-name">{sub.name}</span>
                    </label>
                    {sub.description && (
                      <p className="suite-sync-item-desc">{sub.description}</p>
                    )}
                  </li>
                ))}
              </ul>
            </>
          )}
        </div>
        <div className="modal-footer">
          <button
            className="btn btn-secondary"
            onClick={onRequestClose}
            disabled={loading}
          >
            {t('suiteSync.cancel')}
          </button>
          <button
            className="btn btn-primary"
            onClick={handleConfirm}
            disabled={loading || loadingSubSkills || noneSelected}
          >
            {t('suiteSync.confirm')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default memo(SuiteSyncModal)
