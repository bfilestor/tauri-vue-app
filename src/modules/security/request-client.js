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

function headersToObject(headers) {
  const result = {}

  if (!headers || typeof headers.forEach !== 'function') {
    return result
  }

  headers.forEach((value, key) => {
    result[key] = value
  })

  return result
}

function appendFieldValue(target, key, value) {
  if (key in target) {
    target[key] = Array.isArray(target[key]) ? [...target[key], value] : [target[key], value]
    return
  }

  target[key] = value
}

function normalizeFormDataValue(value) {
  if (typeof File !== 'undefined' && value instanceof File) {
    return `[File name=${value.name} size=${value.size} type=${value.type || 'unknown'}]`
  }

  if (typeof Blob !== 'undefined' && value instanceof Blob) {
    return `[Blob size=${value.size} type=${value.type || 'unknown'}]`
  }

  return value
}

function searchParamsToObject(searchParams) {
  const result = {}

  if (!searchParams || typeof searchParams.forEach !== 'function') {
    return result
  }

  searchParams.forEach((value, key) => {
    appendFieldValue(result, key, value)
  })

  return result
}

function parseQueryParamsFromUrl(requestUrl) {
  try {
    return searchParamsToObject(new URL(requestUrl, 'http://request.log').searchParams)
  } catch {
    return {}
  }
}

function parseStringAsJsonIfPossible(rawText) {
  if (typeof rawText !== 'string') {
    return rawText
  }

  const trimmed = rawText.trim()
  if (!trimmed) {
    return rawText
  }

  try {
    return JSON.parse(trimmed)
  } catch {
    return rawText
  }
}

function normalizeUserId(value) {
  if (value == null || value === '') {
    return null
  }

  if (typeof value === 'number' && Number.isFinite(value)) {
    return value
  }

  if (typeof value === 'string') {
    const trimmed = value.trim()
    if (!trimmed) {
      return null
    }

    const parsed = Number(trimmed)
    return Number.isFinite(parsed) ? parsed : trimmed
  }

  return null
}

function isAbsoluteRequestPath(requestPath) {
  return typeof requestPath === 'string'
    && (requestPath.startsWith('http://') || requestPath.startsWith('https://'))
}

function readStoredUserId(storage) {
  const directUserId = normalizeUserId(storage.getString(SECURITY_STORAGE_KEYS.userId))
  if (directUserId != null) {
    return directUserId
  }

  const userInfo = parseStringAsJsonIfPossible(storage.getString(SECURITY_STORAGE_KEYS.userInfo))
  return normalizeUserId(userInfo?.userId)
}

