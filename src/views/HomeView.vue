<template>
  <div class="home-view">
    <header class="home-view__hero">
      <h1 class="home-view__title">{{ t('home.welcome') }}</h1>
      <p class="home-view__subtitle text-muted">{{ t('home.subtitle') }}</p>
    </header>

    <section class="home-view__section">
      <h2 class="home-view__heading">{{ t('home.recentCleanups') }}</h2>
      <el-row :gutter="16">
        <el-col :span="8">
          <el-card shadow="hover" class="home-view__stat">
            <div class="home-view__stat-label">{{ t('home.totalFreed') }}</div>
            <div class="home-view__stat-value">{{ totalFreedDisplay }}</div>
          </el-card>
        </el-col>
        <el-col :span="8">
          <el-card shadow="hover" class="home-view__stat">
            <div class="home-view__stat-label">{{ t('home.lastCleanup') }}</div>
            <div class="home-view__stat-value home-view__stat-value--text">
              {{ lastCleanupDisplay }}
            </div>
          </el-card>
        </el-col>
        <el-col :span="8">
          <el-card shadow="hover" class="home-view__stat">
            <div class="home-view__stat-label">{{ t('home.diskUsage') }}</div>
            <div class="home-view__stat-value home-view__stat-value--text">
              {{ appVersion }}
            </div>
          </el-card>
        </el-col>
      </el-row>
    </section>

    <section class="home-view__section">
      <h2 class="home-view__heading">{{ t('home.quickActions') }}</h2>
      <el-row :gutter="16">
        <el-col :span="6" v-for="a in actions" :key="a.path">
          <el-card class="home-view__action" shadow="hover" @click="router.push(a.path)">
            <el-icon class="home-view__action-icon"><component :is="a.icon" /></el-icon>
            <h3>{{ t(a.title) }}</h3>
            <p class="text-muted">{{ t(a.desc) }}</p>
          </el-card>
        </el-col>
      </el-row>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'
import { formatSize, formatDate } from '@/utils/format'

const { t } = useI18n()
const router = useRouter()
const store = useSettingsStore()

const actions = [
  { path: '/system-junk', icon: 'Delete', title: 'home.overview.systemJunk', desc: 'home.overview.systemJunkDesc' },
  { path: '/browser-cache', icon: 'ChromeFilled', title: 'home.overview.browserCache', desc: 'home.overview.browserCacheDesc' },
  { path: '/large-files', icon: 'Files', title: 'home.overview.largeFiles', desc: 'home.overview.largeFilesDesc' },
  { path: '/registry', icon: 'Setting', title: 'home.overview.registry', desc: 'home.overview.registryDesc' },
]

const totalFreedDisplay = computed(() => formatSize(store.totalFreedBytes))
const lastCleanupDisplay = computed(() =>
  store.lastCleanupAt ? formatDate(store.lastCleanupAt) : t('home.neverCleaned'),
)
const appVersion = computed(() => store.appInfo ? `v${store.appInfo.version}` : '—')

onMounted(async () => {
  store.loadFromStorage()
  await store.loadAppInfo()
})
</script>

<style lang="scss" scoped>
.home-view {
  &__hero {
    margin-bottom: 24px;
  }
  &__title {
    font-size: 24px;
    font-weight: 600;
    margin: 0 0 8px;
  }
  &__subtitle {
    margin: 0;
  }
  &__section {
    margin-bottom: 24px;
  }
  &__heading {
    font-size: 16px;
    font-weight: 500;
    margin: 0 0 12px;
    color: var(--el-text-color-regular);
  }
  &__stat {
    text-align: left;
  }
  &__stat-label {
    font-size: 13px;
    color: var(--el-text-color-secondary);
    margin-bottom: 4px;
  }
  &__stat-value {
    font-size: 22px;
    font-weight: 600;
    color: var(--ooc-primary);
    font-variant-numeric: tabular-nums;
    &--text {
      font-size: 16px;
      color: var(--el-text-color-regular);
    }
  }
  &__action {
    cursor: pointer;
    text-align: center;
    transition: transform 0.15s ease, box-shadow 0.15s ease;
    &:hover {
      transform: translateY(-2px);
    }
    h3 {
      font-size: 14px;
      margin: 12px 0 4px;
    }
    p {
      font-size: 12px;
      margin: 0;
    }
  }
  &__action-icon {
    font-size: 32px;
    color: var(--ooc-primary);
  }
}
</style>
