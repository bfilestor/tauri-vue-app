# 医疗健康资料 OCR 与 AI 智能分析系统

## 1. 文档信息

- 文档名称：Spring Boot 模块划分与代码结构
- 适用版本：V2.0
- 服务端架构：Spring Boot + RuoYi-Vue
- 配套文档：`docs/产品需求说明书.md`、`docs/服务端接口设计.md`、`docs/数据库表设计.md`、`docs/MySQL建表SQL草案.md`
- 文档目标：明确服务端工程模块边界、包结构、职责分层和推荐开发顺序，便于后续团队按模块并行开发

## 2. 总体目标

本次服务端建设目标不是做完整医疗云平台，而是围绕以下能力搭建一个可运营的业务中台：

- 前台用户登录与设备管理
- 家庭成员管理
- 商品、订单、支付、退款
- OCR/AI 问答次数钱包与台账
- 内置 AI 网关与调用计费
- 后台运营管理、风控与审计

## 3. 设计原则

- 后台管理员继续复用 RuoYi-Vue 权限体系
- 前台 C 端用户独立于 `sys_user`，使用业务用户体系
- 按“领域模块 + 分层结构”组织代码，不建议把所有功能堆进单一 `controller/service/mapper`
- 内置 AI 网关、支付、钱包、订单必须保持清晰边界，避免后期耦合失控
- 尽量做到“前台接口”和“后台接口”分离、“业务编排”和“基础设施”分离

## 4. 工程组织建议

推荐采用多模块 Maven 工程，便于后续拆分与并行开发。

推荐结构：

```text
health-guard-server/
├── pom.xml
├── health-guard-admin/               # 启动模块，整合 RuoYi 后台
├── health-guard-framework/           # 公共配置、鉴权、异常、基础组件
├── health-guard-common/              # 通用工具、枚举、常量、统一响应
├── health-guard-module-auth/         # 前台认证与令牌
├── health-guard-module-user/         # 前台用户与设备
├── health-guard-module-family/       # 家庭成员
├── health-guard-module-product/      # 商品与SKU
├── health-guard-module-order/        # 订单、支付、退款
├── health-guard-module-wallet/       # 钱包与台账
├── health-guard-module-ai/           # 内置AI网关、调用日志、计费编排
├── health-guard-module-risk/         # 风控与黑名单
└── health-guard-infra/               # 第三方集成：短信、支付、AI、缓存等
```

如果项目初期不想做多模块，也建议在单模块中严格按下文包结构划分目录。

## 5. 模块职责划分

## 5.1 `health-guard-admin`

### 职责

- 作为 Spring Boot 启动模块
- 集成 RuoYi-Vue 后台管理能力
- 扫描各业务模块 Bean
- 统一挂载后台菜单、权限、监控、日志

### 不应承担的职责

- 不直接写具体业务逻辑
- 不直接承载 AI、支付、钱包等实现代码

## 5.2 `health-guard-common`

### 职责

- 统一常量、枚举、错误码定义
- 通用 VO / DTO 基类
- 通用工具类
- 业务异常基类

### 推荐内容

- `ApiCode`
- `BizException`
- `PageQuery`
- `PageResult<T>`
- `SceneTypeEnum`
- `CreditTypeEnum`
- `OrderStatusEnum`
- `ChargeStatusEnum`

## 5.3 `health-guard-framework`

### 职责

- 前台 JWT 鉴权过滤器
- TraceId 透传
- 全局异常处理
- MyBatis Plus 或 MyBatis 配置
- Redis、Jackson、线程池、OpenFeign/RestTemplate/WebClient 等基础配置

### 推荐内容

- `SecurityConfig`
- `AppUserLoginInterceptor`
- `GlobalExceptionHandler`
- `TraceIdFilter`
- `JacksonConfig`
- `ThreadPoolConfig`

## 5.4 `health-guard-infra`

### 职责

- 封装所有第三方服务接入
- 将外部协议适配为内部统一接口

### 建议拆分子包

- `infra.sms`
- `infra.payment`
- `infra.ai`
- `infra.cache`
- `infra.id`

### 推荐接口

- `SmsSender`
- `WechatPayClient`
- `BuiltinAiClient`
- `IdGenerator`

## 5.5 `health-guard-module-auth`

### 职责

- 手机号验证码发送
- 验证码登录/注册
- access token / refresh token 签发与刷新
- 登出与会话失效

### 对外接口

- `/app-api/auth/send-code`
- `/app-api/auth/login`
- `/app-api/auth/refresh-token`
- `/app-api/auth/logout`

### 依赖关系

- 依赖 `module-user`
- 依赖 `infra.sms`
- 依赖 `framework` 鉴权与 token 组件

