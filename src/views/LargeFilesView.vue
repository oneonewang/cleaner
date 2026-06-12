<template>
  <div class="large-files-view">
    <header class="view-header">
      <div>
        <h1>{{ t('largeFiles.title') }}</h1>
        <p class="text-muted">{{ t('largeFiles.description') }}</p>
      </div>
    </header>

    <el-card class="large-files-view__filters">
      <el-form :inline="true" label-width="auto">
        <el-form-item :label="t('largeFiles.selectFolder')">
          <div class="flex gap-8 items-center">
            <el-tag
              v-for="(p, i) in roots"
              :key="p"
              closable
              @close="roots.splice(i, 1)"
            >
              {{ p }}
            </el-tag>
            <el-button :icon="Folder" size="small" @click="pickFolder">
              {{ t('largeFiles.selectFolder') }}
            </el-button>
          </div>
        </el-form-item>
        <el-form-item :label="t('largeFiles.minSize')">
          <el-input-number
            v-model="minSizeMB"
            :min="1"
            :max="10240"
            :step="50"
            size="default"
          />
          <span class="ml-8">MB</span>
        </el-form-item>
        <el-form-item :label="t('largeFiles.olderThan')">
          <el-input-number
            v-model="olderThanDays"
            :min="0"
            :max="3650"
            :step="30"
          />
          <span class="ml-8">{{ t('largeFiles.days') }}</span>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" :icon="Search" :loading="scanning" :disabled="roots.length === 0" @click="onScan">
            {{ t('common.scan') }}
          </el-button>
          <el-button v-if="scanning" :icon="CircleClose" @click="onCancel">
            {{ t('common.stop') }}
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <ScanProgressBar
      v-if="scanning"
      :title="t('largeFiles.scanning', { path: cleaningStore.currentPath })"
      :progress="cleaningStore.progress"
      :current-path="cleaningStore.currentPath"
      class="mt-16"
    />

    <el-alert
      v-if="finished && files.length === 0"
      :title="t('common.noData')"
      type="info"
      :closable="false"
      show-icon
      class="mt-16"
    />

    <el-alert
      v-if="finished && files.length > 0"
      :title="t('largeFiles.results', { n: files.length })"
      type="success"
      :closable="false"
      show-icon
      class="mt-16"
    >
      <template #default>
        <div class="mt-8 flex gap-8">
          <el-button size="small" type="danger" :loading="cleaning" :disabled="selectedPaths.length === 0" @click="onClean">
            {{ t('common.delete') }} ({{ selectedPaths.length }})
          </el-button>
        </div>
      </template>
    </el-alert>

    <el-table
      v-if="files.length > 0"
      :data="files"
      @selection-change="(rows: LargeFile[]) => (selectedPaths = rows.map((r) => r.path))"
      stripe
      class="mt-16"
    >
      <el-table-column type="selection" width="50" />
      <el-table-column :label="t('common.openInExplorer')" min-width="320">
        <template #default="{ row }">
          <el-tooltip :content="row.path" placement="top">
            <span class="large-files-view__path" @click="reveal(row.path)">{{ row.path }}</span>
          </el-tooltip>
        </template>
      </el-table-column>
      <el-table-column :label="t('common.bytes')" width="140" align="right">
        <template #default="{ row }">
          <SizeText :bytes="row.size" mode="auto" />
        </template>
      </el-table-column>
      <el-table-column label="Last access" width="200">
        <template #default="{ row }">
          {{ new Date(row.last_access * 1000).toLocaleString() }}
        </template>
      </el-table-column>
    </el-table>

    <ConfirmDialog
      v-model="confirmOpen"
      :count="selectedPaths.length"
      :bytes="selectedBytes"
      @confirm="doClean"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { open } from '@tauri-apps/plugin-dialog'
