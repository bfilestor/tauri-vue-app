import test from 'node:test'
import assert from 'node:assert/strict'

import { resolveCustomModeProviderGuides } from '../../src/modules/security/custom-mode-guide.js'

test('平台链接为空时返回未配置状态', () => {
  const guides = resolveCustomModeProviderGuides([
    {
      key: 'zhipu',
      name: '智谱 AI',
      signupUrl: '',
    },
  ])

  assert.equal(guides.length, 1)
  assert.equal(guides[0].linkConfigured, false)
  assert.equal(guides[0].signupUrl, '')
})

test('平台链接为 http/https 时返回已配置状态', () => {
  const guides = resolveCustomModeProviderGuides([
    {
      key: 'custom',
      name: '自定义',
      signupUrl: ' https://example.com/register ',
    },
  ])

  assert.equal(guides.length, 1)
  assert.equal(guides[0].linkConfigured, true)
  assert.equal(guides[0].signupUrl, 'https://example.com/register')
})

test('非 http 链接按未配置处理，避免误打开未知协议', () => {
  const guides = resolveCustomModeProviderGuides([
    {
      key: 'unsafe',
      name: 'Unsafe',
      signupUrl: 'javascript:alert(1)',
    },
  ])

  assert.equal(guides[0].linkConfigured, false)
})