## 5.6 `health-guard-module-user`

### 职责

- 前台用户信息管理
- 登录设备管理
- 账户信息查询

### 对外接口

- `/app-api/account/profile`
- `/app-api/account/balance`
- `/app-api/account/orders`
- `/app-api/account/ledger`
- `/app-api/account/devices`

### 后台接口

- `/admin-api/users/page`
- `/admin-api/users/{userId}`
- `/admin-api/users/{userId}/status`

## 5.7 `health-guard-module-family`

### 职责

- 家庭成员新增、编辑、删除、查询
- 默认成员设置

### 对外接口

- `/app-api/family-members`
- `/app-api/family-members/{memberId}`
- `/app-api/family-members/{memberId}/set-default`

### 后台接口

- `/admin-api/family-members/page`
- `/admin-api/family-members/{memberId}`

## 5.8 `health-guard-module-product`

### 职责

- 商品与 SKU 管理
- 前台商品展示

### 对外接口

- `/app-api/products`
- `/app-api/products/{productId}`

### 后台接口

- `/admin-api/products/page`
- `/admin-api/products`
- `/admin-api/products/{productId}`
- `/admin-api/products/{productId}/status`

## 5.9 `health-guard-module-order`

### 职责

- 订单创建
- 订单状态流转
- 支付下单
- 支付回调处理
- 退款处理

### 对外接口

- `/app-api/orders`
- `/app-api/orders/{orderNo}`
- `/app-api/orders/{orderNo}/pay-qrcode`
- `/app-api/orders/{orderNo}/status`
- `/app-api/orders/{orderNo}/cancel`

### 后台接口

- `/admin-api/orders/page`
- `/admin-api/orders/{orderNo}`
- `/admin-api/orders/{orderNo}/manual-success`
- `/admin-api/orders/{orderNo}/refund`

### 依赖关系

- 依赖 `module-product`
- 依赖 `module-wallet`
- 依赖 `infra.payment`

## 5.10 `health-guard-module-wallet`

### 职责

- 次数钱包查询
- 次数增减
- 台账记录
- 人工补偿

### 对外接口

- `/app-api/wallet`
- `/app-api/account/balance`
- `/app-api/account/ledger`

### 后台接口

- `/admin-api/wallets/page`
- `/admin-api/wallets/{userId}`
- `/admin-api/wallets/{userId}/compensate`
- `/admin-api/credit-ledgers/page`

### 关键要求

- 钱包余额与台账必须事务一致
- 所有扣次与返还必须有流水

## 5.11 `health-guard-module-ai`

### 职责

- 内置 OCR 网关
- 内置 AI 综合分析网关
- 内置 AI 对话网关
- 调用记录与 AI 请求日志
- 请求幂等处理
- 与钱包模块协作完成扣次与返还

### 对外接口

- `/app-api/usage/precheck`
- `/app-api/usage-records`
- `/app-api/builtin-ai/ocr`
- `/app-api/builtin-ai/analyze`
- `/app-api/builtin-ai/chat`
- `/app-api/builtin-ai/chat/stream`

### 后台接口

- `/admin-api/usage-records/page`
- `/admin-api/ai-request-logs/page`

### 依赖关系

- 依赖 `module-wallet`
- 依赖 `module-family`
- 依赖 `infra.ai`

## 5.12 `health-guard-module-risk`

### 职责

- 用户黑名单管理
- 高频请求限流记录
- 异常登录与异常调用审计

### 后台接口

- `/admin-api/risk/blacklist/users/{userId}`
- `/admin-api/risk/blacklist/users/{userId}` `DELETE`

### 被动参与模块

- `auth` 登录前检查风控状态
- `ai` 调用前检查黑名单与限流状态
- `order` 支付前检查风险状态

## 6. 分层代码结构建议

每个业务模块建议采用统一分层结构：

```text
health-guard-module-xxx/
└── src/main/java/com/healthguard/xxx/
    ├── controller/
    │   ├── app/
    │   └── admin/
    ├── service/
    │   ├── XxxService.java
    │   └── impl/
    ├── manager/
    ├── convert/
    ├── mapper/
    ├── domain/
    │   ├── entity/
    │   ├── dto/
    │   ├── vo/
    │   ├── query/
    │   └── event/
    ├── enums/
    └── listener/
```

### 分层职责说明

- `controller`：处理请求入参、权限、返回值包装
- `service`：编排业务流程，定义业务接口
- `manager`：领域对象操作与复杂业务聚合，适合处理事务性强的逻辑
- `mapper`：数据库访问层
- `convert`：DTO、Entity、VO 之间转换
- `domain/entity`：数据库实体
- `domain/dto`：内部服务传输对象
- `domain/vo`：接口返回对象
- `domain/query`：查询条件对象
- `listener/event`：事件发布与异步处理

