import { SECURITY_STORAGE_KEYS } from './constants.js'
import { createStorageAdapter } from './storage.js'

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
    currentMember: null,
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

function normalizeMemberId(value) {
  const numeric = toNumber(value)
  if (numeric != null) {
    return numeric
  }

  return normalizeText(value) || null
}

function normalizeOwnerUserId(value) {
  const normalized = normalizeMemberId(value)
  return normalized == null ? '' : String(normalized)
}

function normalizeMemberItem(member) {
  if (!member || typeof member !== 'object') {
    return null
  }

  const memberId = normalizeMemberId(
    member?.memberId
    ?? member?.id
    ?? member?.cloudMemberId,
  )

  return {
    ...member,
    memberId,
    memberName: normalizeText(
      member?.memberName
      ?? member?.name
      ?? member?.nickName
      ?? member?.displayName,
    ),
    relationCode: normalizeText(
      member?.relationCode
      ?? member?.relation
      ?? member?.relationType,
    ),
    isDefault: (
      isDefaultMark(member?.defaultFlag)
      || isDefaultMark(member?.isDefault)
      || isDefaultMark(member?.defaultMember)
      || isDefaultMark(member?.defaultSelected)
    ),
  }
}

function normalizeMembers(members) {
  return toArray(members)
    .map((item) => normalizeMemberItem(item))
    .filter((item) => item?.memberId != null)
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

function resolvePriceYuan(product, sku) {
  const directPrice = toNumber(
    sku?.salePrice
    ?? sku?.price
    ?? product?.salePrice
    ?? product?.price,
  )
  if (directPrice != null) {
    return directPrice
  }

  const priceFen = toNumber(
    sku?.priceFen
    ?? product?.priceFen,
  )
  if (priceFen != null) {
    return Math.round((priceFen / 100) * 100) / 100
  }

  return 0
}

function normalizeProductItem(product) {
  const sku = pickDefaultSku(product?.skuList)
  const online = product?.online !== false
  const enabled = sku?.enabled !== false
  const normalized = {
    productId: product?.productId ?? product?.id ?? null,
    productName: normalizeText(product?.productName || product?.name || ''),
    skuName: normalizeText(sku?.skuName || sku?.name || ''),
    skuId: sku?.skuId ?? sku?.id ?? product?.skuId ?? null,
    price: resolvePriceYuan(product, sku),
    callTimes: resolveCallTimes(product, sku),
    online,
    enabled,
    purchasable: online && enabled && Boolean(sku?.skuId ?? sku?.id ?? product?.skuId),
    raw: product,
  }

  return normalized
}

function sortProductsForDisplay(products) {
  return [...products].sort((left, right) => {
    const leftCalls = toNumber(left?.callTimes)
    const rightCalls = toNumber(right?.callTimes)
    if (leftCalls != null && rightCalls != null && leftCalls !== rightCalls) {
      return leftCalls - rightCalls
    }
    if (leftCalls != null && rightCalls == null) {
      return -1
    }
    if (leftCalls == null && rightCalls != null) {
      return 1
    }

    const leftId = toNumber(left?.productId) ?? Number.MAX_SAFE_INTEGER
    const rightId = toNumber(right?.productId) ?? Number.MAX_SAFE_INTEGER
    return leftId - rightId
  })
}

function createDynamicProductCard(product, index) {
  const fallbackCalls = Number.MAX_SAFE_INTEGER - index
  const resolvedCalls = toNumber(product?.callTimes)
  const targetCalls = resolvedCalls != null ? resolvedCalls : fallbackCalls
  const suffix = resolvedCalls != null ? `${resolvedCalls} 次` : '套餐'
  const name = normalizeText(product?.productName)

  return {
    targetCalls,
    title: name || suffix,
    missing: false,
    purchasable: Boolean(product?.purchasable),
    product,
  }
}

function mapProductsToPackageCards(products) {
  const fixedCards = PACKAGE_CARD_CALLS.map((targetCalls) => {
    const matched = products.find((item) => item.callTimes === targetCalls)
    if (!matched) {
      return createInitialPackageCard(targetCalls)
    }

    return {
      targetCalls,
      title: `${targetCalls} 次`,
      missing: false,
      purchasable: Boolean(matched.purchasable),
      product: matched,
    }
  })

  const fixedMatchedCount = fixedCards.filter((item) => !item.missing).length
  if (fixedMatchedCount > 0) {
    return fixedCards
  }

  const dynamicCards = sortProductsForDisplay(products)
    .map((item, index) => createDynamicProductCard(item, index))

  return dynamicCards.length > 0 ? dynamicCards : fixedCards
}

function resolveDefaultMember(members) {
  const matched = members.find((item) => item?.isDefault)
  if (matched) {
    return matched
  }

  if (members.length === 1) {
    return members[0]
  }

  return null
}

function resolveSelfMemberName(profile) {
  const candidates = [
    profile?.realName,
    profile?.real_name,
    profile?.nickName,
    profile?.nick_name,
    profile?.userName,
    profile?.user_name,
  ]

  const resolved = candidates
    .map((item) => normalizeText(item))
    .find(Boolean)

  return resolved || '本人'
}

function createSelfMemberPayload(profile) {
  const payload = {
    memberName: resolveSelfMemberName(profile),
    relationCode: 'SELF',
  }

  const gender = normalizeText(profile?.gender)
  if (gender) {
    payload.gender = gender
  }

  const birthday = normalizeText(profile?.birthday)
  if (birthday) {
    payload.birthday = birthday
  }

  const mobile = normalizeText(profile?.phone)
  if (mobile) {
    payload.mobile = mobile
  }

  return payload
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
  memberRepository = null,
  cacheTtlMs = DEFAULT_CACHE_TTL_MS,
  now = () => Date.now(),
} = {}) {
  if (!client || typeof client.get !== 'function') {
    throw new Error('createAccountContextService requires a request client.')
  }

  const state = createInitialState()
  const storage = createStorageAdapter(client.storage)

  function persistCurrentMemberId(memberId) {
    storage.setString(
      SECURITY_STORAGE_KEYS.currentMemberId,
      memberId == null ? '' : String(memberId),
    )
  }

  function readPersistedCurrentMemberId() {
    return normalizeMemberId(storage.getString(SECURITY_STORAGE_KEYS.currentMemberId))
  }

  function resolveSessionState() {
    if (typeof authApi?.getSessionState === 'function') {
      return authApi.getSessionState() || {}
    }
    return {}
  }

  function resolveOwnerUserId(sessionState = null, fallbackUserId = null) {
    const session = sessionState || resolveSessionState()
    return normalizeOwnerUserId(
      session?.userId
      ?? session?.userInfo?.userId
      ?? fallbackUserId
      ?? state.profile?.userId,
    )
  }

  async function requestAuthed(method, path, body) {
    if (method === 'POST' && typeof client.post === 'function') {
      return client.post(path, body, {}, { requiresAuth: true, includeUserId: true })
    }

    if (typeof client.request === 'function') {
      return client.request(
        path,
        { method, body },
        { requiresAuth: true, includeUserId: true },
      )
    }

    throw new Error(`Unsupported authenticated request method: ${method}`)
  }

  const fallbackMemberRepository = {
    async listMembers() {
      const response = await client.get('/app-api/family-members', {}, { requiresAuth: true, includeUserId: true })
      return normalizeMembers(response)
    },
    async createMember({ payload = {} } = {}) {
      return requestAuthed('POST', '/app-api/family-members', payload)
    },
    async updateMember({ memberId, payload = {} } = {}) {
      return requestAuthed('PUT', `/app-api/family-members/${memberId}`, payload)
    },
    async deleteMember({ memberId } = {}) {
      return requestAuthed('DELETE', `/app-api/family-members/${memberId}`)
    },
    async setDefaultMember({ memberId } = {}) {
      return requestAuthed('PUT', `/app-api/family-members/${memberId}/set-default`)
    },
  }

  const resolvedMemberRepository = memberRepository || fallbackMemberRepository

  async function listMembers(ownerUserId) {
    const response = await resolvedMemberRepository.listMembers({
      ownerUserId,
      requestAuthed,
      client,
    })
    return normalizeMembers(response)
  }

  async function createInitialSelfMember(ownerUserId, profile) {
    return resolvedMemberRepository.createMember({
      ownerUserId,
      payload: createSelfMemberPayload(profile),
      requestAuthed,
      client,
    })
  }

  function findMemberById(members, memberId) {
    return members.find((item) => String(item?.memberId) === String(memberId)) || null
  }

  function resolveCurrentMember(members, defaultMember) {
    const storedMemberId = readPersistedCurrentMemberId()
    if (storedMemberId != null) {
      const storedMember = findMemberById(members, storedMemberId)
      if (storedMember) {
        return storedMember
      }
    }

    return defaultMember || null
  }

  async function fetchProducts() {
    const response = await client.get('/app-api/products')
    const normalizedProducts = toArray(response)
      .map((item) => normalizeProductItem(item))
      .filter((item) => item.productId != null || item.skuId != null || item.callTimes != null)

    state.products = normalizedProducts
    state.packageCards = mapProductsToPackageCards(normalizedProducts)
  }

  async function fetchAuthedContext(sessionState = null) {
    const [profile, balance, wallet] = await Promise.all([
      client.get('/app-api/account/profile', {}, { requiresAuth: true, includeUserId: true }),
      client.get('/app-api/account/balance', {}, { requiresAuth: true, includeUserId: true }),
      client.get('/app-api/wallet', {}, { requiresAuth: true, includeUserId: true }),
    ])

    const ownerUserId = resolveOwnerUserId(sessionState, profile?.userId)
    if (!ownerUserId) {
      throw new Error('Missing owner user id.')
    }

    let members = await listMembers(ownerUserId)
    let memberInitError = ''

    if (members.length === 0) {
      try {
        await createInitialSelfMember(ownerUserId, profile)
        members = await listMembers(ownerUserId)
      } catch (error) {
        memberInitError = error?.message || '系统自动创建本人档案失败，请稍后重试。'
      }
    }

    const defaultMember = resolveDefaultMember(members)
    const currentMember = resolveCurrentMember(members, defaultMember)

    persistCurrentMemberId(currentMember?.memberId ?? '')

    state.profile = profile || null
    state.balance = balance || null
    state.wallet = wallet || null
    state.members = members
    state.defaultMember = defaultMember
    state.currentMember = currentMember
    state.memberBlocked = !currentMember
    state.memberBlockedReason = currentMember
      ? ''
      : (memberInitError || '未找到默认成员，请先在账户中设置默认成员后再继续。')
    state.status = currentMember ? 'ready' : 'blocked'
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
      state.currentMember = null
      state.memberBlocked = false
      state.memberBlockedReason = ''
      persistCurrentMemberId('')
    }

    try {
      if (session?.isAuthenticated) {
        await Promise.all([
          fetchProducts(),
          fetchAuthedContext(session),
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

  function selectMember(memberId) {
    const nextMember = findMemberById(state.members, memberId)
    if (!nextMember) {
      throw new Error('成员不存在或已失效')
    }

    state.currentMember = nextMember
    state.memberBlocked = false
    state.memberBlockedReason = ''
    state.status = 'ready'
    persistCurrentMemberId(nextMember.memberId)
    return state
  }

  async function createMember(payload = {}) {
    const ownerUserId = resolveOwnerUserId(null, state.profile?.userId)
    const response = await resolvedMemberRepository.createMember({
      ownerUserId,
      payload,
      requestAuthed,
      client,
    })
    const createdMemberId = normalizeMemberId(
      response?.memberId
      ?? response?.id
      ?? response?.data?.memberId
      ?? response?.data?.id,
    )

    if (createdMemberId != null) {
      persistCurrentMemberId(createdMemberId)
    }

    await refresh({
      force: true,
      sessionState: typeof authApi?.getSessionState === 'function'
        ? authApi.getSessionState()
        : {},
    })
    return state
  }

  async function updateMember(memberId, payload = {}) {
    const ownerUserId = resolveOwnerUserId(null, state.profile?.userId)
    await resolvedMemberRepository.updateMember({
      memberId: String(memberId),
      ownerUserId,
      payload,
      requestAuthed,
      client,
    })
    await refresh({
      force: true,
      sessionState: typeof authApi?.getSessionState === 'function'
        ? authApi.getSessionState()
        : {},
    })
    return state
  }

  async function deleteMember(memberId) {
    const ownerUserId = resolveOwnerUserId(null, state.profile?.userId)
    if (String(state.currentMember?.memberId) === String(memberId)) {
      persistCurrentMemberId('')
    }

    await resolvedMemberRepository.deleteMember({
      memberId: String(memberId),
      ownerUserId,
      requestAuthed,
      client,
    })
    await refresh({
      force: true,
      sessionState: typeof authApi?.getSessionState === 'function'
        ? authApi.getSessionState()
        : {},
    })
    return state
  }

  async function setDefaultMember(memberId) {
    const ownerUserId = resolveOwnerUserId(null, state.profile?.userId)
    persistCurrentMemberId(memberId)
    await resolvedMemberRepository.setDefaultMember({
      memberId: String(memberId),
      ownerUserId,
      requestAuthed,
      client,
    })
    await refresh({
      force: true,
      sessionState: typeof authApi?.getSessionState === 'function'
        ? authApi.getSessionState()
        : {},
    })
    return state
  }

  function clear() {
    Object.assign(state, createInitialState())
    persistCurrentMemberId('')
    return state
  }

  return {
    state,
    getState,
    refresh,
    selectMember,
    createMember,
    updateMember,
    deleteMember,
    setDefaultMember,
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
