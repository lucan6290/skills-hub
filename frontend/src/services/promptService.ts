import { invokeCommand } from '@/lib/api'
import type { PromptFileDto } from '@/features/prompts/types'

export const promptService = {
  scanPromptFiles(): Promise<PromptFileDto[]> {
    return invokeCommand('scan_prompt_files')
  },

  scanProjectPromptFiles(projectPath: string): Promise<PromptFileDto[]> {
    return invokeCommand('scan_project_prompt_files', { project_path: projectPath })
  },

  getPromptFiles(tool?: string): Promise<PromptFileDto[]> {
    return invokeCommand('get_prompt_files', { tool: tool ?? null })
  },

  readPromptFile(filePath: string): Promise<string> {
    return invokeCommand('read_prompt_file', { file_path: filePath })
  },

  writePromptFile(filePath: string, content: string): Promise<void> {
    return invokeCommand('write_prompt_file', { file_path: filePath, content })
  },

  deletePromptFile(id: string, deleteFromDisk?: boolean): Promise<void> {
    return invokeCommand('delete_prompt_file', { id, delete_from_disk: deleteFromDisk ?? false })
  },
}
