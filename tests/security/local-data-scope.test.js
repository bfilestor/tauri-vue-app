import test from 'node:test'
import assert from 'node:assert/strict'

import {
  resolveLocalChatScope,
  resolveLocalMemberScope,
} from '../../src/modules/security/local-data-scope.js'

test('成员作用域优先使用登录态 userId，并带出当前成员信息', () => {
  const scope = resolveLocalMemberScope(
    {
      userId: ' 1001 ',
      userInfo: {
        userId: 'ignored-user-id',
      },
    },
    {
      profile: {
        userId: 'ignored-profile-id',
      },
      currentMember: {
        memberId: ' 2001 ',
        memberName: '本人',
      },
    },
  )

  assert.deepEqual(scope, {
    ownerUserId: '1001',
    memberId: '2001',
    memberName: '本人',
  })
})

test('成员作用域在缺少 session.userId 时回退到 userInfo 或 profile', () => {
  const fromUserInfo = resolveLocalMemberScope(
    {
      userInfo: {
        userId: 1002,
      },
    },
    {
      currentMember: {
        memberId: 2002,
        memberName: '母亲',
      },
    },
  )
  const fromProfile = resolveLocalMemberScope(
    {},
    {
      profile: {
        userId: 1003,
      },
      currentMember: {
        memberId: 2003,
        memberName: '父亲',
      },
    },
  )

  assert.deepEqual(fromUserInfo, {
    ownerUserId: '1002',
    memberId: '2002',
    memberName: '母亲',
  })
  assert.deepEqual(fromProfile, {
    ownerUserId: '1003',
    memberId: '2003',
    memberName: '父亲',
  })
})

test('成员作用域缺少用户或成员时返回 null，避免发送全局查询', () => {
  assert.equal(
    resolveLocalMemberScope(
      { userId: '' },
      {
        currentMember: {
          memberId: 2001,
        },
      },
    ),
    null,
  )
  assert.equal(
    resolveLocalMemberScope(
      { userId: 1001 },
      {
        currentMember: null,
      },
    ),
    null,
  )
})

test('聊天作用域在成员作用域基础上补齐 conversationId', () => {
  const scope = resolveLocalChatScope(
    {
      userInfo: {
        userId: 1004,
      },
    },
    {
      currentMember: {
        memberId: 2004,
        memberName: '孩子',
      },
    },
    ' conv-001 ',
  )

  assert.deepEqual(scope, {
    ownerUserId: '1004',
    memberId: '2004',
    memberName: '孩子',
    conversationId: 'conv-001',
  })
})

test('聊天作用域在成员未就绪时返回 null', () => {
  assert.equal(
    resolveLocalChatScope(
      { userId: 1005 },
      {
        currentMember: null,
      },
      'conv-002',
    ),
    null,
  )
})
