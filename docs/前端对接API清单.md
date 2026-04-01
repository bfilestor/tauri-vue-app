# 医疗健康项目前端对接 API 清单

## 1. 文档说明

- 文档目的：为前端开发提供当前仓库内“真实可对接”的接口清单与字段说明。
- 整理依据：
  - `doc/README.md`
  - `doc/服务端接口设计.md`
  - `qcy-admin` 下真实 Controller
  - `qcy-ui/src/api/health/*.js`、`qcy-ui/src/api/login.js`、`qcy-ui/src/api/menu.js`
- 适用范围：
  - 平台基础接口：登录、验证码、用户信息、路由、登出
  - 医疗健康业务接口：`/app-api/**`、`/admin-api/**`、`/open-api/**`
- 不纳入本文：
  - RuoYi 原生系统管理 CRUD（如角色、菜单、部门、字典、配置）
  - 监控、任务、代码生成、Swagger、历史 `advisor / AIS / ship` 相关接口

## 2. 当前实现特点

### 2.1 以真实代码为准

当前仓库内的健康业务接口，和 `doc/服务端接口设计.md` 的理想设计相比，已经落地了一批 MVP 接口，但仍有少量路径与返回结构差异。前端对接时，应优先以本文和真实 Controller 为准。

### 2.2 统一返回格式

除首页文本接口外，当前接口统一返回若依 `AjaxResult`：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {}
}
```

说明：

- 普通成功接口通常是 `AjaxResult.success(data)`
- 无返回体时通常是 `AjaxResult.success()`
- 部分接口返回的是字符串、布尔值、数组、整数，而不是对象

### 2.3 当前健康业务列表接口不是标准 `TableDataInfo`

虽然规范建议列表接口返回 `TableDataInfo`，但当前健康业务后台列表接口大多仍返回：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": []
}
```

前端现状：

- `qcy-ui/src/api/health/user.js`
- `qcy-ui/src/api/health/product.js`
- `qcy-ui/src/api/health/order.js`

这些文件已经在前端侧做了 `normalizeListResponse` 兼容，把数组转换成 `{ rows, total }`。

### 2.4 `/app-api/**` 当前仍显式传 `userId`

当前 App 业务接口里，账户中心、家庭成员、钱包、订单、AI 等接口很多仍通过 query/body 显式传 `userId`，尚未完全切换到“只从登录态上下文读取用户”的最终形态。前端对接时需要保留该字段。

### 2.5 当前实现包含 Mock / 内存态能力

当前仓库中的健康业务实现仍以 MVP 为主，主要表现为：

- 手机验证码、邮箱验证码为 Mock 发送
- 商品、订单、钱包、AI 日志、风控名单等大量数据为内存存储
- AI OCR / 分析 / 对话为 Mock 引擎
- 支付回调签名校验为固定 Mock 规则

这意味着：前端可以联调流程，但不要把当前返回样例视为最终生产协议。

## 3. 认证与通道约定

| 通道 | 前缀 | 当前要求 |
| --- | --- | --- |
| 平台接口 | `/login` `/getInfo` 等 | 后台登录链路 |
| App 业务接口 | `/app-api/**` | 认证类接口匿名，其余多数需要业务登录态；但当前实现很多还要求显式 `userId` |
| 后台业务接口 | `/admin-api/**` | 需要后台登录 token |
| 开放回调接口 | `/open-api/**` | 不走后台鉴权，按接口内部规则校验 |

常用请求头：

- `Authorization: Bearer {token}`
- `Content-Type: application/json`

## 4. 平台基础接口

> 这部分是后台前端启动后必须依赖的基础接口，属于若依平台能力，但也是当前项目前端运行的前置条件。

### 4.1 POST `/login`

- 接口名称：后台登录
- 功能说明：管理员登录，成功后返回后台 token
- 认证要求：否
- 请求体：`LoginBody`

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `username` | string | 是 | 后台用户名 |
| `password` | string | 是 | 后台密码 |
| `code` | string | 否 | 图形验证码 |
| `uuid` | string | 否 | 验证码唯一标识 |

- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `token` | string | 后台登录令牌 |

