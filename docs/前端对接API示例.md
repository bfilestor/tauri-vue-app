# 医疗健康项目前端对接 API JSON 示例

## 1. 文档说明

- 文档目的：提供接近 Swagger / OpenAPI 阅读习惯的 JSON 请求响应示例，方便前端联调。
- 对齐范围：以当前仓库内真实已实现接口为准。
- 配套文档：`doc/前端对接API清单.md`
- 统一返回格式：除首页文本接口外，其余接口默认返回 `AjaxResult`

统一成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {}
}
```

统一失败响应示例：

```json
{
  "code": 500,
  "msg": "业务异常说明"
}
```

常用请求头示例：

```http
Authorization: Bearer your-token
Content-Type: application/json
```

## 2. 平台基础接口

### 2.1 POST `/login`

说明：后台管理员登录。

请求示例：

```json
{
  "username": "admin",
  "password": "Admin@123",
  "code": "8x2f",
  "uuid": "8d9d1e7e4c8e4e9bbf0a123456789abc"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "token": "eyJhbGciOiJIUzUxMiJ9.mock-token"
}
```

### 2.2 GET `/captchaImage`

说明：获取后台登录验证码。

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "captchaEnabled": true,
  "uuid": "8d9d1e7e4c8e4e9bbf0a123456789abc",
  "img": "/9j/4AAQSkZJRgABAQAAAQABAAD..."
}
```

### 2.3 GET `/getInfo`

说明：获取当前后台用户信息、角色、权限。

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "user": {
    "userId": 1,
    "deptId": 103,
    "userName": "admin",
    "nickName": "若依",
    "email": "admin@qcy.com",
    "phonenumber": "13800000000"
  },
  "roles": [
    "admin"
  ],
  "permissions": [
    "*:*:*"
  ],
  "isDefaultModifyPwd": false,
  "isPasswordExpired": false
}
```

### 2.4 GET `/getRouters`

说明：获取后台菜单路由。

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "name": "System",
      "path": "/system",
      "hidden": false,
      "component": "Layout",
      "meta": {
        "title": "系统管理",
        "icon": "system"
      },
      "children": [
        {
          "name": "User",
          "path": "user",
          "component": "system/user/index",
          "meta": {
            "title": "用户管理",
            "icon": "user"
          }
        }
      ]
    }
  ]
}
```

### 2.5 POST `/logout`

说明：后台登出。

成功响应示例：

```json
{
  "code": 200,
  "msg": "退出成功"
}
```

## 3. App 业务接口

## 3.1 认证接口

### 3.1.1 POST `/app-api/auth/send-phone-code`

请求示例：

```json
{
  "phone": "13800138000"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": "验证码已发送（当前为 Mock 通道）"
}
```

### 3.1.2 POST `/app-api/auth/login/phone-code`

请求示例：

```json
{
  "phone": "13800138000",
  "code": "123456",
  "deviceId": "desktop-win-001",
  "deviceName": "Windows Desktop"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "userId": 10001,
    "accessToken": "app-access-token-001",
    "refreshToken": "app-refresh-token-001",
    "newlyRegistered": true
  }
}
```

### 3.1.3 POST `/app-api/auth/send-email-code`

请求示例：

```json
{
  "email": "demo@example.com"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": "邮箱验证码已发送（当前为 Mock 通道）"
}
```

### 3.1.4 POST `/app-api/auth/register/email`

请求示例：

```json
{
  "email": "demo@example.com",
  "emailCode": "654321",
  "username": "demoUser",
  "password": "Demo123456",
  "deviceId": "desktop-win-001",
  "deviceName": "Windows Desktop"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "userId": 10002,
    "accessToken": "app-access-token-002",
    "refreshToken": "app-refresh-token-002"
  }
}
```

### 3.1.5 POST `/app-api/auth/login/password`

请求示例：

```json
{
  "username": "demoUser",
  "password": "Demo123456",
  "deviceId": "desktop-win-001",
  "deviceName": "Windows Desktop"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "userId": 10002,
    "accessToken": "app-access-token-003",
    "refreshToken": "app-refresh-token-003"
  }
}
```

### 3.1.6 POST `/app-api/auth/refresh-token`

请求示例：

