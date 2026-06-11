import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export type ScanKind = 'system-junk' | 'browser' | 'large-files' | 'registry' | null

export interface ProgressEvent {
  kind: 'started' | 'progress' | 'category_done' | 'item_found' | 'finished' | 'cancelled' | 'error'
  scan_id: string
  scan_type?: string
  category?: string
  bytes?: number
  total_bytes?: number
  current_path?: string
  message?: string
  percent?: number
}

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

export interface CleanSummary {
  total_files: number
  total_bytes: number
  errors: string[]
}

export const useCleaningStore = defineStore('cleaning', () => {
  const scanning = ref(false)
  const cleaning = ref(false)
  const scanKind = ref<ScanKind>(null)
  const scanId = ref<string | null>(null)
  const progress = ref(0)
  const currentPath = ref('')
  const currentCategory = ref('')
  const totalBytes = ref(0)
  const results = ref<JunkCategoryResult[]>([])
  const errorMsg = ref<string | null>(null)

  let unlisten: UnlistenFn | null = null

  const grandTotal = computed(() =>
    results.value.reduce((acc, c) => acc + c.total_bytes, 0),
  )
  const grandCount = computed(() =>
    results.value.reduce((acc, c) => acc + c.file_count, 0),
  )

  async function attach() {
    if (unlisten) return
    unlisten = await listen<ProgressEvent>('scan-progress', (e) => {
      onEvent(e.payload)
    })
  }

  async function detach() {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  }

  function onEvent(ev: ProgressEvent) {
    if (scanId.value && ev.scan_id !== scanId.value) return
    switch (ev.kind) {
      case 'started':
        progress.value = 0
        break
      case 'progress':
        if (typeof ev.percent === 'number') progress.value = ev.percent
        if (ev.current_path) currentPath.value = ev.current_path
        if (ev.category) currentCategory.value = ev.category
        break
      case 'category_done':
        if (ev.category && typeof ev.bytes === 'number') {
          const cat = results.value.find((c) => c.id === ev.category)
          if (cat) cat.total_bytes = ev.bytes
        }
        break
      case 'item_found':
        if (ev.category && ev.current_path && typeof ev.bytes === 'number') {
          let cat = results.value.find((c) => c.id === ev.category)
          if (!cat) {
            cat = {
              id: ev.category,
              name: ev.category,
              total_bytes: 0,
              file_count: 0,
              files: [],
            }
            results.value.push(cat)
          }
          cat.total_bytes += ev.bytes
          cat.file_count += 1
          if (cat.files.length < 500) {
            cat.files.push({
              path: ev.current_path,
              size: ev.bytes,
              is_dir: false,
            })
          }
        }
        break
      case 'finished':
        scanning.value = false
        if (typeof ev.total_bytes === 'number') totalBytes.value = ev.total_bytes
        progress.value = 100
        break
      case 'cancelled':
        scanning.value = false
        break
      case 'error':
        scanning.value = false
        errorMsg.value = ev.message ?? 'unknown error'
        break
    }
  }

  /** 一次性写入完整结果(用于 system-junk-result、browser-cache-result 等) */
  function setResults(items: JunkCategoryResult[]) {
    results.value = items
  }

  function reset() {
    results.value = []
    progress.value = 0
    currentPath.value = ''
    currentCategory.value = ''
    totalBytes.value = 0
    errorMsg.value = null
  }

  function beginScan(kind: ScanKind, id: string) {
    reset()
    scanKind.value = kind
    scanId.value = id
    scanning.value = true
    progress.value = 0
  }

  function endClean() {
    cleaning.value = false
  }

  function beginClean() {
    cleaning.value = true
  }

  return {
    scanning,
    cleaning,
    scanKind,
    scanId,
    progress,
    currentPath,
    currentCategory,
    totalBytes,
    results,
    errorMsg,
    grandTotal,
    grandCount,
    attach,
    detach,
    reset,
    setResults,
    beginScan,
    beginClean,
    endClean,
  }
})
