import { memo, useCallback, useEffect, useMemo, useState } from 'react'
import {
  Database,
  Download,
  RefreshCw,
  Search,
  Shield,
  Trash2,
  ChevronLeft,
  ChevronRight,
  AlertTriangle,
  CheckCircle,
  HardDrive,
  Table2,
  Wrench,
  ArrowUp,
  ArrowDown,
  Copy,
  FolderOpen,
  Zap,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import { toast } from 'sonner'
import {
  fetchDbOverview,
  fetchDbTableData,
  runDbMaintenance,
  resetDb,
  exportDb,
  openDbFolder,
  type DbOverview,
  type DbTableData,
} from '@/lib/api'

type TabKey = 'overview' | 'tables' | 'maintenance'
type FragStatus = 'normal' | 'warn' | 'danger'
type TableSortKey = 'name' | 'rows' | 'size'

type DatabasePanelProps = {
  t: TFunction
}

type MaintenanceAction = {
  key: string
  label_key: string
  desc_key: string
  danger: boolean
}

const MAINTENANCE_ACTIONS: MaintenanceAction[] = [
  { key: 'integrity_check', label_key: 'db.integrityCheck', desc_key: 'db.integrityCheckDesc', danger: false },
  { key: 'vacuum', label_key: 'db.vacuum', desc_key: 'db.vacuumDesc', danger: false },
  { key: 'analyze', label_key: 'db.analyze', desc_key: 'db.analyzeDesc', danger: false },
  { key: 'clear_cache', label_key: 'db.clearCache', desc_key: 'db.clearCacheDesc', danger: true },
  { key: 'clear_discovered', label_key: 'db.clearDiscovered', desc_key: 'db.clearDiscoveredDesc', danger: true },
]

const FRAG_WARN_THRESHOLD = 20
const FRAG_DANGER_THRESHOLD = 40

function getFragStatus(pct: number): FragStatus {
  if (pct >= FRAG_DANGER_THRESHOLD) return 'danger'
  if (pct >= FRAG_WARN_THRESHOLD) return 'warn'
  return 'normal'
}

function formatTimestamp(ms: number): string {
  if (!ms) return '-'
  const d = new Date(ms)
  const pad = (n: number) => n.toString().padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

function formatValue(val: unknown): string {
  if (val === null || val === undefined) return 'NULL'
  if (typeof val === 'object') return JSON.stringify(val)
  return String(val)
}

const DatabasePanel = ({ t }: DatabasePanelProps) => {
  const [activeTab, setActiveTab] = useState<TabKey>('overview')
  const [overview, setOverview] = useState<DbOverview | null>(null)
  const [loading, setLoading] = useState(false)
  const [loadError, setLoadError] = useState<string | null>(null)
  const [selectedTable, setSelectedTable] = useState<string>('')
  const [tableData, setTableData] = useState<DbTableData | null>(null)
  const [tableLoading, setTableLoading] = useState(false)
  const [page, setPage] = useState(1)
  const [pageSize] = useState(50)
  const [sortCol, setSortCol] = useState<string | null>(null)
  const [sortDir, setSortDir] = useState<'asc' | 'desc'>('asc')
  const [filterText, setFilterText] = useState('')
  const [actionLoading, setActionLoading] = useState<string | null>(null)
  const [resetConfirm, setResetConfirm] = useState('')
  const [showResetConfirm, setShowResetConfirm] = useState(false)
  const [detailRow, setDetailRow] = useState<Record<string, unknown> | null>(null)
  const [showVacuumConfirm, setShowVacuumConfirm] = useState(false)
  const [tableSortKey, setTableSortKey] = useState<TableSortKey>('name')
  const [tableSortDir, setTableSortDir] = useState<'asc' | 'desc'>('asc')

  const loadOverview = useCallback(async () => {
    setLoading(true)
    setLoadError(null)
    try {
      const data = await fetchDbOverview()
      setOverview(data)
      if (!selectedTable && data.tables.length > 0) {
        setSelectedTable(data.tables[0].table_name)
      }
    } catch (err) {
      setLoadError(err instanceof Error ? err.message : t('db.dbLoadError'))
    } finally {
      setLoading(false)
    }
  }, [selectedTable, t])

  useEffect(() => {
    loadOverview()
  }, [loadOverview])

  const loadTableData = useCallback(async () => {
    if (!selectedTable) return
    setTableLoading(true)
    try {
      const data = await fetchDbTableData(selectedTable, {
        page,
        page_size: pageSize,
        sort_col: sortCol,
        sort_dir: sortDir,
        filter_text: filterText || null,
      })
      setTableData(data)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to load table data')
    } finally {
      setTableLoading(false)
    }
  }, [selectedTable, page, pageSize, sortCol, sortDir, filterText])

  useEffect(() => {
    if (activeTab === 'tables') {
      loadTableData()
    }
  }, [activeTab, loadTableData])

  const handleSort = useCallback((col: string) => {
    if (sortCol === col) {
      setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'))
    } else {
      setSortCol(col)
      setSortDir('asc')
    }
    setPage(1)
  }, [sortCol])

  const handleMaintenanceAction = useCallback(async (action: string) => {
    setActionLoading(action)
    try {
      const result = await runDbMaintenance(action)
      if (result.ok) {
        toast.success(result.message)
      } else {
        toast.error(result.message)
      }
      // Reload overview after any maintenance
      loadOverview()
      if (activeTab === 'tables' && selectedTable) {
        loadTableData()
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Operation failed')
    } finally {
      setActionLoading(null)
    }
  }, [loadOverview, activeTab, selectedTable, loadTableData])

  const handleReset = useCallback(async () => {
    if (resetConfirm.trim() !== 'RESET') {
      toast.error(t('db.resetConfirmHint'))
      return
    }
    setActionLoading('reset')
    try {
      const result = await resetDb(resetConfirm)
      toast.success(result.message)
      setShowResetConfirm(false)
      setResetConfirm('')
      setActiveTab('overview')
      loadOverview()
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Reset failed')
    } finally {
      setActionLoading(null)
    }
  }, [resetConfirm, t, loadOverview])

  const handleExport = useCallback(async () => {
    try {
      const result = await exportDb()
      if (result.ok) {
        toast.success(result.message || t('db.exportSuccess'))
      } else {
        toast.error(result.message || 'Export failed')
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Export failed')
    }
  }, [t])

  const handleCopyPath = useCallback(async () => {
    if (!overview?.db_path) return
    try {
      await navigator.clipboard.writeText(overview.db_path)
      toast.success(t('db.copied'))
    } catch {
      toast.error('Copy failed')
    }
  }, [overview, t])

  const handleOpenFolder = useCallback(async () => {
    try {
      const result = await openDbFolder()
      toast.success(result.message)
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to open folder')
    }
  }, [])

  const handleTableStatSort = useCallback((key: TableSortKey) => {
    if (tableSortKey === key) {
      setTableSortDir((d) => (d === 'asc' ? 'desc' : 'asc'))
    } else {
      setTableSortKey(key)
      setTableSortDir('asc')
    }
  }, [tableSortKey])

  const tableList = useMemo(() => overview?.tables ?? [], [overview])

  const sortedTables = useMemo(() => {
    const sorted = [...tableList]
    sorted.sort((a, b) => {
      let cmp = 0
      if (tableSortKey === 'name') cmp = a.display_name.localeCompare(b.display_name)
      else if (tableSortKey === 'rows') cmp = a.row_count - b.row_count
      else cmp = a.size_bytes - b.size_bytes
      return tableSortDir === 'asc' ? cmp : -cmp
    })
    return sorted
  }, [tableList, tableSortKey, tableSortDir])

  const fragStatus = overview ? getFragStatus(overview.fragmentation_pct) : 'normal'
  const fragTooltipKey = fragStatus === 'normal'
    ? 'db.tooltipFragmentationNormal'
    : 'db.tooltipFragmentationWarn'
  const fragTooltip = t(fragTooltipKey, { pct: overview?.fragmentation_pct ?? 0 })

  // ── Render: Overview Tab ──
  const renderOverview = () => {
    if (loadError) {
      return (
        <div className="db-error-state">
          <AlertTriangle size={32} className="db-error-icon" />
          <div className="db-error-title">{t('db.dbLoadError')}</div>
          <div className="db-error-msg">{loadError}</div>
          <div className="db-error-desc">{t('db.dbMissingDesc')}</div>
          <button
            className="btn-secondary db-error-retry"
            onClick={loadOverview}
            disabled={loading}
            type="button"
          >
            <RefreshCw size={14} className={loading ? 'db-spin' : ''} />
            {t('db.refresh')}
          </button>
        </div>
      )
    }

    if (!overview) {
      return (
        <div className="db-loading">
          {loading ? <RefreshCw size={20} className="db-spin" /> : null}
        </div>
      )
    }

    return (
      <div className="db-overview">
        <div className="db-stats-grid">
          <div className="db-stat-card" title={t('db.tooltipFileSize')}>
            <div className="db-stat-icon"><HardDrive size={18} /></div>
            <div className="db-stat-content">
              <div className="db-stat-label">{t('db.fileSize')}</div>
              <div className="db-stat-value">{overview.file_size_human}</div>
            </div>
          </div>
          <div className="db-stat-card" title={t('db.tooltipTablesCount')}>
            <div className="db-stat-icon"><Database size={18} /></div>
            <div className="db-stat-content">
              <div className="db-stat-label">{t('db.tablesCount')}</div>
              <div className="db-stat-value">{overview.tables.length}</div>
            </div>
          </div>
          <div className="db-stat-card" title={t('db.tooltipSqliteVersion')}>
            <div className="db-stat-icon"><Shield size={18} /></div>
            <div className="db-stat-content">
              <div className="db-stat-label">{t('db.sqliteVersion')}</div>
              <div className="db-stat-value">{overview.sqlite_version}</div>
            </div>
          </div>
          <div
            className={`db-stat-card frag-${fragStatus}`}
            title={fragTooltip}
          >
            <div className="db-stat-icon">
              {fragStatus === 'normal' ? <CheckCircle size={18} /> : <AlertTriangle size={18} />}
            </div>
            <div className="db-stat-content">
              <div className="db-stat-label">{t('db.fragmentation')}</div>
              <div className="db-stat-value">{overview.fragmentation_pct}%</div>
            </div>
            {fragStatus !== 'normal' && (
              <button
                className="db-frag-action"
                onClick={() => setShowVacuumConfirm(true)}
                disabled={actionLoading !== null}
                type="button"
              >
                <Zap size={12} />
                {t('db.defragmentNow')}
              </button>
            )}
          </div>
        </div>

        <div className="db-info-section">
          <h4 className="db-info-title">{t('db.fileInfo')}</h4>
          <div className="db-info-grid">
            <div className="db-info-item db-info-path">
              <span className="db-info-label">{t('db.filePath')}</span>
              <span className="db-info-value mono" title={overview.db_path}>{overview.db_path}</span>
              <div className="db-info-actions">
                <button
                  className="db-icon-btn"
                  onClick={handleCopyPath}
                  title={t('db.copyPath')}
                  type="button"
                >
                  <Copy size={13} />
                </button>
                <button
                  className="db-icon-btn"
                  onClick={handleOpenFolder}
                  title={t('db.openFolder')}
                  type="button"
                >
                  <FolderOpen size={13} />
                </button>
              </div>
            </div>
            <div className="db-info-item">
              <span className="db-info-label">{t('db.lastModified')}</span>
              <span className="db-info-value">{formatTimestamp(overview.last_modified)}</span>
            </div>
            <div className="db-info-item">
              <span className="db-info-label">{t('db.pageSize')}</span>
              <span className="db-info-value">{overview.page_size} B</span>
            </div>
            <div className="db-info-item">
              <span className="db-info-label">{t('db.freeSpace')}</span>
              <span className="db-info-value">{overview.free_size_human}</span>
            </div>
          </div>
        </div>

        <div className="db-info-section">
          <h4 className="db-info-title">{t('db.tableStats')}</h4>
          <div className="db-table-stats-wrap">
            <table className="db-table-stats-table">
              <thead>
                <tr>
                  <th
                    className={tableSortKey === 'name' ? 'sorted' : ''}
                    onClick={() => handleTableStatSort('name')}
                  >
                    <span className="db-th-content">
                      {t('db.tabTables')}
                      {tableSortKey === 'name' && (
                        tableSortDir === 'asc' ? <ArrowUp size={11} /> : <ArrowDown size={11} />
                      )}
                    </span>
                  </th>
                  <th
                    className={`db-num-col ${tableSortKey === 'rows' ? 'sorted' : ''}`}
                    onClick={() => handleTableStatSort('rows')}
                  >
                    <span className="db-th-content">
                      {t('db.rowCount')}
                      {tableSortKey === 'rows' && (
                        tableSortDir === 'asc' ? <ArrowUp size={11} /> : <ArrowDown size={11} />
                      )}
                    </span>
                  </th>
                  <th
                    className={`db-num-col ${tableSortKey === 'size' ? 'sorted' : ''}`}
                    onClick={() => handleTableStatSort('size')}
                  >
                    <span className="db-th-content">
                      {t('db.fileSize')}
                      {tableSortKey === 'size' && (
                        tableSortDir === 'asc' ? <ArrowUp size={11} /> : <ArrowDown size={11} />
                      )}
                    </span>
                  </th>
                </tr>
              </thead>
              <tbody>
                {sortedTables.map((tbl) => (
                  <tr
                    key={tbl.table_name}
                    className={`db-table-stat-row ${selectedTable === tbl.table_name ? 'active' : ''}`}
                    onClick={() => {
                      setSelectedTable(tbl.table_name)
                      setPage(1)
                      setActiveTab('tables')
                    }}
                  >
                    <td className="db-table-stat-name">
                      <Table2 size={13} className="db-table-stat-icon" />
                      {tbl.display_name}
                    </td>
                    <td className="db-num-col">{tbl.row_count.toLocaleString()}</td>
                    <td className="db-num-col">{tbl.size_human}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    )
  }

  // ── Render: Tables Tab ──
  const renderTables = () => (
    <div className="db-tables">
      <div className="db-tables-header">
        <select
          className="db-table-select"
          value={selectedTable}
          onChange={(e) => {
            setSelectedTable(e.target.value)
            setPage(1)
            setSortCol(null)
            setFilterText('')
          }}
        >
          {tableList.map((tbl) => (
            <option key={tbl.table_name} value={tbl.table_name}>
              {tbl.display_name} ({tbl.row_count})
            </option>
          ))}
        </select>
        <div className="db-search-wrap">
          <Search size={14} className="db-search-icon" />
          <input
            type="text"
            className="db-search-input"
            placeholder={t('db.filterPlaceholder')}
            value={filterText}
            onChange={(e) => {
              setFilterText(e.target.value)
              setPage(1)
            }}
          />
        </div>
      </div>

      {tableLoading && (
        <div className="db-loading-inline">
          <RefreshCw size={16} className="db-spin" />
        </div>
      )}

      {tableData && !tableLoading && (
        <>
          <div className="db-table-wrap">
            <table className="db-data-table">
              <thead>
                <tr>
                  {tableData.columns.map((col) => (
                    <th
                      key={col.name}
                      onClick={() => handleSort(col.name)}
                      className={sortCol === col.name ? 'sorted' : ''}
                      title={col.type}
                    >
                      <span className="db-th-content">
                        {col.name}
                        {col.pk && <span className="db-pk-badge">PK</span>}
                        {sortCol === col.name && (
                          sortDir === 'asc' ? <ArrowUp size={12} /> : <ArrowDown size={12} />
                        )}
                      </span>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {tableData.rows.length === 0 ? (
                  <tr>
                    <td colSpan={tableData.columns.length} className="db-empty">
                      {t('db.noData')}
                    </td>
                  </tr>
                ) : (
                  tableData.rows.map((row, idx) => (
                    <tr key={idx} onClick={() => setDetailRow(row)} className="db-data-row">
                      {tableData.columns.map((col) => (
                        <td key={col.name} title={formatValue(row[col.name])}>
                          {formatValue(row[col.name])}
                        </td>
                      ))}
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>

          <div className="db-pagination">
            <span className="db-pagination-info">
              {t('db.pagination', {
                from: (page - 1) * pageSize + 1,
                to: Math.min(page * pageSize, tableData.total),
                total: tableData.total,
              })}
            </span>
            <div className="db-pagination-btns">
              <button
                className="btn-secondary db-page-btn"
                disabled={page <= 1}
                onClick={() => setPage((p) => p - 1)}
                type="button"
              >
                <ChevronLeft size={14} />
              </button>
              <span className="db-page-num">{page} / {tableData.total_pages || 1}</span>
              <button
                className="btn-secondary db-page-btn"
                disabled={page >= tableData.total_pages}
                onClick={() => setPage((p) => p + 1)}
                type="button"
              >
                <ChevronRight size={14} />
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  )

  // ── Render: Maintenance Tab ──
  const renderMaintenance = () => (
    <div className="db-maintenance">
      <div className="db-maint-actions">
        {MAINTENANCE_ACTIONS.map((act) => (
          <div key={act.key} className={`db-maint-card ${act.danger ? 'danger' : ''}`}>
            <div className="db-maint-info">
              <div className="db-maint-title">{t(act.label_key)}</div>
              <div className="db-maint-desc">{t(act.desc_key)}</div>
            </div>
            <button
              className={`btn-secondary ${act.danger ? 'btn-danger' : ''}`}
              onClick={() => handleMaintenanceAction(act.key)}
              disabled={actionLoading !== null}
              type="button"
            >
              {actionLoading === act.key ? <RefreshCw size={14} className="db-spin" /> : t('db.execute')}
            </button>
          </div>
        ))}
      </div>

      <div className="db-maint-divider" />

      <div className="db-maint-card">
        <div className="db-maint-info">
          <div className="db-maint-title">
            <Download size={16} className="db-maint-title-icon" />
            {t('db.exportBackup')}
          </div>
          <div className="db-maint-desc">{t('db.exportBackupDesc')}</div>
        </div>
        <button
          className="btn-secondary"
          onClick={handleExport}
          type="button"
        >
          <Download size={14} className="db-maint-btn-icon" />
          {t('db.download')}
        </button>
      </div>

      <div className="db-maint-divider" />

      <div className="db-maint-card danger">
        <div className="db-maint-info">
          <div className="db-maint-title">
            <AlertTriangle size={16} className="db-maint-title-icon" />
            {t('db.resetDatabase')}
          </div>
          <div className="db-maint-desc">{t('db.resetDatabaseDesc')}</div>
        </div>
        {!showResetConfirm ? (
          <button
            className="btn-secondary btn-danger"
            onClick={() => setShowResetConfirm(true)}
            type="button"
          >
            <Trash2 size={14} className="db-maint-btn-icon" />
            {t('db.reset')}
          </button>
        ) : (
          <div className="db-reset-confirm">
            <input
              type="text"
              className="db-reset-input"
              placeholder="RESET"
              value={resetConfirm}
              onChange={(e) => setResetConfirm(e.target.value)}
            />
            <button
              className="btn-secondary btn-danger"
              onClick={handleReset}
              disabled={actionLoading !== null}
              type="button"
            >
              {actionLoading === 'reset' ? <RefreshCw size={14} className="db-spin" /> : t('db.confirm')}
            </button>
            <button
              className="btn-secondary"
              onClick={() => { setShowResetConfirm(false); setResetConfirm('') }}
              type="button"
            >
              {t('db.cancel')}
            </button>
          </div>
        )}
      </div>
    </div>
  )

  return (
    <div className="settings-v2-section settings-v2-db-section">
      <div className="settings-v2-db-header">
        <h3 className="settings-v2-section-title db-panel-title">{t('db.title')}</h3>
        <div className="db-header-actions">
          <button
            className="settings-v2-seg-btn settings-v2-db-refresh"
            onClick={loadOverview}
            disabled={loading}
            type="button"
            title={t('db.refresh')}
          >
            <RefreshCw size={13} className={loading ? 'db-spin' : ''} />
            {t('db.refresh')}
          </button>
          <button
            className="settings-v2-seg-btn settings-v2-db-refresh"
            onClick={handleExport}
            type="button"
            title={t('db.backupDb')}
          >
            <Download size={13} />
            {t('db.backupDb')}
          </button>
        </div>
      </div>

      <div className="db-tabs settings-v2-db-tabs">
        {([
          ['overview', 'db.tabOverview', HardDrive],
          ['tables', 'db.tabTables', Table2],
          ['maintenance', 'db.tabMaintenance', Wrench],
        ] as const).map(([key, labelKey, Icon]) => (
          <button
            key={key}
            className={`db-tab ${activeTab === key ? 'active' : ''}`}
            onClick={() => setActiveTab(key as TabKey)}
            type="button"
          >
            <Icon size={14} />
            {t(labelKey)}
          </button>
        ))}
      </div>

      <div className="db-tab-content">
        {activeTab === 'overview' && renderOverview()}
        {activeTab === 'tables' && renderTables()}
        {activeTab === 'maintenance' && renderMaintenance()}
      </div>

      {/* Row detail modal */}
      {detailRow && (
        <div className="modal-backdrop" onClick={() => setDetailRow(null)}>
          <div className="db-detail-modal" onClick={(e) => e.stopPropagation()}>
            <div className="db-detail-header">
              <h4>{t('db.rowDetail')}</h4>
              <button className="modal-close-btn" onClick={() => setDetailRow(null)} type="button">×</button>
            </div>
            <div className="db-detail-body">
              {tableData?.columns.map((col) => (
                <div key={col.name} className="db-detail-row">
                  <span className="db-detail-key">{col.name}</span>
                  <span className="db-detail-value mono">
                    {typeof detailRow[col.name] === 'object'
                      ? JSON.stringify(detailRow[col.name], null, 2)
                      : formatValue(detailRow[col.name])}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {/* VACUUM confirm dialog */}
      {showVacuumConfirm && (
        <div className="modal-backdrop" onClick={() => setShowVacuumConfirm(false)}>
          <div className="db-confirm-modal" onClick={(e) => e.stopPropagation()}>
            <div className="db-detail-header">
              <h4>{t('db.vacuumConfirmTitle')}</h4>
              <button className="modal-close-btn" onClick={() => setShowVacuumConfirm(false)} type="button">×</button>
            </div>
            <div className="db-confirm-body">
              <AlertTriangle size={24} className="db-confirm-icon" />
              <p className="db-confirm-text">{t('db.vacuumConfirmBody')}</p>
            </div>
            <div className="db-confirm-actions">
              <button
                className="btn-secondary"
                onClick={() => setShowVacuumConfirm(false)}
                type="button"
              >
                {t('db.cancel')}
              </button>
              <button
                className="btn-secondary btn-danger"
                onClick={() => {
                  setShowVacuumConfirm(false)
                  handleMaintenanceAction('vacuum')
                }}
                disabled={actionLoading !== null}
                type="button"
              >
                {actionLoading === 'vacuum' ? <RefreshCw size={14} className="db-spin" /> : t('db.defragmentNow')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default memo(DatabasePanel)
