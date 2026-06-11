import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import enUs from 'element-plus/es/locale/lang/en'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'

import App from './App.vue'
import { router } from './router'
import { i18n } from './locales'
import './styles/index.scss'

const app = createApp(App)

// 注册所有 Element Plus 图标
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component as never)
}

app.use(createPinia())
app.use(router)
app.use(i18n)

app.use(ElementPlus, { locale: i18n.global.locale.value === 'zh-CN' ? zhCn : enUs })

document.documentElement.lang = i18n.global.locale.value
app.mount('#app')

// 挂载后立即隐藏加载占位(避免阻塞主线程,等下一帧再移除)
requestAnimationFrame(() => {
  const loading = document.getElementById('app-loading')
  if (loading) {
    loading.classList.add('is-hiding')
    setTimeout(() => loading.remove(), 380)
  }
})

export default app