### 4.2 GET `/captchaImage`

- 接口名称：获取登录验证码
- 功能说明：返回后台登录页验证码
- 认证要求：否
- 请求参数：无
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `captchaEnabled` | boolean | 是否启用验证码 |
| `uuid` | string | 验证码唯一标识 |
| `img` | string | Base64 图片内容 |

### 4.3 GET `/getInfo`

- 接口名称：获取当前后台用户信息
- 功能说明：登录成功后拉取后台用户、角色、权限
- 认证要求：是
- 请求参数：无
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `user` | object | 若依后台用户对象 |
| `roles` | string[] | 角色编码列表 |
| `permissions` | string[] | 权限码列表 |
| `isDefaultModifyPwd` | boolean | 是否初始密码未修改 |
| `isPasswordExpired` | boolean | 密码是否过期 |

### 4.4 GET `/getRouters`

- 接口名称：获取后台路由
- 功能说明：获取当前后台用户可见菜单路由
- 认证要求：是
- 请求参数：无
- 响应体：`data` 为路由数组

### 4.5 POST `/logout`

- 接口名称：后台登出
- 功能说明：清理后台登录态
- 认证要求：是
- 请求参数：无
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `msg` | string | 一般为“退出成功” |

## 5. App 业务接口

## 5.1 认证接口

### 5.1.1 POST `/app-api/auth/send-phone-code`

- 接口名称：发送手机验证码
- 功能说明：向手机号发送登录验证码，当前为 Mock 通道
- 认证要求：否
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `phone` | string | 是 | 中国大陆手机号，格式需匹配 `^1\\d{10}$` |

- 响应体：`data` 为成功提示字符串

### 5.1.2 POST `/app-api/auth/login/phone-code`

- 接口名称：手机号验证码登录
- 功能说明：校验验证码后登录；若业务用户不存在会自动初始化
- 认证要求：否
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `phone` | string | 是 | 手机号 |
| `code` | string | 是 | 验证码 |
| `deviceId` | string | 否 | 设备 ID |
| `deviceName` | string | 否 | 设备名称 |

- 响应体：`data: PhoneCodeLoginResult`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `userId` | long | 业务用户 ID |
| `accessToken` | string | 业务访问令牌 |
| `refreshToken` | string | 刷新令牌 |
| `newlyRegistered` | boolean | 是否为首次自动注册 |

### 5.1.3 POST `/app-api/auth/send-email-code`

- 接口名称：发送邮箱验证码
- 功能说明：向邮箱发送注册验证码，当前为 Mock 通道
- 认证要求：否
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `email` | string | 是 | 邮箱地址 |

- 响应体：`data` 为成功提示字符串

### 5.1.4 POST `/app-api/auth/register/email`

- 接口名称：邮箱注册
- 功能说明：邮箱验证码校验通过后注册业务用户，并返回登录令牌
- 认证要求：否
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `email` | string | 是 | 邮箱 |
| `emailCode` | string | 是 | 邮箱验证码 |
| `username` | string | 是 | 用户名，长度 4~32 |
| `password` | string | 是 | 密码，8~64 位且需包含字母和数字 |
| `deviceId` | string | 否 | 设备 ID |
| `deviceName` | string | 否 | 设备名称 |

- 响应体：`data: PasswordLoginResult`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `userId` | long | 业务用户 ID |
| `accessToken` | string | 业务访问令牌 |
| `refreshToken` | string | 刷新令牌 |

### 5.1.5 POST `/app-api/auth/login/password`

- 接口名称：用户名密码登录
- 功能说明：使用业务用户名和密码登录
- 认证要求：否
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `username` | string | 是 | 用户名 |
| `password` | string | 是 | 明文密码 |
| `deviceId` | string | 否 | 设备 ID |
| `deviceName` | string | 否 | 设备名称 |

- 响应体：`data: PasswordLoginResult`

### 5.1.6 POST `/app-api/auth/refresh-token`

- 接口名称：刷新业务令牌
- 功能说明：使用业务 `refreshToken` 换新 token
- 认证要求：否
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `refreshToken` | string | 是 | 刷新令牌 |

- 响应体：`data: PasswordLoginResult`

