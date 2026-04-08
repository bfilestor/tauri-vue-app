import test from 'node:test'
import assert from 'node:assert/strict'

import {
  PACKAGE_CARD_CALLS,
  createAccountContextService,
} from '../../src/modules/security/account-context-service.js'

function createClientStub() {
  const requests = []
  const handlers = new Map()

  return {
    requests,
    when(path, handler) {
      handlers.set(path, handler)
    },
    async get(path, init = {}, meta = {}) {
      requests.push({ path, init, meta })
      const handler = handlers.get(path)
      if (!handler) {
        throw new Error(`unexpected request: ${path}`)
      }

      return handler({ path, init, meta })
    },
    async post(path, body, init = {}, meta = {}) {
      requests.push({ path, body, init, meta, method: 'POST' })
      const handler = handlers.get(`POST ${path}`) || handlers.get(path)
      if (!handler) {
        throw new Error(`unexpected request: ${path}`)
      }

      return handler({ path, body, init, meta, method: 'POST' })
    },
  }
}

function createAuthApiStub(sessionState) {
  return {
    getSessionState() {
      return {
        accessToken: '',
        refreshToken: '',
        userInfo: null,
        isGuest: false,
        isAuthenticated: false,
        ...sessionState,
      }
    },
  }
}

test('登录后自动加载账户上下文并写入统一缓存', async () => {
  const client = createClientStub()
  client.when('/app-api/account/profile', () => ({ userId: 1001, nickName: 'Demo' }))
  client.when('/app-api/account/balance', () => ({ ocrBalance: 7, analyzeBalance: 9, chatBalance: 11 }))
  client.when('/app-api/wallet', () => ({ totalBalance: 27 }))
  client.when('/app-api/family-members', () => ([
    { memberId: 2001, memberName: '本人', defaultFlag: true },
  ]))
  client.when('/app-api/products', () => ([
    { productId: 1, productName: '20次包', callTimes: 20, skuList: [{ skuId: 101, salePrice: 10 }] },
    { productId: 2, productName: '100次包', callTimes: 100, skuList: [{ skuId: 102, salePrice: 29.9 }] },
    { productId: 3, productName: '500次包', callTimes: 500, skuList: [{ skuId: 103, salePrice: 99 }] },
  ]))

  const service = createAccountContextService({
    client,
    authApi: createAuthApiStub({ isAuthenticated: true, accessToken: 'token-001' }),
  })

  const state = await service.refreshForCurrentSession({ force: true })

  assert.equal(state.status, 'ready')
  assert.equal(state.profile?.userId, 1001)
  assert.equal(state.balance?.ocrBalance, 7)
  assert.equal(state.wallet?.totalBalance, 27)
  assert.equal(state.defaultMember?.memberId, 2001)
  assert.equal(state.currentMember?.memberId, 2001)
  assert.equal(state.packageCards.length, 3)
  assert.equal(state.lastError, '')

  const paths = client.requests.map((item) => item.path).sort()
  assert.deepEqual(paths, [
    '/app-api/account/balance',
    '/app-api/account/profile',
    '/app-api/family-members',
    '/app-api/products',
    '/app-api/wallet',
  ])

  const authedRequests = client.requests.filter((item) => item.path !== '/app-api/products')
  assert.equal(authedRequests.every((item) => item.meta.includeUserId === true), true)
})

test('访客态只加载公开套餐，不请求登录态余额接口', async () => {
  const client = createClientStub()
  client.when('/app-api/products', () => ([
    { productId: 1, productName: '20次包', callTimes: 20, skuList: [{ skuId: 101, salePrice: 10 }] },
  ]))

  const service = createAccountContextService({
    client,
    authApi: createAuthApiStub({ isGuest: true, isAuthenticated: false }),
  })

  const state = await service.refreshForCurrentSession({ force: true })

  assert.equal(state.status, 'ready')
  assert.equal(state.isGuest, true)
  assert.equal(state.profile, null)
  assert.equal(state.balance, null)
  assert.equal(state.wallet, null)
  assert.equal(state.members.length, 0)
  assert.equal(client.requests.length, 1)
  assert.equal(client.requests[0].path, '/app-api/products')
})

test('无默认成员时进入阻塞状态并给出提示', async () => {
  const client = createClientStub()
  client.when('/app-api/account/profile', () => ({ userId: 1002, nickName: 'NoMember' }))
  client.when('/app-api/account/balance', () => ({ ocrBalance: 1 }))
  client.when('/app-api/wallet', () => ({ totalBalance: 1 }))
  client.when('/app-api/family-members', () => ([
    { memberId: 3001, memberName: '成员A', defaultFlag: false },
    { memberId: 3002, memberName: '成员B', defaultFlag: false },
  ]))
  client.when('/app-api/products', () => ([
    { productId: 1, productName: '20次包', callTimes: 20, skuList: [{ skuId: 101 }] },
  ]))

  const service = createAccountContextService({
    client,
    authApi: createAuthApiStub({ isAuthenticated: true, accessToken: 'token-002' }),
  })

  const state = await service.refreshForCurrentSession({ force: true })

  assert.equal(state.status, 'blocked')
  assert.equal(state.memberBlocked, true)
  assert.match(state.memberBlockedReason, /默认成员/)
  assert.equal(state.defaultMember, null)
})

