export type OnboardingVariant = {
  tool: string
  name: string
  path: string
  fingerprint?: string | null
  is_link: boolean
  link_target?: string | null
}

export type OnboardingGroup = {
  name: string
  variants: OnboardingVariant[]
  has_conflict: boolean
}

export type OnboardingPlan = {
  total_tools_scanned: number
  total_skills_found: number
  groups: OnboardingGroup[]
}

export type ToolOption = {
  id: string
  label: string
  supports_project_scope?: boolean
}

export type TagDto = {
  id: number
  name: string
  sort_order: number
}

export type TagWithCountDto = TagDto & {
  skill_count: number
  updated_at: number
}

export type ManagedSkill = {
  id: string
  name: string
  description?: string | null
  source_type: string
  source_ref?: string | null
  source_subpath?: string | null
  source_url?: string | null
  community_path: string
  created_at: number
  updated_at: number
  last_sync_at?: number | null
  status: string
  tags: TagDto[]
  targets: {
    tool: string
    scope: 'global' | 'project' | string
    project_path?: string | null
    mode: string
    status: string
    target_path: string
    synced_at?: number | null
    suite_skill_id?: string | null
  }[]
  version?: string | null
  author?: string | null
  license?: string | null
  category?: string | null
  homepage?: string | null
  frontmatter_extra?: Record<string, string> | null
  skill_file_count?: number | null
  skill_dir_size?: number | null
  usage?: SkillUsage[] | null
  sort_order: number
  is_suite?: boolean
}

export interface SuiteSubSkill {
  name: string
  subpath: string
  description?: string | null
}

export interface SkillUsage {
  tool: string
  sync_count: number
  last_synced_at: number | null
  last_viewed_at: number | null
  view_count: number
}

export type LocalSkillCandidate = {
  name: string
  description?: string | null
  subpath: string
  valid: boolean
  reason?: string | null
}

export type InstallResultDto = {
  skill_id: string
  name: string
  community_path: string
  content_hash?: string | null
}

export type ToolInfoDto = {
  key: string
  label: string
  installed: boolean
  skills_dir: string
  supports_project_scope: boolean
}

export type ToolStatusDto = {
  tools: ToolInfoDto[]
  installed: string[]
  newly_installed: string[]
}

export type SkillFileEntry = {
  path: string
  size: number
}

export type SkillSource = 'custom' | 'community'
