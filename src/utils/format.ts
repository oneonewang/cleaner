/** 字节数格式化(与 SizeText 一致,但纯函数) */
const UNITS = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']

export function formatSize(bytes: number, mode: 'auto' | 'short' = 'auto', digits = 2): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B'
  const exp = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), UNITS.length - 1)
  const value = bytes / Math.pow(1024, exp)
  const d = mode === 'short' ? digits : value >= 100 ? 0 : value >= 10 ? 1 : digits
  return `${value.toFixed(d)} ${UNITS[exp]}`
}

export function formatDate(ts: number | null | undefined, locale: string = 'zh-CN'): string {
  if (!ts) return '—'
  const d = new Date(ts)
  return d.toLocaleString(locale === 'zh-CN' ? 'zh-CN' : 'en-US', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}
