import {
  APP_CLIENT_ID,
  DEFAULT_COMPATIBILITY_ERROR_CODES,
  HIGH_VALUE_ROUTE_PATTERNS,
  SECURITY_STORAGE_KEYS,
  SIGNATURE_MODE,
} from './constants.js'
import { createStorageAdapter } from './storage.js'

const textEncoder = new TextEncoder()

function toUint8Array(buffer) {
  return buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer)
}

function toBase64(buffer) {
  const bytes = toUint8Array(buffer)

  if (typeof btoa === 'function') {
    let binary = ''
    const chunkSize = 0x8000

    for (let index = 0; index < bytes.length; index += chunkSize) {
      binary += String.fromCharCode(...bytes.subarray(index, index + chunkSize))
    }

    return btoa(binary)
  }

  return Buffer.from(bytes).toString('base64')
}

function parseExpireTime(value) {
  if (value == null || value === '') {
    return 0
  }

  const numericValue = Number(value)
  if (Number.isFinite(numericValue)) {
    return numericValue
  }

  const timestamp = Date.parse(value)
  return Number.isFinite(timestamp) ? timestamp : 0
}

async function hmacSha256Base64(secret, rawValue) {
  const secretKey = await crypto.subtle.importKey(
    'raw',
    textEncoder.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign'],
  )
  const signature = await crypto.subtle.sign('HMAC', secretKey, textEncoder.encode(rawValue))
  return toBase64(signature)
}

async function safeReadJson(response) {
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

function buildRequestUrl(baseUrl, requestPath) {
  if (!baseUrl) {
    return requestPath
  }

  return new URL(requestPath, baseUrl).toString()
}

function normalizeErrorCode(payload, status) {
  if (payload && typeof payload === 'object') {
    if (typeof payload.code === 'string') {
      return payload.code
    }

    if (typeof payload.data?.code === 'string') {
      return payload.data.code
    }
  }

  return `HTTP_${status}`
}

function createServiceError(message, code, cause) {
  const error = new Error(message)
  error.code = code
  error.cause = cause
  return error
}

function resolveRequestPath(requestPath) {
  const placeholderBaseUrl = 'https://health-monitor.local'
  const url = requestPath.startsWith('http://') || requestPath.startsWith('https://')
    ? new URL(requestPath)
    : new URL(requestPath, placeholderBaseUrl)
  const sortedParams = [...url.searchParams.entries()].sort(([leftKey, leftValue], [rightKey, rightValue]) => {
    if (leftKey === rightKey) {
      return leftValue.localeCompare(rightValue)
    }

    return leftKey.localeCompare(rightKey)
  })
  const params = new URLSearchParams()

  for (const [key, value] of sortedParams) {
    params.append(key, value)
  }

  const queryString = params.toString()
  return `${url.pathname}${queryString ? `?${queryString}` : ''}`
}

function inferDeviceType(osName) {
  return /mac/i.test(osName) ? 'MAC' : 'PC'
}

function buildFingerprintSource(identity, runtimeInfo) {
  return JSON.stringify({
    clientId: identity.clientId,
    deviceId: identity.deviceId,
    deviceName: runtimeInfo.deviceName,
    deviceType: runtimeInfo.deviceType,
    osName: runtimeInfo.osName,
    osVersion: runtimeInfo.osVersion,
  })
}

function getCompatibleErrorCodes(inputCodes = DEFAULT_COMPATIBILITY_ERROR_CODES) {
  return new Set(inputCodes)
}

function persistCredential(storage, payload) {
  storage.setString(SECURITY_STORAGE_KEYS.deviceSecret, payload.deviceSecret || '')
  storage.setString(SECURITY_STORAGE_KEYS.privateKey, payload.privateKey || '')
  storage.setString(SECURITY_STORAGE_KEYS.credentialVersion, payload.credentialVersion || '')
  storage.setNumber(SECURITY_STORAGE_KEYS.credentialExpireTime, parseExpireTime(payload.expireTime))
  storage.setString(SECURITY_STORAGE_KEYS.signatureMode, SIGNATURE_MODE.required)
  storage.remove(SECURITY_STORAGE_KEYS.lastActivationError)
}

async function postJson({ fetchImpl, baseUrl, requestPath, payload, headers = {} }) {
  const response = await fetchImpl(buildRequestUrl(baseUrl, requestPath), {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...headers,
    },
    body: JSON.stringify(payload),
  })
  const responsePayload = await safeReadJson(response)

  if (!response.ok) {
    throw createServiceError(
      responsePayload?.msg || responsePayload?.message || `Request failed with ${response.status}`,
      normalizeErrorCode(responsePayload, response.status),
      responsePayload,
    )
  }

  if (responsePayload && typeof responsePayload === 'object' && 'code' in responsePayload && responsePayload.code !== 200) {
    throw createServiceError(
      responsePayload.msg || responsePayload.message || 'Request failed',
      normalizeErrorCode(responsePayload, response.status),
      responsePayload,
    )
  }

  if (responsePayload && typeof responsePayload === 'object' && 'data' in responsePayload) {
    return responsePayload.data
  }

  return responsePayload
}

