import test from 'node:test'
import assert from 'node:assert/strict'

import {
  SECURITY_STORAGE_KEYS,
  SIGNATURE_MODE,
  createMemoryStorage,
  createRequestClient,
  ensureDeviceReady,
  getSecurityState,
  signRequest,
} from '../../src/modules/security/index.js'

function jsonResponse(payload, status = 200) {
  return new Response(JSON.stringify(payload), {
    status,
    headers: {
      'Content-Type': 'application/json',
    },
  })
}

function createRuntimeInfo() {
  return {
    appVersion: '1.0.2',
    deviceName: 'Windows Desktop',
    deviceType: 'PC',
    osName: 'Windows',
    osVersion: '11',
  }
}

test('未登录启动仅生成设备标识，不触发激活请求', async () => {
  const storage = createMemoryStorage()
  const requests = []

  const state = await ensureDeviceReady({
    storage,
    fetchImpl: async (url, init) => {
      requests.push({ url, init })
      return jsonResponse({
        code: 200,
        data: {
          activationStatus: 'ACTIVE',
          credentialVersion: 'cred-v1',
          expireTime: 1893456000000,
          deviceSecret: 'secret-v1',
        },
      })
    },
    baseUrl: 'https://api.example.com',
    runtimeInfo: createRuntimeInfo(),
    now: () => 1775000000000,
    nonceFactory: () => 'nonce-activate',
  })

  assert.equal(requests.length, 0)
  assert.equal(state.clientId, 'desktop-tauri')
  assert.match(state.deviceId, /^desktop-/)
  assert.equal(state.deviceSecret, '')
  assert.equal(getSecurityState(storage).credentialVersion, '')
})

test('登录后激活会携带 token 与 userId', async () => {
  const storage = createMemoryStorage({
    [SECURITY_STORAGE_KEYS.accessToken]: 'access-activate',
    [SECURITY_STORAGE_KEYS.userId]: '10001',
  })
  const requests = []

  const state = await ensureDeviceReady({
    storage,
    fetchImpl: async (url, init) => {
      requests.push({ url, init })
      return jsonResponse({
        code: 200,
        data: {
          activationStatus: 'ACTIVE',
          credentialVersion: 'cred-v1',
          expireTime: 1893456000000,
          deviceSecret: 'secret-v1',
        },
      })
    },
    baseUrl: 'https://api.example.com',
    runtimeInfo: createRuntimeInfo(),
    now: () => 1775000000000,
    nonceFactory: () => 'nonce-activate',
  })

  assert.equal(requests.length, 1)
  assert.equal(new URL(requests[0].url).pathname, '/app-api/client/activate')
  const headers = new Headers(requests[0].init.headers)
  const payload = JSON.parse(requests[0].init.body)
  assert.equal(headers.get('Authorization'), 'Bearer access-activate')
  assert.equal(payload.userId, 10001)
  assert.equal(state.deviceSecret, 'secret-v1')
  assert.equal(getSecurityState(storage).credentialVersion, 'cred-v1')
})

test('高价值接口会自动附加签名头和幂等键', async () => {
  const storage = createMemoryStorage({
    [SECURITY_STORAGE_KEYS.clientId]: 'desktop-tauri',
    [SECURITY_STORAGE_KEYS.deviceId]: 'desktop-fixed-device',
    [SECURITY_STORAGE_KEYS.deviceSecret]: 'secret-v1',
    [SECURITY_STORAGE_KEYS.credentialVersion]: 'cred-v1',
    [SECURITY_STORAGE_KEYS.credentialExpireTime]: '1893456000000',
  })
  const requests = []
  const client = createRequestClient({
    storage,
    fetchImpl: async (url, init) => {
      requests.push({ url, init })
      return jsonResponse({
        code: 200,
        data: {
          ok: true,
        },
      })
    },
    baseUrl: 'https://api.example.com',
    runtimeInfo: createRuntimeInfo(),
    now: () => 1775000000000,
    nonceFactory: () => 'nonce-request',
    traceIdFactory: () => 'trace-001',
  })

  await client.post(
    '/app-api/orders',
    { skuId: 1001 },
    {},
    { highValue: true, idempotent: true, idempotencyKey: 'order-1775000000000' },
  )

  assert.equal(requests.length, 1)
  const headers = new Headers(requests[0].init.headers)
  const body = JSON.stringify({ skuId: 1001 })
  const { signature } = await signRequest({
    secret: 'secret-v1',
    method: 'POST',
    requestPath: '/app-api/orders',
    body,
    timestamp: 1775000000000,
    nonce: 'nonce-request',
    clientId: 'desktop-tauri',
    deviceId: 'desktop-fixed-device',
  })

  assert.equal(headers.get('X-Client-Id'), 'desktop-tauri')
  assert.equal(headers.get('X-Device-Id'), 'desktop-fixed-device')
  assert.equal(headers.get('X-Timestamp'), '1775000000000')
  assert.equal(headers.get('X-Nonce'), 'nonce-request')
  assert.equal(headers.get('X-Signature'), signature)
  assert.equal(headers.get('X-App-Version'), '1.0.2')
  assert.equal(headers.get('Idempotency-Key'), 'order-1775000000000')
  assert.equal(headers.get('X-Trace-Id'), 'trace-001')
})

