import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface AppInfo {
  name: string
  version: string
  tauri_version: string
}

export const useSettingsStore = defineStore('settings', () => {
  const confirmBeforeClean = ref(true)
  const theme = ref<'light' | 'dark'>('light')
  const locale = ref<'zh-CN' | 'en-US'>('zh-CN')
  const lastCleanupAt = ref<number | null>(null)
  const totalFreedBytes = ref(0)
  const appInfo = ref<AppInfo | null>(null)
  const isAdmin = ref(false)

  const themeClass = computed(() => (theme.value === 'dark' ? 'dark' : ''))

  function applyTheme() {
    document.documentElement.classList.toggle('dark', theme.value === 'dark')
  }

  function setTheme(t: 'light' | 'dark') {
    theme.value = t
    localStorage.setItem('theme', t)
    applyTheme()
  }

  function setLocale(loc: 'zh-CN' | 'en-US') {
    locale.value = loc
    localStorage.setItem('locale', loc)
    document.documentElement.lang = loc
  }

  function loadFromStorage() {
    const t = localStorage.getItem('theme') as 'light' | 'dark' | null
    if (t) theme.value = t
    const l = localStorage.getItem('locale') as 'zh-CN' | 'en-US' | null
    if (l) locale.value = l
    const last = localStorage.getItem('lastCleanupAt')
    if (last) lastCleanupAt.value = Number(last)
    const total = localStorage.getItem('totalFreedBytes')
    if (total) totalFreedBytes.value = Number(total)
    const conf = localStorage.getItem('confirmBeforeClean')
    if (conf !== null) confirmBeforeClean.value = conf === 'true'
    applyTheme()
  }

  function recordCleanup(bytes: number) {
    totalFreedBytes.value += bytes
    lastCleanupAt.value = Date.now()
    localStorage.setItem('totalFreedBytes', String(totalFreedBytes.value))
    localStorage.setItem('lastCleanupAt', String(lastCleanupAt.value))
  }

  async function loadAppInfo() {
    try {
      appInfo.value = await invoke<AppInfo>('get_app_info')
    } catch (e) {
      console.warn('get_app_info failed', e)
    }
  }

  async function loadIsAdmin() {
    try {
      const { isAdmin: invokeIsAdmin } = await import('@/api')
      isAdmin.value = await invokeIsAdmin()
    } catch (e) {
      console.warn('is_admin failed', e)
    }
  }

  return {
    confirmBeforeClean,
    theme,
    locale,
    lastCleanupAt,
    totalFreedBytes,
    appInfo,
    isAdmin,
    themeClass,
    setTheme,
    setLocale,
    loadFromStorage,
    recordCleanup,
    loadAppInfo,
    loadIsAdmin,
  }
})