### 5.1.7 POST `/app-api/auth/logout`

- 接口名称：业务登出
- 功能说明：从请求头读取 `Authorization` 中的 Bearer token 并注销
- 认证要求：建议是
- 请求参数：无
- 请求头：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `Authorization` | string | 是 | `Bearer {accessToken}` |

- 响应体：`data` 为成功提示字符串

## 5.2 账户中心接口

### 5.2.1 GET `/app-api/account/profile`

- 接口名称：账户中心-用户资料
- 功能说明：查询当前业务用户的基础资料摘要
- 认证要求：当前实现仍显式传 `userId`
- 请求参数：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 业务用户 ID |

- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `userId` | long | 用户 ID |
| `nickname` | string | 当前为 `U{userId}` |
| `avatar` | string | 头像地址，当前为空字符串 |

### 5.2.2 GET `/app-api/account/balance`

- 接口名称：账户中心-钱包余额
- 功能说明：查询业务用户次数余额
- 请求参数：`userId`
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `userId` | long | 用户 ID |
| `balanceTimes` | int | 当前次数余额 |

### 5.2.3 GET `/app-api/account/orders`

- 接口名称：账户中心-我的订单
- 功能说明：查询业务用户订单列表
- 请求参数：`userId`
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `userId` | long | 用户 ID |
| `items` | `HealthOrderDO[]` | 订单数组 |

### 5.2.4 GET `/app-api/account/ledger`

- 接口名称：账户中心-我的台账
- 功能说明：查询业务用户钱包流水
- 请求参数：`userId`
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `userId` | long | 用户 ID |
| `items` | `HealthWalletLedgerDO[]` | 钱包流水数组 |

### 5.2.5 GET `/app-api/account/devices`

- 接口名称：账户中心-登录设备
- 功能说明：查询业务用户设备列表
- 请求参数：`userId`
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `userId` | long | 用户 ID |
| `items` | array | 当前实现固定返回空数组 |

## 5.3 家庭成员接口

### 5.3.1 POST `/app-api/family/member/create`

- 接口名称：新增家庭成员
- 功能说明：创建家庭成员，首个成员会自动设为默认成员
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 业务用户 ID |
| `name` | string | 是 | 成员姓名 |
| `relation` | string | 是 | 与用户关系 |
| `age` | int | 是 | 年龄，0~150 |

- 响应体：`data: HealthFamilyMemberDO`

### 5.3.2 PUT `/app-api/family/member/update`

- 接口名称：修改家庭成员
- 功能说明：更新家庭成员基本信息
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 业务用户 ID |
| `memberId` | long | 是 | 成员 ID |
| `name` | string | 是 | 成员姓名 |
| `relation` | string | 是 | 与用户关系 |
| `age` | int | 是 | 年龄 |

- 响应体：`data: HealthFamilyMemberDO`

### 5.3.3 DELETE `/app-api/family/member/{memberId}`

- 接口名称：删除家庭成员
- 功能说明：删除指定成员；若删除的是默认成员，会自动重设新的默认成员
- 路径参数：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `memberId` | long | 成员 ID |

- Query 参数：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 业务用户 ID |

- 响应体：无业务 `data`

### 5.3.4 GET `/app-api/family/member/list`

- 接口名称：家庭成员列表
- 功能说明：查询某业务用户的全部家庭成员
- Query 参数：`userId`
- 响应体：`data: HealthFamilyMemberDO[]`

### 5.3.5 POST `/app-api/family/member/{memberId}/set-default`

- 接口名称：设置默认成员
- 功能说明：将指定成员设为默认成员
- 路径参数：`memberId`
- Query 参数：`userId`
- 响应体：无业务 `data`

## 5.4 商品接口

### 5.4.1 GET `/app-api/products`

- 接口名称：前台商品列表
- 功能说明：查询当前可售商品列表
- 请求参数：无
- 响应体：`data: HealthProductDO[]`
- 说明：仅返回 `online=true` 且至少有一个可用 SKU 的商品

### 5.4.2 GET `/app-api/products/{productId}`

