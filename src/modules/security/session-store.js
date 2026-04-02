import { SECURITY_STORAGE_KEYS } from './constants.js'
import { createStorageAdapter } from './storage.js'

function safeParseJson(rawValue) {
  if (!rawValue) {
    return null
  }

  try {
    return JSON.parse(rawValue)
  } catch {
    return null
  }
}

function toBoolean(value) {
  return value === '1' || value === 'true'
}

function writeBoolean(storage, key, enabled) {
  storage.setString(key, enabled ? '1' : '')
}

function normalizeUserInfo(input) {
  return input && typeof input === 'object' ? input : null
}

export function createAuthSessionStore(storageInput) {
  const storage = createStorageAdapter(storageInput)

  function getSessionState() {
    const accessToken = storage.getString(SECURITY_STORAGE_KEYS.accessToken)
    const refreshToken = storage.getString(SECURITY_STORAGE_KEYS.refreshToken)

    return {
      accessToken,
      refreshToken,
      userInfo: safeParseJson(storage.getString(SECURITY_STORAGE_KEYS.userInfo)),
      isGuest: toBoolean(storage.getString(SECURITY_STORAGE_KEYS.guestMode)),
      trialGiftPending: toBoolean(storage.getString(SECURITY_STORAGE_KEYS.trialGiftPending)),
      isAuthenticated: Boolean(accessToken),
    }
  }

  function setAuthenticatedSession(payload = {}, options = {}) {
    const current = getSessionState()
    const userInfo = normalizeUserInfo(
      options.userInfo
      ?? payload.userInfo
      ?? (options.keepExistingUserInfo ? current.userInfo : null),
    )
    const trialGiftPending = options.trialGiftPending ?? current.trialGiftPending

    storage.setString(SECURITY_STORAGE_KEYS.accessToken, payload.accessToken || '')
    storage.setString(SECURITY_STORAGE_KEYS.refreshToken, payload.refreshToken || '')
    storage.setString(SECURITY_STORAGE_KEYS.userInfo, userInfo ? JSON.stringify(userInfo) : '')
    writeBoolean(storage, SECURITY_STORAGE_KEYS.guestMode, false)
    writeBoolean(storage, SECURITY_STORAGE_KEYS.trialGiftPending, Boolean(trialGiftPending))

    return getSessionState()
  }

  function clearAuthSession() {
    storage.remove(SECURITY_STORAGE_KEYS.accessToken)
    storage.remove(SECURITY_STORAGE_KEYS.refreshToken)
    storage.remove(SECURITY_STORAGE_KEYS.userInfo)
    writeBoolean(storage, SECURITY_STORAGE_KEYS.guestMode, false)
    writeBoolean(storage, SECURITY_STORAGE_KEYS.trialGiftPending, false)
    return getSessionState()
  }

  function enterGuestMode() {
    clearAuthSession()
    writeBoolean(storage, SECURITY_STORAGE_KEYS.guestMode, true)
    return getSessionState()
  }

  function exitGuestMode() {
    writeBoolean(storage, SECURITY_STORAGE_KEYS.guestMode, false)
    return getSessionState()
  }

  function setTrialGiftPending(enabled) {
    writeBoolean(storage, SECURITY_STORAGE_KEYS.trialGiftPending, Boolean(enabled))
    return getSessionState()
  }

  return {
    getSessionState,
    setAuthenticatedSession,
    clearAuthSession,
    enterGuestMode,
    exitGuestMode,
    setTrialGiftPending,
    storage,
  }
}
