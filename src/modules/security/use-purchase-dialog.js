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
  onPaymentSuccess: async () => {
    await refreshAccountContext({ force: true })
    controller.close()
  },
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
  async function createOrderAndStartPolling() {
    const result = await controller.createOrder()
    if (!result.ok) {
      return result
    }

    controller.startPolling({
      maxAttempts: 30,
      intervalMs: 3000,
    })
    await controller.pollOrderStatusOnce()
    return result
  }

  async function resumePolling() {
    const started = controller.startPolling({
      maxAttempts: Math.max(state.pollingMaxAttempts || 30, 1),
      intervalMs: Math.max(state.pollingIntervalMs || 3000, 500),
    })
    if (!started.ok) {
      return started
    }
    return controller.pollOrderStatusOnce()
  }

  return {
    state: readonly(state),
    openPurchaseDialog,
    closePurchaseDialog: controller.close,
    selectPackageByCalls: controller.selectPackageByCalls,
    createOrder: createOrderAndStartPolling,
    switchPayChannel: controller.switchPayChannel,
    refreshQrcode: controller.refreshQrcode,
    cancelPayment: controller.cancelPayment,
    resumePolling,
  }
}
