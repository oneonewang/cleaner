<template>
  <el-dialog
    v-model="visible"
    :title="title ?? t('common.confirmDelete')"
    width="480px"
    :close-on-click-modal="false"
    align-center
  >
    <div class="confirm-dialog">
      <p>{{ description }}</p>
      <div class="confirm-dialog__stats">
        <div class="confirm-dialog__stat">
          <div class="confirm-dialog__stat-label">{{ t('common.files') }}</div>
          <div class="confirm-dialog__stat-value">{{ count }}</div>
        </div>
        <div class="confirm-dialog__stat">
          <div class="confirm-dialog__stat-label">{{ t('common.bytes') }}</div>
          <div class="confirm-dialog__stat-value">{{ displaySize }}</div>
        </div>
      </div>
      <el-alert
        v-if="permanent"
        :title="t('errors.permissionDenied')"
        type="warning"
        :closable="false"
        show-icon
      />
    </div>
    <template #footer>
      <el-button @click="onCancel">{{ t('common.cancel') }}</el-button>
      <el-button :type="permanent ? 'danger' : 'primary'" @click="onConfirm">
        {{ t('common.confirm') }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { formatSize } from '@/utils/format'

const props = defineProps<{
  modelValue: boolean
  count: number
  bytes: number
  permanent: boolean
  title?: string
  description?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [val: boolean]
  confirm: []
  cancel: []
}>()

const { t } = useI18n()
const visible = ref(props.modelValue)

watch(
  () => props.modelValue,
  (v) => (visible.value = v),
)
watch(visible, (v) => emit('update:modelValue', v))

const displaySize = computed(() => formatSize(props.bytes))
const desc = computed(
  () =>
    props.description ??
    t('common.confirmDeleteDesc', {
      count: props.count,
      size: displaySize.value,
      action: props.permanent ? t('common.action.permanent') : t('common.action.toTrash'),
    }),
)

function onConfirm() {
  visible.value = false
  emit('confirm')
}
function onCancel() {
  visible.value = false
  emit('cancel')
}
</script>

<style lang="scss" scoped>
.confirm-dialog {
  &__stats {
    display: flex;
    gap: 32px;
    margin: 16px 0;
  }
  &__stat {
    flex: 1;
  }
  &__stat-label {
    font-size: 13px;
    color: var(--el-text-color-secondary);
  }
  &__stat-value {
    font-size: 20px;
    font-weight: 600;
    color: var(--ooc-primary);
    font-variant-numeric: tabular-nums;
  }
}
</style>