- 接口名称：前台商品详情
- 功能说明：查询可售商品详情
- 路径参数：`productId`
- 响应体：`data: HealthProductDO`
- 说明：若商品未上架或无可售 SKU，会返回错误

## 5.5 订单接口

### 5.5.1 POST `/app-api/orders`

- 接口名称：创建订单
- 功能说明：按商品 SKU 创建待支付订单
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 业务用户 ID |
| `productId` | long | 是 | 商品 ID |
| `skuId` | long | 是 | SKU ID |
| `payChannel` | string | 是 | 支付渠道，仅支持 `WECHAT` 或 `ALIPAY` |

- 响应体：`data: HealthOrderDO`

### 5.5.2 GET `/app-api/orders/{orderNo}/pay-qrcode`

- 接口名称：获取支付二维码
- 功能说明：根据订单生成 Mock 支付二维码信息
- 路径参数：`orderNo`
- Query 参数：`userId`
- 响应体：`data: PayQrcodeVO`

### 5.5.3 GET `/app-api/orders/{orderNo}`

- 接口名称：订单详情
- 功能说明：查询当前用户自己的订单详情
- 路径参数：`orderNo`
- Query 参数：`userId`
- 响应体：`data: HealthOrderDO`

### 5.5.4 GET `/app-api/orders/{orderNo}/status`

- 接口名称：订单状态查询
- 功能说明：查询订单当前状态
- 路径参数：`orderNo`
- Query 参数：`userId`
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `data` | string | 订单状态，如 `WAIT_PAY`、`PAY_SUCCESS`、`CANCELED`、`REFUNDED` |

### 5.5.5 POST `/app-api/orders/{orderNo}/cancel`

- 接口名称：取消订单
- 功能说明：仅待支付订单允许取消
- 路径参数：`orderNo`
- Query 参数：`userId`
- 响应体：无业务 `data`

## 5.6 钱包接口

### 5.6.1 GET `/app-api/wallet`

- 接口名称：钱包余额
- 功能说明：查询业务用户当前次数余额
- Query 参数：`userId`
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `data` | int | 当前余额次数 |

### 5.6.2 GET `/app-api/wallet/ledger`

- 接口名称：钱包流水
- 功能说明：查询业务用户的钱包台账
- Query 参数：`userId`
- 响应体：`data: HealthWalletLedgerDO[]`

### 5.6.3 POST `/app-api/wallet/adjust`

- 接口名称：钱包变更
- 功能说明：MVP 通用调整入口，可用于支付到账、退款回收、人工补偿等场景
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 业务用户 ID |
| `deltaTimes` | int | 是 | 次数变更值，正数增加，负数扣减 |
| `bizType` | string | 是 | 业务类型 |
| `bizNo` | string | 是 | 业务编号，参与幂等 |

- 响应体：`data: HealthWalletLedgerDO`
- 说明：若幂等命中，当前实现会返回 `null`

## 5.7 AI 接口

### 5.7.1 POST `/app-api/ai/precheck`

- 接口名称：AI 调用预检查
- 功能说明：校验成员归属、余额、风控、幂等键
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 业务用户 ID |
| `memberId` | long | 是 | 家庭成员 ID |
| `idempotencyKey` | string | 是 | 幂等键 |

- 响应体：`data: AiPrecheckResult`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `pass` | boolean | 是否通过预检 |
| `reasonCode` | string | 拒绝原因编码，通过时通常为空 |
| `reasonMessage` | string | 拒绝原因文案，通过时通常为空 |

### 5.7.2 POST `/app-api/ai/ocr`

- 接口名称：OCR 识别
- 功能说明：执行 OCR 调用，成功扣减 1 次；失败会尝试返还
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 业务用户 ID |
| `memberId` | long | 是 | 成员 ID |
| `idempotencyKey` | string | 是 | 幂等键 |
| `imageUrl` | string | 是 | 图片地址 |

- 响应体：`data: OcrInvokeResult`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `success` | boolean | 是否成功 |
| `text` | string | OCR 文本，失败时可能为空 |
| `traceId` | string | 调用追踪 ID |
| `message` | string | 成功或失败说明 |

### 5.7.3 POST `/app-api/ai/analysis`

