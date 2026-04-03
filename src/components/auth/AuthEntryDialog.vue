<template>
  <el-dialog
    :model-value="state.visible"
    width="420px"
    align-center
    :show-close="false"
    :close-on-click-modal="true"
    class="auth-entry-dialog"
    @update:model-value="handleDialogVisibilityChange"
  >
    <template #header>
      <div class="auth-header">
        <div class="auth-logo">
          <span class="material-symbols-outlined">health_and_safety</span>
        </div>
        <div class="auth-title">健康管家</div>
        <div class="auth-subtitle">登录后解锁完整 AI 分析能力</div>
      </div>
    </template>

    <div class="auth-body">
      <div class="auth-tab-row">
        <button
          type="button"
          class="auth-tab-btn"
          :class="{ active: state.activeTab === AUTH_DIALOG_TABS.login }"
          @click="switchTab(AUTH_DIALOG_TABS.login)"
        >
          登录
        </button>
        <button
          type="button"
          class="auth-tab-btn"
          :class="{ active: state.activeTab === AUTH_DIALOG_TABS.register }"
          @click="switchTab(AUTH_DIALOG_TABS.register)"
        >
          注册
        </button>
      </div>

      <div v-if="state.activeTab === AUTH_DIALOG_TABS.login" class="auth-form">
        <el-form-item :error="state.errors.loginAccount">
          <el-input
            v-model="state.loginForm.account"
            placeholder="手机号或邮箱"
            clearable
            @keyup.enter="submitLogin"
          >
            <template #prefix>
              <span class="material-symbols-outlined auth-input-icon">person</span>
            </template>
          </el-input>
        </el-form-item>
        <el-form-item :error="state.errors.loginPassword">
          <el-input
            v-model="state.loginForm.password"
            type="password"
            show-password
            placeholder="密码"
            @keyup.enter="submitLogin"
          >
            <template #prefix>
              <span class="material-symbols-outlined auth-input-icon">key</span>
            </template>
          </el-input>
        </el-form-item>
        <el-button type="primary" class="w-full" :loading="state.submitting" @click="submitLogin">
          登录
        </el-button>
      </div>

      <div v-else class="auth-form">
        <el-form-item :error="state.errors.registerAccount">
          <el-input
            v-model="state.registerForm.account"
            placeholder="账号（邮箱或手机号）"
            clearable
          >
            <template #prefix>
              <span class="material-symbols-outlined auth-input-icon">alternate_email</span>
            </template>
          </el-input>
        </el-form-item>
        <el-form-item :error="state.errors.registerCode">
          <div class="auth-code-row">
            <el-input
              v-model="state.registerForm.verificationCode"
              placeholder="验证码"
              clearable
              @keyup.enter="submitRegister"
            >
              <template #prefix>
                <span class="material-symbols-outlined auth-input-icon">mark_email_read</span>
              </template>
            </el-input>
            <el-button plain @click="mockSendCode">发送验证码</el-button>
          </div>
        </el-form-item>
        <el-form-item :error="state.errors.registerPassword">
          <el-input
            v-model="state.registerForm.password"
            type="password"
            show-password
            placeholder="设置密码（至少 8 位）"
            @keyup.enter="submitRegister"
          >
            <template #prefix>
              <span class="material-symbols-outlined auth-input-icon">lock</span>
            </template>
          </el-input>
        </el-form-item>
        <div class="gift-notice">
          注册成功即赠 <strong>1 次</strong> 免费 AI 调用体验
        </div>
        <el-checkbox v-model="state.registerForm.confirmAccepted">
          我已确认注册并同意服务条款与隐私政策
        </el-checkbox>
        <div v-if="state.errors.registerConfirm" class="auth-inline-error">{{ state.errors.registerConfirm }}</div>
        <el-button type="primary" class="w-full" :loading="state.submitting" @click="submitRegister">
          立即注册
        </el-button>
      </div>

      <div v-if="state.errors.submit" class="auth-submit-error">{{ state.errors.submit }}</div>

      <div class="auth-divider">
        <span>或</span>
      </div>

      <el-button class="w-full" @click="enterGuestMode">
        以临时访客身份使用
      </el-button>

      <div class="auth-footer">
        继续即表示同意
        <a href="#" @click.prevent>服务条款</a>
        与
        <a href="#" @click.prevent>隐私政策</a>
      </div>
    </div>
  </el-dialog>
