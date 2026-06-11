<template>
  <div class="system-junk-view">
    <header class="view-header">
      <div>
        <h1>{{ t('systemJunk.title') }}</h1>
        <p class="text-muted">{{ t('systemJunk.description') }}</p>
      </div>
      <div class="view-header__actions">
        <el-button
          v-if="!cleaningStore.scanning"
          type="primary"
          :icon="Search"
          @click="onScan"
          :loading="cleaningStore.scanning"
        >
          {{ t('common.scan') }}
        </el-button>
        <el-button v-else :icon="CircleClose" @click="onCancel">
          {{ t('common.stop') }}
        </el-button>
        <el-button
          type="danger"
          :icon="Delete"
          :disabled="!hasResults || selectedIds.length === 0"
          :loading="cleaningStore.cleaning"
          @click="onClean"
        >
          {{ t('common.clean') }}
        </el-button>
      </div>
    </header>

    <ScanProgressBar
      v-if="cleaningStore.scanning"
      :title="t('common.scanning')"
      :progress="cleaningStore.progress"
      :current-path="cleaningStore.currentPath"
    />

    <div v-if="cleaningStore.errorMsg" class="mt-16">
      <el-alert :title="cleaningStore.errorMsg" type="error" :closable="false" show-icon />
    </div>

    <div v-if="hasResults" class="system-junk-view__summary">
      <el-alert
        type="info"
        :closable="false"
        show-icon
      >
        <template #title>
          {{ t('common.files') }}: {{ cleaningStore.grandCount }} ·
          {{ t('common.totalFreed', { n: formatSize(cleaningStore.grandTotal) }) }}
        </template>
        <div class="mt-8 flex gap-8">
          <el-button size="small" @click="selectAll(true)">{{ t('common.selectAll') }}</el-button>
          <el-button size="small" @click="selectAll(false)">{{ t('common.deselectAll') }}</el-button>
        </div>
      </el-alert>
    </div>

    <div v-if="categories.length > 0" class="system-junk-view__list">
      <JunkCategoryItem
        v-for="cat in categories"
        :key="cat.id"
        :id="cat.id"
        :name="catName(cat.id)"
        :description="cat.description ?? ''"
        :total_bytes="cat.total_bytes"
        :file_count="cat.file_count"
        :files="cat.files"
        :checked="selectedIds.includes(cat.id)"
        @update:checked="(v) => toggleOne(cat.id, v)"
      />
    </div>

    <EmptyState
      v-else-if="!cleaningStore.scanning"
      :description="t('common.noData')"
    />

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
import { Search, Delete, CircleClose } from '@element-plus/icons-vue'
import JunkCategoryItem from '@/components/JunkCategoryItem.vue'
import ScanProgressBar from '@/components/ScanProgressBar.vue'
import EmptyState from '@/components/EmptyState.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { useCleaningStore } from '@/stores/cleaning'
import { useSettingsStore } from '@/stores/settings'
import { listSystemJunkCategories, scanSystemJunk, cleanSystemJunk, type JunkCategoryResult, type CleanItem } from '@/api/systemJunk'
import { cancelScan } from '@/api'
import { formatSize } from '@/utils/format'

const { t, locale } = useI18n()
const cleaningStore = useCleaningStore()
const settingsStore = useSettingsStore()

const categories = ref<JunkCategoryResult[]>([])
const selectedIds = ref<string[]>([])
const confirmOpen = ref(false)

const hasResults = computed(() => categories.value.length > 0)
const selectedCount = computed(() => {
  const sel = new Set(selectedIds.value)
  return categories.value
    .filter((c) => sel.has(c.id))
    .reduce((acc, c) => acc + c.file_count, 0)
})
const selectedBytes = computed(() => {
  const sel = new Set(selectedIds.value)
  return categories.value
    .filter((c) => sel.has(c.id))
    .reduce((acc, c) => acc + c.total_bytes, 0)
})

function catName(id: string) {
  return t(`systemJunk.categories.${id}`, id)
}