import { Search, Folder, CircleClose } from '@element-plus/icons-vue'
import SizeText from '@/components/SizeText.vue'
import ScanProgressBar from '@/components/ScanProgressBar.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { scanLargeFiles, deletePaths, type LargeFile } from '@/api/largeFiles'
import { cancelScan, revealInExplorer } from '@/api'
import { useCleaningStore } from '@/stores/cleaning'
import { useSettingsStore } from '@/stores/settings'
import { formatSize } from '@/utils/format'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const { t } = useI18n()
const cleaningStore = useCleaningStore()
const settingsStore = useSettingsStore()

const roots = ref<string[]>([])
const minSizeMB = ref(100)
const olderThanDays = ref(180)
const files = ref<LargeFile[]>([])
const selectedPaths = ref<string[]>([])
const scanning = ref(false)
const cleaning = ref(false)
const finished = ref(false)
const confirmOpen = ref(false)

const selectedBytes = computed(() => {
  const sel = new Set(selectedPaths.value)
  return files.value
    .filter((f) => sel.has(f.path))
    .reduce((acc, f) => acc + f.size, 0)
})

let unFile: UnlistenFn | null = null

async function setupListener() {
  if (unFile) return
  unFile = await listen<LargeFile>('large-file-found', (e) => {
    if (cleaningStore.scanKind !== 'large-files') return
    if (files.value.length < 5000) {
      files.value.push(e.payload)
    }
  })
}

async function pickFolder() {
  try {
    const selected = await open({ directory: true, multiple: true })
    if (!selected) return
    const list = Array.isArray(selected) ? selected : [selected]
    for (const p of list) {
      if (!roots.value.includes(p)) roots.value.push(p)
    }
  } catch (e) {
    console.warn(e)
  }
}

async function onScan() {
  if (roots.value.length === 0) return
  files.value = []
  selectedPaths.value = []
  finished.value = false
  cleaningStore.reset()
  await cleaningStore.attach()
  await setupListener()
  try {
    const { scan_id } = await scanLargeFiles(
      roots.value,
      minSizeMB.value * 1024 * 1024,
      olderThanDays.value,
    )
    cleaningStore.beginScan('large-files', scan_id)
  } catch (e: unknown) {
    ElMessage.error(t('errors.scanFailed', { msg: String(e) }))
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
  if (settingsStore.confirmBeforeClean) confirmOpen.value = true
  else doClean()
}

async function doClean() {
  if (selectedPaths.value.length === 0) return
  cleaning.value = true
  try {
    const summary = await deletePaths(selectedPaths.value)
    settingsStore.recordCleanup(summary.total_bytes)
    ElMessage.success(t('common.totalFreed', { n: formatSize(summary.total_bytes) }))
    const sel = new Set(selectedPaths.value)
    files.value = files.value.filter((f) => !sel.has(f.path))
    selectedPaths.value = []
  } catch (e: unknown) {
    ElMessage.error(t('errors.cleanFailed', { msg: String(e) }))
  } finally {
    cleaning.value = false
  }
}

async function reveal(path: string) {
  try {
    await revealInExplorer(path)
  } catch (e) {
    console.warn(e)
  }
}

// 监听 scanning 状态,用于在 UI 显示进度与结束
const stopWatch = (() => {
  return import('vue').then(({ watch }) => {
    return watch(
      () => cleaningStore.scanning,
      (v) => {
        scanning.value = v
        if (!v && cleaningStore.scanKind === 'large-files') {
          finished.value = true
        }
      },
    )
  })
})()

onMounted(async () => {
  await cleaningStore.attach()
})

onUnmounted(async () => {
  const stop = await stopWatch
  stop()
  if (unFile) {
    unFile()
    unFile = null
  }
  await cleaningStore.detach()
})
</script>

<style lang="scss" scoped>
.large-files-view {
  &__filters {
    margin-bottom: 16px;
  }
  &__path {
    font-family: 'Consolas', 'Courier New', monospace;
    font-size: 12px;
    cursor: pointer;
    color: var(--el-color-primary);
    &:hover {
      text-decoration: underline;
    }
  }
}
.view-header {
  margin-bottom: 16px;
  h1 {
    font-size: 20px;
    margin: 0 0 4px;
  }
  p {
    margin: 0;
    font-size: 13px;
  }
}
</style>