```json
{
  "refreshToken": "app-refresh-token-003"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "userId": 10002,
    "accessToken": "app-access-token-004",
    "refreshToken": "app-refresh-token-004"
  }
}
```

### 3.1.7 POST `/app-api/auth/logout`

说明：从请求头读取 Bearer token。

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": "退出成功"
}
```

## 3.2 账户中心接口

### 3.2.1 GET `/app-api/account/profile?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "userId": 10002,
    "nickname": "U10002",
    "avatar": ""
  }
}
```

### 3.2.2 GET `/app-api/account/balance?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "userId": 10002,
    "balanceTimes": 9
  }
}
```

### 3.2.3 GET `/app-api/account/orders?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "userId": 10002,
    "items": [
      {
        "orderNo": "HG202603301230001001",
        "userId": 10002,
        "productId": 5001,
        "skuId": 9001,
        "skuName": "OCR 10次包",
        "priceFen": 1990,
        "times": 10,
        "payChannel": "WECHAT",
        "status": "PAY_SUCCESS",
        "createdTime": "2026-03-30 12:30:00"
      }
    ]
  }
}
```

### 3.2.4 GET `/app-api/account/ledger?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "userId": 10002,
    "items": [
      {
        "ledgerNo": "WL202603301231002001",
        "userId": 10002,
        "deltaTimes": 10,
        "balanceAfter": 10,
        "bizType": "ORDER_PAY_SUCCESS",
        "bizNo": "HG202603301230001001",
        "createdTime": "2026-03-30 12:31:00"
      }
    ]
  }
}
```

### 3.2.5 GET `/app-api/account/devices?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "userId": 10002,
    "items": []
  }
}
```

## 3.3 家庭成员接口

### 3.3.1 POST `/app-api/family/member/create`

请求示例：

```json
{
  "userId": 10002,
  "name": "张三",
  "relation": "SELF",
  "age": 32
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "memberId": 10001,
    "userId": 10002,
    "name": "张三",
    "relation": "SELF",
    "age": 32,
    "default": true,
    "createdTime": "2026-03-30 12:40:00",
    "updatedTime": "2026-03-30 12:40:00"
  }
}
```

### 3.3.2 PUT `/app-api/family/member/update`

请求示例：

```json
{
  "userId": 10002,
  "memberId": 10001,
  "name": "张三",
  "relation": "SELF",
  "age": 33
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "memberId": 10001,
    "userId": 10002,
    "name": "张三",
    "relation": "SELF",
    "age": 33,
    "default": true,
    "createdTime": "2026-03-30 12:40:00",
    "updatedTime": "2026-03-30 12:45:00"
  }
}
```

### 3.3.3 DELETE `/app-api/family/member/10001?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功"
}
```

### 3.3.4 GET `/app-api/family/member/list?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "memberId": 10001,
      "userId": 10002,
      "name": "张三",
      "relation": "SELF",
      "age": 33,
      "default": true,
      "createdTime": "2026-03-30 12:40:00",
      "updatedTime": "2026-03-30 12:45:00"
    }
  ]
}
```

### 3.3.5 POST `/app-api/family/member/10001/set-default?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功"
}
```

## 3.4 商品接口

### 3.4.1 GET `/app-api/products`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "productId": 5001,
      "productName": "OCR 识别次卡",
      "productDesc": "适用于 OCR 识别场景",
      "online": true,
      "createdTime": "2026-03-30 10:00:00",
      "updatedTime": "2026-03-30 10:30:00",
      "skuList": [
        {
          "skuId": 9001,
          "productId": 5001,
          "skuName": "OCR 10次包",
          "priceFen": 1990,
          "times": 10,
          "enabled": true
        }
      ]
    }
  ]
}
```

