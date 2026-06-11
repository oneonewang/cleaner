<template>
  <el-menu
    :default-active="route.path"
    class="app-sidebar"
    background-color="transparent"
    text-color="var(--el-text-color-regular)"
    active-text-color="var(--ooc-primary)"
    router
  >
    <el-menu-item
      v-for="r in menuItems"
      :key="r.path"
      :index="r.path"
    >
      <el-icon><component :is="r.icon" /></el-icon>
      <span>{{ t(r.titleKey) }}</span>
    </el-menu-item>
  </el-menu>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { computed } from 'vue'

const route = useRoute()
const { t } = useI18n()

const menuItems = computed(() => [
  { path: '/', titleKey: 'nav.home', icon: 'Odometer' },
  { path: '/system-junk', titleKey: 'nav.system', icon: 'Delete' },
  { path: '/browser-cache', titleKey: 'nav.browser', icon: 'ChromeFilled' },
  { path: '/large-files', titleKey: 'nav.large', icon: 'Files' },
  { path: '/registry', titleKey: 'nav.registry', icon: 'Setting' },
])
</script>

<style lang="scss" scoped>
.app-sidebar {
  border-right: none;
  height: 100%;
  padding-top: 8px;
  :deep(.el-menu-item) {
    margin: 4px 12px;
    border-radius: 6px;
    height: 44px;
    line-height: 44px;
    &.is-active {
      background: var(--ooc-primary-soft);
      font-weight: 500;
    }
    &:hover {
      background: var(--el-fill-color-light);
    }
  }
}
</style>