export function createNonce() {
  return crypto.randomUUID().replaceAll('-', '')
}

export function createTraceId() {
  return createNonce()
}

export function createStableDeviceId() {
  return `desktop-${createNonce()}`
}

export function matchHighValueRoute(requestPath) {
  const normalizedPath = resolveRequestPath(requestPath)
  return HIGH_VALUE_ROUTE_PATTERNS.some((pattern) => pattern.test(normalizedPath))
}

export async function sha256Base64(rawValue = '') {
  const digest = await crypto.subtle.digest('SHA-256', textEncoder.encode(rawValue))
  return toBase64(digest)
}

export async function buildCanonicalString({
  method,
  requestPath,
  body = '',
  timestamp,
  nonce,
  clientId,
  deviceId,
}) {
  const bodyHash = await sha256Base64(body || '')

  return [
    method.toUpperCase(),
    resolveRequestPath(requestPath),
    bodyHash,
    String(timestamp),
    nonce,
    clientId,
    deviceId,
  ].join('\n')
}

export async function signRequest({
  secret,
  method,
  requestPath,
  body = '',
  timestamp,
  nonce,
  clientId,
  deviceId,
}) {
  const canonical = await buildCanonicalString({
    method,
    requestPath,
    body,
    timestamp,
    nonce,
    clientId,
    deviceId,
  })

  return {
    canonical,
    signature: await hmacSha256Base64(secret, canonical),
  }
}

export function getSecurityState(storageInput) {
  const storage = createStorageAdapter(storageInput)
  return {
    clientId: storage.getString(SECURITY_STORAGE_KEYS.clientId),
    deviceId: storage.getString(SECURITY_STORAGE_KEYS.deviceId),
    deviceSecret: storage.getString(SECURITY_STORAGE_KEYS.deviceSecret),
    privateKey: storage.getString(SECURITY_STORAGE_KEYS.privateKey),
    credentialVersion: storage.getString(SECURITY_STORAGE_KEYS.credentialVersion),
    credentialExpireTime: storage.getNumber(SECURITY_STORAGE_KEYS.credentialExpireTime, 0),
    signatureMode: storage.getString(SECURITY_STORAGE_KEYS.signatureMode, SIGNATURE_MODE.required),
    accessToken: storage.getString(SECURITY_STORAGE_KEYS.accessToken),
  }
}

export function hasActiveCredential(state, now = Date.now()) {
  if (!state.deviceSecret && !state.privateKey) {
    return false
  }

  if (!state.credentialExpireTime) {
    return true
  }

  return state.credentialExpireTime > now
}

export function ensureDeviceIdentity({
  storage: storageInput,
  defaultClientId = APP_CLIENT_ID,
  deviceIdFactory = createStableDeviceId,
}) {
  const storage = createStorageAdapter(storageInput)
  const clientId = storage.getString(SECURITY_STORAGE_KEYS.clientId) || defaultClientId
  const deviceId = storage.getString(SECURITY_STORAGE_KEYS.deviceId) || deviceIdFactory()

  storage.setString(SECURITY_STORAGE_KEYS.clientId, clientId)
  storage.setString(SECURITY_STORAGE_KEYS.deviceId, deviceId)

  return {
    clientId,
    deviceId,
  }
}

export async function createActivationPayload({
  storage: storageInput,
  runtimeInfo,
  now = () => Date.now(),
  nonceFactory = createNonce,
  activationSecretProof = 'init-secret-proof',
  activateSignatureFactory,
}) {
  const storage = createStorageAdapter(storageInput)
  const identity = ensureDeviceIdentity({ storage })
  const nonce = nonceFactory()
  const activateTime = now()
  const normalizedRuntimeInfo = {
    appVersion: runtimeInfo.appVersion || '1.0.0',
    deviceName: runtimeInfo.deviceName || 'Desktop Client',
    deviceType: runtimeInfo.deviceType || inferDeviceType(runtimeInfo.osName || ''),
    osName: runtimeInfo.osName || 'Desktop',
    osVersion: runtimeInfo.osVersion || 'unknown',
  }
  const payload = {
    clientId: identity.clientId,
    deviceId: identity.deviceId,
    deviceName: normalizedRuntimeInfo.deviceName,
    deviceType: normalizedRuntimeInfo.deviceType,
    osName: normalizedRuntimeInfo.osName,
    osVersion: normalizedRuntimeInfo.osVersion,
    appVersion: normalizedRuntimeInfo.appVersion,
    deviceFingerprint: await sha256Base64(buildFingerprintSource(identity, normalizedRuntimeInfo)),
    credentialMode: 'HMAC',
    secretProof: activationSecretProof,
    activateTime,
    nonce,
  }
  const signature = activateSignatureFactory
    ? await activateSignatureFactory({ payload, storage, runtimeInfo: normalizedRuntimeInfo })
    : 'activate-signature-placeholder'

  return {
    ...payload,
    signature,
  }
}

