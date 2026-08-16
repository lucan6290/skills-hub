import { memo } from 'react'
import type { TFunction } from 'i18next'

type NewToolsModalProps = {
  open: boolean
  loading: boolean
  toolsLabelText: string
  onDismiss: () => void
  t: TFunction
}

const NewToolsModal = ({
  open,
  loading,
  toolsLabelText,
  onDismiss,
  t,
}: NewToolsModalProps) => {
  if (!open) return null

  return (
    <div className="modal-backdrop" onClick={onDismiss}>
      <div className="modal" onClick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
        <div className="modal-header">
          <div className="modal-title">{t('newToolsTitle')}</div>
        </div>
        <div className="modal-body">
          {t('newToolsBody', {
            tools: toolsLabelText,
          })}
        </div>
        <div className="modal-footer">
          <button className="btn btn-primary" onClick={onDismiss} disabled={loading}>
            {t('gotIt')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default memo(NewToolsModal)
