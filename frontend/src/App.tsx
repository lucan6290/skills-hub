import { lazy, Suspense, useCallback, useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Toaster } from 'sonner'
import { getCurrent, onOpenUrl } from '@tauri-apps/plugin-deep-link'
import { Header, LoadingOverlay } from '@/components/layout'
import {
  FilterBar,
  SkillsList,
  useSkills,
  useSkillFilter,
  useScopeState,
  useScopeManager,
  useAddSkill,
  useTagActions,
  useSkillActions,
} from '@/features/skills'
import { useTheme } from '@/features/settings'
import { useImportFlow } from '@/features/import-flow'
import { AppStateProvider, useAppState } from '@/context/AppStateContext'
import { ModalProvider, useModal } from '@/context/ModalContext'
import type { ManagedSkill } from '@/features/skills'

// ─── Lazy-loaded views ──────────────────────────────
const SkillDetailView = lazy(() => import('@/features/skills/components/SkillDetailView'))
const TagsPage = lazy(() => import('@/features/tags/components/TagsPage'))
const SettingsPage = lazy(() => import('@/features/settings/components/SettingsPage'))
const ToolsPage = lazy(() => import('@/features/tools/components/ToolsPage'))
const PromptsPage = lazy(() => import('@/features/prompts/components/PromptsPage'))

// ─── Lazy-loaded modals ─────────────────────────────
const SkillInfoModal = lazy(() => import('@/features/skills/modals/SkillInfoModal'))
const AddSkillModal = lazy(() => import('@/features/skills/modals/AddSkillModal'))
const EditSkillTagsModal = lazy(() => import('@/features/skills/modals/EditSkillTagsModal'))
const ImportModal = lazy(() => import('@/features/import-flow/components/ImportModal'))
const LocalPickModal = lazy(() => import('@/features/import-flow/components/LocalPickModal'))
const SharedDirModal = lazy(() => import('@/features/skills/modals/SharedDirModal'))
const SuiteSyncModal = lazy(() => import('@/features/skills/modals/SuiteSyncModal'))
const ScopeSyncModal = lazy(() => import('@/features/skills/modals/ScopeSyncModal'))
const NewToolsModal = lazy(() => import('@/features/tools/modals/NewToolsModal'))
const DeleteModal = lazy(() => import('@/features/skills/modals/DeleteModal'))

function App() {
  return (
    <AppStateProvider>
      <ModalProvider>
        <AppContent />
      </ModalProvider>
    </AppStateProvider>
  )
}

