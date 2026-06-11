import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN'
import enUS from './en-US'

export const SUPPORTED_LOCALES = ['zh-CN', 'en-US'] as const
export type LocaleCode = (typeof SUPPORTED_LOCALES)[number]

const stored = (typeof localStorage !== 'undefined' && localStorage.getItem('locale')) as LocaleCode | null
const initial: LocaleCode = stored && SUPPORTED_LOCALES.includes(stored) ? stored : 'zh-CN'

export const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: initial,
  fallbackLocale: 'en-US',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
})

export function setLocale(loc: LocaleCode) {
  i18n.global.locale.value = loc
  localStorage.setItem('locale', loc)
  document.documentElement.lang = loc
}
