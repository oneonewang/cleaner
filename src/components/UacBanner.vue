<template>
  <transition name="uac-banner">
    <div v-if="!settingsStore.isAdmin" class="uac-banner">
      <el-icon class="uac-banner__icon"><Lock /></el-icon>
      <div class="uac-banner__text">
        <div class="uac-banner__title">{{ t('uac.title') }}</div>
        <div class="uac-banner__desc">{{ t('uac.desc') }}</div>
      </div>
      <el-button
        type="primary"
        size="small"
        :icon="TopRight"
        :loading="elevating"
        @click="onElevate"
      >
        {{ t('uac.elevate') }}
      </el-button>
      <el-button text size="small" :icon="Close" circle @click="dismissed = true" />
    </div>
  </transition>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Lock, TopRight, Close } from '@element-plus/icons-vue'
import { useSettingsStore } from '@/stores/settings'
import { relaunchAsAdmin } from '@/api'

const { t } = useI18n()
const settingsStore = useSettingsStore()
const elevating = ref(false)
const dismissed = ref(false)

async function onElevate() {
  try {
    await ElMessageBox.confirm(
      t('uac.confirmDesc'),
      t('uac.confirmTitle'),
      {
        type: 'warning',
        confirmButtonText: t('uac.elevate'),
        cancelButtonText: t('common.cancel'),
      },
    )
  } catch {
    return
  }
  elevating.value = true
  try {
    await relaunchAsAdmin()
    // 如果能到这里,说明提权未真正启动,Rust 端已 exit,前端也会随窗口关闭而结束
  } catch (e) {
    elevating.value = false
    ElMessage.error(t('uac.failed', { msg: String(e) }))
  }
}

onMounted(() => {
  // 首次挂载时主动检测一次(在 store init 之后再调用)
  if (settingsStore.isAdmin === false) {
    settingsStore.loadIsAdmin()
  }
})
</script>

<style lang="scss" scoped>
.uac-banner {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px;
  background: linear-gradient(90deg, #fff7e6 0%, #fffbe6 100%);
  border-bottom: 1px solid #ffe58f;
  color: #874d00;
  font-size: 13px;
  &__icon {
    font-size: 18px;
    color: #fa8c16;
  }
  &__text {
    flex: 1;
    line-height: 1.4;
  }
  &__title {
    font-weight: 600;
  }
  &__desc {
    color: #ad6800;
    font-size: 12px;
  }
  :deep(.el-button.is-circle) {
    color: #874d00;
  }
}

html.dark .uac-banner {
  background: linear-gradient(90deg, #2a1f10 0%, #1a1a1a 100%);
  border-bottom-color: #5a3a14;
  color: #ffd591;
  &__desc { color: #d4b896; }
  :deep(.el-button.is-circle) { color: #ffd591; }
}

.uac-banner-enter-active,
.uac-banner-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.uac-banner-enter-from,
.uac-banner-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
