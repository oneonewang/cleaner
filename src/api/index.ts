import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/** 通用:取消扫描 */
export async function cancelScan(scanId: string): Promise<void> {
  return invoke('cancel_scan', { scanId })
}

/** 通用:监听进度事件 */
export async function listenProgress(
  handler: (e: { payload: unknown }) => void,
): Promise<UnlistenFn> {
  return listen('scan-progress', handler)
}

/** 通用:在资源管理器中显示 */
export async function revealInExplorer(path: string): Promise<void> {
  return invoke('reveal_in_explorer', { path })
}

/** 通用:打开路径 */
export async function openPath(path: string): Promise<void> {
  return invoke('open_path', { path })
}
