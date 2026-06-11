<template>
  <div class="registry-view">
    <header class="view-header">
      <div>
        <h1>{{ t('registry.title') }}</h1>
        <p class="text-muted">{{ t('registry.description') }}</p>
      </div>
      <div class="view-header__actions">
        <el-button :icon="Search" type="primary" :loading="scanning" @click="onScan">
          {{ t('common.scan') }}
        </el-button>
        <el-button
          :icon="Document"
          :disabled="issues.length === 0"
          @click="onBackup"
        >
          {{ t('registry.backup') }}
        </el-button>
        <el-button
          :icon="Delete"
          type="danger"
          :disabled="selectedIssues.length === 0"
          :loading="cleaning"
          @click="onClean"
        >
          {{ t('common.clean') }}
        </el-button>
      </div>
    </header>

    <el-alert
      v-if="lastBackupPath"
      :title="t('registry.backed', { path: lastBackupPath })"
      type="success"
      :closable="false"
      show-icon
      class="mt-16 mb-16"
    />

    <el-alert
      v-if="issues.length === 0 && finished"
      :title="t('registry.noIssues')"
      type="success"
      :closable="false"
      show-icon
      class="mt-16 mb-16"
    />

    <el-table
      v-if="issues.length > 0"
      :data="issues"
      @selection-change="(rows: RegistryIssue[]) => (selectedIssues = rows.map((r) => r.id))"
      row-key="id"
      stripe
    >
      <el-table-column type="selection" width="50" />
      <el-table-column :label="t('registry.highRisk')" width="100">
        <template #default="{ row }">
          <el-tag :type="riskTag(row.risk)" size="small">
            {{ t(`registry.${row.risk.toLowerCase()}Risk`) }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column :label="t('registry.scopes.run')" min-width="180">
        <template #default="{ row }">
          {{ t(`registry.scopes.${scopeKey(row.scope)}`) }}
        </template>
      </el-table-column>
      <el-table-column label="Hive" width="80">
        <template #default="{ row }">{{ row.hive }}</template>
      </el-table-column>
      <el-table-column label="Key / Value" min-width="360">
        <template #default="{ row }">
          <div class="registry-view__key">{{ row.key_path }}</div>
          <div v-if="row.value_name" class="text-muted registry-view__value">
            {{ row.value_name }} = {{ row.value_data }}
          </div>
        </template>
      </el-table-column>
      <el-table-column :label="t('common.openInExplorer')" min-width="220">
        <template #default="{ row }">
          <span class="text-muted">{{ row.description }}</span>
        </template>
      </el-table-column>
    </el-table>

    <ConfirmDialog
      v-model="confirmOpen"
      :count="selectedIssues.length"
      :bytes="0"
      :permanent="true"
      :description="t('registry.backupFirst')"
      @confirm="doClean"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search, Delete, Document } from '@element-plus/icons-vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import {
  scanRegistry,
  backupRegistry,
  cleanRegistry,
  type RegistryIssue,
  type RegistryScope,
  type RiskLevel,
} from '@/api/registry'
import { formatSize } from '@/utils/format'

const { t } = useI18n()

const issues = ref<RegistryIssue[]>([])
const selectedIssues = ref<string[]>([])
const scanning = ref(false)
const cleaning = ref(false)
const finished = ref(false)
const lastBackupPath = ref<string | null>(null)
const confirmOpen = ref(false)

function scopeKey(s: RegistryScope) {
  return s.toLowerCase()
}
function riskTag(r: RiskLevel) {
  return r === 'High' ? 'danger' : r === 'Medium' ? 'warning' : 'info'
}

async function onScan() {
  scanning.value = true
  finished.value = false
  try {
    const list = await scanRegistry(['Run', 'RunOnce', 'Uninstall', 'Com'])
    issues.value = list
    finished.value = true
  } catch (e: unknown) {
    ElMessage.error(t('errors.scanFailed', { msg: String(e) }))
  } finally {
    scanning.value = false
  }
}

async function onBackup() {
  try {
    const path = await backupRegistry(issues.value)
    lastBackupPath.value = path
    ElMessage.success(t('registry.backed', { path }))
  } catch (e: unknown) {
    ElMessage.error(t('errors.cleanFailed', { msg: String(e) }))
  }
}

async function onClean() {
  if (selectedIssues.value.length === 0) return
  if (!lastBackupPath.value) {
    ElMessageBox.confirm(t('registry.backupFirst'), t('common.confirm'), {
      confirmButtonText: t('registry.backup'),
      cancelButtonText: t('common.cancel'),
      type: 'warning',
    })
      .then(onBackup)
      .catch(() => {})
    return
  }
  confirmOpen.value = true
}

async function doClean() {
  const sel = new Set(selectedIssues.value)
  const toClean = issues.value.filter((i) => sel.has(i.id))
  if (toClean.length === 0) return
  cleaning.value = true
  try {
    const summary = await cleanRegistry(toClean)
    ElMessage.success(
      `${t('common.clean')} ${toClean.length} ${t('common.items')}` +
        (summary.errors.length ? ` (${summary.errors.length} errors)` : ''),
    )
    issues.value = issues.value.filter((i) => !sel.has(i.id))
    selectedIssues.value = []
  } catch (e: unknown) {
    ElMessage.error(t('errors.cleanFailed', { msg: String(e) }))
  } finally {
    cleaning.value = false
  }
}
</script>

<style lang="scss" scoped>
.registry-view {
  &__key {
    font-family: 'Consolas', 'Courier New', monospace;
    font-size: 12px;
    color: var(--ooc-primary);
  }
  &__value {
    font-family: 'Consolas', 'Courier New', monospace;
    font-size: 12px;
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
