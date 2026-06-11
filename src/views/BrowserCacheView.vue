<template>
  <div class="browser-cache-view">
    <header class="view-header">
      <div>
        <h1>{{ t('browser.title') }}</h1>
        <p class="text-muted">{{ t('browser.description') }}</p>
      </div>
      <div class="view-header__actions">
        <el-button :icon="Search" type="primary" @click="onScan" :loading="scanning">
          {{ t('common.scan') }}
        </el-button>
        <el-button v-if="scanning" :icon="CircleClose" @click="onCancel">
          {{ t('common.stop') }}
        </el-button>
        <el-button
          :icon="Delete"
          type="danger"
          :disabled="selectedIds.length === 0"
          :loading="cleaning"
          @click="onClean"
        >
          {{ t('common.clean') }}
        </el-button>
      </div>
    </header>

    <ScanProgressBar
      v-if="scanning"
      :title="t('common.scanning')"
      :progress="cleaningStore.progress"
      :current-path="cleaningStore.currentPath"
      class="mt-16 mb-16"
    />

    <el-alert
      v-if="profiles.length > 0"
      :title="t('browser.closeBrowserWarn')"
      type="warning"
      :closable="false"
      show-icon
      class="mt-16 mb-16"
    />

    <el-table
      v-if="profiles.length > 0"
      :data="profiles"
      @selection-change="onSelectionChange"
      row-key="id"
      stripe
    >
      <el-table-column type="selection" width="50" />
      <el-table-column :label="t('nav.browser')" min-width="220">
        <template #default="{ row }">
          <div class="flex items-center gap-8">
            <el-icon class="browser-cache-view__icon" :class="row.browser.toLowerCase()">
              <component :is="browserIcon(row.browser)" />
            </el-icon>
            <div>
              <div class="browser-cache-view__name">{{ browserLabel(row.browser) }}</div>
              <div class="text-muted browser-cache-view__profile">{{ row.profile_name }}</div>
            </div>
          </div>
        </template>
      </el-table-column>
      <el-table-column :label="t('common.bytes')" width="160" align="right">
        <template #default="{ row }">
          <SizeText :bytes="row.total_bytes" mode="auto" class="browser-cache-view__size" />
        </template>
      </el-table-column>
      <el-table-column :label="t('common.files')" width="100" align="right">
        <template #default="{ row }">{{ row.cache_paths.length }}</template>
      </el-table-column>
    </el-table>

    <EmptyState v-else :description="t('common.noData')" />

    <ConfirmDialog
      v-model="confirmOpen"
      :count="selectedCount"
      :bytes="selectedBytes"
      :permanent="!settingsStore.toTrash"
      @confirm="doClean"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Search, Delete, CircleClose, ChromeFilled } from '@element-plus/icons-vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import SizeText from '@/components/SizeText.vue'
import ScanProgressBar from '@/components/ScanProgressBar.vue'
import EmptyState from '@/components/EmptyState.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  detectBrowsers,
  scanBrowserCache,
  cleanBrowserCache,
  type BrowserProfile,
  type BrowserKind,
} from '@/api/browserCache'
import { cancelScan } from '@/api'
import { useCleaningStore } from '@/stores/cleaning'
import { useSettingsStore } from '@/stores/settings'
import { formatSize } from '@/utils/format'

const { t } = useI18n()
const cleaningStore = useCleaningStore()
const settingsStore = useSettingsStore()

const profiles = ref<BrowserProfile[]>([])
const selectedIds = ref<string[]>([])
const scanning = ref(false)
const cleaning = ref(false)
const confirmOpen = ref(false)

let unResult: UnlistenFn | null = null

const selectedCount = computed(() => selectedIds.value.length)
const selectedBytes = computed(() => {
  const sel = new Set(selectedIds.value)
  return profiles.value
    .filter((p) => sel.has(p.id))
    .reduce((acc, p) => acc + p.total_bytes, 0)
})

function browserLabel(b: BrowserKind) {
  return t(`browser.browsers.${b.toLowerCase()}`, b)
}
function browserIcon(_b: BrowserKind) {
  return ChromeFilled
}

function onSelectionChange(rows: BrowserProfile[]) {
  selectedIds.value = rows.map((r) => r.id)
}

async function setupListener() {
  if (unResult) return
  unResult = await listen<BrowserProfile[]>('browser-cache-result', (e) => {
    if (cleaningStore.scanKind !== 'browser') return
    const map = new Map(e.payload.map((p) => [p.id, p]))
    profiles.value = profiles.value.map((p) => map.get(p.id) ?? p)
  })
}

async function onScan() {
  scanning.value = true
  selectedIds.value = []
  cleaningStore.reset()
  await cleaningStore.attach()
  await setupListener()
  try {
    const detected = await detectBrowsers()
    if (detected.length === 0) {
      ElMessage.warning(t('common.noData'))
      profiles.value = []
      scanning.value = false
      return
    }
    profiles.value = detected
    const { scan_id } = await scanBrowserCache(detected.map((p) => p.id))
    cleaningStore.beginScan('browser', scan_id)
  } catch (e: unknown) {
    ElMessage.error(t('errors.scanFailed', { msg: String(e) }))
    scanning.value = false
  }
}

async function onCancel() {
  if (cleaningStore.scanId) {
    try {
      await cancelScan(cleaningStore.scanId)
    } catch (e) {
      console.warn(e)
    }
  }
}

function onClean() {
  if (settingsStore.confirmBeforeClean) {
    confirmOpen.value = true
  } else {
    doClean()
  }
}

async function doClean() {
  const sel = new Set(selectedIds.value)
  const paths = profiles.value
    .filter((p) => sel.has(p.id))
    .flatMap((p) => p.cache_paths)
  if (paths.length === 0) return
  cleaning.value = true
  try {
    const summary = await cleanBrowserCache(paths, settingsStore.toTrash)
    settingsStore.recordCleanup(summary.total_bytes)
    ElMessage.success(t('common.totalFreed', { n: formatSize(summary.total_bytes) }))
    profiles.value = profiles.value.filter((p) => !sel.has(p.id))
    selectedIds.value = []
  } catch (e: unknown) {
    ElMessage.error(t('errors.cleanFailed', { msg: String(e) }))
  } finally {
    cleaning.value = false
  }
}

// 监听 scanning 状态变化,扫描结束后关闭 loading
const stopWatch = (async () => {
  const { watch } = await import('vue')
  return watch(
    () => cleaningStore.scanning,
    (v) => {
      if (cleaningStore.scanKind === 'browser') {
        scanning.value = v
      }
    },
  )
})()

onMounted(() => {
  settingsStore.loadFromStorage()
})

onUnmounted(async () => {
  const stop = await stopWatch
  stop()
  if (unResult) {
    unResult()
    unResult = null
  }
})
</script>

<style lang="scss" scoped>
.browser-cache-view {
  &__icon {
    font-size: 20px;
    color: var(--ooc-primary);
    &.edge { color: #0078d4; }
    &.firefox { color: #ff7139; }
    &.opera { color: #ff1b2d; }
    &.brave { color: #fb542b; }
  }
  &__name {
    font-weight: 500;
  }
  &__profile {
    font-size: 12px;
  }
  &__size {
    font-weight: 600;
  }
}
.view-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 16px;
  h1 {
    font-size: 20px;
    margin: 0 0 4px;
  }
  p {
    margin: 0;
    font-size: 13px;
  }
  &__actions {
    display: flex;
    gap: 8px;
  }
}
</style>
