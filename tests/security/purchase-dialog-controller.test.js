import test from 'node:test'
import assert from 'node:assert/strict'

import {
  createInitialPurchaseDialogState,
  createPurchaseDialogController,
} from '../../src/modules/security/purchase-dialog-controller.js'

function createOrderServiceStub() {
  const calls = {
    createOrder: [],
    fetchPayQrcode: [],
    fetchOrderStatus: [],
    cancelOrder: [],
  }

  const responses = {
    createOrder: null,
    fetchPayQrcode: null,
    fetchOrderStatus: null,
    cancelOrder: null,
  }

  function shiftResponse(value, fallback) {
    if (Array.isArray(value)) {
      if (value.length === 0) {
        return fallback
      }
      return value.shift()
    }
    return value ?? fallback
  }

  return {
    calls,
    responses,
    async createOrder(payload) {
      calls.createOrder.push(payload)
      if (responses.createOrder instanceof Error) {
        throw responses.createOrder
      }
      return responses.createOrder || {
        orderNo: 'ORDER-001',
        payableAmount: 29.9,
        orderStatus: 'CREATED',
      }
    },
    async fetchPayQrcode(orderNo, payChannel) {
      calls.fetchPayQrcode.push({ orderNo, payChannel })
      if (responses.fetchPayQrcode instanceof Error) {
        throw responses.fetchPayQrcode
      }
      return responses.fetchPayQrcode || {
        payChannel,
        qrcodeUrl: `https://pay.example.com/${payChannel}/qr.png`,
        expireTime: '2099-01-01T00:00:00Z',
      }
    },
    async fetchOrderStatus(orderNo) {
      calls.fetchOrderStatus.push({ orderNo })
      const value = shiftResponse(responses.fetchOrderStatus, {
        orderStatus: 'CREATED',
        payStatus: 'UNPAID',
      })
      if (value instanceof Error) {
        throw value
      }
      return value
    },
    async cancelOrder(orderNo) {
      calls.cancelOrder.push({ orderNo })
      const value = shiftResponse(responses.cancelOrder, { success: true })
      if (value instanceof Error) {
        throw value
      }
      return value
    },
  }
}

function createPackageCards() {
  return [
    {
      targetCalls: 20,
      title: '20 次',
      missing: false,
      purchasable: true,
      product: {
        productId: 20001,
        skuId: 21001,
        price: 10,
      },
    },
    {
      targetCalls: 100,
      title: '100 次',
      missing: false,
      purchasable: true,
      product: {
        productId: 20002,
        skuId: 21002,
        price: 29.9,
      },
    },
    {
      targetCalls: 500,
      title: '500 次',
      missing: true,
      purchasable: false,
      product: null,
    },
  ]
}

test('选择套餐后可创建订单并展示二维码摘要', async () => {
  const state = createInitialPurchaseDialogState()
  const orderService = createOrderServiceStub()
  const controller = createPurchaseDialogController({
    state,
    orderService,
    now: () => new Date('2026-04-03T09:00:00Z').getTime(),
    idempotencyKeyFactory: () => 'idem-fixed-001',
  })

  controller.open({
    packageCards: createPackageCards(),
    preferredCalls: 100,
  })
  const result = await controller.createOrder()

  assert.equal(result.ok, true)
  assert.equal(state.orderNo, 'ORDER-001')
  assert.equal(state.payableAmount, 29.9)
  assert.equal(state.selectedCard?.targetCalls, 100)
  assert.equal(state.qrcodeUrl, 'https://pay.example.com/WECHAT/qr.png')
  assert.equal(orderService.calls.createOrder.length, 1)
  assert.deepEqual(orderService.calls.createOrder[0], {
    skuId: 21002,
    sourceChannel: 'DESKTOP',
    idempotencyKey: 'idem-fixed-001',
  })
})

test('切换支付渠道会重新拉取对应二维码', async () => {
  const state = createInitialPurchaseDialogState()
  const orderService = createOrderServiceStub()
  const controller = createPurchaseDialogController({
    state,
    orderService,
    now: () => new Date('2026-04-03T09:00:00Z').getTime(),
  })

  controller.open({ packageCards: createPackageCards() })
  await controller.createOrder()
  orderService.responses.fetchPayQrcode = {
    payChannel: 'ALIPAY',
    qrcodeUrl: 'https://pay.example.com/ALIPAY/qr.png',
    expireTime: '2099-01-01T00:00:00Z',
  }

  const switched = await controller.switchPayChannel('ALIPAY')

  assert.equal(switched.ok, true)
  assert.equal(state.payChannel, 'ALIPAY')
  assert.equal(state.qrcodeUrl, 'https://pay.example.com/ALIPAY/qr.png')
  assert.equal(orderService.calls.fetchPayQrcode.at(-1).payChannel, 'ALIPAY')
})