## 7. 包名建议

统一包名前缀建议：

```text
com.healthguard
```

示例：

```text
com.healthguard.auth
com.healthguard.user
com.healthguard.family
com.healthguard.product
com.healthguard.order
com.healthguard.wallet
com.healthguard.ai
com.healthguard.risk
com.healthguard.infra
com.healthguard.framework
com.healthguard.common
```

## 8. 模块内部示例结构

## 8.1 认证模块示例

```text
com.healthguard.auth
├── controller/app/AppAuthController.java
├── service/AuthService.java
├── service/impl/AuthServiceImpl.java
├── manager/AuthManager.java
├── mapper/AppUserTokenMapper.java
├── domain/dto/LoginCmd.java
├── domain/dto/RefreshTokenCmd.java
├── domain/vo/LoginRespVO.java
└── convert/AuthConvert.java
```

## 8.2 订单模块示例

```text
com.healthguard.order
├── controller/app/AppOrderController.java
├── controller/admin/AdminOrderController.java
├── controller/open/OpenPayCallbackController.java
├── service/OrderService.java
├── service/PayService.java
├── manager/OrderManager.java
├── manager/PayCallbackManager.java
├── mapper/OrderInfoMapper.java
├── mapper/PaymentTransactionMapper.java
├── domain/entity/OrderInfoDO.java
├── domain/entity/PaymentTransactionDO.java
├── domain/dto/CreateOrderCmd.java
├── domain/vo/OrderDetailRespVO.java
└── convert/OrderConvert.java
```

## 8.3 AI 模块示例

```text
com.healthguard.ai
├── controller/app/AppBuiltinAiController.java
├── controller/admin/AdminUsageRecordController.java
├── service/BuiltinAiService.java
├── service/UsageRecordService.java
├── manager/BuiltinAiManager.java
├── manager/ChargeManager.java
├── mapper/UsageRecordMapper.java
├── mapper/AiRequestLogMapper.java
├── domain/entity/UsageRecordDO.java
├── domain/entity/AiRequestLogDO.java
├── domain/dto/BuiltinChatCmd.java
├── domain/dto/BuiltinAnalyzeCmd.java
├── domain/vo/BuiltinChatRespVO.java
└── listener/UsageFailedRefundListener.java
```

## 9. Controller 设计建议

建议严格区分三类入口：

- `controller.app`：桌面端前台接口
- `controller.admin`：RuoYi-Vue 后台管理接口
- `controller.open`：三方回调接口，如微信支付回调

示例：

```text
controller/
├── app/
│   ├── AppAuthController.java
│   ├── AppAccountController.java
│   ├── AppFamilyMemberController.java
│   ├── AppProductController.java
│   ├── AppOrderController.java
│   └── AppBuiltinAiController.java
├── admin/
│   ├── AdminUserController.java
│   ├── AdminProductController.java
│   ├── AdminOrderController.java
│   ├── AdminWalletController.java
│   └── AdminRiskController.java
└── open/
    └── OpenWechatPayCallbackController.java
```

## 10. Service 与 Manager 职责建议

## 10.1 Service 层

适合做：

- 对外暴露业务接口
- 参数组合与流程编排
- 事务边界控制
- 调用多个 manager 完成业务

不建议：

- 直接堆砌大量 SQL 或第三方调用细节

## 10.2 Manager 层

适合做：

- 复杂领域逻辑
- 钱包扣减与返还
- 支付回调状态迁移
- AI 调用幂等与网关日志处理

### 典型 manager 示例

- `WalletManager`
- `OrderStatusManager`
- `PayCallbackManager`
- `AiGatewayManager`
- `RiskCheckManager`

## 11. DTO / VO / DO 规范

建议命名统一：

- `DO`：数据库实体，如 `AppUserDO`
- `ReqVO`：接口入参，如 `AppLoginReqVO`
- `RespVO`：接口出参，如 `WalletRespVO`
- `Cmd`：服务层命令对象，如 `CreateOrderCmd`
- `Query`：查询对象，如 `UsageRecordQuery`

示例：

```text
domain/
├── entity/AppUserDO.java
├── dto/CreateOrderCmd.java
├── query/OrderPageQuery.java
├── vo/OrderDetailRespVO.java
└── vo/ProductSimpleRespVO.java
```

## 12. 基础能力组件建议

## 12.1 鉴权组件

建议抽象：

- `TokenService`
- `LoginUserContext`
- `AppLoginInterceptor`

## 12.2 业务编号生成组件

建议抽象：

- `OrderNoGenerator`
- `RefundNoGenerator`
- `LedgerNoGenerator`

