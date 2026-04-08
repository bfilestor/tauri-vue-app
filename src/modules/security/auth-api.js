import { createAuthSessionStore } from './session-store.js'

function createAuthError(message, code) {
  const error = new Error(message)
  error.code = code
  return error
}

function normalizeAuthPayload(payload) {
  if (!payload || typeof payload !== 'object') {
    throw createAuthError('Invalid auth response payload.', 'AUTH_PAYLOAD_INVALID')
  }

  if (!payload.accessToken || !payload.refreshToken) {
    throw createAuthError('Missing auth token field in response.', 'AUTH_TOKEN_MISSING')
  }

  const userId = payload.userId ?? payload.userInfo?.userId ?? null
  const userInfo = payload.userInfo && typeof payload.userInfo === 'object'
    ? payload.userInfo
    : (userId != null ? { userId } : null)

  return {
    accessToken: payload.accessToken,
    refreshToken: payload.refreshToken,
    expireIn: payload.expireIn,
    userId,
    userInfo,
  }
}

export function createAuthApi({ client, storage, autoBindRefresh = true } = {}) {
  if (!client || typeof client.post !== 'function') {
    throw createAuthError('createAuthApi requires a request client.', 'AUTH_CLIENT_REQUIRED')
  }

  const sessionStore = createAuthSessionStore(storage ?? client.storage)

  async function initializeDeviceCredentialAfterAuth() {
    if (typeof client.initialize !== 'function') {
      return
    }

    try {
      await client.initialize()
    } catch (error) {
      console.warn('[auth] device activation after auth failed', error)
    }
  }

  async function loginByPassword(payload) {
    const response = await client.post('/app-api/auth/login/password', payload, {}, { skipAuthRefresh: true })
    const nextSession = sessionStore.setAuthenticatedSession(normalizeAuthPayload(response), {
      trialGiftPending: false,
    })
    await initializeDeviceCredentialAfterAuth()

    return {
      ...response,
      session: nextSession,
    }
  }

  async function registerByEmail(payload) {
    const response = await client.post('/app-api/auth/register/email', payload, {}, { skipAuthRefresh: true })
    const nextSession = sessionStore.setAuthenticatedSession(normalizeAuthPayload(response), {
      trialGiftPending: true,
    })
    await initializeDeviceCredentialAfterAuth()

    return {
      ...response,
      session: nextSession,
    }
  }

  async function refreshSessionToken(refreshToken) {
    const state = sessionStore.getSessionState()
    const resolvedRefreshToken = refreshToken || state.refreshToken

    if (!resolvedRefreshToken) {
      throw createAuthError('Missing refresh token.', 'REFRESH_TOKEN_MISSING')
    }

    const response = await client.post(
      '/app-api/auth/refresh-token',
      { refreshToken: resolvedRefreshToken },
      {},
      { skipAuthRefresh: true },
    )
    const nextSession = sessionStore.setAuthenticatedSession(normalizeAuthPayload(response), {
      keepExistingUserInfo: true,
      trialGiftPending: state.trialGiftPending,
    })

    return {
      ...response,
      session: nextSession,
    }
  }

  async function logout() {
    const state = sessionStore.getSessionState()

    try {
      if (state.accessToken) {
        await client.post('/app-api/auth/logout', undefined, {}, {
          requiresAuth: true,
          skipAuthRefresh: true,
        })
      }
    } finally {
      sessionStore.clearAuthSession()
    }
  }

  function enterGuestMode() {
    return sessionStore.enterGuestMode()
  }

  function leaveGuestMode() {
    return sessionStore.exitGuestMode()
  }

  function markTrialGiftConsumed() {
    return sessionStore.setTrialGiftPending(false)
  }

  function getSessionState() {
    return sessionStore.getSessionState()
  }

  if (autoBindRefresh && typeof client.configure === 'function') {
    client.configure({
      refreshAccessToken: async () => {
        const result = await refreshSessionToken()
        return result.accessToken
      },
      onAuthRefreshFailure: () => {
        sessionStore.clearAuthSession()
      },
    })
  }

  return {
    loginByPassword,
    registerByEmail,
    refreshSessionToken,
    logout,
    enterGuestMode,
    leaveGuestMode,
    markTrialGiftConsumed,
    getSessionState,
    sessionStore,
  }
}
