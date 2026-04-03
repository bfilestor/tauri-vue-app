import test from 'node:test'
import assert from 'node:assert/strict'

import {
  buildAccountMenuEntries,
  resolveUsageFeedback,
} from '../../src/modules/security/account-view.js'

test('余额刷新后侧边栏与通用模式面板读取同一份最新次数数据', () => {
  const accountStateBefore = {
    status: 'ready',
    wallet: { totalBalance: 9, totalCredits: 20 },
    packageCards: [{ targetCalls: 20 }, { targetCalls: 100 }],
  }
  const sidebarBefore = resolveUsageFeedback(accountStateBefore)
  const panelBefore = resolveUsageFeedback(accountStateBefore)
  assert.equal(sidebarBefore.remaining, panelBefore.remaining)
  assert.equal(sidebarBefore.remaining, 9)

  const accountStateAfter = {
    ...accountStateBefore,
    wallet: { totalBalance: 3, totalCredits: 20 },
  }
  const sidebarAfter = resolveUsageFeedback(accountStateAfter)
  const panelAfter = resolveUsageFeedback(accountStateAfter)
  assert.equal(sidebarAfter.remaining, panelAfter.remaining)
  assert.equal(sidebarAfter.remaining, 3)
})

test('登录态账号菜单包含设置/购买/退出入口', () => {
  const entries = buildAccountMenuEntries(true)
  assert.deepEqual(entries.map((item) => item.key), ['account', 'purchase', 'logout'])
})

test('余额为 0 时进度为 0 且购买入口保持可用', () => {
  const usage = resolveUsageFeedback({
    status: 'ready',
    wallet: { totalBalance: 0, totalCredits: 20 },
  })

  assert.equal(usage.remaining, 0)
  assert.equal(usage.percent, 0)
  assert.equal(usage.canPurchase, true)
})

test('余额接口失败时保留上次缓存并标记为 stale', () => {
  const previousUsage = resolveUsageFeedback({
    status: 'ready',
    wallet: { totalBalance: 6, totalCredits: 20 },
  })

  const usage = resolveUsageFeedback({
    status: 'error',
    lastError: 'balance timeout',
  }, previousUsage)

  assert.equal(usage.remaining, 6)
  assert.equal(usage.stale, true)
  assert.equal(usage.errorMessage, 'balance timeout')
})
