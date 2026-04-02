import { createRequestClient } from './request-client.js'

export const appRequestClient = createRequestClient()

function detectOsName(platform) {
  if (/win/i.test(platform)) {
    return 'Windows'
  }

  if (/mac/i.test(platform)) {
    return 'macOS'
  }

  if (/linux/i.test(platform)) {
    return 'Linux'
  }

  return 'Desktop'
}

export function detectRuntimeInfo(appVersion = '1.0.0') {
  const platform = typeof navigator !== 'undefined'
    ? navigator.userAgentData?.platform || navigator.platform || 'Desktop'
    : 'Desktop'
  const osName = detectOsName(platform)

  return {
    appVersion,
    deviceName: `${platform} Desktop`,
    deviceType: /mac/i.test(osName) ? 'MAC' : 'PC',
    osName,
    osVersion: typeof navigator !== 'undefined' ? navigator.userAgent || 'unknown' : 'unknown',
  }
}

export async function bootstrapClientSecurity(options = {}) {
  const runtimeInfo = {
    ...detectRuntimeInfo(options.appVersion),
    ...(options.runtimeInfo || {}),
  }

  appRequestClient.configure({
    baseUrl: options.baseUrl || '',
    fetchImpl: options.fetchImpl,
    runtimeInfo,
    activationSecretProof: options.activationSecretProof || 'init-secret-proof',
    activateSignatureFactory: options.activateSignatureFactory,
    compatibilityErrorCodes: options.compatibilityErrorCodes,
    signatureMode: options.signatureMode,
    nonceFactory: options.nonceFactory,
    traceIdFactory: options.traceIdFactory,
    now: options.now,
  })

  try {
    return await appRequestClient.initialize()
  } catch (error) {
    console.warn('[security] bootstrap failed', error)
    return null
  }
}
