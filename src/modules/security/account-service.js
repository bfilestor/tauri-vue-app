import { appRequestClient } from './bootstrap.js'
import { getAuthApi } from './auth-service.js'
import { createAccountContextService } from './account-context-service.js'

let sharedAccountContextService = null

function normalizeOwnerUserId(value) {
  if (value == null) {
    return ''
  }

  const text = String(value).trim()
  return text
}

function resolveOwnerUserId(authApi) {
  const session = typeof authApi?.getSessionState === 'function'
    ? (authApi.getSessionState() || {})
    : {}

  return normalizeOwnerUserId(
    session?.userId
    ?? session?.userInfo?.userId,
  )
}

async function resolveTauriInvoke() {
  try {
    const tauriCore = await import('@tauri-apps/api/core')
    return typeof tauriCore?.invoke === 'function' ? tauriCore.invoke : null
  } catch {
    return null
  }
}

function isUnavailableLocalCommandError(error) {
  const message = String(error?.message || '')
  return (
    message.includes('__TAURI_INTERNALS__')
    || message.includes('window is not defined')
    || message.includes('Cannot find module')
    || message.includes('Cannot find package')
  )
}

function createMemberRepository(client, authApi) {
  async function invokeLocal(command, payload) {
    const invoke = await resolveTauriInvoke()
    if (!invoke) {
      return { handled: false, result: null }
    }

    try {
      const result = await invoke(command, payload)
      return { handled: true, result }
    } catch (error) {
      if (isUnavailableLocalCommandError(error)) {
        return { handled: false, result: null }
      }
      throw error
    }
  }

  async function fallbackRequest(method, path, body) {
    if (method === 'GET') {
      return client.get(path, {}, { requiresAuth: true, includeUserId: true })
    }
    if (method === 'POST' && typeof client.post === 'function') {
      return client.post(path, body, {}, { requiresAuth: true, includeUserId: true })
    }
    if (typeof client.request === 'function') {
      return client.request(path, { method, body }, { requiresAuth: true, includeUserId: true })
    }
    throw new Error(`Unsupported request method: ${method}`)
  }

  function resolveOwnerIdFromInput(ownerUserId) {
    const resolved = normalizeOwnerUserId(ownerUserId) || resolveOwnerUserId(authApi)
    if (!resolved) {
      throw new Error('Missing owner user id.')
    }
    return resolved
  }

  return {
    async listMembers({ ownerUserId } = {}) {
      const resolvedOwnerUserId = resolveOwnerIdFromInput(ownerUserId)
      const localResult = await invokeLocal('list_family_members', {
        ownerUserId: resolvedOwnerUserId,
      })
      if (localResult.handled) {
        return localResult.result
      }
      return fallbackRequest('GET', '/app-api/family-members')
    },
    async createMember({ ownerUserId, payload = {} } = {}) {
      const resolvedOwnerUserId = resolveOwnerIdFromInput(ownerUserId)
      const localResult = await invokeLocal('create_family_member', {
        ownerUserId: resolvedOwnerUserId,
        input: payload,
      })
      if (localResult.handled) {
        return localResult.result
      }
      return fallbackRequest('POST', '/app-api/family-members', payload)
    },
    async updateMember({ ownerUserId, memberId, payload = {} } = {}) {
      const resolvedOwnerUserId = resolveOwnerIdFromInput(ownerUserId)
      const localResult = await invokeLocal('update_family_member', {
        ownerUserId: resolvedOwnerUserId,
        memberId: String(memberId ?? ''),
        input: payload,
      })
      if (localResult.handled) {
        return localResult.result
      }
      return fallbackRequest('PUT', `/app-api/family-members/${memberId}`, payload)
    },
    async deleteMember({ ownerUserId, memberId } = {}) {
      const resolvedOwnerUserId = resolveOwnerIdFromInput(ownerUserId)
      const localResult = await invokeLocal('delete_family_member', {
        ownerUserId: resolvedOwnerUserId,
        memberId: String(memberId ?? ''),
      })
      if (localResult.handled) {
        return localResult.result
      }
      return fallbackRequest('DELETE', `/app-api/family-members/${memberId}`)
    },
    async setDefaultMember({ ownerUserId, memberId } = {}) {
      const resolvedOwnerUserId = resolveOwnerIdFromInput(ownerUserId)
      const localResult = await invokeLocal('set_default_family_member', {
        ownerUserId: resolvedOwnerUserId,
        memberId: String(memberId ?? ''),
      })
      if (localResult.handled) {
        return localResult.result
      }
      return fallbackRequest('PUT', `/app-api/family-members/${memberId}/set-default`)
    },
  }
}

export function getAccountContextService() {
  if (!sharedAccountContextService) {
    const authApi = getAuthApi()
    sharedAccountContextService = createAccountContextService({
      client: appRequestClient,
      authApi,
      memberRepository: createMemberRepository(appRequestClient, authApi),
    })
  }

  return sharedAccountContextService
}
