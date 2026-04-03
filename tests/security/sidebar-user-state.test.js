import test from 'node:test'
import assert from 'node:assert/strict'

import {
  createAuthSessionStore,
  createMemoryStorage,
} from '../../src/modules/security/index.js'
import { resolveSidebarUserState } from '../../src/modules/security/sidebar-user-state.js'

test('未登录状态展示登录注册入口与试用提示', () => {
  const state = resolveSidebarUserState({
    isAuthenticated: false,
    isGuest: false,
    userInfo: null,
  })

  assert.equal(state.mode, 'anonymous')
  assert.equal(state.showAuthActions, true)
  assert.equal(state.showSensitiveActions, false)
  assert.match(state.trialHint, /注册即送/)
})

test('登录状态展示昵称、在线状态和次数进度', () => {
  const state = resolveSidebarUserState({
    isAuthenticated: true,
    isGuest: false,
    userInfo: {
      nickName: '健康用户',
      remainingTimes: 7,
      totalTimes: 20,
    },
  })

  assert.equal(state.mode, 'authenticated')
  assert.equal(state.displayName, '健康用户')
  assert.equal(state.subtitle, '在线')
  assert.equal(state.usage.remaining, 7)
  assert.equal(state.usage.total, 20)
  assert.equal(state.usage.percent, 35)
  assert.equal(state.showSensitiveActions, true)
})

test('重启后可从本地会话恢复用户区登录展示', () => {
  const storage = createMemoryStorage()
  const sessionStore = createAuthSessionStore(storage)
  sessionStore.setAuthenticatedSession({
    accessToken: 'access-restored',
    refreshToken: 'refresh-restored',
    userInfo: {
      userName: 'restore-user',
      remainingTimes: 5,
      totalTimes: 20,
    },
  })

  const restored = createAuthSessionStore(storage).getSessionState()
  const state = resolveSidebarUserState(restored)

  assert.equal(state.mode, 'authenticated')
  assert.equal(state.displayName, 'restore-user')
  assert.equal(state.usage.remaining, 5)
})

test('访客态不展示账号敏感入口', () => {
  const state = resolveSidebarUserState({
    isAuthenticated: false,
    isGuest: true,
    userInfo: null,
  })

  assert.equal(state.mode, 'guest')
  assert.equal(state.showSensitiveActions, false)
  assert.equal(state.quickActions.length, 0)
  assert.match(state.guestHint, /访客/)
})

test('用户信息缺失时使用兜底昵称与头像首字母', () => {
  const state = resolveSidebarUserState({
    isAuthenticated: true,
    isGuest: false,
    userInfo: {},
  })

  assert.equal(state.displayName, '健康用户')
  assert.equal(state.avatarText, '健')
})