</template>

<script setup>
import { reactive, watch } from 'vue'
import { ElMessage } from 'element-plus'

import {
  AUTH_DIALOG_TABS,
  createAuthDialogController,
  createInitialAuthDialogState,
} from '@/modules/security/auth-dialog-controller.js'
import { getAuthApi, resolveAuthDeviceContext } from '@/modules/security/auth-service.js'

const props = defineProps({
  modelValue: {
    type: Boolean,
    default: false,
  },
  defaultTab: {
    type: String,
    default: AUTH_DIALOG_TABS.login,
  },
})

const emit = defineEmits([
  'update:modelValue',
  'auth-success',
  'guest-entered',
])

const state = reactive(createInitialAuthDialogState())
const controller = createAuthDialogController({
  state,
  authApi: getAuthApi(),
  resolveDeviceContext: resolveAuthDeviceContext,
})

watch(
  () => props.modelValue,
  (visible) => {
    if (visible) {
      controller.open(props.defaultTab)
    } else {
      controller.close()
    }
  },
  { immediate: true },
)

watch(
  () => props.defaultTab,
  (tab) => {
    if (props.modelValue) {
      controller.switchTab(tab)
    }
  },
)

watch(
  () => state.visible,
  (visible) => {
    if (visible !== props.modelValue) {
      emit('update:modelValue', visible)
    }
  },
)

function handleDialogVisibilityChange(visible) {
  if (visible) {
    controller.open(props.defaultTab)
  } else {
    controller.close()
  }
}

function switchTab(tab) {
  controller.switchTab(tab)
}

async function submitLogin() {
  const result = await controller.submitLogin()

  if (result.ok) {
    ElMessage.success('登录成功')
    emit('auth-success', { type: AUTH_DIALOG_TABS.login })
    return
  }

  if (result.reason === 'api') {
    ElMessage.error(result.message)
  }
}

async function submitRegister() {
  const result = await controller.submitRegister()

  if (result.ok) {
    ElMessage.success('注册成功，已赠送 1 次免费体验')
    emit('auth-success', { type: AUTH_DIALOG_TABS.register, trialGiftPending: true })
    return
  }

  if (result.reason === 'api') {
    ElMessage.error(result.message)
  }
}

function enterGuestMode() {
  const result = controller.enterGuestMode()
  ElMessage.warning(result.notice)
  emit('guest-entered', result)
}

function mockSendCode() {
  ElMessage.info('验证码发送接口将在后续 Issue 对接')
}
</script>

<style scoped>
.auth-header {
  text-align: center;
}

.auth-logo {
  width: 52px;
  height: 52px;
  margin: 0 auto 10px;
  border-radius: 14px;
  background: linear-gradient(135deg, #1565c0, #2196f3);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
}

.auth-title {
  font-size: 20px;
  font-weight: 700;
  color: #1e293b;
}

.auth-subtitle {
  margin-top: 4px;
  font-size: 13px;
  color: #64748b;
}

.auth-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.auth-tab-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
  padding: 4px;
  border-radius: 10px;
  background: #f1f5f9;
}

.auth-tab-btn {
  border: 0;
  border-radius: 8px;
  padding: 9px 0;
  background: transparent;
  color: #64748b;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.auth-tab-btn.active {
  background: #fff;
  color: #1e293b;
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.1);
}

.auth-form {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.auth-input-icon {
  font-size: 18px;
  color: #94a3b8;
}

.auth-code-row {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 8px;
  width: 100%;
}

.gift-notice {
  border: 1px solid #86efac;
  background: #ecfdf5;
  border-radius: 10px;
  padding: 10px 12px;
  color: #065f46;
  font-size: 12px;
}

.gift-notice strong {
  font-weight: 700;
}

.auth-inline-error {
  color: #f56c6c;
  font-size: 12px;
  margin-top: -6px;
}

.auth-submit-error {
  color: #f56c6c;
  font-size: 13px;
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 8px;
  padding: 8px 10px;
}

.auth-divider {
  display: flex;
  align-items: center;
  gap: 10px;
  color: #94a3b8;
  font-size: 12px;
}

.auth-divider::before,
.auth-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: #e2e8f0;
}

.auth-footer {
  text-align: center;
  color: #94a3b8;
  font-size: 12px;
}

.auth-footer a {
  color: #2563eb;
  text-decoration: none;
}
</style>
