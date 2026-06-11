import { invoke } from '@tauri-apps/api/core'

export interface JunkFile {
  path: string
  size: number
  is_dir: boolean
}

export interface JunkCategoryResult {
  id: string
  name: string
  description?: string | null
  total_bytes: number
  file_count: number
  files: JunkFile[]
}

export interface CleanItem {
  category: string
  paths: string[]
}

export interface CleanSummary {
  total_files: number
  total_bytes: number
  errors: string[]
}

/** 列出所有系统垃圾类别(无需扫描) */
export async function listSystemJunkCategories(): Promise<JunkCategoryResult[]> {
  return invoke<JunkCategoryResult[]>('list_system_junk_categories')
}

/** 启动系统垃圾扫描,返回 scanId */
export async function scanSystemJunk(
  categories: string[] | null,
): Promise<{ scan_id: string }> {
  return invoke('scan_system_junk', { categories })
}

/** 按勾选项清理系统垃圾 */
export async function cleanSystemJunk(
  items: CleanItem[],
  toTrash: boolean,
): Promise<CleanSummary> {
  return invoke<CleanSummary>('clean_system_junk', { items, toTrash })
}