### 3.4.2 GET `/app-api/products/5001`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "productId": 5001,
    "productName": "OCR 识别次卡",
    "productDesc": "适用于 OCR 识别场景",
    "online": true,
    "createdTime": "2026-03-30 10:00:00",
    "updatedTime": "2026-03-30 10:30:00",
    "skuList": [
      {
        "skuId": 9001,
        "productId": 5001,
        "skuName": "OCR 10次包",
        "priceFen": 1990,
        "times": 10,
        "enabled": true
      }
    ]
  }
}
```

## 3.5 订单接口

### 3.5.1 POST `/app-api/orders`

请求示例：

```json
{
  "userId": 10002,
  "productId": 5001,
  "skuId": 9001,
  "payChannel": "WECHAT"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "orderNo": "HG202603301250001002",
    "userId": 10002,
    "productId": 5001,
    "skuId": 9001,
    "skuName": "OCR 10次包",
    "priceFen": 1990,
    "times": 10,
    "payChannel": "WECHAT",
    "status": "WAIT_PAY",
    "createdTime": "2026-03-30 12:50:00"
  }
}
```

### 3.5.2 GET `/app-api/orders/HG202603301250001002/pay-qrcode?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "orderNo": "HG202603301250001002",
    "payChannel": "WECHAT",
    "qrcodeUrl": "mockpay://wechat/qrcode/HG202603301250001002"
  }
}
```

### 3.5.3 GET `/app-api/orders/HG202603301250001002?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "orderNo": "HG202603301250001002",
    "userId": 10002,
    "productId": 5001,
    "skuId": 9001,
    "skuName": "OCR 10次包",
    "priceFen": 1990,
    "times": 10,
    "payChannel": "WECHAT",
    "status": "WAIT_PAY",
    "createdTime": "2026-03-30 12:50:00"
  }
}
```

### 3.5.4 GET `/app-api/orders/HG202603301250001002/status?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": "WAIT_PAY"
}
```

### 3.5.5 POST `/app-api/orders/HG202603301250001002/cancel?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功"
}
```

## 3.6 钱包接口

### 3.6.1 GET `/app-api/wallet?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": 10
}
```

### 3.6.2 GET `/app-api/wallet/ledger?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "ledgerNo": "WL202603301255002001",
      "userId": 10002,
      "deltaTimes": 10,
      "balanceAfter": 10,
      "bizType": "ORDER_PAY_SUCCESS",
      "bizNo": "HG202603301250001002",
      "createdTime": "2026-03-30 12:55:00"
    }
  ]
}
```

### 3.6.3 POST `/app-api/wallet/adjust`

请求示例：

```json
{
  "userId": 10002,
  "deltaTimes": 5,
  "bizType": "MANUAL_COMPENSATE",
  "bizNo": "OPS-20260330-001"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "ledgerNo": "WL202603301300002002",
    "userId": 10002,
    "deltaTimes": 5,
    "balanceAfter": 15,
    "bizType": "MANUAL_COMPENSATE",
    "bizNo": "OPS-20260330-001",
    "createdTime": "2026-03-30 13:00:00"
  }
}
```

## 3.7 AI 接口

### 3.7.1 POST `/app-api/ai/precheck`

请求示例：

```json
{
  "userId": 10002,
  "memberId": 10001,
  "idempotencyKey": "ai-precheck-20260330-001"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "pass": true,
    "reasonCode": null,
    "reasonMessage": null
  }
}
```

失败响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "pass": false,
    "reasonCode": "BALANCE_NOT_ENOUGH",
    "reasonMessage": "钱包次数不足"
  }
}
```

### 3.7.2 POST `/app-api/ai/ocr`

请求示例：

```json
{
  "userId": 10002,
  "memberId": 10001,
  "idempotencyKey": "ocr-20260330-001",
  "imageUrl": "https://static.example.com/ocr/report-001.png"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "success": true,
    "text": "姓名：张三；检查项：血常规；结论：建议复查。",
    "traceId": "a1b2c3d4e5f678901234567890abcdef",
    "message": "OCR 调用成功"
  }
}
```

