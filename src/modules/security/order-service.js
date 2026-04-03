function normalizePayChannel(channel) {
  if (!channel) {
    return 'WECHAT'
  }

  const value = String(channel).trim().toUpperCase()
  if (value === 'ALIPAY') {
    return 'ALIPAY'
  }

  return 'WECHAT'
}

function buildQrcodePath(orderNo, payChannel) {
  const channel = normalizePayChannel(payChannel)
  const query = new URLSearchParams({ payChannel: channel })
  return `/app-api/orders/${encodeURIComponent(orderNo)}/pay-qrcode?${query.toString()}`
}

function normalizeOrderResponse(response) {
  return {
    orderNo: response?.orderNo || '',
    orderStatus: response?.orderStatus || '',
    payableAmount: response?.payableAmount ?? response?.amount ?? 0,
    raw: response || null,
  }
}

function normalizeQrcodeResponse(response, payChannel) {
  return {
    payChannel: normalizePayChannel(response?.payChannel || payChannel),
    qrcodeUrl: response?.qrcodeUrl || response?.qrCodeUrl || '',
    expireTime: response?.expireTime || response?.expiredAt || '',
    raw: response || null,
  }
}

function normalizeOrderStatusResponse(response) {
  return {
    orderStatus: response?.orderStatus || '',
    payStatus: response?.payStatus || '',
    raw: response || null,
  }
}

export function createOrderService({ client } = {}) {
  if (!client || typeof client.post !== 'function' || typeof client.get !== 'function') {
    throw new Error('createOrderService requires request client with get/post.')
  }

  async function createOrder({ skuId, sourceChannel = 'DESKTOP', idempotencyKey }) {
    const response = await client.post(
      '/app-api/orders',
      {
        skuId,
        sourceChannel,
        idempotencyKey,
      },
      {},
      {
        requiresAuth: true,
        includeUserId: true,
        idempotencyKey,
      },
    )

    return normalizeOrderResponse(response)
  }

  async function fetchPayQrcode(orderNo, payChannel = 'WECHAT') {
    const channel = normalizePayChannel(payChannel)
    const primaryPath = buildQrcodePath(orderNo, channel)
    try {
      const response = await client.get(primaryPath, {}, { requiresAuth: true, includeUserId: true })
      return normalizeQrcodeResponse(response, channel)
    } catch (error) {
      // Some backends infer pay channel from order and may reject query args.
      const fallbackPath = `/app-api/orders/${encodeURIComponent(orderNo)}/pay-qrcode`
      const response = await client.get(fallbackPath, {}, { requiresAuth: true, includeUserId: true })
      return normalizeQrcodeResponse(response, channel)
    }
  }

  async function fetchOrderStatus(orderNo) {
    const response = await client.get(
      `/app-api/orders/${encodeURIComponent(orderNo)}/status`,
      {},
      { requiresAuth: true, includeUserId: true },
    )

    return normalizeOrderStatusResponse(response)
  }

  async function cancelOrder(orderNo) {
    return client.post(
      `/app-api/orders/${encodeURIComponent(orderNo)}/cancel`,
      undefined,
      {},
      { requiresAuth: true, includeUserId: true, skipAuthRefresh: false },
    )
  }

  return {
    createOrder,
    fetchPayQrcode,
    fetchOrderStatus,
    cancelOrder,
  }
}
