import test from 'node:test'
import assert from 'node:assert/strict'

import {
  SECURITY_STORAGE_KEYS,
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

test('includeUserId 会自动在请求 query 中附加 userId', async () => {
  const storage = createMemoryStorage({
    [SECURITY_STORAGE_KEYS.accessToken]: 'access-uid',
    [SECURITY_STORAGE_KEYS.userId]: '10003',
  })
  const requests = []
  const client = createRequestClient({
    storage,
    baseUrl: 'https://api.example.com',
    fetchImpl: async (url, init) => {
      requests.push({ url, init })
      return jsonResponse({ code: 200, data: { ok: true } })
    },
  })

  await client.get('/app-api/orders?status=PAID', {}, { requiresAuth: true, includeUserId: true })

  assert.equal(requests.length, 1)
  const url = new URL(requests[0].url)
  assert.equal(url.pathname, '/app-api/orders')
  assert.equal(url.searchParams.get('status'), 'PAID')
  assert.equal(url.searchParams.get('userId'), '10003')
})

test('includeUserId 且缺失 userId 时会抛出 USER_ID_MISSING', async () => {
  const storage = createMemoryStorage({
    [SECURITY_STORAGE_KEYS.accessToken]: 'access-uid',
  })
  const client = createRequestClient({
    storage,
    baseUrl: 'https://api.example.com',
    fetchImpl: async () => jsonResponse({ code: 200, data: { ok: true } }),
  })

  await assert.rejects(
    () => client.get('/app-api/orders', {}, { requiresAuth: true, includeUserId: true }),
    (error) => {
      assert.equal(error.code, 'USER_ID_MISSING')
      return true
    },
  )
})