- 接口名称：综合分析
- 功能说明：执行分析调用，成功扣减 1 次；失败会尝试返还
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 业务用户 ID |
| `memberId` | long | 是 | 成员 ID |
| `idempotencyKey` | string | 是 | 幂等键 |
| `content` | string | 是 | 待分析文本 |

- 响应体：`data: AnalysisInvokeResult`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `success` | boolean | 是否成功 |
| `summary` | string | 分析摘要，失败时可能为空 |
| `traceId` | string | 调用追踪 ID |
| `message` | string | 成功或失败说明 |

### 5.7.4 POST `/app-api/ai/chat`

- 接口名称：AI 问答
- 功能说明：执行问答调用，支持前端声明是否流式，但当前仍以普通 JSON 返回
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 业务用户 ID |
| `memberId` | long | 是 | 成员 ID |
| `idempotencyKey` | string | 是 | 幂等键 |
| `question` | string | 是 | 问题内容 |
| `stream` | boolean | 否 | 是否声明流式 |
| `interrupted` | boolean | 否 | 是否模拟中断 |

- 响应体：`data: ChatInvokeResult`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `success` | boolean | 是否成功 |
| `traceId` | string | 调用追踪 ID |
| `answer` | string | 完整答案 |
| `chunks` | string[] | 分片内容 |
| `message` | string | 成功或失败说明 |

### 5.7.5 GET `/app-api/ai/logs`

- 接口名称：我的 AI 调用日志
- 功能说明：查询业务用户自己的 AI 调用记录
- Query 参数：`userId`
- 响应体：`data: AiCallLogDO[]`

## 6. 后台业务接口

## 6.1 业务用户接口

### 6.1.1 POST `/admin-api/health-users/{userId}/bootstrap`

- 接口名称：初始化业务用户占位数据
- 功能说明：若用户不存在则创建最小业务用户对象，便于后台演示/测试
- 路径参数：`userId`
- 响应体：`data: HealthUserDO`

### 6.1.2 GET `/admin-api/health-users/page`

- 接口名称：业务用户分页列表
- 功能说明：查询业务用户列表
- Query 参数：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `pageNum` | int | 否 | 默认 `1` |
| `pageSize` | int | 否 | 默认 `10` |

- 响应体：`data: HealthUserPageItem[]`

`HealthUserPageItem` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `userId` | long | 用户 ID |
| `username` | string | 用户名 |
| `phone` | string | 手机号 |
| `email` | string | 邮箱 |
| `lastLoginTime` | datetime | 最后登录时间 |
| `enabled` | boolean | 是否启用 |

### 6.1.3 GET `/admin-api/health-users/{userId}`

- 接口名称：业务用户详情
- 功能说明：返回用户详情、启停状态、家庭成员列表
- 路径参数：`userId`
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `user` | `HealthUserDO` | 业务用户对象 |
| `enabled` | boolean | 启停状态 |
| `members` | `HealthFamilyMemberDO[]` | 家庭成员列表 |

### 6.1.4 PUT `/admin-api/health-users/{userId}/status`

- 接口名称：切换业务用户状态
- 功能说明：启用或禁用业务用户
- 路径参数：`userId`
- Query 参数：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | int | 是 | `1` 表示启用，其他值视为禁用 |

- 响应体：无业务 `data`

## 6.2 商品管理接口

### 6.2.1 GET `/admin-api/products/page`

- 接口名称：商品分页列表
- 功能说明：查询全部商品
- Query 参数：`pageNum`、`pageSize`
- 响应体：`data: HealthProductDO[]`

### 6.2.2 POST `/admin-api/products`

- 接口名称：新增商品
- 功能说明：创建商品及其 SKU 列表
- 请求体：`ProductSaveReqVO`

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `productName` | string | 是 | 商品名称 |
| `productDesc` | string | 否 | 商品描述 |
| `skuList` | `ProductSkuReqVO[]` | 是 | SKU 列表，不能为空 |

`ProductSkuReqVO` 字段：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `skuName` | string | 是 | SKU 名称 |
| `priceFen` | int | 是 | 价格，单位分 |
| `times` | int | 是 | 可兑换次数 |

