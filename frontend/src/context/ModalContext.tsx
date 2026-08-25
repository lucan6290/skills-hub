import React, { createContext, useCallback, useContext, useMemo, useState } from 'react'
import type { ManagedSkill, TagWithCountDto } from '@/features/skills'

// ─── Types ────────────────────────────────────────────
type ModalState = {
  // Skill info modal
  infoModalSkill: ManagedSkill | null
  // Import modal
  showImportModal: boolean
  // New tools notification modal
  showNewToolsModal: boolean
  // Delete skill confirmation
  pendingDeleteId: string | null
  // Tag editing modal
  tagEditorSkill: ManagedSkill | null
  // Delete tag confirmation
  pendingDeleteTag: TagWithCountDto | null
  // Active view
  activeView: 'myskills' | 'detail' | 'settings' | 'tags' | 'tools' | 'prompts'
  activeSkillSource: 'custom' | 'community'
  detailSkill: ManagedSkill | null
}

type ModalActions = {
  openInfoModal: (skill: ManagedSkill) => void
  closeInfoModal: () => void
  setShowImportModal: (show: boolean) => void
  setShowNewToolsModal: (show: boolean) => void
  setPendingDeleteId: (id: string | null) => void
  openEditTags: (skill: ManagedSkill) => void
  closeEditTags: () => void
  setPendingDeleteTag: (tag: TagWithCountDto | null) => void
  handleViewChange: (view: 'myskills' | 'tags' | 'tools' | 'prompts') => void
  setActiveSkillSource: (source: 'custom' | 'community') => void
  setDetailSkill: (skill: ManagedSkill | null) => void
  viewSkillFiles: (skill: ManagedSkill) => void
  backToList: () => void
  openSettings: () => void
  closeSettings: () => void
}

type ModalContextValue = ModalState & ModalActions

const ModalContext = createContext<ModalContextValue | null>(null)

// ─── Provider ─────────────────────────────────────────
export function ModalProvider({ children }: { children: React.ReactNode }) {
  const [infoModalSkill, setInfoModalSkill] = useState<ManagedSkill | null>(null)
  const [showImportModal, setShowImportModal] = useState(false)
  const [showNewToolsModal, setShowNewToolsModal] = useState(false)
  const [pendingDeleteId, setPendingDeleteId] = useState<string | null>(null)
  const [tagEditorSkill, setTagEditorSkill] = useState<ManagedSkill | null>(null)
  const [pendingDeleteTag, setPendingDeleteTag] = useState<TagWithCountDto | null>(null)
  const [activeView, setActiveView] = useState<ModalState['activeView']>('myskills')
  const [activeSkillSource, setActiveSkillSource] = useState<ModalState['activeSkillSource']>('custom')
  const [detailSkill, setDetailSkill] = useState<ManagedSkill | null>(null)

  const openInfoModal = useCallback((skill: ManagedSkill) => {
    setInfoModalSkill(skill)
  }, [])

  const closeInfoModal = useCallback(() => {
    setInfoModalSkill(null)
  }, [])

  const openEditTags = useCallback((skill: ManagedSkill) => {
    setTagEditorSkill(skill)
  }, [])

  const closeEditTags = useCallback(() => {
    setTagEditorSkill(null)
  }, [])

  const handleViewChange = useCallback((view: 'myskills' | 'tags' | 'tools' | 'prompts') => {
    setInfoModalSkill(null)
    setActiveView(view)
    if (view === 'myskills') {
      setDetailSkill(null)
    }
  }, [])

  const openSettings = useCallback(() => {
    setActiveView('settings')
  }, [])

  const closeSettings = useCallback(() => {
    setActiveView('myskills')
  }, [])

  const viewSkillFiles = useCallback((skill: ManagedSkill) => {
    setInfoModalSkill(null)
    setDetailSkill(skill)
    setActiveView('detail')
  }, [])

  const backToList = useCallback(() => {
    setDetailSkill(null)
    setActiveView('myskills')
  }, [])

  const value = useMemo<ModalContextValue>(
    () => ({
      infoModalSkill,
      showImportModal,
      showNewToolsModal,
      pendingDeleteId,
      tagEditorSkill,
      pendingDeleteTag,
      activeView,
      activeSkillSource,
      detailSkill,
      openInfoModal,
      closeInfoModal,
      setShowImportModal,
      setShowNewToolsModal,
      setPendingDeleteId,
      openEditTags,
      closeEditTags,
      setPendingDeleteTag,
      handleViewChange,
      setActiveSkillSource,
      setDetailSkill,
      viewSkillFiles,
      backToList,
      openSettings,
      closeSettings,
    }),
    [
      infoModalSkill,
      showImportModal,
      showNewToolsModal,
      pendingDeleteId,
      tagEditorSkill,
      pendingDeleteTag,
      activeView,
      activeSkillSource,
      detailSkill,
      openInfoModal,
      closeInfoModal,
      openEditTags,
      closeEditTags,
      handleViewChange,
      viewSkillFiles,
      backToList,
      openSettings,
      closeSettings,
    ],
  )

  return (
    <ModalContext.Provider value={value}>
      {children}
    </ModalContext.Provider>
  )
}

// ─── Hook ─────────────────────────────────────────────
export function useModal() {
  const ctx = useContext(ModalContext)
  if (!ctx) {
    throw new Error('useModal must be used within ModalProvider')
  }
  return ctx
}
