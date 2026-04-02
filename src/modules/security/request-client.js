import {
  SECURITY_STORAGE_KEYS,
  SIGNATURE_MODE,
} from './constants.js'
import {
  createNonce,
  createTraceId,
  ensureDeviceReady,
  getSecurityState,
  matchHighValueRoute,
  signRequest,
} from './device-security.js'
import { createStorageAdapter } from './storage.js'

const DEFAULT_UNAUTHORIZED_CODES = new Set([
  401,
  '401',
  'HTTP_401',
  'UNAUTHORIZED',
  'ACCESS_TOKEN_EXPIRED',
  'TOKEN_EXPIRED',
  'TOKEN_INVALID',
])

function isJsonLikeBody(value) {
  if (value == null) {
    return false
  }

  if (typeof value === 'string') {
    return false
  }

  if (value instanceof FormData) {
    return false
  }

  if (value instanceof URLSearchParams) {
    return false
  }

  if (typeof Blob !== 'undefined' && value instanceof Blob) {
    return false
  }

  return typeof value === 'object'
}

function buildAbsoluteUrl(baseUrl, requestPath) {
  if (!baseUrl) {
    return requestPath
  }

  return new URL(requestPath, baseUrl).toString()
}

function resolveRequestPathname(requestPath) {
  const placeholderBaseUrl = 'https://health-monitor.local'
  const url = requestPath.startsWith('http://') || requestPath.startsWith('https://')
    ? new URL(requestPath)
    : new URL(requestPath, placeholderBaseUrl)

  return url.pathname
}

function isAuthRefreshRequest(requestPath) {
  return resolveRequestPathname(requestPath) === '/app-api/auth/refresh-token'
}

function isUnauthorizedError(response, payload, error) {
  if (response?.status === 401) {
    return true
  }

  if (payload && typeof payload === 'object' && DEFAULT_UNAUTHORIZED_CODES.has(payload.code)) {
    return true
  }

  return DEFAULT_UNAUTHORIZED_CODES.has(error?.code)
}

function shouldAttemptAuthRefresh({ config, response, payload, error, requestPath, meta, retried }) {
  if (retried || meta.skipAuthRefresh) {
    return false
  }

  if (isAuthRefreshRequest(requestPath) || typeof config.refreshAccessToken !== 'function') {
    return false
  }

  if (typeof config.shouldRefreshAuth === 'function') {
    return Boolean(config.shouldRefreshAuth({ response, payload, error, requestPath, meta }))
  }

  return isUnauthorizedError(response, payload, error)
}

async function readResponseBody(response) {
  const rawText = await response.text()

  if (!rawText) {
    return null
  }

  try {
    return JSON.parse(rawText)
  } catch {
    return rawText
  }
}

function normalizeResponseError(response, payload) {
  const error = new Error(payload?.msg || payload?.message || `Request failed with ${response.status}`)
  error.code = payload?.code || `HTTP_${response.status}`
  error.payload = payload
  return error
}

