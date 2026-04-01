# Vue3 客户端对接示例

## 1. 文档说明

- 文档目标：给 Vue3 或 Tauri + Vue3 客户端提供统一请求封装、签名处理和业务接口调用示例。
- 配套总览文档：`doc/桌面客户端对接文档.md`
- 适用场景：
  - Vue3 桌面客户端
  - Tauri + Vue3 客户端
  - 前端联调示例工程

本文示例基于：

- Vue3
- Axios
- CryptoJS

如果项目是 Tauri 客户端，建议把设备凭证存储、签名计算优先下沉到 Tauri Rust 侧或系统安全区，而不是长期只放在浏览器侧存储。

---

## 2. 推荐目录结构

```text
src/
├── api/
│   ├── auth.ts
│   ├── family.ts
│   ├── order.ts
│   └── ai.ts
├── utils/
│   ├── clientSecurity.ts
│   └── request.ts
└── stores/
    └── session.ts
```

---

## 3. 依赖安装

```bash
npm install axios crypto-js
```

如果使用 TypeScript，建议额外安装类型：

```bash
npm install -D @types/crypto-js
```

---

## 4. 安全签名工具

### 4.1 `src/utils/clientSecurity.ts`

```ts
import CryptoJS from 'crypto-js'

export interface ClientSecurityContext {
  clientId: string
  deviceId: string
  appVersion: string
  deviceSecret: string
}

export function sha256Base64(raw: string) {
  const hash = CryptoJS.SHA256(CryptoJS.enc.Utf8.parse(raw))
  return CryptoJS.enc.Base64.stringify(hash)
}

export function buildCanonicalString(
  method: string,
  requestPath: string,
  body: string,
  timestamp: string,
  nonce: string,
  clientId: string,
  deviceId: string
) {
  return [
    method.toUpperCase(),
    requestPath,
    sha256Base64(body || ''),
    timestamp,
    nonce,
    clientId,
    deviceId
  ].join('\n')
}

export function signRequest(
  ctx: ClientSecurityContext,
  method: string,
  requestPath: string,
  body: string,
  timestamp: string,
  nonce: string
) {
  const canonical = buildCanonicalString(
    method,
    requestPath,
    body,
    timestamp,
    nonce,
    ctx.clientId,
    ctx.deviceId
  )

  const sign = CryptoJS.HmacSHA256(canonical, ctx.deviceSecret)
  return CryptoJS.enc.Base64.stringify(sign)
}

export function createNonce() {
  return crypto.randomUUID().replace(/-/g, '')
}
```

说明：

- 首期按 HMAC 模式示例
- 后续如果切换为非对称签名，可只替换 `signRequest`
- 业务 API 层不需要关心签名细节

---

## 5. Axios 请求封装

### 5.1 `src/utils/request.ts`

```ts
import axios from 'axios'
import { createNonce, signRequest, type ClientSecurityContext } from './clientSecurity'

const request = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
  timeout: 15000
})

function getSecurityContext(): ClientSecurityContext | null {
  const raw = localStorage.getItem('health-client-security')
  return raw ? JSON.parse(raw) : null
}

function getAccessToken() {
  return localStorage.getItem('health-access-token') || ''
}

request.interceptors.request.use((config) => {
  const token = getAccessToken()
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }

  const security = getSecurityContext()
  if (!security) {
    return config
  }

  const method = (config.method || 'get').toUpperCase()
  const url = config.url || ''
  const queryString = config.params
    ? '?' + new URLSearchParams(config.params as Record<string, string>).toString()
    : ''
  const requestPath = `${url}${queryString}`
  const body = config.data ? JSON.stringify(config.data) : ''
  const timestamp = Date.now().toString()
  const nonce = createNonce()
  const signature = signRequest(security, method, requestPath, body, timestamp, nonce)

  config.headers['X-Client-Id'] = security.clientId
  config.headers['X-Device-Id'] = security.deviceId
  config.headers['X-Timestamp'] = timestamp
  config.headers['X-Nonce'] = nonce
  config.headers['X-Signature'] = signature
  config.headers['X-App-Version'] = security.appVersion

  return config
})

request.interceptors.response.use(
  (response) => response,
  async (error) => {
    const code = error?.response?.data?.code
    const msg = error?.response?.data?.msg || '请求失败'

    if (code === 'CLIENT_DEVICE_FROZEN') {
      window.alert('当前设备已被冻结，请联系管理员')
    }

    if (code === 'CLIENT_DEVICE_REVOKED') {
      window.alert('当前设备已被吊销，请重新激活客户端')
    }

    if (code === 'CLIENT_VERSION_BLOCKED') {
      window.alert('当前客户端版本已被限制，请升级后再试')
    }

    return Promise.reject(new Error(msg))
  }
)

export default request
```

