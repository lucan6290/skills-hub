import { apiGet } from './api'

/**
 * 通过后端打开系统原生文件夹选择对话框，返回用户选择的文件夹路径字符串。
 * 如果后端不可用，回退到 prompt() 文本输入。
 */
export async function pickFolder(promptTitle: string): Promise<string | null> {
  try {
    const res = await apiGet<{ path: string | null }>('pick_folder')
    return res.path
  } catch {
    // 后端不可用时回退到 prompt
    const result = window.prompt(promptTitle)
    return result || null
  }
}
