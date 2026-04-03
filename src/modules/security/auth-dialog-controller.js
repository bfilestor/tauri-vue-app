export const AUTH_DIALOG_TABS = {
  login: 'login',
  register: 'register',
}

const MIN_PASSWORD_LENGTH = 8

function createEmptyErrors() {
  return {
    loginAccount: '',
    loginPassword: '',
    registerAccount: '',
    registerCode: '',
    registerPassword: '',
    registerConfirm: '',
    submit: '',
  }
}

function trimText(value) {
  return typeof value === 'string' ? value.trim() : ''
}

function mergeErrors(target, nextErrors) {
  Object.assign(target, createEmptyErrors(), nextErrors)
}

function validateLoginForm(form) {
  const errors = {}
  const account = trimText(form.account)
  const password = trimText(form.password)

  if (!account) {
    errors.loginAccount = '请输入账号'
  }

  if (!password) {
    errors.loginPassword = '请输入密码'
  }

  return {
    valid: Object.keys(errors).length === 0,
    errors,
    values: {
      account,
      password,
    },
  }
}

function validateRegisterForm(form) {
  const errors = {}
  const account = trimText(form.account)
  const verificationCode = trimText(form.verificationCode)
  const password = trimText(form.password)

  if (!account) {
    errors.registerAccount = '请输入账号'
  }

  if (!verificationCode) {
    errors.registerCode = '请输入验证码'
  }

  if (!password) {
    errors.registerPassword = '请输入密码'
  } else if (password.length < MIN_PASSWORD_LENGTH) {
    errors.registerPassword = `密码至少 ${MIN_PASSWORD_LENGTH} 位`
  }

  if (!form.confirmAccepted) {
    errors.registerConfirm = '请确认注册并同意条款'
  }

  return {
    valid: Object.keys(errors).length === 0,
    errors,
    values: {
      account,
      verificationCode,
      password,
    },
  }
}

async function defaultResolveDeviceContext() {
  return {
    deviceId: '',
    deviceName: 'Desktop',
    clientInfo: '',
  }
}

export function createInitialAuthDialogState() {
  return {
    visible: false,
    activeTab: AUTH_DIALOG_TABS.login,
    submitting: false,
    errors: createEmptyErrors(),
    successMessage: '',
    guestNotice: '',
    loginForm: {
      account: '',
      password: '',
    },
    registerForm: {
      account: '',
      verificationCode: '',
      password: '',
      confirmAccepted: false,
    },
  }
}

export function createAuthDialogController({
  state = createInitialAuthDialogState(),
  authApi,
  resolveDeviceContext = defaultResolveDeviceContext,
} = {}) {
  if (!authApi || typeof authApi.loginByPassword !== 'function' || typeof authApi.registerByEmail !== 'function') {
    throw new Error('createAuthDialogController requires an authApi instance.')
  }

  function setSubmitError(message) {
    mergeErrors(state.errors, { submit: message || '' })
  }

  function open(tab = AUTH_DIALOG_TABS.login) {
    state.visible = true
    switchTab(tab)
    setSubmitError('')
    state.successMessage = ''
    return state
  }

  function close() {
    state.visible = false
    setSubmitError('')
    return state
  }

  function switchTab(tab) {
    state.activeTab = tab === AUTH_DIALOG_TABS.register ? AUTH_DIALOG_TABS.register : AUTH_DIALOG_TABS.login
    setSubmitError('')
    return state
  }

  async function submitLogin() {
    const validation = validateLoginForm(state.loginForm)
    mergeErrors(state.errors, validation.errors)
    if (!validation.valid) {
      return {
        ok: false,
        reason: 'validation',
        errors: validation.errors,
      }
    }

    state.submitting = true
    setSubmitError('')
    state.successMessage = ''

    try {
      const device = await resolveDeviceContext()
      await authApi.loginByPassword({
        userName: validation.values.account,
        password: validation.values.password,
        deviceId: device.deviceId || '',
        deviceName: device.deviceName || 'Desktop',
        clientInfo: device.clientInfo || '',
      })
      close()
      state.successMessage = '登录成功'
      return {
        ok: true,
      }
    } catch (error) {
      const message = error?.message || '登录失败，请稍后重试'
      setSubmitError(message)
      return {
        ok: false,
        reason: 'api',
        message,
        error,
      }
    } finally {
      state.submitting = false
    }
  }

  async function submitRegister() {
    const validation = validateRegisterForm(state.registerForm)
    mergeErrors(state.errors, validation.errors)
    if (!validation.valid) {
      return {
        ok: false,
        reason: 'validation',
        errors: validation.errors,
      }
    }

    state.submitting = true
    setSubmitError('')
    state.successMessage = ''

    try {
      const device = await resolveDeviceContext()
      await authApi.registerByEmail({
        email: validation.values.account,
        emailCode: validation.values.verificationCode,
        userName: validation.values.account,
        password: validation.values.password,
        confirmPassword: validation.values.password,
        deviceId: device.deviceId || '',
      })
      close()
      state.successMessage = '注册成功，已赠送 1 次免费 AI 调用体验'
      return {
        ok: true,
      }
    } catch (error) {
      const message = error?.message || '注册失败，请稍后重试'
      setSubmitError(message)
      return {
        ok: false,
        reason: 'api',
        message,
        error,
      }
    } finally {
      state.submitting = false
    }
  }

  function enterGuestMode() {
    if (typeof authApi.enterGuestMode === 'function') {
      authApi.enterGuestMode()
    }
    state.guestNotice = '已进入临时访客模式，部分权益功能受限。'
    close()
    return {
      ok: true,
      notice: state.guestNotice,
    }
  }

  return {
    state,
    open,
    close,
    switchTab,
    submitLogin,
    submitRegister,
    enterGuestMode,
  }
}
