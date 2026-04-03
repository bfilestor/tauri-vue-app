const DEFAULT_USAGE_TOTAL = 20

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

function normalizeText(value) {
  return typeof value === 'string' ? value.trim() : ''
}

function resolveDisplayName(userInfo) {
  const candidates = [
    userInfo?.nickName,
    userInfo?.nickname,
    userInfo?.userName,
    userInfo?.username,
    userInfo?.email,
    userInfo?.phone,
  ]

  for (const candidate of candidates) {
    const text = normalizeText(candidate)
    if (text) {
      return text
    }
  }

  return '健康用户'
}

function resolveAvatarText(displayName) {
  const value = normalizeText(displayName)
  return value ? value.slice(0, 1) : '健'
}

function resolveUsage(userInfo) {
  const remaining = toNumber(
    userInfo?.remainingTimes
    ?? userInfo?.remainingCount
    ?? userInfo?.usageRemaining
    ?? userInfo?.quotaRemaining,
  )
  const total = toNumber(
    userInfo?.totalTimes
    ?? userInfo?.totalCount
    ?? userInfo?.usageTotal
    ?? userInfo?.quotaTotal
    ?? userInfo?.packageTotal,
  )

  const normalizedRemaining = Math.max(0, remaining ?? 0)
  const normalizedTotal = Math.max(1, total ?? DEFAULT_USAGE_TOTAL)
  const percent = clamp(Math.round((normalizedRemaining / normalizedTotal) * 100), 0, 100)

  return {
    remaining: normalizedRemaining,
    total: normalizedTotal,
    percent,
    label: `${normalizedRemaining} / ${normalizedTotal} 次`,
  }
}

function createAnonymousState() {
  return {
    mode: 'anonymous',
    displayName: '未登录',
    subtitle: '登录后可同步账号权益',
    avatarText: '未',
    usage: {
      remaining: 0,
      total: DEFAULT_USAGE_TOTAL,
      percent: 0,
      label: `0 / ${DEFAULT_USAGE_TOTAL} 次`,
    },
    showAuthActions: true,
    showSensitiveActions: false,
    trialHint: '注册即送 1 次免费 AI 调用体验',
    guestHint: '',
    quickActions: [],
  }
}

function createGuestState() {
  return {
    mode: 'guest',
    displayName: '临时访客',
    subtitle: '访客模式',
    avatarText: '访',
    usage: {
      remaining: 0,
      total: DEFAULT_USAGE_TOTAL,
      percent: 0,
      label: `0 / ${DEFAULT_USAGE_TOTAL} 次`,
    },
    showAuthActions: true,
    showSensitiveActions: false,
    trialHint: '登录或注册后可保存历史记录',
    guestHint: '当前为临时访客，账号菜单与历史权益不可用',
    quickActions: [],
  }
}

function createAuthenticatedState(sessionState) {
  const userInfo = sessionState?.userInfo && typeof sessionState.userInfo === 'object'
    ? sessionState.userInfo
    : {}
  const displayName = resolveDisplayName(userInfo)

  return {
    mode: 'authenticated',
    displayName,
    subtitle: '在线',
    avatarText: resolveAvatarText(displayName),
    usage: resolveUsage(userInfo),
    showAuthActions: false,
    showSensitiveActions: true,
    trialHint: '',
    guestHint: '',
    quickActions: [
      { key: 'purchase', label: '购买次数' },
      { key: 'account', label: '账号菜单' },
      { key: 'logout', label: '退出登录' },
    ],
  }
}

export function resolveSidebarUserState(sessionState = {}) {
  if (sessionState.isAuthenticated) {
    return createAuthenticatedState(sessionState)
  }

  if (sessionState.isGuest) {
    return createGuestState()
  }

  return createAnonymousState()
}
