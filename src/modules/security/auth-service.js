import { createAuthApi } from './auth-api.js'
import { appRequestClient, detectRuntimeInfo } from './bootstrap.js'
import { SECURITY_STORAGE_KEYS } from './constants.js'

let sharedAuthApi = null

function readUserAgent() {
  if (typeof navigator === 'undefined') {
    return 'unknown'
  }

  return navigator.userAgent || 'unknown'
}

export function getAuthApi() {
  if (!sharedAuthApi) {
    sharedAuthApi = createAuthApi({
      client: appRequestClient,
      storage: appRequestClient.storage,
    })
  }

  return sharedAuthApi
}

export async function resolveAuthDeviceContext() {
  try {
    await appRequestClient.initialize()
  } catch {
    // Keep graceful fallback for local/offline mode.
  }

  const runtime = detectRuntimeInfo()
  const storage = appRequestClient.storage

  return {
    deviceId: storage.getString(SECURITY_STORAGE_KEYS.deviceId) || '',
    deviceName: runtime.deviceName || 'Desktop',
    clientInfo: readUserAgent(),
  }
}
