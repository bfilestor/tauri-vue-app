<template>
  <el-dialog
    :model-value="state.visible"
    width="560px"
    align-center
    :close-on-click-modal="false"
    title="购买次数包"
    @close="closePurchaseDialog"
  >
    <div class="purchase-body">
      <div class="package-grid">
        <button
          v-for="card in state.packageCards"
          :key="card.targetCalls"
          class="package-card"
          :class="{
            active: state.selectedCard?.targetCalls === card.targetCalls,
            missing: card.missing,
          }"
          :disabled="!card.purchasable"
          @click="handleSelectCard(card)"
        >
          <p class="package-title">{{ card.title }}</p>
          <p class="package-price">
            {{ card.missing ? '暂不可用' : `¥${card.product?.price ?? '-'}` }}
          </p>
          <p class="package-meta">
            {{ card.missing ? '缺少对应 SKU' : `SKU: ${card.product?.skuId}` }}
          </p>
        </button>
      </div>

      <div class="order-section">
        <div class="pay-tabs">
          <button
            type="button"
            class="pay-tab"
            :class="{ active: state.payChannel === 'WECHAT' }"
            @click="switchChannel('WECHAT')"
          >
            微信支付
          </button>
          <button
            type="button"
            class="pay-tab"
            :class="{ active: state.payChannel === 'ALIPAY' }"
            @click="switchChannel('ALIPAY')"
          >
            支付宝
          </button>
        </div>

        <div class="order-summary">
          <p>订单号：{{ state.orderNo || '未创建' }}</p>
          <p>套餐：{{ state.selectedCard?.title || '-' }}</p>
          <p>金额：{{ state.orderNo ? `¥${state.payableAmount}` : '-' }}</p>
        </div>

        <div class="qrcode-box">
          <img v-if="state.qrcodeUrl" :src="state.qrcodeUrl" alt="支付二维码" class="qrcode-image">
          <p v-else class="qrcode-placeholder">创建订单后显示二维码</p>
        </div>

        <p v-if="state.qrcodeExpireTime" class="expire-text">
          二维码有效期至：{{ state.qrcodeExpireTime }}
          <span v-if="state.qrcodeExpired" class="expire-badge">已过期</span>
        </p>
        <p v-if="state.errorMessage" class="error-text">{{ state.errorMessage }}</p>
      </div>
    </div>

    <template #footer>
      <div class="purchase-footer">
        <el-button @click="closePurchaseDialog">关闭</el-button>
        <el-button
          v-if="state.orderNo"
          plain
          :loading="state.loadingQrcode"
          @click="refreshQrcode"
        >
          刷新二维码
        </el-button>
        <el-button
          type="primary"
          :loading="state.creatingOrder"
          @click="createOrder"
        >
          {{ state.orderNo ? '重新下单' : '创建订单' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup>
import { ElMessage } from 'element-plus'
import { usePurchaseDialog } from '@/modules/security/index.js'

const {
  state,
  closePurchaseDialog,
  selectPackageByCalls,
  createOrder: createOrderAction,
  switchPayChannel: switchPayChannelAction,
  refreshQrcode: refreshQrcodeAction,
} = usePurchaseDialog()

function handleSelectCard(card) {
  const result = selectPackageByCalls(card.targetCalls)
  if (!result.ok) {
    ElMessage.warning('该套餐暂不可购买')
  }
}

async function switchChannel(channel) {
  const result = await switchPayChannelAction(channel)
  if (!result.ok) {
    ElMessage.error(state.errorMessage || '切换支付渠道失败')
  }
}

async function createOrder() {
  const result = await createOrderAction()
  if (!result.ok) {
    ElMessage.error(state.errorMessage || '创建订单失败')
    return
  }
  ElMessage.success('订单已创建，请扫码支付')
}

async function refreshQrcode() {
  const result = await refreshQrcodeAction()
  if (!result.ok) {
    ElMessage.error(state.errorMessage || '二维码刷新失败')
    return
  }
  ElMessage.success('二维码已刷新')
}
</script>

<style scoped>
.purchase-body {
  display: grid;
  grid-template-columns: 1fr;
  gap: 14px;
}

.package-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.package-card {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 10px;
  text-align: left;
  background: #fff;
  cursor: pointer;
}

.package-card.active {
  border-color: #3b82f6;
  background: #eff6ff;
}

.package-card.missing {
  background: #f8fafc;
  color: #94a3b8;
  cursor: not-allowed;
}

.package-title {
  font-size: 13px;
  font-weight: 700;
}

.package-price {
  margin-top: 6px;
  font-size: 18px;
  font-weight: 700;
  color: #2563eb;
}

.package-card.missing .package-price {
  color: #94a3b8;
}

.package-meta {
  margin-top: 4px;
  font-size: 11px;
  color: #64748b;
}

.order-section {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 12px;
}

.pay-tabs {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.pay-tab {
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  padding: 8px;
  font-size: 13px;
  color: #475569;
  background: #fff;
  cursor: pointer;
}

.pay-tab.active {
  border-color: #3b82f6;
  background: #eff6ff;
  color: #2563eb;
}

.order-summary {
  margin-top: 10px;
  font-size: 12px;
  color: #475569;
  line-height: 1.8;
}

.qrcode-box {
  margin-top: 10px;
  border: 1px dashed #cbd5e1;
  border-radius: 10px;
  min-height: 180px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f8fafc;
}

.qrcode-image {
  width: 170px;
  height: 170px;
  object-fit: contain;
  background: #fff;
}

.qrcode-placeholder {
  color: #94a3b8;
  font-size: 12px;
}

.expire-text {
  margin-top: 8px;
  font-size: 12px;
  color: #64748b;
}

.expire-badge {
  color: #dc2626;
  margin-left: 6px;
}

.error-text {
  margin-top: 6px;
  color: #dc2626;
  font-size: 12px;
}

.purchase-footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
</style>