## 12.3 钱包扣费组件

建议抽象：

- `WalletOperateService`
- `CreditLedgerService`

## 12.4 AI 网关组件

建议抽象：

- `BuiltinAiClient`
- `BuiltinOcrExecutor`
- `BuiltinAnalyzeExecutor`
- `BuiltinChatExecutor`

## 13. 事务边界建议

以下场景必须单独明确定义事务：

- 登录成功后首次注册用户 + 初始化钱包
- 设置默认成员时清空旧默认成员
- 订单支付成功 + 钱包加次 + 台账落账
- AI 调用成功 + 扣次 + 调用记录更新
- AI 调用失败 + 返还次数 + 台账落账
- 后台人工补偿 + 钱包变更 + 流水记录

建议把这些事务边界放在 `service` 或 `manager` 中统一控制，不分散到 controller。

## 14. 事件驱动建议

为了降低耦合，可将部分非核心同步逻辑改为事件通知：

- 订单支付成功后发布 `OrderPaidEvent`
- 钱包加次后发布 `WalletChangedEvent`
- AI 调用失败后发布 `UsageFailedEvent`
- 高频风控命中后发布 `RiskHitEvent`

适合异步处理的事情：

- 统计报表更新
- 运营告警通知
- 日志汇总
- 非关键缓存刷新

## 15. 推荐开发顺序

建议按以下顺序落地：

### 第一阶段

- `common`
- `framework`
- `infra`
- `module-user`
- `module-auth`
- `module-family`

### 第二阶段

- `module-product`
- `module-wallet`
- `module-order`

### 第三阶段

- `module-ai`
- `module-risk`

### 第四阶段

- 后台菜单与运营页面
- 报表、告警、补偿工具

## 16. 模块依赖关系建议

建议依赖方向如下：

```text
admin
 ├── framework
 ├── common
 ├── infra
 ├── module-auth
 ├── module-user
 ├── module-family
 ├── module-product
 ├── module-order
 ├── module-wallet
 ├── module-ai
 └── module-risk

module-auth   -> module-user, framework, infra, common
module-user   -> framework, common
module-family -> framework, common
module-product-> framework, common
module-order  -> module-product, module-wallet, infra, common
module-wallet -> common
module-ai     -> module-wallet, module-family, module-risk, infra, common
module-risk   -> common
infra         -> common
framework     -> common
```

原则：

- 下游业务模块不要反向依赖 `admin`
- `wallet` 不应依赖 `order`，避免循环依赖
- `ai` 可以依赖 `wallet`，但 `wallet` 不应依赖 `ai`

## 17. 配置文件建议

建议在 `application.yml` 中按领域拆分配置：

```yaml
health-guard:
  auth:
    access-token-expire-seconds: 7200
    refresh-token-expire-seconds: 2592000
  sms:
    enabled: true
  pay:
    wechat:
      mch-id: xxx
      app-id: xxx
  ai:
    builtin:
      base-url: https://xxx
      api-key: xxx
      chat-model: xxx
      ocr-model: xxx
  wallet:
    default-ocr-balance: 0
    default-qa-balance: 0
  risk:
    login-limit-per-hour: 10
    ai-call-limit-per-minute: 20
```

## 18. 后台菜单建议

RuoYi-Vue 后台建议新增一级菜单：

- 用户中心
- 家庭成员
- 商品中心
- 订单中心
- 支付管理
- 钱包台账
- AI 调用管理
- 风控管理

对应二级菜单示例：

- 用户列表
- 设备列表
- 成员列表
- 商品列表
- SKU 列表
- 订单列表
- 支付流水
- 钱包余额
- 次数流水
- 调用记录
- AI 请求日志
- 黑名单管理

## 19. 单模块简化版结构

如果你初期不想做 Maven 多模块，也可以先在 RuoYi 的业务模块中按目录隔离：

```text
ruoyi-business/
└── src/main/java/com/ruoyi/business/
    ├── auth/
    ├── user/
    ├── family/
    ├── product/
    ├── order/
    ├── wallet/
    ├── ai/
    ├── risk/
    ├── infra/
    └── common/
```

这种方式更适合前期快速起步，但中后期建议还是升级为多模块结构。

## 20. 结论

这份模块划分方案的重点是把“账号、订单、钱包、AI 网关、风控”几个高耦合但职责不同的领域拆清楚。按该结构落地后，既能兼容 RuoYi-Vue 现有后台体系，也能支持前台 C 端接口、计费闭环和后续业务扩展。下一步最适合继续输出的内容是：

- Java 实体类与 DTO 草案
- Mapper / Service 接口骨架
- RuoYi-Vue 后台菜单与权限标识清单
