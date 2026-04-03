import test from 'node:test'
import assert from 'node:assert/strict'

import {
  SECURITY_STORAGE_KEYS,
  createAuthApi,
  createAuthSessionStore,
  createMemoryStorage,
  createRequestClient,
} from '../../src/modules/security/index.js'

function jsonResponse(payload, status = 200) {
  return new Response(JSON.stringify(payload), {
    status,
    headers: {
      'Content-Type': 'application/json',
    },
  })
}

test('密码登录会持久化会话并支持恢复', async () => {
  const storage = createMemoryStorage()
  const client = createRequestClient({
    storage,
    fetchImpl: async (url) => {
      const pathname = new URL(url).pathname
      assert.equal(pathname, '/app-api/auth/login/password')
      return jsonResponse({
        code: 200,
        data: {
          accessToken: 'access-001',
          refreshToken: 'refresh-001',
          expireIn: 7200,
          userInfo: {
            userId: 1001,
            userName: 'demo-user',
            nickName: 'Demo',
          },
        },
      })
    },
    baseUrl: 'https://api.example.com',
  })
  const authApi = createAuthApi({ client, storage })

  await authApi.loginByPassword({
    userName: 'demo-user',
    password: 'demo-pass',
    deviceId: 'desktop-fixed',
    deviceName: 'Windows Desktop',
  })

  const state = authApi.getSessionState()
  assert.equal(state.accessToken, 'access-001')
  assert.equal(state.refreshToken, 'refresh-001')
  assert.equal(state.userId, 1001)
  assert.equal(state.userInfo?.userId, 1001)
  assert.equal(state.isGuest, false)

  const restored = createAuthSessionStore(storage).getSessionState()
  assert.equal(restored.accessToken, 'access-001')
  assert.equal(restored.refreshToken, 'refresh-001')
  assert.equal(restored.userId, 1001)
  assert.equal(restored.userInfo?.userName, 'demo-user')
})

test('退出登录会清理认证凭证但保留设备身份', async () => {
  const storage = createMemoryStorage({
    [SECURITY_STORAGE_KEYS.clientId]: 'desktop-tauri',
    [SECURITY_STORAGE_KEYS.deviceId]: 'desktop-keep',
    [SECURITY_STORAGE_KEYS.accessToken]: 'access-existing',
    [SECURITY_STORAGE_KEYS.refreshToken]: 'refresh-existing',
    [SECURITY_STORAGE_KEYS.userInfo]: JSON.stringify({ userId: 2002, userName: 'neo' }),
  })
  let logoutRequested = false
  const client = createRequestClient({
    storage,
    fetchImpl: async (url, init) => {
      const pathname = new URL(url).pathname
      if (pathname === '/app-api/auth/logout') {
        const headers = new Headers(init.headers)
        assert.equal(headers.get('Authorization'), 'Bearer access-existing')
        logoutRequested = true
        return jsonResponse({ code: 200, data: true })
      }

      throw new Error(`unexpected request: ${pathname}`)
    },
    baseUrl: 'https://api.example.com',
  })
  const authApi = createAuthApi({ client, storage })

  await authApi.logout()

  assert.equal(logoutRequested, true)
  const state = authApi.getSessionState()
  assert.equal(state.accessToken, '')
  assert.equal(state.refreshToken, '')
  assert.equal(state.userId, null)
  assert.equal(state.userInfo, null)
  assert.equal(state.isGuest, false)
  assert.equal(client.storage.getString(SECURITY_STORAGE_KEYS.clientId), 'desktop-tauri')
  assert.equal(client.storage.getString(SECURITY_STORAGE_KEYS.deviceId), 'desktop-keep')
})

test('登录响应仅返回 userId 时会写入会话 userId', async () => {
  const storage = createMemoryStorage()
  const client = createRequestClient({
    storage,
    fetchImpl: async (url) => {
      const pathname = new URL(url).pathname
      assert.equal(pathname, '/app-api/auth/login/password')
      return jsonResponse({
        code: 200,
        data: {
          accessToken: 'access-flat-userid',
          refreshToken: 'refresh-flat-userid',
          userId: 10003,
        },
      })
    },
    baseUrl: 'https://api.example.com',
  })
  const authApi = createAuthApi({ client, storage })

  await authApi.loginByPassword({
    userName: 'flat-user',
    password: 'demo-pass',
  })

  const state = authApi.getSessionState()
  assert.equal(state.userId, 10003)
  assert.equal(state.userInfo?.userId, 10003)
  assert.equal(client.storage.getString(SECURITY_STORAGE_KEYS.userId), '10003')
})

