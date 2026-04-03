const DEFAULT_USAGE_TYPE = 'CHAT'

function toFiniteNumber(value, fallback = 0) {
  if (typeof value === 'number' && Number.isFinite(value)) {
    return value
  }

  if (typeof value === 'string' && value.trim()) {
    const parsed = Number(value)
    if (Number.isFinite(parsed)) {
      return parsed
    }
  }

  return fallback
}

function normalizeUsageType(usageType) {
  if (typeof usageType !== 'string' || usageType.trim() === '') {
    return DEFAULT_USAGE_TYPE
  }

  return usageType.trim().toUpperCase()
}

function resolveAllowed(payload) {
  if (typeof payload?.canUse === 'boolean') {
    return payload.canUse
  }

  if (typeof payload?.allowed === 'boolean') {
    return payload.allowed
  }

  return false
}

export function createUsageService({
  client,
  idempotencyKeyFactory = () => `usage-${Date.now()}`,
} = {}) {
  if (!client || typeof client.post !== 'function') {
    throw new Error('createUsageService requires a request client.')
  }

  return {
    async precheck({ memberId, usageType = DEFAULT_USAGE_TYPE } = {}) {
      const normalizedUsageType = normalizeUsageType(usageType)
      const idempotencyKey = idempotencyKeyFactory(normalizedUsageType)
      const payload = await client.post(
        '/app-api/usage/precheck',
        {
          memberId,
          usageType: normalizedUsageType,
          idempotencyKey,
        },
        {},
        {
          requiresAuth: true,
          idempotent: true,
          idempotencyKey,
        },
      )

      return {
        allowed: resolveAllowed(payload),
        usageType: normalizedUsageType,
        requiredCredits: toFiniteNumber(payload?.requiredCredits ?? payload?.needCredits, 1),
        availableCredits: toFiniteNumber(payload?.availableCredits ?? payload?.currentBalance, 0),
        reason: typeof payload?.reason === 'string' ? payload.reason : '',
        raw: payload,
      }
    },
  }
}
