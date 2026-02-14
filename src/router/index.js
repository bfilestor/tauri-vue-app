import { createRouter, createWebHashHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'Layout',
    component: () => import('@/components/layout/index.vue'),
    redirect: '/upload',
    children: [
      {
        path: '/upload',
        name: 'Upload',
        meta: {
          keepAlive: true,
          title: '数据上传',
          icon: 'cloud_upload',
        },
        component: () => import('@/views/upload/index.vue'),
      },
      {
        path: '/desensitize',
        name: 'Desensitize',
        meta: {
          keepAlive: true,
          title: '数据脱敏',
          icon: 'healing',
        },
        component: () => import('@/views/desensitize/index.vue'),
      },
      {
        path: '/trends',
        name: 'Trends',
        meta: {
          keepAlive: true,
          title: '趋势分析',
          icon: 'insights',
        },
        component: () => import('@/views/trends/index.vue'),
      },
      {
        path: '/history',
        name: 'History',
        meta: {
          keepAlive: true,
          title: '历史记录',
          icon: 'history',
        },
        component: () => import('@/views/history/index.vue'),
      },
      {
        path: '/aiqa',
        name: 'AIQA',
        meta: {
          keepAlive: true,
          title: 'AI 问答',
          icon: 'smart_toy',
        },
        component: () => import('@/views/aiqa/index.vue'),
      },
      {
        path: '/settings',
        name: 'Settings',
        meta: {
          keepAlive: true,
          title: '系统设置',
          icon: 'settings',
        },
        component: () => import('@/views/settings/index.vue'),
      },
    ],
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router
