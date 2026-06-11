import { invoke } from '@tauri-apps/api/core'

export type RegistryScope = 'Run' | 'RunOnce' | 'Uninstall' | 'Com'
export type RiskLevel = 'High' | 'Medium' | 'Low'

export interface RegistryIssue {
  id: string
  scope: RegistryScope
  hive: 'HKLM' | 'HKCU'
  key_path: string
  value_name: string | null
  value_data: string | null
  description: string
  risk: RiskLevel
  /** 是否在白名单中(白名单内的项目不应被自动选中) */
  whitelisted: boolean
}

export interface CleanSummary {
  total_files: number
  total_bytes: number
  errors: string[]
}

/** 扫描注册表 */
export async function scanRegistry(
  scopes: RegistryScope[],
): Promise<RegistryIssue[]> {
  return invoke<RegistryIssue[]>('scan_registry', { scopes })
}

/** 备份注册表,返回 .reg 文件路径 */
export async function backupRegistry(
  issues: RegistryIssue[],
): Promise<string> {
  return invoke<string>('backup_registry', { issues })
}

/** 清理注册表(必须先备份) */
export async function cleanRegistry(
  issues: RegistryIssue[],
): Promise<CleanSummary> {
  return invoke<CleanSummary>('clean_registry', { issues })
}