export async function activateDevice({
  storage: storageInput,
  fetchImpl,
  baseUrl,
  runtimeInfo,
  now = () => Date.now(),
  nonceFactory = createNonce,
  activationSecretProof,
  activateSignatureFactory,
}) {
  const storage = createStorageAdapter(storageInput)
  const activationPayload = await createActivationPayload({
    storage,
    runtimeInfo,
    now,
    nonceFactory,
    activationSecretProof,
    activateSignatureFactory,
  })
  const responseData = await postJson({
    fetchImpl,
    baseUrl,
    requestPath: '/app-api/client/activate',
    payload: activationPayload,
  })

  persistCredential(storage, responseData || {})
  return getSecurityState(storage)
}

export async function refreshCredential({
  storage: storageInput,
  fetchImpl,
  baseUrl,
  runtimeInfo,
  now = () => Date.now(),
  nonceFactory = createNonce,
}) {
  const storage = createStorageAdapter(storageInput)
  const state = getSecurityState(storage)
  const nonce = nonceFactory()
  const timestamp = now()
  const requestPath = '/app-api/client/refresh-credential'
  const payload = {
    clientId: state.clientId,
    deviceId: state.deviceId,
    credentialVersion: state.credentialVersion,
    appVersion: runtimeInfo.appVersion || '1.0.0',
    nonce,
    timestamp,
  }
  const body = JSON.stringify(payload)
  const { signature } = await signRequest({
    secret: state.deviceSecret,
    method: 'POST',
    requestPath,
    body,
    timestamp,
    nonce,
    clientId: state.clientId,
    deviceId: state.deviceId,
  })
  const responseData = await postJson({
    fetchImpl,
    baseUrl,
    requestPath,
    payload: {
      ...payload,
      signature,
    },
    headers: {
      'X-Client-Id': state.clientId,
      'X-Device-Id': state.deviceId,
      'X-Timestamp': String(timestamp),
      'X-Nonce': nonce,
      'X-Signature': signature,
      'X-App-Version': runtimeInfo.appVersion || '1.0.0',
    },
  })

  persistCredential(storage, responseData || {})
  return getSecurityState(storage)
}

export async function ensureDeviceReady({
  storage: storageInput,
  fetchImpl = globalThis.fetch?.bind(globalThis),
  baseUrl = '',
  runtimeInfo = {},
  now = () => Date.now(),
  nonceFactory = createNonce,
  deviceIdFactory = createStableDeviceId,
  activationSecretProof = 'init-secret-proof',
  activateSignatureFactory,
  compatibilityErrorCodes = DEFAULT_COMPATIBILITY_ERROR_CODES,
  signatureMode = SIGNATURE_MODE.required,
}) {
  const storage = createStorageAdapter(storageInput)
  ensureDeviceIdentity({ storage, deviceIdFactory })

  if (signatureMode === SIGNATURE_MODE.compat) {
    storage.setString(SECURITY_STORAGE_KEYS.signatureMode, SIGNATURE_MODE.compat)
    return getSecurityState(storage)
  }

  const state = getSecurityState(storage)
  if (hasActiveCredential(state, now())) {
    return state
  }

  if (typeof fetchImpl !== 'function') {
    throw createServiceError('Missing fetch implementation for security bootstrap.', 'FETCH_IMPL_MISSING')
  }

  try {
    if (state.deviceSecret && state.credentialVersion) {
      return await refreshCredential({
        storage,
        fetchImpl,
        baseUrl,
        runtimeInfo,
        now,
        nonceFactory,
      })
    }

    return await activateDevice({
      storage,
      fetchImpl,
      baseUrl,
      runtimeInfo,
      now,
      nonceFactory,
      activationSecretProof,
      activateSignatureFactory,
    })
  } catch (error) {
    const compatibleCodes = getCompatibleErrorCodes(compatibilityErrorCodes)
    if (compatibleCodes.has(error.code)) {
      storage.setString(SECURITY_STORAGE_KEYS.signatureMode, SIGNATURE_MODE.compat)
      storage.setString(SECURITY_STORAGE_KEYS.lastActivationError, error.code)
      return getSecurityState(storage)
    }

    throw error
  }
}