test('邮箱注册成功后进入登录态并标记试用提示', async () => {
  const storage = createMemoryStorage()
  const client = createRequestClient({
    storage,
    fetchImpl: async (url) => {
      const pathname = new URL(url).pathname
      assert.equal(pathname, '/app-api/auth/register/email')
      return jsonResponse({
        code: 200,
        data: {
          accessToken: 'access-register',
          refreshToken: 'refresh-register',
          expireIn: 7200,
          userInfo: {
            userId: 3003,
            userName: 'mail-user',
          },
        },
      })
    },
    baseUrl: 'https://api.example.com',
  })
  const authApi = createAuthApi({ client, storage })

  await authApi.registerByEmail({
    email: 'demo@example.com',
    code: '123456',
    password: 'Demo123456',
    confirmPassword: 'Demo123456',
  })

  const state = authApi.getSessionState()
  assert.equal(state.accessToken, 'access-register')
  assert.equal(state.refreshToken, 'refresh-register')
  assert.equal(state.userId, 3003)
  assert.equal(state.isGuest, false)
  assert.equal(state.trialGiftPending, true)
})

test('登录态接口遇到 401 时会自动刷新并重试一次', async () => {
  const storage = createMemoryStorage({
    [SECURITY_STORAGE_KEYS.accessToken]: 'access-old',
    [SECURITY_STORAGE_KEYS.refreshToken]: 'refresh-old',
    [SECURITY_STORAGE_KEYS.userInfo]: JSON.stringify({ userId: 4004, userName: 'retry-user' }),
  })
  const requests = []
  const client = createRequestClient({
    storage,
    fetchImpl: async (url, init) => {
      const pathname = new URL(url).pathname
      requests.push({ pathname, init })

      if (pathname === '/app-api/account/profile') {
        const authorization = new Headers(init.headers).get('Authorization')
        if (authorization === 'Bearer access-old') {
          return jsonResponse({ code: 401, msg: 'token expired' }, 401)
        }

        if (authorization === 'Bearer access-new') {
          return jsonResponse({
            code: 200,
            data: {
              userId: 4004,
              nickName: 'Retry Success',
            },
          })
        }
      }

      if (pathname === '/app-api/auth/refresh-token') {
        const payload = JSON.parse(init.body)
        assert.equal(payload.refreshToken, 'refresh-old')
        return jsonResponse({
          code: 200,
          data: {
            accessToken: 'access-new',
            refreshToken: 'refresh-new',
            expireIn: 7200,
          },
        })
      }

      throw new Error(`unexpected request: ${pathname}`)
    },
    baseUrl: 'https://api.example.com',
  })
  const authApi = createAuthApi({ client, storage })

  const profile = await client.get('/app-api/account/profile', {}, { requiresAuth: true })

  assert.equal(profile.userId, 4004)
  const refreshRequests = requests.filter((item) => item.pathname === '/app-api/auth/refresh-token')
  const profileRequests = requests.filter((item) => item.pathname === '/app-api/account/profile')
  assert.equal(refreshRequests.length, 1)
  assert.equal(profileRequests.length, 2)

  const state = authApi.getSessionState()
  assert.equal(state.accessToken, 'access-new')
  assert.equal(state.refreshToken, 'refresh-new')
  assert.equal(state.userId, 4004)
})

test('刷新令牌失败时会回退到未登录态', async () => {
  const storage = createMemoryStorage({
    [SECURITY_STORAGE_KEYS.accessToken]: 'access-expired',
    [SECURITY_STORAGE_KEYS.refreshToken]: 'refresh-expired',
    [SECURITY_STORAGE_KEYS.userInfo]: JSON.stringify({ userId: 5005, userName: 'expired-user' }),
  })
  const client = createRequestClient({
    storage,
    fetchImpl: async (url) => {
      const pathname = new URL(url).pathname
      if (pathname === '/app-api/account/profile') {
        return jsonResponse({ code: 401, msg: 'access expired' }, 401)
      }

      if (pathname === '/app-api/auth/refresh-token') {
        return jsonResponse({ code: 401, msg: 'refresh expired' }, 401)
      }

      throw new Error(`unexpected request: ${pathname}`)
    },
    baseUrl: 'https://api.example.com',
  })
  const authApi = createAuthApi({ client, storage })

  await assert.rejects(
    () => client.get('/app-api/account/profile', {}, { requiresAuth: true }),
    (error) => {
      assert.equal(error.code, 401)
      return true
    },
  )

  const state = authApi.getSessionState()
  assert.equal(state.accessToken, '')
  assert.equal(state.refreshToken, '')
  assert.equal(state.userId, null)
  assert.equal(state.userInfo, null)
  assert.equal(state.isGuest, false)
})

test('临时访客会写入访客态标记并清空登录凭证', () => {
  const storage = createMemoryStorage({
    [SECURITY_STORAGE_KEYS.accessToken]: 'access-before-guest',
    [SECURITY_STORAGE_KEYS.refreshToken]: 'refresh-before-guest',
  })
  const client = createRequestClient({
    storage,
    fetchImpl: async () => jsonResponse({ code: 200, data: true }),
    baseUrl: 'https://api.example.com',
  })
  const authApi = createAuthApi({ client, storage })

  authApi.enterGuestMode()

  const state = authApi.getSessionState()
  assert.equal(state.isGuest, true)
  assert.equal(state.accessToken, '')
  assert.equal(state.refreshToken, '')
  assert.equal(state.userId, null)
})


