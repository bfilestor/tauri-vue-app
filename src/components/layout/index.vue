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
      <div class="p-4 border-t border-slate-200 space-y-3">
        <div v-if="userAreaState.mode === 'authenticated'" class="rounded-xl border border-slate-200 bg-slate-50 p-3">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-full bg-[#2b8cee]/15 flex items-center justify-center text-[#2b8cee] text-sm font-bold">
              {{ userAreaState.avatarText }}
            </div>
            <div class="min-w-0">
              <p class="text-sm font-bold text-slate-800 truncate">{{ userAreaState.displayName }}</p>
              <p class="text-xs text-slate-400 flex items-center gap-1">
                <span class="inline-block w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
                {{ userAreaState.subtitle }}
              </p>
            </div>
          </div>
          <div class="mt-3">
            <div class="flex items-center justify-between text-[11px] text-slate-500 mb-1">
              <span>剩余次数</span>
              <span class="font-semibold text-[#2b8cee]">{{ userAreaState.usage.label }}</span>
            </div>
            <div class="h-1.5 w-full rounded-full bg-slate-200 overflow-hidden">
              <div class="h-full rounded-full bg-gradient-to-r from-[#2b8cee] to-cyan-400" :style="{ width: `${userAreaState.usage.percent}%` }"></div>
            </div>
          </div>
          <p v-if="usageFeedback.stale" class="text-[11px] text-amber-600 mt-2">
            次数刷新失败，当前显示缓存数据
          </p>
          <div class="grid grid-cols-3 gap-2 mt-3">
            <button class="text-[11px] rounded-md border border-slate-200 bg-white px-1 py-1.5 text-slate-600 hover:border-[#2b8cee] hover:text-[#2b8cee] transition-colors" @click="handlePurchaseClick">
              购买次数
            </button>
            <button class="text-[11px] rounded-md border border-slate-200 bg-white px-1 py-1.5 text-slate-600 hover:border-[#2b8cee] hover:text-[#2b8cee] transition-colors" @click="handleAccountMenu">
              账号菜单
            </button>
            <button class="text-[11px] rounded-md border border-red-100 bg-red-50 px-1 py-1.5 text-red-500 hover:bg-red-100 transition-colors" @click="handleLogout">
              退出登录
            </button>
          </div>
        </div>
        <div v-else class="space-y-2">
          <button
            @click="openAuthDialog(AUTH_DIALOG_TABS.login)"
            class="w-full flex items-center justify-center gap-2 px-3 py-2 text-sm font-semibold text-white bg-gradient-to-r from-[#2196f3] to-[#1565c0] rounded-lg hover:opacity-95 transition-opacity"
          >
            <span class="material-symbols-outlined text-lg">person</span>
            登录账号
          </button>
          <button
            @click="openAuthDialog(AUTH_DIALOG_TABS.register)"
            class="w-full flex items-center justify-center gap-2 px-3 py-2 text-sm font-semibold text-[#2196f3] bg-white border border-slate-200 rounded-lg hover:border-[#2196f3] hover:bg-blue-50 transition-colors"
          >
            <span class="material-symbols-outlined text-lg">auto_awesome</span>
            免费注册
          </button>
          <p class="text-[11px] text-center text-slate-400">
            {{ userAreaState.mode === 'guest' ? userAreaState.guestHint : userAreaState.trialHint }}
          </p>
        </div>
        <button
          @click="handleQuit"
          class="w-full flex items-center justify-center gap-2 px-4 py-2 text-sm text-red-500 bg-red-50 hover:bg-red-200 hover:text-red-600 rounded-lg transition-colors"
        >
          <span class="material-symbols-outlined text-xl">logout</span>
          退出程序
        </button>
      </div>
    </aside>

    <!-- 主内容区 -->
    <main class="flex-1 ml-64 h-full flex flex-col bg-slate-50">
      <!-- 窗口控制栏 -->
      <header class="h-10 flex items-center justify-between px-4 select-none shrink-0" data-tauri-drag-region>
        <div class="flex-1 h-full" data-tauri-drag-region></div>
        <div class="flex items-center gap-1 relative z-50">
          <template v-if="showHeaderAuthButtons">
            <button
              @click="openAuthDialog(AUTH_DIALOG_TABS.login)"
              class="px-3 py-1.5 rounded-lg border border-slate-300 text-xs text-slate-700 hover:bg-slate-100 transition-colors"
              title="登录账号"
            >
              登录
            </button>
            <button
              @click="openAuthDialog(AUTH_DIALOG_TABS.register)"
              class="px-3 py-1.5 rounded-lg bg-[#2b8cee] text-xs text-white hover:bg-[#1f7dd9] transition-colors"
              title="注册账号"
            >
              注册
            </button>
          </template>
          <div v-else class="px-2 py-1 rounded-lg border border-slate-200 bg-white text-xs text-slate-600">
            {{ userAreaState.displayName }}
          </div>
          <button 
            @click="appWindow.minimize()"
            class="p-2 rounded-lg hover:bg-slate-200 text-slate-600 transition-colors flex items-center justify-center w-8 h-8"
            title="最小化"
          >
            <span class="material-symbols-outlined text-lg">remove</span>
          </button>
          <button 
            @click="appWindow.toggleMaximize()"
            class="p-2 rounded-lg hover:bg-slate-200 text-slate-600 transition-colors flex items-center justify-center w-8 h-8"
            title="最大化/还原"
          >
            <span class="material-symbols-outlined text-lg">crop_square</span>
          </button>
          <button 
            @click="appWindow.close()"
            class="p-2 rounded-lg hover:bg-red-500 hover:text-white text-slate-600 transition-colors flex items-center justify-center w-8 h-8"
            title="关闭"
          >
            <span class="material-symbols-outlined text-lg">close</span>
          </button>
        </div>
      </header>

      <!-- 内容滚动区 -->
      <div class="flex-1 overflow-y-auto w-full">
        <div class="p-8 max-w-6xl mx-auto">
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
      </div>
    </main>
  </div>
  <auth-entry-dialog
    v-model="authDialogVisible"
    :default-tab="authDialogTab"
    @auth-success="handleAuthSuccess"
    @guest-entered="handleGuestEntered"
  />
</template>

<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { ElMessage, ElMessageBox } from 'element-plus'
import AuthEntryDialog from '@/components/auth/AuthEntryDialog.vue'
import { AUTH_DIALOG_TABS } from '@/modules/security/auth-dialog-controller.js'
import {
  buildAccountMenuEntries,
  getAuthApi,
  resolveSidebarUserState,
  resolveUsageFeedback,
  useAccountContext,
} from '@/modules/security/index.js'

const route = useRoute()
const appWindow = getCurrentWindow()
const authApi = getAuthApi()
const { state: accountContextState, refresh: refreshAccountContext } = useAccountContext()
const authDialogVisible = ref(false)
const authDialogTab = ref(AUTH_DIALOG_TABS.login)
const sessionState = ref(authApi.getSessionState())
const latestUsageFeedback = ref(resolveUsageFeedback(accountContextState))
const usageFeedback = computed(() => {
  const usage = resolveUsageFeedback(accountContextState, latestUsageFeedback.value)
  if (!usage.stale) {
    latestUsageFeedback.value = usage
  }
  return usage
})
const userAreaState = computed(() => resolveSidebarUserState({
  ...sessionState.value,
  userInfo: {
    ...(sessionState.value?.userInfo || {}),
    remainingTimes: usageFeedback.value.remaining,
    totalTimes: usageFeedback.value.total,
  },
}))
const accountMenuEntries = computed(() => buildAccountMenuEntries(userAreaState.value.mode === 'authenticated'))
const showHeaderAuthButtons = computed(() => userAreaState.value.mode !== 'authenticated')

const menuItems = [
  { path: '/upload', title: '数据上传', icon: 'cloud_upload' },
  { path: '/desensitize', title: '数据脱敏', icon: 'healing' },
  { path: '/trends', title: '趋势分析', icon: 'insights' },
  { path: '/history', title: '历史记录', icon: 'history' },
  { path: '/aiqa', title: 'AI 问答', icon: 'smart_toy' },
  { path: '/settings', title: '系统设置', icon: 'settings' },
]

const isActive = (path) => {
  return route.path === path
}

const openAuthDialog = (tab = AUTH_DIALOG_TABS.login) => {
  authDialogTab.value = tab
  authDialogVisible.value = true
}

const refreshSessionState = () => {
  sessionState.value = authApi.getSessionState()
}

const refreshAccountContextWithFeedback = async (options = {}) => {
  try {
    await refreshAccountContext(options)
  } catch (error) {
    const message = error?.message || '账户上下文加载失败'
    ElMessage.error(message)
  }
}

const handleAuthSuccess = () => {
  authDialogVisible.value = false
  refreshSessionState()
  void refreshAccountContextWithFeedback({ force: true })
}

const handleGuestEntered = () => {
  authDialogVisible.value = false
  refreshSessionState()
  void refreshAccountContextWithFeedback({ force: true })
}

const handlePurchaseClick = () => {
  ElMessage.info('购买次数入口将在 E2-S2-I1 接入')
}

const handleAccountMenu = () => {
  const labels = accountMenuEntries.value.map((item) => item.label).join(' / ')
  ElMessage.info(`账号快捷入口：${labels}`)
}

const handleLogout = async () => {
  try {
    await authApi.logout()
    refreshSessionState()
    await refreshAccountContextWithFeedback({ force: true })
    ElMessage.success('已退出登录')
  } catch (error) {
    ElMessage.error(error?.message || '退出登录失败')
  }
}

const handleQuit = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要退出程序吗？',
      '退出确认',
      {
        confirmButtonText: '退出',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
    await invoke('quit')
  } catch (e) {
    // 用户取消退出
  }
}

onMounted(() => {
  refreshSessionState()
  void refreshAccountContextWithFeedback({ force: true })
})
</script>

<style scoped>
</style>
