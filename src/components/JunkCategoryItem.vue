<template>
  <el-card class="junk-category-item" shadow="hover">
    <div class="junk-category-item__head">
      <el-checkbox
        :model-value="checked"
        :indeterminate="indeterminate"
        @change="onToggle"
        :disabled="disabled"
      >
        <span class="junk-category-item__name">
          <el-icon v-if="icon" class="junk-category-item__icon"><component :is="icon" /></el-icon>
          {{ name }}
        </span>
      </el-checkbox>
      <div class="junk-category-item__stats">
        <SizeText :bytes="total_bytes" mode="auto" class="junk-category-item__size" />
        <span class="text-muted">({{ file_count }} {{ t('common.files') }})</span>
      </div>
    </div>
    <div v-if="description" class="junk-category-item__desc text-muted">{{ description }}</div>
    <el-collapse-transition>
      <div v-show="expanded" class="junk-category-item__files">
        <el-table :data="files" max-height="280" size="small" stripe>
          <el-table-column :label="t('common.openInExplorer')" min-width="200">
            <template #default="{ row }">
              <el-tooltip :content="row.path" placement="top">
                <span class="junk-category-item__path" @click="reveal(row.path)">{{ row.path }}</span>
              </el-tooltip>
            </template>
          </el-table-column>
          <el-table-column :label="t('common.bytes')" width="120" align="right">
            <template #default="{ row }">
              <SizeText :bytes="row.size" mode="short" />
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-collapse-transition>
    <div class="junk-category-item__foot">
      <el-button text size="small" @click="expanded = !expanded">
        {{ expanded ? t('common.collapseAll') : t('common.expandAll') }}
      </el-button>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SizeText from './SizeText.vue'
import { revealInExplorer } from '@/api'
import type { JunkFile } from '@/api/systemJunk'

const props = defineProps<{
  id: string
  name: string
  description?: string
  total_bytes: number
  file_count: number
  files: JunkFile[]
  checked: boolean
  indeterminate?: boolean
  disabled?: boolean
  icon?: string
}>()

const emit = defineEmits<{ 'update:checked': [val: boolean] }>()

const { t } = useI18n()
const expanded = ref(false)

const checked = computed(() => props.checked)
const indeterminate = computed(() => props.indeterminate ?? false)

function onToggle(val: string | number | boolean) {
  emit('update:checked', Boolean(val))
}

async function reveal(path: string) {
  try {
    await revealInExplorer(path)
  } catch (e) {
    console.warn('reveal failed', e)
  }
}
</script>

<style lang="scss" scoped>
.junk-category-item {
  &__head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  &__name {
    font-weight: 500;
    margin-left: 8px;
  }
  &__icon {
    margin-right: 4px;
    color: var(--ooc-primary);
  }
  &__stats {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  &__size {
    font-weight: 600;
    color: var(--ooc-primary);
    font-size: 16px;
  }
  &__desc {
    margin: 8px 0;
    font-size: 13px;
  }
  &__files {
    margin-top: 8px;
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
  &__foot {
    margin-top: 8px;
    text-align: right;
  }
}
</style>