- 响应体：`data: HealthProductDO`

### 6.2.3 PUT `/admin-api/products/{productId}`

- 接口名称：修改商品
- 功能说明：更新商品与 SKU 列表
- 路径参数：`productId`
- 请求体：同新增商品
- 响应体：`data: HealthProductDO`

### 6.2.4 DELETE `/admin-api/products/{productId}`

- 接口名称：删除商品
- 功能说明：删除指定商品
- 路径参数：`productId`
- 响应体：无业务 `data`

### 6.2.5 PUT `/admin-api/products/{productId}/status`

- 接口名称：商品上下架
- 功能说明：切换商品是否可售
- 路径参数：`productId`
- Query 参数：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | int | 是 | `1` 上架，其他值下架 |

- 响应体：无业务 `data`

## 6.3 订单管理接口

### 6.3.1 GET `/admin-api/orders/page`

- 接口名称：订单分页列表
- 功能说明：查询全部订单
- Query 参数：`pageNum`、`pageSize`
- 响应体：`data: HealthOrderDO[]`

### 6.3.2 GET `/admin-api/orders/{orderNo}`

- 接口名称：订单详情
- 功能说明：查询指定订单详情
- 路径参数：`orderNo`
- 响应体：`data: HealthOrderDO`

### 6.3.3 POST `/admin-api/orders/mark-paid`

- 接口名称：后台人工置支付成功
- 功能说明：把订单置为支付成功，并给用户发放次数
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `orderNo` | string | 是 | 订单号 |

- 响应体：无业务 `data`

### 6.3.4 POST `/admin-api/orders/refund`

- 接口名称：后台订单退款
- 功能说明：把支付成功订单置为退款，并回收次数
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `orderNo` | string | 是 | 订单号 |

- 响应体：无业务 `data`

## 6.4 钱包管理接口

### 6.4.1 GET `/admin-api/wallet/{userId}`

- 接口名称：钱包摘要
- 功能说明：查询指定用户的钱包余额与台账
- 路径参数：`userId`
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `userId` | long | 用户 ID |
| `balance` | int | 当前余额 |
| `ledgerList` | `HealthWalletLedgerDO[]` | 流水列表 |

### 6.4.2 POST `/admin-api/wallet/compensate`

- 接口名称：钱包补偿
- 功能说明：后台人工增加次数
- 请求体：`AdminWalletAdjustReqVO`

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 用户 ID |
| `times` | int | 是 | 补偿次数，必须大于 0 |
| `bizNo` | string | 是 | 业务编号 |

- 响应体：`data: HealthWalletLedgerDO`

### 6.4.3 POST `/admin-api/wallet/refund`

- 接口名称：钱包回收
- 功能说明：后台人工回收次数
- 请求体：同上
- 响应体：`data: HealthWalletLedgerDO`
- 说明：内部会将 `times` 转为负数写入台账

## 6.5 AI 审计与风控接口

### 6.5.1 GET `/admin-api/ai/audit/logs`

- 接口名称：AI 全量日志
- 功能说明：查询全部 AI 调用日志
- 请求参数：无
- 响应体：`data: AiCallLogDO[]`

### 6.5.2 GET `/admin-api/ai/audit/exceptions`

- 接口名称：AI 异常日志
- 功能说明：仅返回状态为 `FAILED` 的 AI 调用日志
- 请求参数：无
- 响应体：`data: AiCallLogDO[]`

### 6.5.3 POST `/admin-api/ai/audit/risk/block`

- 接口名称：拉黑 AI 用户
- 功能说明：把指定用户加入风控黑名单
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 用户 ID |

- 响应体：无业务 `data`

### 6.5.4 POST `/admin-api/ai/audit/risk/unblock`

- 接口名称：解除 AI 用户拉黑
- 功能说明：把指定用户从风控黑名单移除
- 请求体：同上
- 响应体：无业务 `data`

### 6.5.5 GET `/admin-api/ai/audit/risk/blocked-users`

- 接口名称：拉黑用户列表
- 功能说明：查询当前被拉黑的用户 ID 列表
- 请求参数：无
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `data` | long[] | 被拉黑的用户 ID 列表 |