---

## 6. 会话与设备信息保存

### 6.1 建议保存项

| Key | 说明 |
| --- | --- |
| `health-client-security` | `clientId`、`deviceId`、`appVersion`、`deviceSecret` |
| `health-access-token` | 登录 token |
| `health-refresh-token` | 刷新 token |
| `health-user-info` | 当前登录用户资料 |

### 6.2 安全建议

- 如果是纯 Web 环境，可先临时放在 `localStorage`
- 如果是 Tauri 客户端，优先迁移到安全存储能力
- 不建议把 `deviceSecret` 明文写死在前端源码

---

## 7. API 文件示例

### 7.1 `src/api/auth.ts`

```ts
import request from '@/utils/request'

export interface LoginResp {
  accessToken: string
  refreshToken: string
  expireIn: number
  userInfo: {
    userId: number
    userName?: string
    nickName?: string
  }
}

export function activateClient(payload: Record<string, unknown>) {
  return request.post('/app-api/client/activate', payload)
}

export function loginByPassword(payload: {
  userName: string
  password: string
  deviceId: string
  deviceName: string
}) {
  return request.post<{ data: LoginResp }>('/app-api/auth/login/password', payload)
}

export function loginByPhoneCode(payload: {
  phone: string
  code: string
  deviceId: string
  deviceName: string
}) {
  return request.post('/app-api/auth/login/phone-code', payload)
}

export function refreshToken(refreshToken: string) {
  return request.post('/app-api/auth/refresh-token', { refreshToken })
}

export function logout() {
  return request.post('/app-api/auth/logout')
}
```

### 7.2 `src/api/family.ts`

```ts
import request from '@/utils/request'

export function listFamilyMembers() {
  return request.get('/app-api/family-members')
}

export function createFamilyMember(payload: {
  memberName: string
  relationCode: string
  gender: string
  birthday: string
  mobile?: string
  healthNote?: string
}) {
  return request.post('/app-api/family-members', payload)
}

export function updateFamilyMember(memberId: number, payload: Record<string, unknown>) {
  return request.put(`/app-api/family-members/${memberId}`, payload)
}

export function deleteFamilyMember(memberId: number) {
  return request.delete(`/app-api/family-members/${memberId}`)
}

export function setDefaultMember(memberId: number) {
  return request.put(`/app-api/family-members/${memberId}/set-default`)
}
```

### 7.3 `src/api/order.ts`

```ts
import request from '@/utils/request'

export function listProducts() {
  return request.get('/app-api/products')
}

export function createOrder(payload: {
  skuId: number
  sourceChannel: string
  idempotencyKey: string
}) {
  return request.post('/app-api/orders', payload, {
    headers: {
      'Idempotency-Key': payload.idempotencyKey
    }
  })
}

export function getOrderStatus(orderNo: string) {
  return request.get(`/app-api/orders/${orderNo}/status`)
}

export function getPayQrcode(orderNo: string) {
  return request.get(`/app-api/orders/${orderNo}/pay-qrcode`)
}
```

### 7.4 `src/api/ai.ts`

```ts
import request from '@/utils/request'

export function usagePrecheck(payload: {
  memberId: number
  usageType: 'OCR' | 'ANALYZE' | 'CHAT'
  idempotencyKey: string
}) {
  return request.post('/app-api/usage/precheck', payload, {
    headers: {
      'Idempotency-Key': payload.idempotencyKey
    }
  })
}

export function invokeOcr(payload: {
  memberId: number
  fileUrlList: string[]
  sceneCode: string
  idempotencyKey: string
}) {
  return request.post('/app-api/builtin-ai/ocr', payload, {
    headers: {
      'Idempotency-Key': payload.idempotencyKey
    }
  })
}

export function invokeAnalyze(payload: {
  memberId: number
  sourceText: string
  sourceUsageNo?: string
  sceneCode: string
  idempotencyKey: string
}) {
  return request.post('/app-api/builtin-ai/analyze', payload, {
    headers: {
      'Idempotency-Key': payload.idempotencyKey
    }
  })
}

export function invokeChat(payload: {
  memberId: number
  question: string
  conversationId?: string
  idempotencyKey: string
}) {
  return request.post('/app-api/builtin-ai/chat', payload, {
    headers: {
      'Idempotency-Key': payload.idempotencyKey
    }
  })
}
```

