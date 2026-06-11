<template>
  <el-tooltip :content="tooltip" placement="top">
    <span class="size-text" :class="textClass">{{ display }}</span>
  </el-tooltip>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  bytes: number
  /** "auto" 自动选择单位;"raw" 始终字节;"short" 强制最大两位小数 */
  mode?: 'auto' | 'raw' | 'short'
  digits?: number
  textClass?: string
}>()

const UNITS = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']

function format(bytes: number, mode: 'auto' | 'raw' | 'short'): { display: string; full: string } {
  if (!Number.isFinite(bytes) || bytes < 0) return { display: '0 B', full: '0 B' }
  if (mode === 'raw') {
    return { display: `${bytes} B`, full: `${bytes} B` }
  }
  if (bytes === 0) return { display: '0 B', full: '0 B' }
  const exp = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), UNITS.length - 1)
  const value = bytes / Math.pow(1024, exp)
  const digits = props.digits ?? (mode === 'short' ? 2 : value >= 100 ? 0 : value >= 10 ? 1 : 2)
  const display = `${value.toFixed(digits)} ${UNITS[exp]}`
  return { display, full: `${bytes.toLocaleString('en-US')} B (${display})` }
}

const formatted = computed(() => format(props.bytes, props.mode ?? 'auto'))
const display = computed(() => formatted.value.display)
const tooltip = computed(() => formatted.value.full)
</script>

<style lang="scss" scoped>
.size-text {
  font-variant-numeric: tabular-nums;
}
</style>
