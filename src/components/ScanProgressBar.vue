<template>
  <div class="scan-progress">
    <div class="scan-progress__head">
      <span class="scan-progress__title">{{ title }}</span>
      <span class="scan-progress__path text-muted" :title="currentPath">{{ currentPath }}</span>
    </div>
    <el-progress
      :percentage="Math.round(progress)"
      :stroke-width="14"
      :show-text="true"
      :status="status"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  title: string
  progress: number
  currentPath?: string
  done?: boolean
  error?: boolean
}>()

const status = computed<'success' | 'exception' | undefined>(() => {
  if (props.error) return 'exception'
  if (props.done) return 'success'
  return undefined
})
</script>

<style lang="scss" scoped>
.scan-progress {
  &__head {
    display: flex;
    justify-content: space-between;
    margin-bottom: 8px;
    font-size: 13px;
  }
  &__title {
    font-weight: 500;
  }
  &__path {
    max-width: 60%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: 'Consolas', 'Courier New', monospace;
  }
}
</style>
