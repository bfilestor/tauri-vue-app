import { appRequestClient } from './bootstrap.js'
import { getAuthApi } from './auth-service.js'
import { createAccountContextService } from './account-context-service.js'

let sharedAccountContextService = null

export function getAccountContextService() {
  if (!sharedAccountContextService) {
    sharedAccountContextService = createAccountContextService({
      client: appRequestClient,
      authApi: getAuthApi(),
    })
  }

  return sharedAccountContextService
}
