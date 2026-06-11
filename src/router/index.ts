import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/HomeView.vue'),
    meta: { titleKey: 'nav.home', icon: 'Odometer' },
  },
  {
    path: '/system-junk',
    name: 'system-junk',
    component: () => import('@/views/SystemJunkView.vue'),
    meta: { titleKey: 'nav.system', icon: 'Delete' },
  },
  {
    path: '/browser-cache',
    name: 'browser-cache',
    component: () => import('@/views/BrowserCacheView.vue'),
    meta: { titleKey: 'nav.browser', icon: 'ChromeFilled' },
  },
  {
    path: '/large-files',
    name: 'large-files',
    component: () => import('@/views/LargeFilesView.vue'),
    meta: { titleKey: 'nav.large', icon: 'Files' },
  },
  {
    path: '/registry',
    name: 'registry',
    component: () => import('@/views/RegistryView.vue'),
    meta: { titleKey: 'nav.registry', icon: 'Setting' },
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/',
  },
]

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
})