失败响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "success": false,
    "text": null,
    "traceId": "a1b2c3d4e5f678901234567890abcdef",
    "message": "OCR_RUNTIME_ERROR:图片地址不可访问"
  }
}
```

### 3.7.3 POST `/app-api/ai/analysis`

请求示例：

```json
{
  "userId": 10002,
  "memberId": 10001,
  "idempotencyKey": "analysis-20260330-001",
  "content": "血红蛋白偏低，白细胞正常，建议结合临床症状判断。"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "success": true,
    "summary": "综合分析结果：当前报告主要提示轻度贫血倾向，建议结合既往病史进一步复查。",
    "traceId": "f1e2d3c4b5a678901234567890abcdef",
    "message": "综合分析调用成功"
  }
}
```

### 3.7.4 POST `/app-api/ai/chat`

请求示例：

```json
{
  "userId": 10002,
  "memberId": 10001,
  "idempotencyKey": "chat-20260330-001",
  "question": "这份报告需要重点关注什么？",
  "stream": false,
  "interrupted": false
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "success": true,
    "traceId": "00112233445566778899aabbccddeeff",
    "answer": "建议重点关注异常指标变化趋势，并结合医生意见决定是否复查。",
    "chunks": [
      "建议重点关注异常指标变化趋势，",
      "并结合医生意见决定是否复查。"
    ],
    "message": "问答成功"
  }
}
```

### 3.7.5 GET `/app-api/ai/logs?userId=10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "traceId": "00112233445566778899aabbccddeeff",
      "userId": 10002,
      "memberId": 10001,
      "bizType": "CHAT",
      "status": "SUCCESS",
      "requestPayload": "这份报告需要重点关注什么？",
      "responsePayload": "建议重点关注异常指标变化趋势，并结合医生意见决定是否复查。",
      "errorMessage": null,
      "createdTime": "2026-03-30 13:10:00"
    }
  ]
}
```

## 4. 后台业务接口

## 4.1 业务用户接口

### 4.1.1 POST `/admin-api/health-users/10002/bootstrap`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "userId": 10002,
    "username": "U10002",
    "phone": null,
    "email": null,
    "encryptedPassword": null,
    "lastLoginTime": null
  }
}
```

### 4.1.2 GET `/admin-api/health-users/page?pageNum=1&pageSize=10`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "userId": 10002,
      "username": "U10002",
      "phone": "13800138000",
      "email": "demo@example.com",
      "lastLoginTime": "2026-03-30 12:20:00",
      "enabled": true
    }
  ]
}
```

### 4.1.3 GET `/admin-api/health-users/10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "user": {
      "userId": 10002,
      "username": "U10002",
      "phone": "13800138000",
      "email": "demo@example.com",
      "encryptedPassword": "$2a$10$mock-password-hash",
      "lastLoginTime": "2026-03-30 12:20:00"
    },
    "enabled": true,
    "members": [
      {
        "memberId": 10001,
        "userId": 10002,
        "name": "张三",
        "relation": "SELF",
        "age": 33,
        "default": true,
        "createdTime": "2026-03-30 12:40:00",
        "updatedTime": "2026-03-30 12:45:00"
      }
    ]
  }
}
```

### 4.1.4 PUT `/admin-api/health-users/10002/status?status=1`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功"
}
```

## 4.2 商品管理接口

### 4.2.1 GET `/admin-api/products/page?pageNum=1&pageSize=10`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "productId": 5001,
      "productName": "OCR 识别次卡",
      "productDesc": "适用于 OCR 识别场景",
      "online": true,
      "createdTime": "2026-03-30 10:00:00",
      "updatedTime": "2026-03-30 10:30:00",
      "skuList": [
        {
          "skuId": 9001,
          "productId": 5001,
          "skuName": "OCR 10次包",
          "priceFen": 1990,
          "times": 10,
          "enabled": true
        }
      ]
    }
  ]
}
```

### 4.2.2 POST `/admin-api/products`

请求示例：

```json
{
  "productName": "综合分析次卡",
  "productDesc": "适用于 AI 综合分析场景",
  "skuList": [
    {
      "skuName": "分析 10次包",
      "priceFen": 2990,
      "times": 10
    },
    {
      "skuName": "分析 30次包",
      "priceFen": 7990,
      "times": 30
    }
  ]
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "productId": 5002,
    "productName": "综合分析次卡",
    "productDesc": "适用于 AI 综合分析场景",
    "online": false,
    "createdTime": "2026-03-30 13:20:00",
    "updatedTime": "2026-03-30 13:20:00",
    "skuList": [
      {
        "skuId": 9002,
        "productId": 5002,
        "skuName": "分析 10次包",
        "priceFen": 2990,
        "times": 10,
        "enabled": true
      },
      {
        "skuId": 9003,
        "productId": 5002,
        "skuName": "分析 30次包",
        "priceFen": 7990,
        "times": 30,
        "enabled": true
      }
    ]
  }
}
```

### 4.2.3 PUT `/admin-api/products/5002`

请求示例：

```json
{
  "productName": "综合分析升级次卡",
  "productDesc": "适用于 AI 综合分析场景",
  "skuList": [
    {
      "skuName": "分析 20次包",
      "priceFen": 4990,
      "times": 20
    }
  ]
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "productId": 5002,
    "productName": "综合分析升级次卡",
    "productDesc": "适用于 AI 综合分析场景",
    "online": false,
    "createdTime": "2026-03-30 13:20:00",
    "updatedTime": "2026-03-30 13:25:00",
    "skuList": [
      {
        "skuId": 9004,
        "productId": 5002,
        "skuName": "分析 20次包",
        "priceFen": 4990,
        "times": 20,
        "enabled": true
      }
    ]
  }
}
```

### 4.2.4 DELETE `/admin-api/products/5002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功"
}
```

### 4.2.5 PUT `/admin-api/products/5002/status?status=1`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功"
}
```

