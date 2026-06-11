<template>
  <div class="app-header">
    <div class="app-header__brand">
      <el-icon class="app-header__logo"><BrushFilled /></el-icon>
      <span class="app-header__title">{{ t('app.title') }}</span>
      <span class="app-header__tagline text-muted">{{ t('app.tagline') }}</span>
    </div>
    <div class="app-header__actions">
      <el-tooltip :content="t('settings.theme')" placement="bottom">
        <el-button text :icon="theme === 'dark' ? Moon : Sunny" circle @click="toggleTheme" />
      </el-tooltip>
      <el-dropdown @command="onLocale">
        <el-button text circle>
          <el-icon><Position /></el-icon>
        </el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item command="zh-CN" :disabled="locale === 'zh-CN'">简体中文</el-dropdown-item>
            <el-dropdown-item command="en-US" :disabled="locale === 'en-US'">English</el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
      <el-tooltip :content="t('nav.settings')" placement="bottom">
        <el-button text :icon="Setting" circle @click="settingsOpen = true" />
      </el-tooltip>
    </div>

    <el-drawer v-model="settingsOpen" :title="t('settings.title')" direction="rtl" size="380px">
      <el-form label-position="top">
        <el-divider content-position="left">{{ t('settings.appearance') }}</el-divider>
        <el-form-item :label="t('settings.theme')">
          <el-radio-group :model-value="theme" @change="changeTheme">
            <el-radio-button value="light">{{ t('settings.light') }}</el-radio-button>
            <el-radio-button value="dark">{{ t('settings.dark') }}</el-radio-button>
          </el-radio-group>
        </el-form-item>
        <el-form-item :label="t('settings.language')">
          <el-radio-group :model-value="locale" @change="changeLocale">
            <el-radio-button value="zh-CN">简体中文</el-radio-button>
            <el-radio-button value="en-US">English</el-radio-button>
          </el-radio-group>
        </el-form-item>

        <el-divider content-position="left">{{ t('settings.cleaning') }}</el-divider>
        <el-form-item>
          <template #label>
            <span>{{ t('settings.sendToTrash') }}</span>
            <div class="text-muted" style="font-size: 12px">
              {{ t('settings.sendToTrashDesc') }}
            </div>
          </template>
          <el-switch v-model="toTrash" />
        </el-form-item>
        <el-form-item :label="t('settings.confirmBeforeClean')">
          <el-switch v-model="confirmBeforeClean" />
        </el-form-item>
      </el-form>
    </el-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { storeToRefs } from 'pinia'
import { Setting, Position, Moon, Sunny, BrushFilled } from '@element-plus/icons-vue'
import { useSettingsStore } from '@/stores/settings'

const { t, locale: i18nLocale } = useI18n()
const store = useSettingsStore()
const { theme, locale, toTrash, confirmBeforeClean } = storeToRefs(store)
const settingsOpen = ref(false)

function toggleTheme() {
  store.setTheme(theme.value === 'dark' ? 'light' : 'dark')
}

function onLocale(loc: string) {
  store.setLocale(loc as 'zh-CN' | 'en-US')
  i18nLocale.value = loc as 'zh-CN' | 'en-US'
}

function changeTheme(v: string | number | boolean | undefined) {
  if (v === undefined) return
  store.setTheme(v as 'light' | 'dark')
}
function changeLocale(v: string | number | boolean | undefined) {
  if (v === undefined) return
  store.setLocale(v as 'zh-CN' | 'en-US')
  i18nLocale.value = v as 'zh-CN' | 'en-US'
}

onMounted(() => {
  store.loadFromStorage()
  i18nLocale.value = store.locale
})
</script>

<style lang="scss" scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  padding: 0 20px;
  &__brand {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  &__logo {
    font-size: 24px;
    color: var(--ooc-primary);
  }
  &__title {
    font-size: 18px;
    font-weight: 600;
  }
  &__tagline {
    margin-left: 8px;
    font-size: 12px;
  }
  &__actions {
    display: flex;
    gap: 4px;
  }
}
</style>
