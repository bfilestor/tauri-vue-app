import test from 'node:test'
import assert from 'node:assert/strict'

import { createUsageService } from '../../src/modules/security/usage-service.js'

function createClientStub() {
  const calls = []
  let responsePayload = {}

  return {
    calls,
    setResponse(payload) {
      responsePayload = payload
    },
    async post(path, body, init = {}, meta = {}) {
      calls.push({ path, body, init, meta })
      return responsePayload
    },
  }
}

test('usage precheck 默认使用 CHAT 并兼容 canUse 返回字段', async () => {
  const client = createClientStub()
  client.setResponse({
    canUse: true,
    availableCredits: 9,
    requiredCredits: 1,
  })

  const service = createUsageService({
    client,
    idempotencyKeyFactory: () => 'idem-chat-001',
  })
  const result = await service.precheck({ memberId: 20001 })

  assert.equal(client.calls.length, 1)
  assert.equal(client.calls[0].path, '/app-api/usage/precheck')
  assert.deepEqual(client.calls[0].body, {
    memberId: 20001,
    usageType: 'CHAT',
    idempotencyKey: 'idem-chat-001',
  })
  assert.equal(client.calls[0].meta.requiresAuth, true)
  assert.equal(client.calls[0].meta.idempotent, true)
  assert.equal(result.allowed, true)
  assert.equal(result.availableCredits, 9)
  assert.equal(result.requiredCredits, 1)
})

test('usage precheck 兼容 allowed/currentBalance/needCredits 字段', async () => {
  const client = createClientStub()
  client.setResponse({
    allowed: false,
    currentBalance: 0,
    needCredits: 1,
    reason: 'insufficient_balance',
  })

  const service = createUsageService({ client })
  const result = await service.precheck({ memberId: 30001, usageType: 'OCR' })

  assert.equal(result.allowed, false)
  assert.equal(result.availableCredits, 0)
  assert.equal(result.requiredCredits, 1)
  assert.equal(result.reason, 'insufficient_balance')
  assert.equal(result.usageType, 'OCR')
})
