function normalizeId(value) {
  if (value == null) {
    return ''
  }

  const text = String(value).trim()
  return text
}

export function resolveLocalMemberScope(sessionState, accountContextState) {
  const ownerUserId = normalizeId(
    sessionState?.userId
    ?? sessionState?.userInfo?.userId
    ?? accountContextState?.profile?.userId,
  )
  const memberId = normalizeId(accountContextState?.currentMember?.memberId)

  if (!ownerUserId || !memberId) {
    return null
  }

  return {
    ownerUserId,
    memberId,
    memberName: accountContextState?.currentMember?.memberName || '',
  }
}

export function resolveLocalChatScope(sessionState, accountContextState, conversationId = '') {
  const memberScope = resolveLocalMemberScope(sessionState, accountContextState)
  if (!memberScope) {
    return null
  }

  return {
    ...memberScope,
    conversationId: normalizeId(conversationId),
  }
}