### 6.5.6 POST `/admin-api/ai/audit/risk/rate-limit`

- 接口名称：设置限流
- 功能说明：为指定用户设置每分钟限额；传空或小于等于 0 会移除限流
- 请求体：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `userId` | long | 是 | 用户 ID |
| `perMinute` | int | 否 | 每分钟上限 |

- 响应体：无业务 `data`

### 6.5.7 GET `/admin-api/ai/audit/risk/rate-limits`

- 接口名称：限流配置列表
- 功能说明：查询当前所有用户限流配置
- 请求参数：无
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `data` | object | `key=userId`，`value=perMinute` |

## 6.6 运营辅助接口

### 6.6.1 GET `/admin-api/ops/daily-report`

- 接口名称：运营日报
- 功能说明：返回 MVP 版日报摘要
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `totalOrders` | int | 订单总数 |
| `aiCalls` | int | AI 调用总数 |
| `note` | string | 说明文案 |

### 6.6.2 GET `/admin-api/ops/alerts/check`

- 接口名称：告警检查
- 功能说明：返回 AI 异常调用告警状态
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `failedAiCalls` | int | 异常 AI 调用数 |
| `alert` | string | `OK` 或 `AI_FAILURE_HIGH` |

### 6.6.3 POST `/admin-api/ops/compensation/run`

- 接口名称：补偿工具执行
- 功能说明：执行补偿工具占位流程
- 请求参数：无
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `executed` | boolean | 是否执行 |
| `message` | string | 返回说明 |

### 6.6.4 GET `/admin-api/ops/schedule/prepare`

- 接口名称：计划任务准备信息
- 功能说明：返回建议的 Cron 配置
- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `cronDailyReport` | string | 日报 cron |
| `cronAlertCheck` | string | 告警检查 cron |
| `cronCompensate` | string | 补偿 cron |
| `status` | string | 当前为 `prepared` |

## 6.7 后台权限辅助接口

> 这一组接口更偏后台管理辅助能力，不是最终业务接口形态，但当前仓库中已真实存在。

### 6.7.1 GET `/admin-api/system/permissions`

- 接口名称：权限码列表
- 功能说明：返回当前健康后台预置权限码
- 请求参数：无
- 响应体：`data: string[]`

当前预置值包括：

- `health:user:query`
- `health:user:update`
- `health:user:bootstrap`
- `health:product:query`
- `health:product:operate`
- `health:order:query`
- `health:order:operate`
- `health:order:refund`
- `health:wallet:query`
- `health:wallet:operate`
- `health:ai:audit`
- `health:ai:risk`
- `health:ops:query`
- `health:ops:operate`

### 6.7.2 GET `/admin-api/system/action-logs`

- 接口名称：后台操作日志
- 功能说明：查询后台动作日志列表
- 请求参数：无
- 响应体：`data: AdminActionLogDO[]`

### 6.7.3 GET `/admin-api/system/record-action`

- 接口名称：记录后台动作
- 功能说明：根据权限码记录一条后台操作日志
- 说明：当前使用 `GET` 写数据，仅适合内部演示与测试，不建议作为正式对外协议
- Query 参数：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `permissionCode` | string | 是 | 权限码，必须存在于预置清单中 |
| `operator` | string | 否 | 操作人，缺省为 `system` |
| `target` | string | 否 | 操作目标 |

- 响应体：无业务 `data`

## 7. 开放回调接口

### 7.1 POST `/open-api/pay/wechat/callback`

- 接口名称：微信支付回调
- 功能说明：处理支付回调，并在成功时驱动订单状态更新与钱包发放
- 认证要求：否
- 请求体：`PayCallbackReqVO`

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `orderNo` | string | 是 | 订单号 |
| `channel` | string | 是 | 支付渠道 |
| `callbackId` | string | 是 | 回调唯一 ID，参与幂等 |
| `sign` | string | 是 | 当前 Mock 实现要求固定为 `mock-sign-ok` |

- 响应体：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `data` | boolean | `true` 表示首次处理成功；`false` 表示重复回调、订单已处理或已取消 |

## 8. 主要数据结构

