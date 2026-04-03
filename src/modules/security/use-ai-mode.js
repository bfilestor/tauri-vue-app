import { reactive, readonly } from 'vue'

import { getAiModeService } from './ai-mode-store.js'

const aiModeService = getAiModeService()
const sharedState = reactive({
  mode: aiModeService.getMode(),
})

function syncModeState() {
  sharedState.mode = aiModeService.getMode()
  return sharedState.mode
}

function setMode(nextMode) {
  aiModeService.setMode(nextMode)
  return syncModeState()
}

function resetMode() {
  aiModeService.reset()
  return syncModeState()
}

export function useAiMode() {
  return {
    state: readonly(sharedState),
    setMode,
    resetMode,
    syncModeState,
    isGeneralMode: () => aiModeService.isGeneralMode(),
    isCustomMode: () => aiModeService.isCustomMode(),
  }
}