---

## 8. 页面调用示例

### 8.1 激活客户端

```ts
import { activateClient } from '@/api/auth'

async function doActivate() {
  const resp = await activateClient({
    clientId: 'desktop-tauri',
    deviceId: 'win10-7c1d9a8f',
    deviceName: 'Windows Desktop',
    deviceType: 'PC',
    osName: 'Windows',
    osVersion: '11',
    appVersion: '1.0.0',
    deviceFingerprint: 'fingerprint-sha256-value',
    credentialMode: 'HMAC',
    secretProof: 'init-secret-proof',
    activateTime: Date.now(),
    nonce: crypto.randomUUID().replace(/-/g, ''),
    signature: 'activate-signature-placeholder'
  })

  const data = resp.data.data
  localStorage.setItem('health-client-security', JSON.stringify({
    clientId: 'desktop-tauri',
    deviceId: 'win10-7c1d9a8f',
    appVersion: '1.0.0',
    deviceSecret: data.deviceSecret
  }))
}
```

### 8.2 登录并保存会话

```ts
import { loginByPassword } from '@/api/auth'

async function doLogin() {
  const resp = await loginByPassword({
    userName: 'demoUser',
    password: 'Demo123456',
    deviceId: 'win10-7c1d9a8f',
    deviceName: 'Windows Desktop'
  })

  const data = resp.data.data
  localStorage.setItem('health-access-token', data.accessToken)
  localStorage.setItem('health-refresh-token', data.refreshToken)
  localStorage.setItem('health-user-info', JSON.stringify(data.userInfo))
}
```

### 8.3 OCR 调用流程

```ts
import { usagePrecheck, invokeOcr } from '@/api/ai'

async function doOcr(memberId: number, fileUrl: string) {
  const idempotencyKey = `ocr-${Date.now()}`

  const precheckResp = await usagePrecheck({
    memberId,
    usageType: 'OCR',
    idempotencyKey
  })

  if (!precheckResp.data.data.canUse) {
    throw new Error('当前次数不足或命中风控，无法继续调用')
  }

  const ocrResp = await invokeOcr({
    memberId,
    fileUrlList: [fileUrl],
    sceneCode: 'MEDICAL_REPORT',
    idempotencyKey
  })

  return ocrResp.data.data
}
```

### 8.4 下单流程

```ts
import { createOrder, getPayQrcode, getOrderStatus } from '@/api/order'

async function doCreateOrder(skuId: number) {
  const idempotencyKey = `order-${Date.now()}`

  const orderResp = await createOrder({
    skuId,
    sourceChannel: 'DESKTOP',
    idempotencyKey
  })

  const orderNo = orderResp.data.data.orderNo
  const qrcodeResp = await getPayQrcode(orderNo)
  const qrcodeUrl = qrcodeResp.data.data.qrcodeUrl

  return {
    orderNo,
    qrcodeUrl
  }
}

async function pollOrderStatus(orderNo: string) {
  const resp = await getOrderStatus(orderNo)
  return resp.data.data
}
```

---

## 9. 流式对话示例

如果服务端最终采用 SSE，可以单独使用 `fetch` 处理流式响应：

```ts
export async function chatStream(accessToken: string, body: Record<string, unknown>) {
  const response = await fetch(`${import.meta.env.VITE_API_BASE_URL}/app-api/builtin-ai/chat/stream`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${accessToken}`
    },
    body: JSON.stringify(body)
  })

  const reader = response.body?.getReader()
  const decoder = new TextDecoder('utf-8')

  while (reader) {
    const { done, value } = await reader.read()
    if (done) break
    const chunk = decoder.decode(value, { stream: true })
    console.log(chunk)
  }
}
```

如果流式接口也要求设备签名，建议：

- 不要在页面组件里手工补签名
- 单独封装一个 `signedFetch` 工具函数

---

## 10. 接入建议

- 所有签名头统一在请求层处理，不要散落到页面逻辑中
- 登录态与设备凭证分开存储、分开失效
- AI、订单接口统一生成 `Idempotency-Key`
- 对 `CLIENT_DEVICE_FROZEN`、`CLIENT_DEVICE_REVOKED`、`CLIENT_VERSION_BLOCKED` 做明确用户提示
- 如果当前联调的是仓库内 MVP 实现，需额外确认接口是否仍要求显式传 `userId`