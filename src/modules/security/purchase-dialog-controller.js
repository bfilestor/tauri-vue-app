const PAY_CHANNELS = {
  wechat: 'WECHAT',
  alipay: 'ALIPAY',
}

function normalizePayChannel(channel) {
  const value = String(channel || '').trim().toUpperCase()
  if (value === PAY_CHANNELS.alipay) {
    return PAY_CHANNELS.alipay
  }

  return PAY_CHANNELS.wechat
}

function createEmptyCard() {
  return {
    targetCalls: 0,
    title: '',
    missing: true,
    purchasable: false,
    product: null,
  }
}

function cloneCards(cards) {
  return Array.isArray(cards)
    ? cards.map((card) => ({ ...createEmptyCard(), ...card }))
    : []
}

function resolvePreferredCard(cards, preferredCalls) {
  if (!Array.isArray(cards) || cards.length === 0) {
    return null
  }

  if (preferredCalls != null) {
    const byCalls = cards.find((item) => item.targetCalls === preferredCalls && item.purchasable)
    if (byCalls) {
      return byCalls
    }
  }

  const firstPurchasable = cards.find((item) => item.purchasable)
  return firstPurchasable || cards[0]
}

function parseExpireTime(input) {
  if (!input) {
    return ''
  }

  const raw = String(input)
  const parsed = Date.parse(raw)
  if (Number.isNaN(parsed)) {
    return raw
  }

  return new Date(parsed).toISOString()
}

function isExpired(expireTime, now) {
  if (!expireTime) {
    return false
  }

  const parsed = Date.parse(expireTime)
  if (Number.isNaN(parsed)) {
    return false
  }

  return now() >= parsed
}

function defaultIdempotencyKeyFactory() {
  const random = Math.random().toString(16).slice(2, 10)
  return `order-${Date.now()}-${random}`
}

export function createInitialPurchaseDialogState() {
  return {
    visible: false,
    opening: false,
    creatingOrder: false,
    loadingQrcode: false,
    payChannel: PAY_CHANNELS.wechat,
    packageCards: [],
    selectedCard: null,
    orderNo: '',
    orderStatus: '',
    payableAmount: 0,
    qrcodeUrl: '',
    qrcodeExpireTime: '',
    qrcodeExpired: false,
    errorMessage: '',
    openReason: '',
  }
}

export function createPurchaseDialogController({
  state = createInitialPurchaseDialogState(),
  orderService,
  now = () => Date.now(),
  idempotencyKeyFactory = defaultIdempotencyKeyFactory,
} = {}) {
  if (!orderService || typeof orderService.createOrder !== 'function' || typeof orderService.fetchPayQrcode !== 'function') {
    throw new Error('createPurchaseDialogController requires orderService instance.')
  }

  function resetOrderRuntime() {
    state.orderNo = ''
    state.orderStatus = ''
    state.payableAmount = 0
    state.qrcodeUrl = ''
    state.qrcodeExpireTime = ''
    state.qrcodeExpired = false
    state.errorMessage = ''
  }

  function open({ packageCards = [], preferredCalls = null, reason = '' } = {}) {
    state.visible = true
    state.openReason = reason || ''
    state.packageCards = cloneCards(packageCards)
    state.selectedCard = resolvePreferredCard(state.packageCards, preferredCalls)
    state.payChannel = PAY_CHANNELS.wechat
    resetOrderRuntime()
    return state
  }

  function close() {
    state.visible = false
    state.errorMessage = ''
    return state
  }

  function selectPackageByCalls(targetCalls) {
    const matched = state.packageCards.find((item) => item.targetCalls === targetCalls)
    if (!matched || !matched.purchasable) {
      return {
        ok: false,
        reason: 'not_purchasable',
      }
    }

    state.selectedCard = matched
    resetOrderRuntime()
    return {
      ok: true,
    }
  }

  async function loadQrcode(payChannel = state.payChannel) {
    if (!state.orderNo) {
      return { ok: false, reason: 'order_missing' }
    }

    state.loadingQrcode = true
    state.errorMessage = ''
    try {
      const qr = await orderService.fetchPayQrcode(state.orderNo, payChannel)
      state.payChannel = normalizePayChannel(payChannel)
      state.qrcodeUrl = qr.qrcodeUrl || ''
      state.qrcodeExpireTime = parseExpireTime(qr.expireTime)
      state.qrcodeExpired = isExpired(state.qrcodeExpireTime, now)
      return {
        ok: true,
      }
    } catch (error) {
      state.errorMessage = error?.message || '二维码加载失败，请稍后重试'
      return {
        ok: false,
        reason: 'qrcode_failed',
        error,
      }
    } finally {
      state.loadingQrcode = false
    }
  }

  async function createOrder() {
    if (state.creatingOrder) {
      return {
        ok: false,
        reason: 'creating',
      }
    }

    if (!state.selectedCard?.purchasable || !state.selectedCard?.product?.skuId) {
      state.errorMessage = '当前套餐不可购买，请选择可用套餐'
      return {
        ok: false,
        reason: 'invalid_package',
      }
    }

    state.creatingOrder = true
    state.errorMessage = ''

    try {
      const idempotencyKey = idempotencyKeyFactory()
      const order = await orderService.createOrder({
        skuId: state.selectedCard.product.skuId,
        sourceChannel: 'DESKTOP',
        idempotencyKey,
      })

      state.orderNo = order.orderNo || ''
      state.orderStatus = order.orderStatus || ''
      state.payableAmount = order.payableAmount ?? 0

      if (!state.orderNo) {
        throw new Error('订单创建成功但未返回订单号')
      }

      await loadQrcode(state.payChannel)
      return {
        ok: true,
        orderNo: state.orderNo,
      }
    } catch (error) {
      state.errorMessage = error?.message || '创建订单失败，请稍后重试'
      return {
        ok: false,
        reason: 'create_failed',
        error,
      }
    } finally {
      state.creatingOrder = false
    }
  }

  async function switchPayChannel(channel) {
    state.payChannel = normalizePayChannel(channel)
    if (!state.orderNo) {
      return {
        ok: true,
      }
    }

    return loadQrcode(state.payChannel)
  }

  async function refreshQrcode() {
    return loadQrcode(state.payChannel)
  }

  return {
    state,
    open,
    close,
    selectPackageByCalls,
    createOrder,
    switchPayChannel,
    refreshQrcode,
  }
}