function AppContent() {
  const { t } = useTranslation()
  const appState = useAppState()
  const modal = useModal()

  // ─── Layer 1：基础数据 ─────────────────────────
  const scopeState = useScopeState()
  const skills = useSkills(t, appState.setError, appState.setSuccessToastMessage)
  const theme = useTheme(t, skills.loadManagedSkills, appState.setError)

  // ─── 共享 loading 状态（供 useAddSkill 等使用）─
  const [loading, setLoading] = useState(false)
  const [loadingStartAt, setLoadingStartAt] = useState<number | null>(null)

  // ─── Layer 1.5：派生 helper ──────────────────
  const getSkillScope = useCallback(
    (skill: ManagedSkill): 'global' | 'project' => {
      const hasGlobalTarget = skill.targets.some((t) => (t.scope ?? 'global') === 'global')
      const hasProjectTarget = skill.targets.some((t) => (t.scope ?? 'global') === 'project')
      if (hasGlobalTarget && !hasProjectTarget) return 'global'
      if (hasProjectTarget && !hasGlobalTarget) return 'project'
      const stored = scopeState.skillScopeState[skill.id]?.scope
      if (stored === 'global' || stored === 'project') return stored
      return hasProjectTarget ? 'project' : 'global'
    },
    [scopeState.skillScopeState],
  )

  // ─── Layer 2：功能 hooks ──────────────────────
  const sourceSkills = useMemo(
    () => skills.managedSkills.filter((skill) => {
      const normalizedSource = skill.source_type === 'custom' ? 'custom' : 'community'
      return normalizedSource === modal.activeSkillSource
    }),
    [modal.activeSkillSource, skills.managedSkills],
  )

  const customSkillCount = useMemo(
    () => skills.managedSkills.filter((skill) => skill.source_type === 'custom').length,
    [skills.managedSkills],
  )

  const communitySkillCount = skills.managedSkills.length - customSkillCount

  const filter = useSkillFilter(
    sourceSkills,
    getSkillScope,
    skills.toolSkillNamesByTool,
  )

  const loadTags = skills.loadTags
  const loadManagedSkills = skills.loadManagedSkills

  useEffect(() => {
    void loadTags(modal.activeSkillSource, filter.sortBy)
  }, [modal.activeSkillSource, loadTags, filter.sortBy])

  useEffect(() => {
    void loadManagedSkills(false, undefined, filter.sortBy)
  }, [filter.sortBy, loadManagedSkills])

  const importFlow = useImportFlow({
    t,
    tools: skills.tools,
    installedToolIds: skills.installedToolIds,
    isInstalled: skills.isInstalled,
    uniqueToolIdsBySkillsDir: skills.uniqueToolIdsBySkillsDir,
    sharedToolIdsByToolId: skills.sharedToolIdsByToolId,
    toolLabelById: skills.toolLabelById,
    loadManagedSkills: skills.loadManagedSkills,
    isSkillNameTaken: skills.isSkillNameTaken,
    showActionErrors: appState.showActionErrors,
    setError: appState.setError,
    setActionMessage: appState.setActionMessage,
    setSuccessToastMessage: appState.setSuccessToastMessage,
  })

  const scopeManager = useScopeManager({
    t,
    tools: skills.tools,
    installedToolIds: skills.installedToolIds,
    installedProjectToolIds: skills.installedProjectToolIds,
    toolSupportsProjectScope: skills.toolSupportsProjectScope,
    sharedToolIdsByToolId: skills.sharedToolIdsByToolId,
    toolLabelById: skills.toolLabelById,
    skillScopeState: scopeState.skillScopeState,
    setSkillScopeState: scopeState.setSkillScopeState,
    managedSkills: skills.managedSkills,
    loadManagedSkills: skills.loadManagedSkills,
    setError: appState.setError,
    setActionMessage: appState.setActionMessage,
    setSuccessToastMessage: appState.setSuccessToastMessage,
  })

  const addSkill = useAddSkill({
    t,
    tools: skills.tools,
    isInstalled: skills.isInstalled,
    uniqueToolIdsBySkillsDir: skills.uniqueToolIdsBySkillsDir,
    syncTargets: importFlow.syncTargets,
    setSyncTargets: importFlow.setSyncTargets,
    loadManagedSkills: skills.loadManagedSkills,
    loadTags: skills.loadTags,
    isSkillNameTaken: skills.isSkillNameTaken,
    showActionErrors: appState.showActionErrors,
    setError: appState.setError,
    loading,
    setLoading,
    setLoadingStartAt,
    setActionMessage: appState.setActionMessage,
    setSuccessToastMessage: appState.setSuccessToastMessage,
  })

  // ─── 合并 loading 状态 ──────────────────────────
  const globalLoading = loading || importFlow.loading || scopeManager.loading
  const globalLoadingStartAt =
    loadingStartAt || importFlow.loadingStartAt || scopeManager.loadingStartAt

  // ─── toolFilter 有效性 ──────────────────────────
  const { toolFilter, handleToolFilterChange } = filter

  useEffect(() => {
    if (toolFilter === 'all') return
    if (!skills.installedToolIds.includes(toolFilter)) {
      handleToolFilterChange('all')
    }
  }, [skills.installedToolIds, toolFilter, handleToolFilterChange])

  // ─── showNewToolsModal ──────────────────────────
  useEffect(() => {
    const loadStatus = async () => {
      const status = await skills.loadToolStatus()
      if (status && status.newly_installed.length > 0) {
        modal.setShowNewToolsModal(true)
      }
    }
    void loadStatus()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // ─── Deep Link handler (skillshub://) ──────────
  useEffect(() => {
    const handleDeepLink = (url: string) => {
      // skillshub://skill/{id} → 打开 skill 详情
      // skillshub://import → 打开导入界面
      const match = url.match(/^skillshub:\/\/([^/]+)(?:\/(.+))?/)
      if (!match) return
      const [, action, param] = match
      if (action === 'skill' && param) {
        const skill = skills.managedSkills.find((s) => s.id === param)
        if (skill) modal.openInfoModal(skill)
      } else if (action === 'import') {
        modal.handleViewChange('myskills')
      }
    }

    // 处理启动时传入的 deep link
    getCurrent()
      .then((urls) => {
        if (urls) {
          for (const url of urls) handleDeepLink(url)
        }
      })
      .catch(() => {})

    // 监听后续 deep link 事件
    const unlisten = onOpenUrl((urls) => {
      for (const url of urls) handleDeepLink(url)
    })

    return () => {
      void unlisten.then((fn) => fn())
    }
  }, [skills.managedSkills, modal])

  // ─── 计算衍生值 ────────────────────────────────
  const currentInfoModalSkill = useMemo(() => {
    if (!modal.infoModalSkill) return null
    return (
      skills.managedSkills.find((s) => s.id === modal.infoModalSkill!.id) ??
      modal.infoModalSkill
    )
  }, [modal.infoModalSkill, skills.managedSkills])

  const pendingDeleteSkill = useMemo(
    () =>
      modal.pendingDeleteId
        ? skills.managedSkills.find((s) => s.id === modal.pendingDeleteId) ?? null
        : null,
    [skills.managedSkills, modal.pendingDeleteId],
  )

  // ─── Tag & Skill actions (extracted hooks) ───────
  const {
    handleCreateTag,
    handleRenameTag,
    handleDeleteTag,
    handleCloseDeleteTag,
    handleConfirmDeleteTag,
  } = useTagActions({
    t,
    loadManagedSkills: skills.loadManagedSkills,
    loadTags: skills.loadTags,
    activeSkillSource: modal.activeSkillSource,
    setError: appState.setError,
    setSuccessToastMessage: appState.setSuccessToastMessage,
    setActionMessage: appState.setActionMessage,
    selectedTagIds: filter.selectedTagIds,
    setSelectedTagIds: filter.setSelectedTagIds,
    pendingDeleteTag: modal.pendingDeleteTag,
    setPendingDeleteTag: modal.setPendingDeleteTag,
    globalLoading,
    setLoading,
    setLoadingStartAt,
  })

  const {
    handleDeleteManaged,
    handleSaveSkillTags,
    handleCloseDelete,
  } = useSkillActions({
    t,
    loadManagedSkills: skills.loadManagedSkills,
    loadTags: skills.loadTags,
    activeSkillSource: modal.activeSkillSource,
    setError: appState.setError,
    setSuccessToastMessage: appState.setSuccessToastMessage,
    setActionMessage: appState.setActionMessage,
    setSkillScopeState: scopeState.setSkillScopeState,
    pendingDeleteId: modal.pendingDeleteId,
    setPendingDeleteId: modal.setPendingDeleteId,
    closeEditTags: modal.closeEditTags,
    globalLoading,
    setLoading,
    setLoadingStartAt,
  })

  // ─── Review import 触发 showImportModal ────────
  const handleReviewImport = useCallback(async () => {
    if (importFlow.plan) {
      modal.setShowImportModal(true)
      return
    }
    const result = await importFlow.loadPlan(true)
    if (result) {
      modal.setShowImportModal(true)
    }
  }, [importFlow, modal])

  // ─── Tags page nav ───────────────────────────────
  const handleOpenTagsPage = useCallback(() => {
    modal.handleViewChange('tags')
  }, [modal])

  const handleReviewUntagged = useCallback(() => {
    filter.setSelectedTagIds([])
    filter.setIncludeUntagged(true)
    modal.handleViewChange('myskills')
  }, [filter, modal])

  const handleViewTag = useCallback(
    (tagId: number) => {
      filter.setSelectedTagIds([tagId])
      filter.setIncludeUntagged(false)
      modal.handleViewChange('myskills')
    },
    [filter, modal],
  )

  const handleSyncAllNewTools = useCallback(() => {
    modal.setShowNewToolsModal(false)
  }, [modal])

  // ─── Render ──────────────────────────────────────
  return (
    <div className="skills-app">
      <Toaster position="top-right" richColors toastOptions={{ duration: 1800 }} />
      <LoadingOverlay
        loading={globalLoading}
        actionMessage={appState.actionMessage}
        loadingStartAt={globalLoadingStartAt}
        onCancel={importFlow.handleCancelLoading}
        t={t}
      />

      <Header
        language={appState.language}
        loading={globalLoading}
        activeView={modal.activeView}
        activeSkillSource={modal.activeSkillSource}
        skillCount={skills.managedSkills.length}
        customSkillCount={customSkillCount}
        communitySkillCount={communitySkillCount}
        toolCount={skills.installedTools.length}
        onToggleLanguage={appState.toggleLanguage}
        onOpenSettings={modal.openSettings}
        onViewChange={modal.handleViewChange}
        onSkillSourceChange={(source) => {
          modal.setActiveSkillSource(source)
          modal.backToList()
        }}
        t={t}
      />

      <main className="skills-main">
        <Suspense fallback={<div className="view-loading" />}>
        {modal.activeView === 'detail' && modal.detailSkill ? (
          <SkillDetailView
            skill={modal.detailSkill}
            onBack={modal.backToList}
            formatRelative={skills.formatRelative}
            t={t}
          />
        ) : modal.activeView === 'myskills' ? (
          <div className="dashboard-stack">
            <FilterBar
              sortBy={filter.sortBy}
              searchQuery={filter.searchQuery}
              scopeFilter={filter.scopeFilter}
              toolFilter={filter.toolFilter}
              installedTools={skills.installedTools}
              tags={skills.tags}
              selectedTagIds={filter.selectedTagIds}
              includeUntagged={filter.includeUntagged}
              untaggedCount={filter.untaggedCount}
              totalCount={filter.visibleSkills.length}
              refreshing={skills.refreshingSkills}
              loading={globalLoading}
              onSortChange={filter.handleSortChange}
              onSearchChange={filter.handleSearchChange}
              onScopeFilterChange={filter.handleScopeFilterChange}
              onToolFilterChange={filter.handleToolFilterChange}
              onRefresh={() => skills.handleRefreshSkills(modal.activeSkillSource)}
              onOpenAdd={() => addSkill.handleOpenAdd(modal.activeSkillSource)}
              onToggleTag={filter.handleToggleTagFilter}
              onToggleUntagged={filter.handleToggleUntaggedFilter}
              onClearTags={filter.handleClearTagFilters}
              onManageTags={handleOpenTagsPage}
              t={t}
            />
            <SkillsList
              plan={modal.activeSkillSource === 'community' ? importFlow.plan : null}
              visibleSkills={filter.visibleSkills}
              installedTools={skills.installedTools}
              loading={globalLoading}
              getSkillSourceLabel={skills.getSkillSourceLabel}
              formatRelative={skills.formatRelative}
              onReviewImport={handleReviewImport}
              onDeleteSkill={modal.setPendingDeleteId}
              onToggleTool={scopeManager.handleToggleToolForSkill}
              onOpenScope={scopeManager.handleOpenScope}
              onOpenDetail={modal.openInfoModal}
              onEditTags={modal.openEditTags}
              getSkillScope={getSkillScope}
              getSkillProjects={skills.getSkillProjects}
              draggable={filter.sortBy === 'manual'}
              onReorder={skills.reorderSkills}
              t={t}
            />
          </div>
        ) : modal.activeView === 'tags' ? (
          <TagsPage
            tags={skills.tags}
            untaggedCount={filter.untaggedCount}
            loading={globalLoading}
            formatRelative={skills.formatRelative}
            onReviewUntagged={handleReviewUntagged}
            onViewTag={handleViewTag}
            onCreateTag={handleCreateTag}
            onRenameTag={handleRenameTag}
            onDeleteTag={handleDeleteTag}
            onReorder={skills.reorderTags}
            t={t}
          />
        ) : modal.activeView === 'settings' ? (
          <SettingsPage
            language={appState.language}
            storagePath={theme.storagePath}
            customRepoPath={theme.customRepoPath}
            themePreference={theme.themePreference}
            onBack={() => modal.handleViewChange('myskills')}
            onPickStoragePath={theme.handlePickStoragePath}
            onPickCustomRepoPath={theme.handlePickCustomRepoPath}
            onOpenFolder={theme.handleOpenFolder}
            onResetDefaults={theme.handleResetDefaults}
            onSetLanguage={appState.setLanguage}
            onThemeChange={theme.handleThemeChange}
            t={t}
          />
        ) : modal.activeView === 'tools' ? (
          <ToolsPage t={t} />
        ) : modal.activeView === 'prompts' ? (
          <PromptsPage t={t} />
        ) : null}
        </Suspense>
      </main>

      {/* Modals – lazy-loaded & rendered only when needed */}
      <Suspense fallback={null}>
        {currentInfoModalSkill ? (
          <SkillInfoModal
            skill={currentInfoModalSkill}
            installedTools={skills.installedTools}
            loading={globalLoading}
            getSkillSourceLabel={skills.getSkillSourceLabel}
            formatRelative={skills.formatRelative}
            onRequestClose={modal.closeInfoModal}
            onViewFiles={modal.viewSkillFiles}
            onDelete={(skillId) => {
              modal.closeInfoModal()
              modal.setPendingDeleteId(skillId)
            }}
            onEditTags={(skill) => {
              modal.closeInfoModal()
              modal.openEditTags(skill)
            }}
            onOpenScope={(skill) => {
              modal.closeInfoModal()
              scopeManager.handleOpenScope(skill)
            }}
            getSkillScope={getSkillScope}
            getSkillProjects={skills.getSkillProjects}
            onUpdateSourceUrl={skills.handleUpdateSourceUrl}
            t={t}
          />
        ) : null}

        {addSkill.showAddModal ? (
          <AddSkillModal
            open={addSkill.showAddModal}
            loading={globalLoading}
            canClose={!globalLoading}
            localPath={addSkill.localPath}
            localName={addSkill.localName}
            sourceType={addSkill.addSourceType}
            tags={skills.tags}
            selectedTagIds={addSkill.addModalTagIds}
            syncTargets={importFlow.syncTargets}
            installedTools={skills.installedTools}
            toolStatus={skills.toolStatus}
            onRequestClose={addSkill.handleCloseAdd}
            onLocalPathChange={addSkill.setLocalPath}
            onPickLocalPath={addSkill.handlePickLocalPath}
            onLocalNameChange={addSkill.setLocalName}
            onSourceTypeChange={addSkill.setAddSourceType}
            onToggleTag={addSkill.handleToggleAddModalTag}
            onSyncTargetChange={importFlow.handleSyncTargetChange}
            onSubmit={addSkill.handleCreateLocal}
            t={t}
          />
        ) : null}

        {modal.tagEditorSkill ? (
          <EditSkillTagsModal
            key={`${modal.tagEditorSkill.id}-${modal.tagEditorSkill.tags.map((tag) => tag.id).join('-')}`}
            open={Boolean(modal.tagEditorSkill)}
            loading={globalLoading}
            skill={
              skills.managedSkills.find((s) => s.id === modal.tagEditorSkill!.id) ?? modal.tagEditorSkill
            }
            tags={skills.tags}
            onRequestClose={modal.closeEditTags}
            onSave={handleSaveSkillTags}
            t={t}
          />
        ) : null}

        {modal.showImportModal && importFlow.plan ? (
          <ImportModal
            open={modal.showImportModal}
            loading={globalLoading}
            plan={importFlow.plan}
            selected={importFlow.selected}
            variantChoice={importFlow.variantChoice}
            storagePath={theme.storagePath}
            onRequestClose={() => {
              if (!globalLoading) modal.setShowImportModal(false)
            }}
            onToggleGroup={importFlow.handleToggleGroup}
            onSelectVariant={importFlow.handleSelectVariant}
            onImport={async () => {
              const ok = await importFlow.handleImport()
              if (ok) modal.setShowImportModal(false)
            }}
            t={t}
          />
        ) : null}

        {scopeManager.pendingSharedToggle ? (
          <SharedDirModal
            open={Boolean(scopeManager.pendingSharedToggle)}
            loading={globalLoading}
            toolLabel={scopeManager.pendingSharedLabels?.toolLabel ?? ''}
            otherLabels={scopeManager.pendingSharedLabels?.otherLabels ?? ''}
            onRequestClose={scopeManager.handleSharedCancel}
            onConfirm={scopeManager.handleSharedConfirm}
            t={t}
          />
        ) : null}

        {scopeManager.suiteSyncState ? (
          <SuiteSyncModal
            open={Boolean(scopeManager.suiteSyncState)}
            loading={globalLoading}
            toolLabel={scopeManager.suiteSyncToolLabel}
            subSkills={scopeManager.suiteSyncState.subSkills}
            loadingSubSkills={scopeManager.suiteSyncState.loadingSubSkills}
            onRequestClose={scopeManager.handleSuiteSyncClose}
            onConfirm={scopeManager.handleSuiteSyncConfirm}
            t={t}
          />
        ) : null}

        {scopeManager.currentScopeModalSkill ? (
          <ScopeSyncModal
            key={`${scopeManager.currentScopeModalSkill.id}-${getSkillScope(scopeManager.currentScopeModalSkill)}`}
            open={Boolean(scopeManager.currentScopeModalSkill)}
            loading={globalLoading}
            skill={scopeManager.currentScopeModalSkill}
            scope={getSkillScope(scopeManager.currentScopeModalSkill)}
            projects={skills.getSkillProjects(scopeManager.currentScopeModalSkill)}
            recentProjects={scopeManager.recentProjects}
            onRequestClose={scopeManager.handleCloseScope}
            onScopeChange={scopeManager.handleScopeChange}
            onPickProject={scopeManager.handlePickProject}
            t={t}
          />
        ) : null}

        {modal.showNewToolsModal && skills.newlyInstalledToolsText ? (
          <NewToolsModal
            open={Boolean(modal.showNewToolsModal && skills.newlyInstalledToolsText)}
            loading={globalLoading}
            toolsLabelText={skills.newlyInstalledToolsText}
            onDismiss={handleSyncAllNewTools}
            t={t}
          />
        ) : null}

        {modal.pendingDeleteId ? (
          <DeleteModal
            open={Boolean(modal.pendingDeleteId)}
            loading={globalLoading}
            skillName={pendingDeleteSkill?.name ?? null}
            onRequestClose={handleCloseDelete}
            onConfirm={() => {
              if (pendingDeleteSkill) void handleDeleteManaged(pendingDeleteSkill)
            }}
            t={t}
          />
        ) : null}
      </Suspense>

      {/* Inline tag delete modal (simple, no lazy needed) */}
      {modal.pendingDeleteTag ? (
        <div
          className="modal-backdrop"
          onClick={globalLoading ? undefined : handleCloseDeleteTag}
        >
          <div
            className="modal modal-delete tag-delete-modal"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="modal-header">
              <div className="modal-title">{t('deleteTagTitle')}</div>
              <button
                className="modal-close"
                type="button"
                onClick={handleCloseDeleteTag}
                disabled={globalLoading}
              >
                {'×'}
              </button>
            </div>
            <div className="modal-body tag-delete-body">
              {t('deleteTagConfirm', {
                name: modal.pendingDeleteTag.name,
                count: modal.pendingDeleteTag.skill_count,
              })}
            </div>
            <div className="modal-footer">
              <button
                className="btn btn-secondary"
                type="button"
                onClick={handleCloseDeleteTag}
                disabled={globalLoading}
              >
                {t('cancel')}
              </button>
              <button
                className="btn btn-danger"
                type="button"
                onClick={() => void handleConfirmDeleteTag()}
                disabled={globalLoading}
              >
                {t('deleteAction')}
              </button>
            </div>
          </div>
        </div>
      ) : null}

      <Suspense fallback={null}>
        {addSkill.showLocalPickModal ? (
          <LocalPickModal
            open={addSkill.showLocalPickModal}
            loading={globalLoading}
            localCandidates={addSkill.localCandidates}
            localCandidateSelected={addSkill.localCandidateSelected}
            onRequestClose={addSkill.handleCloseLocalPick}
            onCancel={addSkill.handleCancelLocalPick}
            onToggleCandidate={addSkill.handleToggleLocalCandidate}
            onInstall={addSkill.handleInstallSelectedLocalCandidates}
            t={t}
          />
        ) : null}
      </Suspense>
    </div>
  )
}

export default App
