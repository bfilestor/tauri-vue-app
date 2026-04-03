import { reactive, readonly } from 'vue'

import { appRequestClient } from './bootstrap.js'
import { createOrderService } from './order-service.js'
import {
  createInitialPurchaseDialogState,
  createPurchaseDialogController,
} from './purchase-dialog-controller.js'
import { useAccountContext } from './use-account-context.js'

const { state: accountContextState, refresh: refreshAccountContext } = useAccountContext()
const state = reactive(createInitialPurchaseDialogState())

const controller = createPurchaseDialogController({
  state,
  orderService: createOrderService({
    client: appRequestClient,
  }),
})

async function openPurchaseDialog({ preferredCalls = null, reason = 'user_click', forceRefresh = false } = {}) {
  try {
    await refreshAccountContext({ force: forceRefresh })
  } catch {
    // Keep existing cached package cards for graceful fallback.
  }

  controller.open({
    packageCards: accountContextState.packageCards || [],
    preferredCalls,
    reason,
  })

  return state
}

export function usePurchaseDialog() {
  return {
    state: readonly(state),
    openPurchaseDialog,
    closePurchaseDialog: controller.close,
    selectPackageByCalls: controller.selectPackageByCalls,
    createOrder: controller.createOrder,
    switchPayChannel: controller.switchPayChannel,
    refreshQrcode: controller.refreshQrcode,
  }
}
