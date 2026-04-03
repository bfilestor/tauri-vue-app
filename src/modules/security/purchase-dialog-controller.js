const PAY_CHANNELS = {
  wechat: 'WECHAT',
  alipay: 'ALIPAY',
}

const SUCCESS_ORDER_STATUS = new Set(['PAID', 'SUCCESS', 'COMPLETED', 'FINISHED'])
const SUCCESS_PAY_STATUS = new Set(['PAID', 'SUCCESS', 'PAY_SUCCESS'])
const FAILED_ORDER_STATUS = new Set(['FAILED', 'CANCELLED', 'CLOSED', 'EXPIRED'])
const FAILED_PAY_STATUS = new Set(['PAY_FAILED', 'FAILED', 'CANCELLED', 'CLOSED'])

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

function normalizeStatus(value) {
  return String(value || '').trim().toUpperCase()
}

function isSuccessStatus(orderStatus, payStatus) {
  return SUCCESS_ORDER_STATUS.has(orderStatus) || SUCCESS_PAY_STATUS.has(payStatus)
}

function isFailedStatus(orderStatus, payStatus) {
  return FAILED_ORDER_STATUS.has(orderStatus) || FAILED_PAY_STATUS.has(payStatus)
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
    payStatus: '',
    payableAmount: 0,
    qrcodeUrl: '',
    qrcodeExpireTime: '',
    qrcodeExpired: false,
    pollingActive: false,
    pollingStatus: 'idle',
    pollingAttempts: 0,
    pollingMaxAttempts: 40,
    pollingIntervalMs: 3000,
    pollingMessage: '',
    lastPolledAt: '',
    errorMessage: '',
    openReason: '',
  }
}

export function createPurchaseDialogController({
  state = createInitialPurchaseDialogState(),
  orderService,
  now = () => Date.now(),
  idempotencyKeyFactory = defaultIdempotencyKeyFactory,
  scheduler = {
    setInterval: (...args) => globalThis.setInterval(...args),
    clearInterval: (...args) => globalThis.clearInterval(...args),
  },
  onPaymentSuccess = undefined,
} = {}) {
  if (!orderService || typeof orderService.createOrder !== 'function' || typeof orderService.fetchPayQrcode !== 'function') {
    throw new Error('createPurchaseDialogController requires orderService instance.')
  }

  let pollingTimerId = null

  function stopPolling(reason = 'stopped') {
    if (pollingTimerId != null) {
      scheduler.clearInterval(pollingTimerId)
      pollingTimerId = null
    }
    state.pollingActive = false
    if (reason === 'cancelled') {
      state.pollingStatus = 'cancelled'
      state.pollingMessage = '支付查询已取消'
    }
  }

  function resetOrderRuntime() {
    stopPolling('reset')
    state.orderNo = ''
    state.orderStatus = ''
    state.payStatus = ''
    state.payableAmount = 0
    state.qrcodeUrl = ''
    state.qrcodeExpireTime = ''
    state.qrcodeExpired = false
    state.pollingStatus = 'idle'
    state.pollingAttempts = 0
    state.pollingMessage = ''
    state.lastPolledAt = ''
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
    stopPolling('closed')
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

  function startPolling({ maxAttempts = 40, intervalMs = 3000 } = {}) {
    if (!state.orderNo) {
      return {
        ok: false,
        reason: 'order_missing',
      }
    }

    stopPolling('restart')
    state.pollingActive = true
    state.pollingStatus = 'polling'
    state.pollingAttempts = 0
    state.pollingMaxAttempts = Math.max(1, maxAttempts)
    state.pollingIntervalMs = Math.max(500, intervalMs)
    state.pollingMessage = '正在查询支付状态...'

    pollingTimerId = scheduler.setInterval(() => {
      void pollOrderStatusOnce()
    }, state.pollingIntervalMs)

    return {
      ok: true,
    }
  }

  async function pollOrderStatusOnce() {
    if (!state.orderNo) {
      return {
        ok: false,
        reason: 'order_missing',
      }
    }

    if (!state.pollingActive) {
      state.pollingActive = true
      state.pollingStatus = 'polling'
    }

    state.pollingAttempts += 1
    state.lastPolledAt = new Date(now()).toISOString()

    try {
      const statusResp = await orderService.fetchOrderStatus(state.orderNo)
      const orderStatus = normalizeStatus(statusResp?.orderStatus)
      const payStatus = normalizeStatus(statusResp?.payStatus)
      state.orderStatus = orderStatus || state.orderStatus
      state.payStatus = payStatus

      if (isSuccessStatus(orderStatus, payStatus)) {
        stopPolling('success')
        state.pollingStatus = 'success'
        state.pollingMessage = '支付成功，正在刷新余额...'
        if (typeof onPaymentSuccess === 'function') {
          await onPaymentSuccess({
            orderNo: state.orderNo,
            orderStatus,
            payStatus,
          })
        }
        return {
          ok: true,
          final: 'success',
        }
      }

      if (isFailedStatus(orderStatus, payStatus)) {
        stopPolling('failed')
        state.pollingStatus = 'failed'
        state.pollingMessage = '订单已关闭或支付失败，请重新下单'
        return {
          ok: true,
          final: 'failed',
        }
      }

      if (state.pollingAttempts >= state.pollingMaxAttempts) {
        stopPolling('timeout')
        state.pollingStatus = 'timeout'
        state.pollingMessage = '查询支付状态超时，请继续支付或稍后重试'
        return {
          ok: true,
          final: 'timeout',
        }
      }

      state.pollingStatus = 'polling'
      state.pollingMessage = '等待支付中...'
      return {
        ok: true,
        final: 'pending',
      }
    } catch (error) {
      stopPolling('error')
      state.pollingStatus = 'error'
      state.pollingMessage = error?.message || '查询支付状态失败'
      return {
        ok: false,
        reason: 'poll_error',
        error,
      }
    }
  }

  async function cancelPayment() {
    stopPolling('cancelled')
    if (!state.orderNo || typeof orderService.cancelOrder !== 'function') {
      return {
        ok: true,
      }
    }

    try {
      await orderService.cancelOrder(state.orderNo)
      return {
        ok: true,
      }
    } catch (error) {
      state.errorMessage = error?.message || '取消订单失败'
      return {
        ok: false,
        reason: 'cancel_failed',
        error,
      }
    }
  }

  return {
    state,
    open,
    close,
    selectPackageByCalls,
    createOrder,
    switchPayChannel,
    refreshQrcode,
    startPolling,
    stopPolling,
    pollOrderStatusOnce,
    cancelPayment,
  }
}