test('创建订单失败时保留套餐选择并提示错误', async () => {
  const state = createInitialPurchaseDialogState()
  const orderService = createOrderServiceStub()
  orderService.responses.createOrder = new Error('order failed')
  const controller = createPurchaseDialogController({
    state,
    orderService,
  })

  controller.open({ packageCards: createPackageCards(), preferredCalls: 20 })
  const result = await controller.createOrder()

  assert.equal(result.ok, false)
  assert.equal(state.visible, true)
  assert.equal(state.selectedCard?.targetCalls, 20)
  assert.match(state.errorMessage, /order failed/)
  assert.equal(state.orderNo, '')
})

test('二维码过期时标记过期并支持刷新', async () => {
  const state = createInitialPurchaseDialogState()
  const orderService = createOrderServiceStub()
  const controller = createPurchaseDialogController({
    state,
    orderService,
    now: () => new Date('2026-04-03T09:00:00Z').getTime(),
  })

  controller.open({ packageCards: createPackageCards(), preferredCalls: 20 })
  orderService.responses.fetchPayQrcode = {
    payChannel: 'WECHAT',
    qrcodeUrl: 'https://pay.example.com/expired.png',
    expireTime: '2026-04-03T08:59:00Z',
  }
  await controller.createOrder()

  assert.equal(state.qrcodeExpired, true)

  orderService.responses.fetchPayQrcode = {
    payChannel: 'WECHAT',
    qrcodeUrl: 'https://pay.example.com/refreshed.png',
    expireTime: '2026-04-03T09:30:00Z',
  }
  const refreshed = await controller.refreshQrcode()

  assert.equal(refreshed.ok, true)
  assert.equal(state.qrcodeExpired, false)
  assert.equal(state.qrcodeUrl, 'https://pay.example.com/refreshed.png')
})

test('轮询检测到支付成功后进入 success 并触发到账回调', async () => {
  const state = createInitialPurchaseDialogState()
  const orderService = createOrderServiceStub()
  const callbacks = []
  const controller = createPurchaseDialogController({
    state,
    orderService,
    onPaymentSuccess: async (payload) => {
      callbacks.push(payload)
    },
  })

  controller.open({ packageCards: createPackageCards(), preferredCalls: 20 })
  await controller.createOrder()
  orderService.responses.fetchOrderStatus = [
    { orderStatus: 'CREATED', payStatus: 'UNPAID' },
    { orderStatus: 'PAID', payStatus: 'SUCCESS' },
  ]

  await controller.pollOrderStatusOnce()
  assert.equal(state.pollingStatus, 'polling')
  await controller.pollOrderStatusOnce()

  assert.equal(state.pollingStatus, 'success')
  assert.equal(state.pollingActive, false)
  assert.equal(callbacks.length, 1)
  assert.equal(callbacks[0].orderNo, 'ORDER-001')
})

test('轮询达到上限后进入 timeout 状态', async () => {
  const state = createInitialPurchaseDialogState()
  const orderService = createOrderServiceStub()
  const controller = createPurchaseDialogController({
    state,
    orderService,
  })

  controller.open({ packageCards: createPackageCards(), preferredCalls: 20 })
  await controller.createOrder()
  orderService.responses.fetchOrderStatus = [
    { orderStatus: 'CREATED', payStatus: 'UNPAID' },
    { orderStatus: 'CREATED', payStatus: 'UNPAID' },
    { orderStatus: 'CREATED', payStatus: 'UNPAID' },
  ]
  controller.startPolling({ maxAttempts: 2, intervalMs: 2000 })

  await controller.pollOrderStatusOnce()
  await controller.pollOrderStatusOnce()

  assert.equal(state.pollingStatus, 'timeout')
  assert.equal(state.pollingActive, false)
  assert.match(state.pollingMessage, /超时/)
})

test('轮询异常时进入 error，用户可取消支付轮询', async () => {
  const state = createInitialPurchaseDialogState()
  const orderService = createOrderServiceStub()
  const controller = createPurchaseDialogController({
    state,
    orderService,
  })

  controller.open({ packageCards: createPackageCards(), preferredCalls: 20 })
  await controller.createOrder()
  orderService.responses.fetchOrderStatus = new Error('poll failed')
  controller.startPolling({ maxAttempts: 5, intervalMs: 2000 })

  await controller.pollOrderStatusOnce()

  assert.equal(state.pollingStatus, 'error')
  assert.equal(state.pollingActive, false)
  assert.match(state.pollingMessage, /poll failed/)

  controller.startPolling({ maxAttempts: 5, intervalMs: 2000 })
  const cancelled = await controller.cancelPayment()
  assert.equal(cancelled.ok, true)
  assert.equal(state.pollingStatus, 'cancelled')
  assert.equal(state.pollingActive, false)
  assert.equal(orderService.calls.cancelOrder.length, 1)
})
