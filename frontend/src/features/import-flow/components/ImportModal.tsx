import { memo, useMemo, useState } from 'react'
import { Download, Search } from 'lucide-react'
import type { TFunction } from 'i18next'
import type { OnboardingGroup, OnboardingPlan, OnboardingVariant } from '@/features/skills/types'

type ImportModalProps = {
  open: boolean
  loading: boolean
  plan: OnboardingPlan
  selected: Record<string, boolean>
  variantChoice: Record<string, string>
  storagePath: string
  onRequestClose: () => void
  onToggleGroup: (groupName: string, checked: boolean) => void
  onSelectVariant: (groupName: string, path: string) => void
  onImport: () => void
  t: TFunction
}

const ImportModal = ({
  open,
  loading,
  plan,
  selected,
  variantChoice,
  storagePath,
  onRequestClose,
  onToggleGroup,
  onSelectVariant,
  onImport,
  t,
}: ImportModalProps) => {
  const [query, setQuery] = useState('')
  const normalizedQuery = query.trim().toLowerCase()
  const filteredGroups = useMemo(() => {
    if (!normalizedQuery) return plan.groups
    return plan.groups.filter((group) => {
      const fields = [
        group.name,
        ...group.variants.flatMap((variant) => [variant.path, variant.tool]),
      ]
      return fields.some((value) => value.toLowerCase().includes(normalizedQuery))
    })
  }, [normalizedQuery, plan.groups])
  const byTool = useMemo(() => {
    const map = new Map<string, Array<{ group: OnboardingGroup; variant: OnboardingVariant }>>()
    for (const group of filteredGroups) {
      for (const variant of group.variants) {
        const list = map.get(variant.tool)
        if (list) list.push({ group, variant })
        else map.set(variant.tool, [{ group, variant }])
      }
    }
    for (const list of map.values()) {
      list.sort((a, b) => a.variant.name.localeCompare(b.variant.name))
    }
    return [...map.entries()].sort(
      (a, b) => b[1].length - a[1].length || a[0].localeCompare(b[0]),
    )
  }, [filteredGroups])
  const selectedCount = filteredGroups.filter((group) => selected[group.name]).length
  const totalSelectedCount = plan.groups.filter((group) => selected[group.name]).length
  const allVisibleSelected =
    filteredGroups.length > 0 &&
    filteredGroups.every((group) => selected[group.name])

  const toggleVisibleGroups = (checked: boolean) => {
    filteredGroups.forEach((group) => onToggleGroup(group.name, checked))
  }

  if (!open) return null

  return (
    <div className="modal-backdrop" onClick={onRequestClose}>
      <div
        className="modal modal-lg modal-discovered"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <div className="modal-title">{t('importTitle')}</div>
          <button
            className="modal-close"
            type="button"
            onClick={onRequestClose}
            aria-label={t('close')}
          >
            ✕
          </button>
        </div>
        <div className="modal-body">
          <div className="import-summary">
            <div>{t('importSummary')}</div>
            <div className="import-metrics">
              <span>{t('toolsScanned', { count: plan.total_tools_scanned })}</span>
              <span>{t('skillsFound', { count: plan.total_skills_found })}</span>
            </div>
            <div className="import-target-hint">
              {t('importTargetHint', { path: storagePath })}
            </div>
          </div>
          <details className="tool-overview">
            <summary>{t('toolOverviewTitle')}</summary>
            <div className="groups">
              {byTool.map(([tool, items]) => (
                <div className="group-card" key={tool}>
                  <div className="group-title">
                    <span>{tool}</span>
                    <span className="badge">{t('skillCount', { count: items.length })}</span>
                  </div>
                  <div className="group-variants">
                    {items.map(({ variant }) => (
                      <div
                        className="variant-row"
                        key={`${tool}-${variant.name}-${variant.path}`}
                      >
                        <div className="variant-info">
                          <span className="path">{variant.name}</span>
                        </div>
                        {variant.is_link ? (
                          <span className="meta">
                            {t('linkLabel', {
                              target: variant.link_target ?? t('unknown'),
                            })}
                          </span>
                        ) : (
                          <span className="meta">{t('directory')}</span>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </details>
          <div className="search-container import-search">
            <Search size={16} className="search-icon-abs" />
            <input
              className="search-input"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder={t('searchPlaceholder')}
            />
          </div>
          <div className="sync-row">
            <label className="inline-checkbox">
              <input
                type="checkbox"
                checked={allVisibleSelected}
                onChange={(event) => toggleVisibleGroups(event.target.checked)}
                disabled={filteredGroups.length === 0}
              />
              {t('selectAll')}
            </label>
            <span className="pick-toolbar-count">
              {t('selectedCount', {
                selected: selectedCount,
                total: filteredGroups.length,
              })}
            </span>
          </div>
          <div className="groups discovered-list">
            {filteredGroups.length === 0 ? (
              <div className="empty">{t('importSearchEmpty')}</div>
            ) : null}
            {filteredGroups.map((group) => (
              <div className="group-card" key={group.name}>
                <div className="group-title">
                  <label className="group-select">
                    <input
                      type="checkbox"
                      checked={Boolean(selected[group.name])}
                      onChange={(event) =>
                        onToggleGroup(group.name, event.target.checked)
                      }
                    />
                    <span>{group.name}</span>
                  </label>
                  {group.has_conflict ? (
                    <span className="badge danger">{t('conflict')}</span>
                  ) : (
                    <span className="badge">{t('consistent')}</span>
                  )}
                </div>
                <div className="group-variants">
                  {group.variants.map((variant) => (
                    <div
                      className="variant-row"
                      key={`${group.name}-${variant.tool}-${variant.path}`}
                    >
                      {group.has_conflict ? (
                        <input
                          type="radio"
                          name={`variant-${group.name}`}
                          checked={variantChoice[group.name] === variant.path}
                          onChange={() => onSelectVariant(group.name, variant.path)}
                        />
                      ) : (
                        <span className="variant-spacer" />
                      )}
                      <div className="variant-info">
                        <span className="path">{variant.path}</span>
                        <span className="found-pill">
                          {t('foundIn')} {variant.tool}
                        </span>
                      </div>
                      {variant.is_link ? (
                        <span className="meta">
                          {t('linkLabel', {
                            target: variant.link_target ?? t('unknown'),
                          })}
                        </span>
                      ) : (
                        <span className="meta">{t('directory')}</span>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
        <div className="modal-footer">
          <button
            className="btn btn-primary"
            onClick={onImport}
            disabled={loading || totalSelectedCount === 0}
          >
            <Download size={14} />
            {t('importAndSync')}
          </button>
          <button
            className="btn btn-secondary"
            onClick={onRequestClose}
            disabled={loading}
          >
            {t('close')}
          </button>
        </div>
      </div>
    </div>
  )
}

export default memo(ImportModal)
