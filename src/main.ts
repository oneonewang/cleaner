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

// Element Plus 主题随 i18n locale 切换
const setupElementLocale = () => {
  const locale = i18n.global.locale.value
  // 通过自定义 provide 在 App 中读取,这里只设置全局 locale
  document.documentElement.lang = locale
}
app.use(ElementPlus, { locale: i18n.global.locale.value === 'zh-CN' ? zhCn : enUs })

setupElementLocale()
// 监听语言变化更新 document lang 与 el-locale
i18n.global.locale.value
app.mount('#app')

export default app
