import { invokeCommand } from '@/lib/api'

export const tagService = {
  createTag(name: string): Promise<void> {
    return invokeCommand('create_tag', { name })
  },

  renameTag(tagId: number, name: string): Promise<{ id: number; name: string }> {
    return invokeCommand<{ id: number; name: string }>('rename_tag', { tag_id: tagId, name })
  },

  deleteTag(tagId: number): Promise<void> {
    return invokeCommand('delete_tag', { tag_id: tagId })
  },
}
