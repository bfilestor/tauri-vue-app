import test from 'node:test'
import assert from 'node:assert/strict'

import { createMemberRepository } from '../../src/modules/security/account-service.js'

function createClientStub() {
  const calls = []

  return {
    calls,
    async get(path, init = {}, meta = {}) {
      calls.push({ method: 'GET', path, init, meta })
      return []
    },
    async post(path, body, init = {}, meta = {}) {
      calls.push({ method: 'POST', path, body, init, meta })
      return { ok: true }
    },
    async request(path, init = {}, meta = {}) {
      calls.push({ method: init?.method || 'GET', path, init, meta })
      return { ok: true }
    },
  }
}

function createAuthApiStub(session = {}) {
  return {
    getSessionState() {
      return {
        isAuthenticated: true,
        userId: '1001',
        userInfo: null,
        ...session,
      }
    },
  }
}

test('成员列表查询在登录态下使用后端接口', async () => {
  const client = createClientStub()
  const repo = createMemberRepository(client, createAuthApiStub({ userId: 'u-1' }))

  await repo.listMembers({})

  assert.equal(client.calls.length, 1)
  assert.equal(client.calls[0].method, 'GET')
  assert.equal(client.calls[0].path, '/app-api/family-members')
  assert.equal(client.calls[0].meta.requiresAuth, true)
  assert.equal(client.calls[0].meta.includeUserId, true)
})

test('成员 CRUD 与设默认统一走后端接口', async () => {
  const client = createClientStub()
  const repo = createMemberRepository(client, createAuthApiStub({ userId: 'u-2' }))

  await repo.createMember({
    payload: { memberName: '本人', relationCode: 'SELF' },
  })
  await repo.updateMember({
    memberId: 'm-1',
    payload: { memberName: '父亲' },
  })
  await repo.deleteMember({ memberId: 'm-1' })
  await repo.setDefaultMember({ memberId: 'm-2' })

  assert.deepEqual(
    client.calls.map((item) => `${item.method} ${item.path}`),
    [
      'POST /app-api/family-members',
      'PUT /app-api/family-members/m-1',
      'DELETE /app-api/family-members/m-1',
      'PUT /app-api/family-members/m-2/set-default',
    ],
  )
})

test('缺少 owner user id 时拒绝成员操作', async () => {
  const client = createClientStub()
  const repo = createMemberRepository(client, createAuthApiStub({ userId: '' }))

  await assert.rejects(() => repo.listMembers({}), /Missing owner user id/i)
  assert.equal(client.calls.length, 0)
})
