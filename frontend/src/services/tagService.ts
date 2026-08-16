import { apiCall } from '@/lib/api'

export const tagService = {
  createTag(name: string): Promise<void> {
    return apiCall('create_tag', { name })
  },

  renameTag(tagId: number, name: string): Promise<{ id: number; name: string }> {
    return apiCall<{ id: number; name: string }>('rename_tag', { tag_id: tagId, name })
  },

  deleteTag(tagId: number): Promise<void> {
    return apiCall('delete_tag', { tag_id: tagId })
  },
}