export function createRequestClient(initialConfig = {}) {
  const storage = createStorageAdapter(initialConfig.storage)
  const config = {
    fetchImpl: globalThis.fetch?.bind(globalThis),
    baseUrl: '',
    runtimeInfo: {
      appVersion: '1.0.0',
    },
    activationSecretProof: 'init-secret-proof',
    compatibilityErrorCodes: undefined,
    signatureMode: SIGNATURE_MODE.required,
    nonceFactory: createNonce,
    traceIdFactory: createTraceId,
    now: () => Date.now(),
    refreshAccessToken: undefined,
    onAuthRefreshFailure: undefined,
    shouldRefreshAuth: undefined,
    ...initialConfig,
  }

  function configure(nextConfig = {}) {
    if (nextConfig.runtimeInfo) {
      config.runtimeInfo = {
        ...config.runtimeInfo,
        ...nextConfig.runtimeInfo,
      }
    }

    Object.assign(config, {
      ...nextConfig,
      runtimeInfo: config.runtimeInfo,
    })
  }

  async function initialize(runtimeInfo = {}) {
    if (Object.keys(runtimeInfo).length > 0) {
      configure({ runtimeInfo })
    }

    return ensureDeviceReady({
      storage,
      fetchImpl: config.fetchImpl,
      baseUrl: config.baseUrl,
      runtimeInfo: config.runtimeInfo,
      now: config.now,
      nonceFactory: config.nonceFactory,
      activationSecretProof: config.activationSecretProof,
      activateSignatureFactory: config.activateSignatureFactory,
      compatibilityErrorCodes: config.compatibilityErrorCodes,
      signatureMode: config.signatureMode,
    })
  }

  async function executeRequest(requestPath, init = {}, meta = {}, context = { retried: false }) {
    const method = (init.method || 'GET').toUpperCase()
    const fetchImpl = config.fetchImpl

    if (typeof fetchImpl !== 'function') {
      throw new Error('Missing fetch implementation.')
    }

    const highValue = meta.highValue ?? matchHighValueRoute(requestPath)
    if (highValue) {
      await initialize()
    }

    const state = getSecurityState(storage)
    const headers = new Headers(init.headers || {})
    const traceId = headers.get('X-Trace-Id') || config.traceIdFactory()
    headers.set('X-Trace-Id', traceId)

    const token = state.accessToken || storage.getString(SECURITY_STORAGE_KEYS.accessToken)
    if (token) {
      headers.set('Authorization', `Bearer ${token}`)
    } else if (meta.requiresAuth) {
      const error = new Error('Missing access token.')
      error.code = 'ACCESS_TOKEN_MISSING'
      throw error
    }

    let body = init.body
    let bodyString = ''

    if (isJsonLikeBody(body)) {
      bodyString = JSON.stringify(body)
      body = bodyString
      if (!headers.has('Content-Type')) {
        headers.set('Content-Type', 'application/json')
      }
    } else if (typeof body === 'string') {
      bodyString = body
      if (!headers.has('Content-Type')) {
        headers.set('Content-Type', 'application/json')
      }
    } else if (body instanceof URLSearchParams) {
      bodyString = body.toString()
      body = bodyString
      if (!headers.has('Content-Type')) {
        headers.set('Content-Type', 'application/x-www-form-urlencoded;charset=UTF-8')
      }
    }

    if (meta.idempotent || meta.idempotencyKey) {
      headers.set('Idempotency-Key', meta.idempotencyKey || `${meta.idempotencyPrefix || 'req'}-${config.now()}`)
    }

    if (highValue && state.deviceSecret) {
      const timestamp = config.now()
      const nonce = config.nonceFactory()
      const { signature } = await signRequest({
        secret: state.deviceSecret,
        method,
        requestPath,
        body: bodyString,
        timestamp,
        nonce,
        clientId: state.clientId,
        deviceId: state.deviceId,
      })

      headers.set('X-Client-Id', state.clientId)
      headers.set('X-Device-Id', state.deviceId)
      headers.set('X-Timestamp', String(timestamp))
      headers.set('X-Nonce', nonce)
      headers.set('X-Signature', signature)
      headers.set('X-App-Version', config.runtimeInfo.appVersion || '1.0.0')
    } else if (highValue && state.signatureMode !== SIGNATURE_MODE.compat) {
      const error = new Error('Signed request requires an active device credential.')
      error.code = 'DEVICE_CREDENTIAL_MISSING'
      throw error
    }

    const response = await fetchImpl(buildAbsoluteUrl(config.baseUrl, requestPath), {
      ...init,
      method,
      headers,
      body,
    })
    const payload = await readResponseBody(response)
    const hasBusinessError = payload && typeof payload === 'object' && 'code' in payload && payload.code !== 200

    if (!response.ok || hasBusinessError) {
      const error = normalizeResponseError(response, payload)

      if (shouldAttemptAuthRefresh({
        config,
        response,
        payload,
        error,
        requestPath,
        meta,
        retried: context.retried,
      })) {
        try {
          await config.refreshAccessToken({
            requestPath,
            method,
            meta,
            payload,
            responseStatus: response.status,
            storage,
          })

          return executeRequest(requestPath, init, meta, { retried: true })
        } catch (refreshError) {
          if (typeof config.onAuthRefreshFailure === 'function') {
            await config.onAuthRefreshFailure({
              requestPath,
              method,
              meta,
              payload,
              responseStatus: response.status,
              error,
              refreshError,
              storage,
            })
          }
        }
      }

      throw error
    }

    if (meta.unwrapData === false) {
      return payload
    }

    return payload && typeof payload === 'object' && 'data' in payload
      ? payload.data
      : payload
  }

  return {
    configure,
    initialize,
    request(requestPath, init = {}, meta = {}) {
      return executeRequest(requestPath, init, meta)
    },
    get(requestPath, init = {}, meta = {}) {
      return executeRequest(requestPath, { ...init, method: 'GET' }, meta)
    },
    post(requestPath, body, init = {}, meta = {}) {
      return executeRequest(requestPath, { ...init, method: 'POST', body }, meta)
    },
    storage,
  }
}
