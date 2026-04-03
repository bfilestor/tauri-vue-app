import { SECURITY_STORAGE_KEYS } from './constants.js'
import { createStorageAdapter } from './storage.js'

export const AI_MODES = {
  general: 'general',
  custom: 'custom',
}

export const DEFAULT_AI_MODE = AI_MODES.general

function normalizeAiMode(mode) {
  return mode === AI_MODES.custom
    ? AI_MODES.custom
    : AI_MODES.general
}

export function createAiModeService({ storage, storageKey = SECURITY_STORAGE_KEYS.aiMode } = {}) {
  const storageAdapter = createStorageAdapter(storage)
  let currentMode = normalizeAiMode(storageAdapter.getString(storageKey, DEFAULT_AI_MODE))

  storageAdapter.setString(storageKey, currentMode)

  return {
    getMode() {
      return currentMode
    },
    setMode(nextMode) {
      currentMode = normalizeAiMode(nextMode)
      storageAdapter.setString(storageKey, currentMode)
      return currentMode
    },
    reset() {
      currentMode = DEFAULT_AI_MODE
      storageAdapter.setString(storageKey, currentMode)
      return currentMode
    },
    isGeneralMode() {
      return currentMode === AI_MODES.general
    },
    isCustomMode() {
      return currentMode === AI_MODES.custom
    },
  }
}
