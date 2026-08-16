import { useEffect, useState } from 'react'
import { fetchScopePreferences } from '@/lib/api'

type SkillScopeState = Record<
  string,
  {
    scope: 'global' | 'project'
    projects: string[]
  }
>

const STORAGE_KEY = 'skills-project-scope-state-v1'

/**
 * Scope 持久化 hook。
 * 从 App.tsx 提取 skillScopeState 初始化 + fetchScopePreferences + localStorage 同步逻辑。
 */
export function useScopeState() {
  const [skillScopeState, setSkillScopeState] = useState<SkillScopeState>({})

  useEffect(() => {
    if (typeof window === 'undefined') return
    fetchScopePreferences()
      .then((prefs) => {
        if (prefs.length > 0) {
          const dbState: SkillScopeState = {}
          for (const p of prefs) {
            let projectPaths: string[] = []
            try {
              projectPaths = JSON.parse(p.project_paths) as string[]
            } catch { /* ignore parse errors */ }
            dbState[p.skill_id] = { scope: p.scope as 'global' | 'project', projects: projectPaths }
          }
          setSkillScopeState(dbState)
          try {
            window.localStorage.setItem(STORAGE_KEY, JSON.stringify(dbState))
          } catch { /* ignore */ }
        } else {
          try {
            const raw = window.localStorage.getItem(STORAGE_KEY)
            if (raw) {
              setSkillScopeState(JSON.parse(raw) as SkillScopeState)
            }
          } catch {
            setSkillScopeState({})
          }
        }
      })
      .catch(() => {
        try {
          const raw = window.localStorage.getItem(STORAGE_KEY)
          if (raw) {
            setSkillScopeState(JSON.parse(raw) as SkillScopeState)
          }
        } catch {
          setSkillScopeState({})
        }
      })
  }, [])

  useEffect(() => {
    if (typeof window === 'undefined') return
    try {
      window.localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify(skillScopeState),
      )
    } catch {
      // ignore storage failures
    }
  }, [skillScopeState])

  return { skillScopeState, setSkillScopeState }
}
