import { invoke } from '@tauri-apps/api/core'

export type BrowserKind = 'Chrome' | 'Edge' | 'Brave' | 'Opera' | 'Firefox'

export interface BrowserProfile {
  id: string
  browser: BrowserKind
  profile_name: string
  display_name: string
  cache_paths: string[]
  total_bytes: number
}

/** 检测本机已安装的浏览器 */
export async function detectBrowsers(): Promise<BrowserProfile[]> {
  return invoke<BrowserProfile[]>('detect_browsers')
}

/** 扫描选定 profile 的缓存大小 */
export async function scanBrowserCache(
  profileIds: string[],
): Promise<BrowserProfile[]> {
  return invoke<BrowserProfile[]>('scan_browser_cache', { profileIds })
}

export interface CleanSummary {
  total_files: number
  total_bytes: number
  errors: string[]
}

/** 清理浏览器缓存 */
export async function cleanBrowserCache(
  cachePaths: string[],
  toTrash: boolean,
): Promise<CleanSummary> {
  return invoke<CleanSummary>('clean_browser_cache', { cachePaths, toTrash })
}
