import test from 'node:test'
import assert from 'node:assert/strict'

import {
  AUTH_DIALOG_TABS,
  createAuthDialogController,
  createInitialAuthDialogState,
} from '../../src/modules/security/auth-dialog-controller.js'

function createAuthApiStub() {
  return {
    loginCalls: [],
    registerCalls: [],
    guestCalls: 0,
    async loginByPassword(payload) {
      this.loginCalls.push(payload)
      return { session: { isAuthenticated: true } }
    },
    async registerByEmail(payload) {
      this.registerCalls.push(payload)
      return { session: { isAuthenticated: true, trialGiftPending: true } }
    },
    enterGuestMode() {
      this.guestCalls += 1
      return { isGuest: true }
    },
  }
}

test('登录与注册 Tab 可切换且已输入内容保持不丢失', () => {
  const state = createInitialAuthDialogState()
  const authApi = createAuthApiStub()
  const controller = createAuthDialogController({ state, authApi })

  state.loginForm.account = 'demo-user'
  state.loginForm.password = 'demo-pass'
  state.registerForm.account = 'demo@example.com'
  state.registerForm.verificationCode = '123456'

  controller.open()
  assert.equal(state.activeTab, AUTH_DIALOG_TABS.login)
  controller.switchTab(AUTH_DIALOG_TABS.register)
  assert.equal(state.activeTab, AUTH_DIALOG_TABS.register)
  controller.switchTab(AUTH_DIALOG_TABS.login)

  assert.equal(state.loginForm.account, 'demo-user')
  assert.equal(state.loginForm.password, 'demo-pass')
  assert.equal(state.registerForm.account, 'demo@example.com')
  assert.equal(state.registerForm.verificationCode, '123456')
})

test('登录表单校验账号和密码非空', async () => {
  const state = createInitialAuthDialogState()
  const authApi = createAuthApiStub()
  const controller = createAuthDialogController({ state, authApi })

  const result = await controller.submitLogin()

  assert.equal(result.ok, false)
  assert.equal(result.reason, 'validation')
  assert.equal(state.errors.loginAccount, '请输入账号')
  assert.equal(state.errors.loginPassword, '请输入密码')
  assert.equal(authApi.loginCalls.length, 0)
})

test('注册表单校验账号、验证码、密码长度和确认动作', async () => {
  const state = createInitialAuthDialogState()
  const authApi = createAuthApiStub()
  const controller = createAuthDialogController({ state, authApi })

  state.registerForm.account = 'demo@example.com'
  state.registerForm.verificationCode = ''
  state.registerForm.password = '123'
  state.registerForm.confirmAccepted = false

  const result = await controller.submitRegister()

  assert.equal(result.ok, false)
  assert.equal(result.reason, 'validation')
  assert.equal(state.errors.registerCode, '请输入验证码')
  assert.equal(state.errors.registerPassword, '密码至少 8 位')
  assert.equal(state.errors.registerConfirm, '请确认注册并同意条款')
  assert.equal(authApi.registerCalls.length, 0)
})

test('关闭弹框不会改动当前会话状态', () => {
  const state = createInitialAuthDialogState()
  const authApi = createAuthApiStub()
  const controller = createAuthDialogController({ state, authApi })

  controller.open()
  controller.close()

  assert.equal(state.visible, false)
  assert.equal(authApi.loginCalls.length, 0)
  assert.equal(authApi.registerCalls.length, 0)
  assert.equal(authApi.guestCalls, 0)
})

test('临时访客入口会写入访客态并关闭弹框', () => {
  const state = createInitialAuthDialogState()
  const authApi = createAuthApiStub()
  const controller = createAuthDialogController({ state, authApi })

  controller.open()
  const result = controller.enterGuestMode()

  assert.equal(result.ok, true)
  assert.equal(state.visible, false)
  assert.equal(authApi.guestCalls, 1)
  assert.match(state.guestNotice, /临时访客/)
})

test('登录提交会组装认证请求并关闭弹框', async () => {
  const state = createInitialAuthDialogState()
  const authApi = createAuthApiStub()
  const controller = createAuthDialogController({
    state,
    authApi,
    resolveDeviceContext: async () => ({
      deviceId: 'device-001',
      deviceName: 'Windows Desktop',
      clientInfo: 'UA-TEST',
    }),
  })

  state.loginForm.account = 'demo-user'
  state.loginForm.password = 'Demo123456'
  controller.open(AUTH_DIALOG_TABS.login)

  const result = await controller.submitLogin()

  assert.equal(result.ok, true)
  assert.equal(state.visible, false)
  assert.equal(authApi.loginCalls.length, 1)
  assert.deepEqual(authApi.loginCalls[0], {
    userName: 'demo-user',
    password: 'Demo123456',
    deviceId: 'device-001',
    deviceName: 'Windows Desktop',
    clientInfo: 'UA-TEST',
  })
})

test('注册提交成功后展示赠送试用提示并关闭弹框', async () => {
  const state = createInitialAuthDialogState()
  const authApi = createAuthApiStub()
  const controller = createAuthDialogController({
    state,
    authApi,
    resolveDeviceContext: async () => ({
      deviceId: 'device-002',
      deviceName: 'Windows Desktop',
      clientInfo: 'UA-TEST',
    }),
  })

  state.registerForm.account = 'mail@example.com'
  state.registerForm.verificationCode = '952700'
  state.registerForm.password = 'Demo123456'
  state.registerForm.confirmAccepted = true
  controller.open(AUTH_DIALOG_TABS.register)

  const result = await controller.submitRegister()

  assert.equal(result.ok, true)
  assert.equal(state.visible, false)
  assert.match(state.successMessage, /赠送 1 次/)
  assert.equal(authApi.registerCalls.length, 1)
  assert.deepEqual(authApi.registerCalls[0], {
    email: 'mail@example.com',
    emailCode: '952700',
    userName: 'mail@example.com',
    password: 'Demo123456',
    confirmPassword: 'Demo123456',
    deviceId: 'device-002',
  })
})
