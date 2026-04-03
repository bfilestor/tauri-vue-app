function toNumber(value) {
  if (typeof value === 'number' && Number.isFinite(value)) {
    return value
  }

  if (typeof value === 'string' && value.trim()) {
    const parsed = Number(value)
    if (Number.isFinite(parsed)) {
      return parsed
    }
  }

  return null
}

function clamp(value, min, max) {
  return Math.min(max, Math.max(min, value))
}

function resolveRemainingFromState(accountState) {
  return toNumber(
    accountState?.wallet?.totalBalance
    ?? accountState?.wallet?.availableCredits
    ?? accountState?.wallet?.remainingCredits
    ?? accountState?.balance?.chatBalance
    ?? accountState?.balance?.totalBalance
    ?? accountState?.balance?.ocrBalance,
  )
}

function resolveTotalFromState(accountState, fallbackTotal) {
  const explicitTotal = toNumber(
    accountState?.wallet?.totalCredits
    ?? accountState?.wallet?.packageTotal
    ?? accountState?.balance?.totalCredits,
  )
  if (explicitTotal != null) {
    return explicitTotal
  }

  const packageTargets = Array.isArray(accountState?.packageCards)
    ? accountState.packageCards
      .map((card) => toNumber(card?.targetCalls))
      .filter((item) => item != null)
    : []
  if (packageTargets.length > 0) {
    return Math.max(...packageTargets)
  }

  return fallbackTotal
}

function createUsageModel(remaining, total, stale = false, errorMessage = '') {
  const normalizedTotal = Math.max(1, total)
  const normalizedRemaining = Math.max(0, remaining)
  const percent = clamp(Math.round((normalizedRemaining / normalizedTotal) * 100), 0, 100)

  return {
    remaining: normalizedRemaining,
    total: normalizedTotal,
    percent,
    label: `${normalizedRemaining} / ${normalizedTotal} 次`,
    canPurchase: normalizedRemaining <= 0,
    stale,
    errorMessage,
  }
}

export function resolveUsageFeedback(accountState, previousUsage = null, fallbackTotal = 20) {
  const remaining = resolveRemainingFromState(accountState)
  const total = resolveTotalFromState(accountState, fallbackTotal)
  const stateStatus = accountState?.status || 'idle'

  if (stateStatus === 'error' && previousUsage) {
    return {
      ...previousUsage,
      stale: true,
      errorMessage: accountState?.lastError || '余额刷新失败，已展示上次缓存',
    }
  }

  return createUsageModel(remaining ?? 0, total, false, '')
}

export function buildAccountMenuEntries(isAuthenticated) {
  if (!isAuthenticated) {
    return []
  }

  return [
    { key: 'account', label: '账号设置' },
    { key: 'purchase', label: '购买次数' },
    { key: 'logout', label: '退出登录' },
  ]
}
