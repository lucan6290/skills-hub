import { memo } from 'react'
import type { TFunction } from 'i18next'

type SharedDirModalProps = {
  open: boolean
  loading: boolean
  toolLabel: string
  otherLabels: string
  onRequestClose: () => void
  onConfirm: () => void
  t: TFunction
}

const SharedDirModal = ({
  open,
  loading,
  toolLabel,
  otherLabels,
  onRequestClose,
  onConfirm,
  t,
}: SharedDirModalProps) => {
  if (!open) return null

  return (
    <div className="modal-backdrop" onClick={onRequestClose}>
      <div
        className="modal"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
      >
        <div className="modal-header">
          <div className="modal-title">{t('appName')}</div>
        </div>
        <div className="modal-body">
          {t('sharedDirConfirm', {
            tool: toolLabel,
            others: otherLabels,
          })}
        </div>
        <div className="modal-footer">
          <button
            className="btn btn-secondary"
            onClick={onRequestClose}
            disabled={loading}
          >
            {t('cancel')}
          </button>
          <button className="btn btn-primary" onClick={onConfirm} disabled={loading}>
            {t('confirm')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default memo(SharedDirModal)
