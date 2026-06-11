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
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { Search, Delete, ChromeFilled } from '@element-plus/icons-vue'
import SizeText from '@/components/SizeText.vue'
import EmptyState from '@/components/EmptyState.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { detectBrowsers, scanBrowserCache, cleanBrowserCache, type BrowserProfile, type BrowserKind } from '@/api/browserCache'
import { useSettingsStore } from '@/stores/settings'
import { formatSize } from '@/utils/format'

const { t } = useI18n()
const settingsStore = useSettingsStore()

const profiles = ref<BrowserProfile[]>([])
const selectedIds = ref<string[]>([])
const scanning = ref(false)
const cleaning = ref(false)
const confirmOpen = ref(false)

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
function browserIcon(b: BrowserKind) {
  // 暂用 ChromeFilled 代替 Edge / Firefox / Opera 的专属图标
  return ChromeFilled
}

function onSelectionChange(rows: BrowserProfile[]) {
  selectedIds.value = rows.map((r) => r.id)
}

async function onScan() {
  scanning.value = true
  try {
    const list = await detectBrowsers()
    if (list.length === 0) {
      ElMessage.warning(t('common.noData'))
      profiles.value = []
      return
    }
    profiles.value = list
    const scanned = await scanBrowserCache(list.map((p) => p.id))
    // 合并扫描结果
    const map = new Map(scanned.map((s) => [s.id, s]))
    profiles.value = profiles.value.map((p) => {
      const s = map.get(p.id)
      return s ? { ...p, total_bytes: s.total_bytes } : p
    })
  } catch (e: unknown) {
    ElMessage.error(t('errors.scanFailed', { msg: String(e) }))
  } finally {
    scanning.value = false
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
    // 移除已清理
    profiles.value = profiles.value.filter((p) => !sel.has(p.id))
    selectedIds.value = []
  } catch (e: unknown) {
    ElMessage.error(t('errors.cleanFailed', { msg: String(e) }))
  } finally {
    cleaning.value = false
  }
}

onMounted(() => {
  settingsStore.loadFromStorage()
})
</script>

<style lang="scss" scoped>
.browser-cache-view {
  &__icon {
    font-size: 20px;
    color: var(--ooc-primary);
    &.edge {
      color: #0078d4;
    }
    &.firefox {
      color: #ff7139;
    }
    &.opera {
      color: #ff1b2d;
    }
    &.brave {
      color: #fb542b;
    }
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
