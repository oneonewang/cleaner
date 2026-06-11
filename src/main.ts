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

// 挂载后立即隐藏加载占位
requestAnimationFrame(() => {
  const loading = document.getElementById('app-loading')
  if (loading) {
    loading.classList.add('is-hiding')
    setTimeout(() => loading.remove(), 380)
  }
})

// 通知 Rust 显示主窗口(之前在 tauri.conf.json 中 visible: false,
// 是为了等 WebView2 启动白屏结束、HTML 加载占位也出现后,再一次性显示窗口)
import('@tauri-apps/api/core').then(({ invoke }) => {
  // 给浏览器一次绘制后再触发,确保 loading 占位已渲染
  setTimeout(() => {
    invoke('show_main_window').catch((e) => console.warn('show_main_window failed', e))
  }, 16)
})

export default app