## 4.3 订单管理接口

### 4.3.1 GET `/admin-api/orders/page?pageNum=1&pageSize=10`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "orderNo": "HG202603301250001002",
      "userId": 10002,
      "productId": 5001,
      "skuId": 9001,
      "skuName": "OCR 10次包",
      "priceFen": 1990,
      "times": 10,
      "payChannel": "WECHAT",
      "status": "WAIT_PAY",
      "createdTime": "2026-03-30 12:50:00"
    }
  ]
}
```

### 4.3.2 GET `/admin-api/orders/HG202603301250001002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "orderNo": "HG202603301250001002",
    "userId": 10002,
    "productId": 5001,
    "skuId": 9001,
    "skuName": "OCR 10次包",
    "priceFen": 1990,
    "times": 10,
    "payChannel": "WECHAT",
    "status": "WAIT_PAY",
    "createdTime": "2026-03-30 12:50:00"
  }
}
```

### 4.3.3 POST `/admin-api/orders/mark-paid`

请求示例：

```json
{
  "orderNo": "HG202603301250001002"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功"
}
```

### 4.3.4 POST `/admin-api/orders/refund`

请求示例：

```json
{
  "orderNo": "HG202603301250001002"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功"
}
```

## 4.4 钱包管理接口

### 4.4.1 GET `/admin-api/wallet/10002`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "userId": 10002,
    "balance": 15,
    "ledgerList": [
      {
        "ledgerNo": "WL202603301255002001",
        "userId": 10002,
        "deltaTimes": 10,
        "balanceAfter": 10,
        "bizType": "ORDER_PAY_SUCCESS",
        "bizNo": "HG202603301250001002",
        "createdTime": "2026-03-30 12:55:00"
      },
      {
        "ledgerNo": "WL202603301300002002",
        "userId": 10002,
        "deltaTimes": 5,
        "balanceAfter": 15,
        "bizType": "MANUAL_COMPENSATE",
        "bizNo": "OPS-20260330-001",
        "createdTime": "2026-03-30 13:00:00"
      }
    ]
  }
}
```

### 4.4.2 POST `/admin-api/wallet/compensate`

请求示例：

```json
{
  "userId": 10002,
  "times": 3,
  "bizNo": "OPS-COMPENSATE-20260330-001"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "ledgerNo": "WL202603301330002003",
    "userId": 10002,
    "deltaTimes": 3,
    "balanceAfter": 18,
    "bizType": "MANUAL_COMPENSATE",
    "bizNo": "OPS-COMPENSATE-20260330-001",
    "createdTime": "2026-03-30 13:30:00"
  }
}
```

### 4.4.3 POST `/admin-api/wallet/refund`

请求示例：

```json
{
  "userId": 10002,
  "times": 2,
  "bizNo": "OPS-REFUND-20260330-001"
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "ledgerNo": "WL202603301331002004",
    "userId": 10002,
    "deltaTimes": -2,
    "balanceAfter": 16,
    "bizType": "ORDER_REFUND",
    "bizNo": "OPS-REFUND-20260330-001",
    "createdTime": "2026-03-30 13:31:00"
  }
}
```

## 4.5 AI 审计与风控接口