## 8.1 `HealthFamilyMemberDO`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `memberId` | long | 成员 ID |
| `userId` | long | 所属用户 ID |
| `name` | string | 成员姓名 |
| `relation` | string | 与用户关系 |
| `age` | int | 年龄 |
| `default` | boolean | 是否默认成员 |
| `createdTime` | datetime | 创建时间 |
| `updatedTime` | datetime | 更新时间 |

## 8.2 `HealthProductDO`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `productId` | long | 商品 ID |
| `productName` | string | 商品名称 |
| `productDesc` | string | 商品描述 |
| `online` | boolean | 是否上架 |
| `createdTime` | datetime | 创建时间 |
| `updatedTime` | datetime | 更新时间 |
| `skuList` | `HealthProductSkuDO[]` | SKU 列表 |

`HealthProductSkuDO` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `skuId` | long | SKU ID |
| `productId` | long | 所属商品 ID |
| `skuName` | string | SKU 名称 |
| `priceFen` | int | 价格，单位分 |
| `times` | int | 可兑换次数 |
| `enabled` | boolean | 是否启用 |

## 8.3 `HealthOrderDO`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `orderNo` | string | 订单号 |
| `userId` | long | 用户 ID |
| `productId` | long | 商品 ID |
| `skuId` | long | SKU ID |
| `skuName` | string | SKU 名称 |
| `priceFen` | int | 订单金额，单位分 |
| `times` | int | 对应次数权益 |
| `payChannel` | string | 支付渠道 |
| `status` | string | 订单状态 |
| `createdTime` | datetime | 创建时间 |

## 8.4 `PayQrcodeVO`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `orderNo` | string | 订单号 |
| `payChannel` | string | 支付渠道 |
| `qrcodeUrl` | string | Mock 二维码地址 |

## 8.5 `HealthWalletLedgerDO`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `ledgerNo` | string | 台账号 |
| `userId` | long | 用户 ID |
| `deltaTimes` | int | 本次变更值 |
| `balanceAfter` | int | 变更后余额 |
| `bizType` | string | 业务类型 |
| `bizNo` | string | 业务编号 |
| `createdTime` | datetime | 创建时间 |

## 8.6 `AiCallLogDO`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `traceId` | string | 调用追踪 ID |
| `userId` | long | 用户 ID |
| `memberId` | long | 成员 ID |
| `bizType` | string | 业务类型，如 `OCR`、`ANALYSIS`、`CHAT` |
| `status` | string | 调用状态，如 `SUCCESS`、`FAILED` |
| `requestPayload` | string | 请求内容 |
| `responsePayload` | string | 响应内容 |
| `errorMessage` | string | 错误信息 |
| `createdTime` | datetime | 创建时间 |

## 8.7 `AdminActionLogDO`

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `actionId` | string | 动作 ID |
| `permissionCode` | string | 权限码 |
| `operator` | string | 操作人 |
| `target` | string | 操作目标 |
| `createdTime` | datetime | 创建时间 |

## 9. 前端对接建议

### 9.1 建议优先对接的后台模块

- 平台基础登录：`/login`、`/captchaImage`、`/getInfo`、`/getRouters`、`/logout`
- 健康后台：用户、商品、订单、钱包、AI 审计、运营工具

### 9.2 当前最需要前端显式兼容的点

- 健康后台列表接口当前多数返回数组，不是 `TableDataInfo`
- App 业务接口当前很多仍需传 `userId`
- AI / 支付 / 短信 / 邮件等为 Mock 流程，前端联调时以状态流转是否正常为主
- `/admin-api/system/record-action` 使用 `GET` 写数据，仅可作为临时能力

### 9.3 如果后续服务端继续按规范收敛

后续若服务端继续对齐 `AGENTS.md` 规范，优先会发生的变化通常是：

- 列表接口统一改成 `TableDataInfo`
- App 用户身份改为从登录态上下文读取，不再显式传 `userId`
- AI / 支付 / 钱包等对象结构会从当前 MVP 结构演进为更明确的 `ReqVO / RespVO`

因此，前端在封装 API 层时建议保留一层适配层，不要把当前返回结构直接散落到页面中。