test('登录后若成员列表为空则自动创建本人成员并恢复可用状态', async () => {
  const client = createClientStub()
  let memberListRequestCount = 0

  client.when('/app-api/account/profile', () => ({
    userId: 1003,
    nickName: '新用户',
    phone: '13800000000',
  }))
  client.when('/app-api/account/balance', () => ({ ocrBalance: 0 }))
  client.when('/app-api/wallet', () => ({ totalBalance: 0 }))
  client.when('/app-api/family-members', () => {
    memberListRequestCount += 1
    if (memberListRequestCount === 1) {
      return []
    }

    return [
      { memberId: 4001, memberName: '新用户', relationCode: 'SELF', defaultFlag: true },
    ]
  })
  client.when('POST /app-api/family-members', ({ body, meta }) => {
    assert.equal(body.memberName, '新用户')
    assert.equal(body.relationCode, 'SELF')
    assert.equal(body.mobile, '13800000000')
    assert.equal(meta.requiresAuth, true)
    assert.equal(meta.includeUserId, true)
    return { memberId: 4001 }
  })
  client.when('/app-api/products', () => ([]))

  const service = createAccountContextService({
    client,
    authApi: createAuthApiStub({ isAuthenticated: true, accessToken: 'token-003' }),
  })

  const state = await service.refreshForCurrentSession({ force: true })

  assert.equal(state.status, 'ready')
  assert.equal(state.memberBlocked, false)
  assert.equal(state.defaultMember?.memberId, 4001)
  assert.equal(state.currentMember?.memberId, 4001)

  const postRequest = client.requests.find((item) => item.method === 'POST' && item.path === '/app-api/family-members')
  assert.ok(postRequest)
})

test('自动创建本人成员失败时保持阻塞状态并返回明确提示', async () => {
  const client = createClientStub()

  client.when('/app-api/account/profile', () => ({ userId: 1004, nickName: '创建失败用户' }))
  client.when('/app-api/account/balance', () => ({ ocrBalance: 0 }))
  client.when('/app-api/wallet', () => ({ totalBalance: 0 }))
  client.when('/app-api/family-members', () => ([]))
  client.when('POST /app-api/family-members', () => {
    throw new Error('创建成员接口不可用')
  })
  client.when('/app-api/products', () => ([]))

  const service = createAccountContextService({
    client,
    authApi: createAuthApiStub({ isAuthenticated: true, accessToken: 'token-004' }),
  })

  const state = await service.refreshForCurrentSession({ force: true })

  assert.equal(state.status, 'blocked')
  assert.equal(state.memberBlocked, true)
  assert.equal(state.defaultMember, null)
  assert.equal(state.currentMember, null)
  assert.match(state.memberBlockedReason, /创建成员接口不可用/)
})

test('商品列表缺少档位时对应卡片保持缺失态，不伪造 SKU', async () => {
  const client = createClientStub()
  client.when('/app-api/products', () => ([
    { productId: 1, productName: '20次包', callTimes: 20, skuList: [{ skuId: 101, salePrice: 10 }] },
    { productId: 3, productName: '500次包', callTimes: 500, skuList: [{ skuId: 103, salePrice: 99 }] },
  ]))

  const service = createAccountContextService({
    client,
    authApi: createAuthApiStub({ isGuest: true, isAuthenticated: false }),
  })

  const state = await service.refreshForCurrentSession({ force: true })

  assert.deepEqual(PACKAGE_CARD_CALLS, [20, 100, 500])
  assert.equal(state.packageCards.length, 3)

  const card20 = state.packageCards.find((item) => item.targetCalls === 20)
  const card100 = state.packageCards.find((item) => item.targetCalls === 100)
  const card500 = state.packageCards.find((item) => item.targetCalls === 500)

  assert.equal(card20.missing, false)
  assert.equal(card20.product?.skuId, 101)
  assert.equal(card100.missing, true)
  assert.equal(card100.product, null)
  assert.equal(card100.purchasable, false)
  assert.equal(card500.missing, false)
  assert.equal(card500.product?.skuId, 103)
})

test('后端返回非固定档位时按真实商品动态展示并兼容 priceFen', async () => {
  const client = createClientStub()
  client.when('/app-api/products', () => ([
    {
      productId: 20001,
      productName: '初级AI次数包',
      online: true,
      skuList: [
        { skuId: 21001, skuName: '初级包（10次）', priceFen: 1000, times: 10, enabled: true },
      ],
    },
    {
      productId: 20002,
      productName: '中级AI次数包',
      online: true,
      skuList: [
        { skuId: 21002, skuName: '中级包（80次）', priceFen: 5000, times: 80, enabled: true },
      ],
    },
    {
      productId: 20003,
      productName: '高级AI次数包',
      online: true,
      skuList: [
        { skuId: 21003, skuName: '高级包（200次）', priceFen: 10000, times: 200, enabled: true },
      ],
    },
  ]))

  const service = createAccountContextService({
    client,
    authApi: createAuthApiStub({ isGuest: true, isAuthenticated: false }),
  })

  const state = await service.refreshForCurrentSession({ force: true })

  assert.equal(state.packageCards.length, 3)
  assert.deepEqual(state.packageCards.map((item) => item.targetCalls), [10, 80, 200])
  assert.equal(state.packageCards[0].title, '初级AI次数包')
  assert.equal(state.packageCards[0].product?.price, 10)
  assert.equal(state.packageCards[1].product?.price, 50)
  assert.equal(state.packageCards[2].product?.price, 100)
  assert.equal(state.packageCards.every((item) => item.missing === false), true)
  assert.equal(state.packageCards.every((item) => item.purchasable === true), true)
})