### 4.5.1 GET `/admin-api/ai/audit/logs`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "traceId": "00112233445566778899aabbccddeeff",
      "userId": 10002,
      "memberId": 10001,
      "bizType": "CHAT",
      "status": "SUCCESS",
      "requestPayload": "这份报告需要重点关注什么？",
      "responsePayload": "建议重点关注异常指标变化趋势，并结合医生意见决定是否复查。",
      "errorMessage": null,
      "createdTime": "2026-03-30 13:10:00"
    }
  ]
}
```

### 4.5.2 GET `/admin-api/ai/audit/exceptions`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "traceId": "a1b2c3d4e5f678901234567890abcdef",
      "userId": 10002,
      "memberId": 10001,
      "bizType": "OCR",
      "status": "FAILED",
      "requestPayload": "https://static.example.com/ocr/report-002.png",
      "responsePayload": null,
      "errorMessage": "OCR_RUNTIME_ERROR:图片地址不可访问",
      "createdTime": "2026-03-30 13:12:00"
    }
  ]
}
```

### 4.5.3 POST `/admin-api/ai/audit/risk/block`

请求示例：

```json
{
  "userId": 10002
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功"
}
```

### 4.5.4 POST `/admin-api/ai/audit/risk/unblock`

请求示例：

```json
{
  "userId": 10002
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功"
}
```

### 4.5.5 GET `/admin-api/ai/audit/risk/blocked-users`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    10002,
    10003
  ]
}
```

### 4.5.6 POST `/admin-api/ai/audit/risk/rate-limit`

请求示例：

```json
{
  "userId": 10002,
  "perMinute": 30
}
```

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功"
}
```

### 4.5.7 GET `/admin-api/ai/audit/risk/rate-limits`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "10002": 30,
    "10003": 15
  }
}
```

## 4.6 运营辅助接口

### 4.6.1 GET `/admin-api/ops/daily-report`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "totalOrders": 12,
    "aiCalls": 56,
    "note": "MVP日报，后续接入真实统计仓库"
  }
}
```

### 4.6.2 GET `/admin-api/ops/alerts/check`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "failedAiCalls": 3,
    "alert": "OK"
  }
}
```

### 4.6.3 POST `/admin-api/ops/compensation/run`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "executed": true,
    "message": "补偿工具占位执行完成（MVP）"
  }
}
```

### 4.6.4 GET `/admin-api/ops/schedule/prepare`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": {
    "cronDailyReport": "0 0 1 * * ?",
    "cronAlertCheck": "0 */5 * * * ?",
    "cronCompensate": "0 30 2 * * ?",
    "status": "prepared"
  }
}
```

## 4.7 后台权限辅助接口

### 4.7.1 GET `/admin-api/system/permissions`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    "health:user:query",
    "health:user:update",
    "health:user:bootstrap",
    "health:product:query",
    "health:product:operate",
    "health:order:query",
    "health:order:operate",
    "health:order:refund",
    "health:wallet:query",
    "health:wallet:operate",
    "health:ai:audit",
    "health:ai:risk",
    "health:ops:query",
    "health:ops:operate"
  ]
}
```

### 4.7.2 GET `/admin-api/system/action-logs`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": [
    {
      "actionId": "11aa22bb33cc44dd55ee66ff77gg88hh",
      "permissionCode": "health:product:operate",
      "operator": "admin",
      "target": "product:5001",
      "createdTime": "2026-03-30 13:40:00"
    }
  ]
}
```

### 4.7.3 GET `/admin-api/system/record-action?permissionCode=health:product:operate&operator=admin&target=product:5001`

成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功"
}
```

## 5. 开放回调接口

### 5.1 POST `/open-api/pay/wechat/callback`

请求示例：

```json
{
  "orderNo": "HG202603301250001002",
  "channel": "WECHAT",
  "callbackId": "wechat-callback-20260330-001",
  "sign": "mock-sign-ok"
}
```

首次处理成功响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": true
}
```

重复回调响应示例：

```json
{
  "code": 200,
  "msg": "操作成功",
  "data": false
}
```

## 6. 前端联调建议

- 如果是后台前端，优先用本文件中的平台基础接口和 `/admin-api/**` 示例。
- 如果是桌面端 / C 端前端，优先用 `/app-api/**` 示例。
- 当前健康业务很多接口仍是 MVP 协议，建议前端保留一层 API 适配，不要把返回结构直接写死在页面里。
- 如果你们后面准备接 Swagger UI、YApi 或 Apifox，这份文档已经可以直接作为导入前的字段参考稿。