function appendUserIdToRequestPath(requestPath, userId) {
  if (userId == null || requestPath == null || requestPath === '') {
    return requestPath
  }

  const absolute = isAbsoluteRequestPath(requestPath)
  const url = absolute
    ? new URL(requestPath)
    : new URL(requestPath, 'http://request.user')

  if (!url.searchParams.has('userId')) {
    url.searchParams.set('userId', String(userId))
  }

  if (absolute) {
    return url.toString()
  }

  const normalizedPath = `${url.pathname}${url.search}${url.hash}`
  return requestPath.startsWith('/') ? normalizedPath : normalizedPath.replace(/^\//, '')
}

function normalizeRequestPayloadForLog(originalBody, bodyString) {
  if (originalBody == null) {
    if (typeof bodyString === 'string' && bodyString.length > 0) {
      return parseStringAsJsonIfPossible(bodyString)
    }
    return null
  }

  if (originalBody instanceof FormData) {
    const result = {}
    originalBody.forEach((value, key) => {
      appendFieldValue(result, key, normalizeFormDataValue(value))
    })
    return result
  }

  if (originalBody instanceof URLSearchParams) {
    return searchParamsToObject(originalBody)
  }

  if (typeof originalBody === 'string') {
    return parseStringAsJsonIfPossible(originalBody)
  }

  if (typeof originalBody === 'object') {
    return originalBody
  }

  return originalBody
}

function getDefinedConfigPatch(nextConfig = {}) {
  const patch = {}

  Object.entries(nextConfig).forEach(([key, value]) => {
    if (value !== undefined && key !== 'runtimeInfo') {
      patch[key] = value
    }
  })

  return patch
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

    const definedConfigPatch = getDefinedConfigPatch(nextConfig)

    Object.assign(config, {
      ...definedConfigPatch,
      runtimeInfo: config.runtimeInfo,
    })
  }

  function resolveFetchImpl() {
    if (typeof config.fetchImpl === 'function') {
      return config.fetchImpl
    }

    if (typeof globalThis.fetch === 'function') {
      return globalThis.fetch.bind(globalThis)
    }

    return undefined
  }

  function createMissingFetchError(extra = {}) {
    const error = new Error('Missing fetch implementation.')
    error.code = 'FETCH_IMPL_MISSING'
    console.error('[request-client] Request aborted: fetch implementation missing', extra)
    return error
  }

  async function initialize(runtimeInfo = {}) {
    if (Object.keys(runtimeInfo).length > 0) {
      configure({ runtimeInfo })
    }

    const fetchImpl = resolveFetchImpl()
    if (typeof fetchImpl !== 'function') {
      throw createMissingFetchError({
        stage: 'initialize',
      })
    }

    const activationAuth = {
      accessToken: storage.getString(SECURITY_STORAGE_KEYS.accessToken),
      userId: readStoredUserId(storage),
    }

    return ensureDeviceReady({
      storage,
      fetchImpl,
      baseUrl: config.baseUrl,
      runtimeInfo: config.runtimeInfo,
      now: config.now,
      nonceFactory: config.nonceFactory,
      activationSecretProof: config.activationSecretProof,
      activateSignatureFactory: config.activateSignatureFactory,
      activationAuth,
      compatibilityErrorCodes: config.compatibilityErrorCodes,
      signatureMode: config.signatureMode,
    })
  }

  async function executeRequest(requestPath, init = {}, meta = {}, context = { retried: false }) {
    const includeUserId = meta.includeUserId === true
    const currentUserId = includeUserId ? readStoredUserId(storage) : null
    if (includeUserId && currentUserId == null) {
      const error = new Error('Missing user id.')
      error.code = 'USER_ID_MISSING'
      throw error
    }

    const effectiveRequestPath = includeUserId
      ? appendUserIdToRequestPath(requestPath, currentUserId)
      : requestPath
    const method = (init.method || 'GET').toUpperCase()
    const fetchImpl = resolveFetchImpl()

    const highValue = meta.highValue ?? matchHighValueRoute(effectiveRequestPath)
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
    const originalBody = init.body
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
        requestPath: effectiveRequestPath,
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

    const requestUrl = buildAbsoluteUrl(config.baseUrl, effectiveRequestPath)
    const requestPayload = normalizeRequestPayloadForLog(originalBody, bodyString)
    const requestHeaders = headersToObject(headers)
    const requestQuery = parseQueryParamsFromUrl(requestUrl)

    if (typeof fetchImpl !== 'function') {
      throw createMissingFetchError({
        stage: 'executeRequest',
        url: requestUrl,
        method,
        headers: requestHeaders,
        query: requestQuery,
        params: requestPayload,
      })
    }

    console.info('[request-client] Request ->', {
      url: requestUrl,
      method,
      headers: requestHeaders,
      query: requestQuery,
      params: requestPayload,
      meta,
    })

    let response
    let payload

    try {
      response = await fetchImpl(requestUrl, {
        ...init,
        method,
        headers,
        body,
      })
      payload = await readResponseBody(response)
    } catch (error) {
      console.error('[request-client] Request exception <-', {
        url: requestUrl,
        method,
        headers: requestHeaders,
        query: requestQuery,
        params: requestPayload,
        error: {
          name: error?.name,
          code: error?.code,
          message: error?.message,
        },
      })
      throw error
    }

    console.info('[request-client] Response <-', {
      url: requestUrl,
      method,
      status: response.status,
      ok: response.ok,
      headers: headersToObject(response.headers),
      data: payload,
    })

    const hasBusinessError = payload && typeof payload === 'object' && 'code' in payload && payload.code !== 200

    if (!response.ok || hasBusinessError) {
      const error = normalizeResponseError(response, payload)

      if (shouldAttemptAuthRefresh({
        config,
        response,
        payload,
        error,
        requestPath: effectiveRequestPath,
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
