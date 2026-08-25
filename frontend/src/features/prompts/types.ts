export type PromptFileDto = {
  id: string
  tool: string
  scope: string
  file_name: string
  file_path: string
  content_hash: string | null
  exists_on_disk: boolean
  last_scanned_at: number
  created_at: number
  updated_at: number
}
