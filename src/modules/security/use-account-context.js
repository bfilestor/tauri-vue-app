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

function selectMember(memberId) {
  accountContextService.selectMember(memberId)
  return syncFromService()
}

async function createMember(payload = {}) {
  await accountContextService.createMember(payload)
  return syncFromService()
}

async function updateMember(memberId, payload = {}) {
  await accountContextService.updateMember(memberId, payload)
  return syncFromService()
}

async function deleteMember(memberId) {
  await accountContextService.deleteMember(memberId)
  return syncFromService()
}

async function setDefaultMember(memberId) {
  await accountContextService.setDefaultMember(memberId)
  return syncFromService()
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
    selectMember,
    createMember,
    updateMember,
    deleteMember,
    setDefaultMember,
    clear,
    syncFromService,
  }
}
