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

function createMemberRepository(client, authApi) {
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
      resolveOwnerIdFromInput(ownerUserId)
      return fallbackRequest('GET', '/app-api/family-members')
    },
    async createMember({ ownerUserId, payload = {} } = {}) {
      resolveOwnerIdFromInput(ownerUserId)
      return fallbackRequest('POST', '/app-api/family-members', payload)
    },
    async updateMember({ ownerUserId, memberId, payload = {} } = {}) {
      resolveOwnerIdFromInput(ownerUserId)
      return fallbackRequest('PUT', `/app-api/family-members/${memberId}`, payload)
    },
    async deleteMember({ ownerUserId, memberId } = {}) {
      resolveOwnerIdFromInput(ownerUserId)
      return fallbackRequest('DELETE', `/app-api/family-members/${memberId}`)
    },
    async setDefaultMember({ ownerUserId, memberId } = {}) {
      resolveOwnerIdFromInput(ownerUserId)
      return fallbackRequest('PUT', `/app-api/family-members/${memberId}/set-default`)
    },
  }
}

export { createMemberRepository }

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
