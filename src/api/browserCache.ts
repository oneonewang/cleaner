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

/** 启动扫描,返回 { scan_id };结果通过 `browser-cache-result` 事件回传,进度通过 `scan-progress` 事件推送 */
export async function scanBrowserCache(
  profileIds: string[],
): Promise<{ scan_id: string }> {
  return invoke<{ scan_id: string }>('scan_browser_cache', { profileIds })
}

export interface CleanSummary {
  total_files: number
  total_bytes: number
  errors: string[]
}

/** 清理浏览器缓存 */
export async function cleanBrowserCache(
  cachePaths: string[],
): Promise<CleanSummary> {
  return invoke<CleanSummary>('clean_browser_cache', { cachePaths })
}