function toggleOne(id: string, v: boolean) {
  if (v) {
    if (!selectedIds.value.includes(id)) selectedIds.value.push(id)
  } else {
    selectedIds.value = selectedIds.value.filter((x) => x !== id)
  }
}

function selectAll(v: boolean) {
  selectedIds.value = v ? categories.value.map((c) => c.id) : []
}

async function onScan() {
  cleaningStore.reset()
  categories.value = []
  selectedIds.value = []
  await cleaningStore.attach()
  try {
    const { scan_id } = await scanSystemJunk(null)
    cleaningStore.beginScan('system-junk', scan_id)
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
  if (settingsStore.confirmBeforeClean) {
    confirmOpen.value = true
  } else {
    doClean()
  }
}

async function doClean() {
  const sel = new Set(selectedIds.value)
  const items: CleanItem[] = categories.value
    .filter((c) => sel.has(c.id))
    .map((c) => ({ category: c.id, paths: c.files.map((f) => f.path) }))

  if (items.length === 0) return
  cleaningStore.beginClean()
  try {
    const summary = await cleanSystemJunk(items, settingsStore.toTrash)
    settingsStore.recordCleanup(summary.total_bytes)
    ElMessage.success(
      t('common.totalFreed', { n: formatSize(summary.total_bytes) }),
    )
    // 移除已清理的类别
    const cleanedIds = new Set(items.map((i) => i.category))
    categories.value = categories.value.filter((c) => !cleanedIds.has(c.id))
    selectedIds.value = selectedIds.value.filter((id) => !cleanedIds.has(id))
  } catch (e: unknown) {
    ElMessage.error(t('errors.cleanFailed', { msg: String(e) }))
  } finally {
    cleaningStore.endClean()
  }
}

// 监听 progress 事件,把分类结果同步到 categories
function syncFromStore() {
  // cleaningStore.results 实时更新
  for (const r of cleaningStore.results) {
    const existing = categories.value.find((c) => c.id === r.id)
    if (existing) {
      existing.total_bytes = r.total_bytes
      existing.file_count = r.file_count
      existing.files = r.files
    } else {
      categories.value.push({ ...r })
    }
  }
}

let stopWatch: (() => void) | null = null
let stopProgress: (() => void) | null = null
onMounted(async () => {
  await cleaningStore.attach()
  // 监听 results 变化
  const stop = (await import('vue')).watch(
    () => cleaningStore.results,
    () => syncFromStore(),
    { deep: true },
  )
  stopWatch = stop

  // 监听 scanning 状态变化,完成后拉一次最终结果
  stopProgress = (await import('vue')).watch(
    () => cleaningStore.scanning,
    async (scanning) => {
      if (!scanning && cleaningStore.scanKind === 'system-junk') {
        // 调用 list 拉取最终分类元数据
        try {
          const list = await listSystemJunkCategories()
          // 合并已扫描的数据
          for (const c of list) {
            const existing = categories.value.find((x) => x.id === c.id)
            if (existing) {
              existing.name = c.name
              existing.description = c.description
            } else {
              categories.value.push(c)
            }
          }
          // 应用 i18n
          applyI18nNames()
        } catch (e) {
          console.warn(e)
        }
      }
    },
  )

  // 预拉类别元数据
  try {
    categories.value = await listSystemJunkCategories()
    applyI18nNames()
  } catch (e) {
    console.warn(e)
  }
})

function applyI18nNames() {
  for (const c of categories.value) {
    const localized = t(`systemJunk.categories.${c.id}`, '')
    if (localized) c.name = localized
  }
}

onUnmounted(async () => {
  stopWatch?.()
  stopProgress?.()
  await cleaningStore.detach()
})

// 语言切换时刷新名称
import { watch as vueWatch } from 'vue'
vueWatch(locale, () => applyI18nNames())
</script>

<style lang="scss" scoped>
.system-junk-view {
  &__summary {
    margin: 16px 0;
  }
  &__list {
    display: grid;
    grid-template-columns: 1fr;
    gap: 12px;
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
