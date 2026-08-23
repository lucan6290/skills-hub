import { invokeCommand } from '@/lib/api'

export const skillService = {
  deleteManagedSkill(skillId: string): Promise<void> {
    return invokeCommand('delete_managed_skill', { skill_id: skillId })
  },

  setSkillTags(skillId: string, tagIds: number[]): Promise<void> {
    return invokeCommand('set_skill_tags', { skill_id: skillId, tag_ids: tagIds })
  },
}
