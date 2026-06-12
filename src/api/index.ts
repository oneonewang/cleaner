import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/** 通知 Rust 显示主窗口(由 Vue 挂载后调用,避免 WebView2 启动白屏) */
export async function showMainWindow(): Promise<void> {
  return invoke('show_main_window')
}

/** 当前进程是否以管理员身份运行 */
export async function isAdmin(): Promise<boolean> {
  return invoke<boolean>('is_admin')
}

/** 以管理员身份重启当前应用 */
export async function relaunchAsAdmin(): Promise<void> {
  return invoke('relaunch_as_admin')
}

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
