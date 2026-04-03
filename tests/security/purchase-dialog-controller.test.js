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
  }

  const responses = {
    createOrder: null,
    fetchPayQrcode: null,
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
