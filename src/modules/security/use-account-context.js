import { reactive, readonly } from 'vue'

import { getAccountContextService } from './account-service.js'

const accountContextService = getAccountContextService()
const sharedState = reactive({
  ...accountContextService.getState(),
})

function syncFromService() {
  Object.assign(sharedState, accountContextService.getState())
  return sharedState
}

async function refresh(options = {}) {
  try {
    await accountContextService.refreshForCurrentSession(options)
    return syncFromService()
  } catch (error) {
    syncFromService()
    throw error
  }
}

function clear() {
  accountContextService.clear()
  return syncFromService()
}

syncFromService()

export function useAccountContext() {
  return {
    state: readonly(sharedState),
    refresh,
    clear,
    syncFromService,
  }
}
