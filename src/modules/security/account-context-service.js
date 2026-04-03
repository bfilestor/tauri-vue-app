const DEFAULT_CACHE_TTL_MS = 30_000
export const PACKAGE_CARD_CALLS = [20, 100, 500]

function createInitialPackageCard(targetCalls) {
  return {
    targetCalls,
    title: `${targetCalls} 次`,
    missing: true,
    purchasable: false,
    product: null,
  }
}

function createInitialPackageCards() {
  return PACKAGE_CARD_CALLS.map((target) => createInitialPackageCard(target))
}

function createInitialState() {
  return {
    status: 'idle',
    isGuest: false,
    profile: null,
    balance: null,
    wallet: null,
    members: [],
    defaultMember: null,
    memberBlocked: false,
    memberBlockedReason: '',
    products: [],
    packageCards: createInitialPackageCards(),
    lastFetchedAt: 0,
    lastError: '',
    cacheKey: '',
  }
}

function toNumber(value) {
  if (typeof value === 'number' && Number.isFinite(value)) {
    return value
  }

  if (typeof value === 'string' && value.trim()) {
    const parsed = Number(value)
    if (Number.isFinite(parsed)) {
      return parsed
    }
  }

  return null
}

function normalizeText(value) {
  return typeof value === 'string' ? value.trim() : ''
}

function isDefaultMark(value) {
  if (value === true || value === 1 || value === '1') {
    return true
  }

  if (typeof value === 'string') {
    const normalized = value.trim().toLowerCase()
    return normalized === 'y' || normalized === 'yes' || normalized === 'true'
  }

  return false
}

function toArray(value) {
  if (Array.isArray(value)) {
    return value
  }

  if (!value || typeof value !== 'object') {
    return []
  }

  if (Array.isArray(value.list)) {
    return value.list
  }

  if (Array.isArray(value.records)) {
    return value.records
  }

  if (Array.isArray(value.items)) {
    return value.items
  }

  return []
}

function pickDefaultSku(skuList) {
  const list = Array.isArray(skuList) ? skuList : []
  if (list.length === 0) {
    return null
  }

  const defaultSku = list.find((item) => isDefaultMark(item?.defaultFlag) || isDefaultMark(item?.isDefault))
  return defaultSku || list[0]
}

function resolveCallTimes(product, sku) {
  return toNumber(
    product?.callTimes
    ?? product?.times
    ?? product?.credits
    ?? product?.quota
    ?? product?.totalCredits
    ?? sku?.callTimes
    ?? sku?.times
    ?? sku?.credits,
  )
}

function normalizeProductItem(product) {
  const sku = pickDefaultSku(product?.skuList)
  const normalized = {
    productId: product?.productId ?? product?.id ?? null,
    productName: normalizeText(product?.productName || product?.name || ''),
    skuId: sku?.skuId ?? sku?.id ?? product?.skuId ?? null,
    price: toNumber(sku?.salePrice ?? sku?.price ?? product?.salePrice ?? product?.price ?? 0) ?? 0,
    callTimes: resolveCallTimes(product, sku),
    raw: product,
  }

  return normalized
}

function mapProductsToPackageCards(products) {
  return PACKAGE_CARD_CALLS.map((targetCalls) => {
    const matched = products.find((item) => item.callTimes === targetCalls)
    if (!matched) {
      return createInitialPackageCard(targetCalls)
    }

    return {
      targetCalls,
      title: `${targetCalls} 次`,
      missing: false,
      purchasable: Boolean(matched.skuId),
      product: matched,
    }
  })
}

function resolveDefaultMember(members) {
  return members.find((item) => (
    isDefaultMark(item?.defaultFlag)
    || isDefaultMark(item?.isDefault)
    || isDefaultMark(item?.defaultMember)
    || isDefaultMark(item?.defaultSelected)
  )) || null
}

function computeCacheKey(session) {
  if (session?.isAuthenticated) {
    return `auth:${session?.accessToken || 'token'}`
  }

  if (session?.isGuest) {
    return 'guest'
  }

  return 'anonymous'
}

export function createAccountContextService({
  client,
  authApi,
  cacheTtlMs = DEFAULT_CACHE_TTL_MS,
  now = () => Date.now(),
} = {}) {
  if (!client || typeof client.get !== 'function') {
    throw new Error('createAccountContextService requires a request client.')
  }

  const state = createInitialState()

  async function fetchProducts() {
    const response = await client.get('/app-api/products')
    const normalizedProducts = toArray(response)
      .map((item) => normalizeProductItem(item))
      .filter((item) => item.productId != null || item.skuId != null || item.callTimes != null)

    state.products = normalizedProducts
    state.packageCards = mapProductsToPackageCards(normalizedProducts)
  }

  async function fetchAuthedContext() {
    const [profile, balance, wallet, membersResp] = await Promise.all([
      client.get('/app-api/account/profile', {}, { requiresAuth: true }),
      client.get('/app-api/account/balance', {}, { requiresAuth: true }),
      client.get('/app-api/wallet', {}, { requiresAuth: true }),
      client.get('/app-api/family-members', {}, { requiresAuth: true }),
    ])

    const members = toArray(membersResp)
    const defaultMember = resolveDefaultMember(members)

    state.profile = profile || null
    state.balance = balance || null
    state.wallet = wallet || null
    state.members = members
    state.defaultMember = defaultMember
    state.memberBlocked = !defaultMember
    state.memberBlockedReason = defaultMember
      ? ''
      : '未找到默认成员，请先在账户中设置默认成员后再继续。'
    state.status = defaultMember ? 'ready' : 'blocked'
  }

  function shouldUseCache({ force, cacheKey }) {
    if (force) {
      return false
    }

    if (!state.lastFetchedAt || state.cacheKey !== cacheKey) {
      return false
    }

    const age = now() - state.lastFetchedAt
    if (age > cacheTtlMs) {
      return false
    }

    return state.status === 'ready' || state.status === 'blocked'
  }

  async function refresh({ force = false, sessionState } = {}) {
    const session = sessionState || (typeof authApi?.getSessionState === 'function' ? authApi.getSessionState() : null) || {}
    const cacheKey = computeCacheKey(session)

    if (shouldUseCache({ force, cacheKey })) {
      return state
    }

    state.status = 'loading'
    state.lastError = ''
    state.isGuest = Boolean(session?.isGuest) || !Boolean(session?.isAuthenticated)
    state.cacheKey = cacheKey

    if (!session?.isAuthenticated) {
      state.profile = null
      state.balance = null
      state.wallet = null
      state.members = []
      state.defaultMember = null
      state.memberBlocked = false
      state.memberBlockedReason = ''
    }

    try {
      if (session?.isAuthenticated) {
        await Promise.all([
          fetchProducts(),
          fetchAuthedContext(),
        ])
      } else {
        await fetchProducts()
        state.status = 'ready'
      }
      state.lastFetchedAt = now()
      return state
    } catch (error) {
      state.status = 'error'
      state.lastError = error?.message || '账户上下文加载失败'
      throw error
    }
  }

  function getState() {
    return state
  }

  function clear() {
    Object.assign(state, createInitialState())
    return state
  }

  return {
    state,
    getState,
    refresh,
    refreshForCurrentSession(options = {}) {
      return refresh({
        ...options,
        sessionState: typeof authApi?.getSessionState === 'function'
          ? authApi.getSessionState()
          : {},
      })
    },
    clear,
  }
}