test('设备凭证过期时会先续期再继续高价值请求', async () => {
  const storage = createMemoryStorage({
    [SECURITY_STORAGE_KEYS.clientId]: 'desktop-tauri',
    [SECURITY_STORAGE_KEYS.deviceId]: 'desktop-fixed-device',
    [SECURITY_STORAGE_KEYS.deviceSecret]: 'expired-secret',
    [SECURITY_STORAGE_KEYS.credentialVersion]: 'cred-v1',
    [SECURITY_STORAGE_KEYS.credentialExpireTime]: '1000',
  })
  const requests = []
  const client = createRequestClient({
    storage,
    baseUrl: 'https://api.example.com',
    runtimeInfo: createRuntimeInfo(),
    now: () => 1775000000000,
    nonceFactory: () => 'nonce-refresh',
    fetchImpl: async (url, init) => {
      requests.push({ url, init })
      const pathname = new URL(url).pathname

      if (pathname === '/app-api/client/refresh-credential') {
        return jsonResponse({
          code: 200,
          data: {
            credentialVersion: 'cred-v2',
            expireTime: 1893456000000,
            deviceSecret: 'new-secret',
          },
        })
      }

      return jsonResponse({
        code: 200,
        data: {
          canUse: true,
        },
      })
    },
  })

  await client.post('/app-api/usage/precheck', { usageType: 'CHAT' }, {}, { highValue: true })

  assert.equal(requests.length, 2)
  assert.equal(new URL(requests[0].url).pathname, '/app-api/client/refresh-credential')
  assert.equal(new URL(requests[1].url).pathname, '/app-api/usage/precheck')
  assert.equal(getSecurityState(storage).credentialVersion, 'cred-v2')
  assert.equal(getSecurityState(storage).deviceSecret, 'new-secret')
})

test('兼容模式会记录状态且不阻断低价值接口', async () => {
  const storage = createMemoryStorage({
    [SECURITY_STORAGE_KEYS.accessToken]: 'access-compat',
    [SECURITY_STORAGE_KEYS.userId]: '10002',
  })

  const state = await ensureDeviceReady({
    storage,
    fetchImpl: async () => jsonResponse({
      code: 'CLIENT_ACTIVATION_DISABLED',
      msg: 'activation disabled',
    }, 403),
    baseUrl: 'https://api.example.com',
    runtimeInfo: createRuntimeInfo(),
  })

  assert.equal(state.signatureMode, SIGNATURE_MODE.compat)

  const requests = []
  const client = createRequestClient({
    storage,
    fetchImpl: async (url, init) => {
      requests.push({ url, init })
      return jsonResponse({
        code: 200,
        data: [{ productId: 1 }],
      })
    },
    baseUrl: 'https://api.example.com',
    runtimeInfo: createRuntimeInfo(),
    signatureMode: SIGNATURE_MODE.compat,
  })

  await client.get('/app-api/products')

  const headers = new Headers(requests[0].init.headers)
  assert.equal(headers.get('X-Signature'), null)
  assert.equal(headers.get('X-Trace-Id') !== null, true)
})
