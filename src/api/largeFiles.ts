import { invoke } from '@tauri-apps/api/core'

export interface LargeFile {
  path: string
  size: number
  last_access: number
  modified: number
  is_dir: boolean
}

export interface CleanSummary {
  total_files: number
  total_bytes: number
  errors: string[]
}

/** 扫描大文件,返回 scanId */
export async function scanLargeFiles(
  roots: string[],
  minSize: number,
  olderThanDays: number,
): Promise<{ scan_id: string }> {
  return invoke('scan_large_files', { roots, minSize, olderThanDays })
}

/** 批量删除文件 */
export async function deletePaths(
  paths: string[],
): Promise<CleanSummary> {
  return invoke<CleanSummary>('delete_paths', { paths })
}
