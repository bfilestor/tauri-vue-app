<template>
  <div class="flex h-screen overflow-hidden bg-slate-50 font-sans">
    <!-- 侧边栏 -->
    <aside class="w-64 bg-white border-r border-slate-200 flex flex-col fixed h-full z-50" data-tauri-drag-region>
      <!-- Logo -->
      <div class="p-6 flex items-center gap-3" data-tauri-drag-region>
        <div class="w-10 h-10 bg-[#2b8cee] rounded-xl flex items-center justify-center text-white shadow-lg shadow-[#2b8cee]/30">
          <span class="material-symbols-outlined">health_and_safety</span>
        </div>
        <span class="font-bold text-xl tracking-tight text-slate-900">健康管家</span>
      </div>

      <!-- 导航菜单 -->
      <nav class="flex-1 px-4 space-y-1 mt-2">
        <router-link
          v-for="item in menuItems"
          :key="item.path"
          :to="item.path"
          :class="[
            'flex items-center gap-3 px-4 py-3 rounded-xl transition-all text-sm font-medium',
            isActive(item.path)
              ? 'text-[#2b8cee] bg-[#2b8cee]/5 font-bold'
              : 'text-slate-500 hover:text-[#2b8cee] hover:bg-slate-50'
          ]"
        >
          <span class="material-symbols-outlined text-2xl">{{ item.icon }}</span>
          {{ item.title }}
        </router-link>
      </nav>

      <!-- 底部用户信息 -->
      <div class="p-4 border-t border-slate-200">
        <div class="flex items-center gap-3 px-2 py-2">
          <div class="w-10 h-10 rounded-full bg-[#2b8cee]/10 flex items-center justify-center text-[#2b8cee]">
            <span class="material-symbols-outlined">person</span>
          </div>
          <div>
            <p class="text-sm font-bold text-slate-800">健康用户</p>
            <p class="text-xs text-slate-400">本地管理</p>
          </div>
        </div>
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="flex-1 ml-64 overflow-y-auto">
      <div class="p-8 max-w-6xl">
        <router-view v-slot="{ Component }">
          <keep-alive>
            <component
              :is="Component"
              v-if="$route.meta.keepAlive"
              :key="$route.name"
            />
          </keep-alive>
          <component
            :is="Component"
            v-if="!$route.meta.keepAlive"
            :key="$route.name"
          />
        </router-view>
      </div>
    </main>
  </div>
</template>

<script setup>
import { useRoute } from 'vue-router'

const route = useRoute()

const menuItems = [
  { path: '/upload', title: '数据上传', icon: 'cloud_upload' },
  { path: '/trends', title: '趋势分析', icon: 'insights' },
  { path: '/settings', title: '系统设置', icon: 'settings' },
]

const isActive = (path) => {
  return route.path === path
}
</script>

<style scoped>
</style>
