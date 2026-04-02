export const APP_CLIENT_ID = 'desktop-tauri'

export const SECURITY_STORAGE_KEYS = {
  clientId: 'health.security.clientId',
  deviceId: 'health.security.deviceId',
  deviceSecret: 'health.security.deviceSecret',
  privateKey: 'health.security.privateKey',
  credentialVersion: 'health.security.credentialVersion',
  credentialExpireTime: 'health.security.credentialExpireTime',
  signatureMode: 'health.security.signatureMode',
  lastActivationError: 'health.security.lastActivationError',
  accessToken: 'health.auth.accessToken',
}

export const SIGNATURE_MODE = {
  required: 'required',
  compat: 'compat',
}

export const DEFAULT_COMPATIBILITY_ERROR_CODES = [
  'CLIENT_ACTIVATION_DISABLED',
  'CLIENT_SECURITY_OPTIONAL',
  'CLIENT_SIGNATURE_NOT_REQUIRED',
]

export const HIGH_VALUE_ROUTE_PATTERNS = [
  /^\/app-api\/orders(?:\/|$)/,
  /^\/app-api\/wallet(?:\/|$)/,
  /^\/app-api\/usage(?:\/|$)/,
  /^\/app-api\/builtin-ai(?:\/|$)/,
  /^\/app-api\/account\/balance(?:\/|$)/,
  /^\/app-api\/account\/ledger(?:\/|$)/,
]
