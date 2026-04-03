import test from 'node:test'
import assert from 'node:assert/strict'

import { createAiModeService, AI_MODES, DEFAULT_AI_MODE } from '../../src/modules/security/ai-mode-service.js'
import { SECURITY_STORAGE_KEYS } from '../../src/modules/security/constants.js'
import { createMemoryStorage } from '../../src/modules/security/storage.js'

test('默认使用通用模式并写入持久化存储', () => {
  const storage = createMemoryStorage()
  const service = createAiModeService({ storage })

  assert.equal(service.getMode(), DEFAULT_AI_MODE)
  assert.equal(service.isGeneralMode(), true)
  assert.equal(storage.getItem(SECURITY_STORAGE_KEYS.aiMode), AI_MODES.general)
})

test('读取非法模式值时自动回退到通用模式', () => {
  const storage = createMemoryStorage({
    [SECURITY_STORAGE_KEYS.aiMode]: 'unknown-mode',
  })

  const service = createAiModeService({ storage })

  assert.equal(service.getMode(), AI_MODES.general)
  assert.equal(storage.getItem(SECURITY_STORAGE_KEYS.aiMode), AI_MODES.general)
})

test('切换到自定义模式后重建服务可恢复该模式', () => {
  const storage = createMemoryStorage()
  const firstService = createAiModeService({ storage })
  firstService.setMode(AI_MODES.custom)

  const secondService = createAiModeService({ storage })

  assert.equal(secondService.getMode(), AI_MODES.custom)
  assert.equal(secondService.isCustomMode(), true)
  assert.equal(storage.getItem(SECURITY_STORAGE_KEYS.aiMode), AI_MODES.custom)
})

test('重置模式会恢复到通用模式', () => {
  const storage = createMemoryStorage({
    [SECURITY_STORAGE_KEYS.aiMode]: AI_MODES.custom,
  })
  const service = createAiModeService({ storage })

  service.reset()

  assert.equal(service.getMode(), AI_MODES.general)
  assert.equal(storage.getItem(SECURITY_STORAGE_KEYS.aiMode), AI_MODES.general)
})
