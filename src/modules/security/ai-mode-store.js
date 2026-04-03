import { appRequestClient } from './bootstrap.js'
import { createAiModeService } from './ai-mode-service.js'

let sharedAiModeService = null

export function getAiModeService() {
  if (!sharedAiModeService) {
    sharedAiModeService = createAiModeService({
      storage: appRequestClient.storage,
    })
  }

  return sharedAiModeService
}
