import { invokeCommand } from './api'

/**
 * 通过 Tauri Rust command 打开系统原生文件夹选择对话框，返回用户选择的文件夹路径字符串。
 * 如果 Rust command 不可用，回退到 prompt() 文本输入。
 */
export async function pickFolder(promptTitle: string): Promise<string | null> {
  try {
    const res = await invokeCommand<{ path: string | null }>('pick_folder')
    return res.path
  } catch {
    // Rust command 不可用时回退到 prompt
    const result = window.prompt(promptTitle)
    return result || null
  }
}
