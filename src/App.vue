<template>
  <el-config-provider :locale="elLocale">
    <el-container class="app-container">
      <el-header class="app-header" height="56px">
        <AppHeader />
      </el-header>
      <el-container class="app-body">
        <el-aside class="app-aside" width="220px">
          <AppSidebar />
        </el-aside>
        <el-main class="app-main">
          <router-view v-slot="{ Component }">
            <transition name="fade" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </el-main>
      </el-container>
    </el-container>
  </el-config-provider>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ElConfigProvider } from 'element-plus'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import enUs from 'element-plus/es/locale/lang/en'
import AppHeader from '@/components/AppHeader.vue'
import AppSidebar from '@/components/AppSidebar.vue'
import { useI18n } from 'vue-i18n'

const { locale } = useI18n()
const elLocale = computed(() => (locale.value === 'zh-CN' ? zhCn : enUs))
</script>

<style lang="scss">
/* 整个应用 = 100vh 视口,内部不再允许整体滚动 */
.app-container {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

/* 嵌套的 el-container 需要填满 header 之外的高度 */
.app-body {
  flex: 1 1 auto;
  min-height: 0;          /* 关键:flex 子项需要 min-height:0 才能让子元素溢出滚动 */
  overflow: hidden;
}

.app-header {
  border-bottom: 1px solid var(--el-border-color-lighter);
  padding: 0;
  background: var(--el-bg-color);
  flex-shrink: 0;          /* 防止 header 被压缩 */
}

.app-aside {
  background: var(--el-bg-color);
  border-right: 1px solid var(--el-border-color-lighter);
  overflow: hidden;        /* 侧边栏不滚动,菜单项不多 */
  flex-shrink: 0;
}

.app-main {
  background: var(--el-bg-color-page);
  padding: 24px;
  overflow-y: auto;        /* 只有主区允许垂直滚动 */
  overflow-x: hidden;
  min-height: 0;           /* 关键:让 overflow-y 在 flex 中生效 */
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
