# 医疗健康资料 OCR 与 AI 智能分析系统 MySQL 建表 SQL 草案

## 1. 文档说明

- 文档名称：MySQL建表SQL草案
- 适用数据库版本：MySQL 5.5
- 适用范围：`qcy-health` 业务域新增业务表
- 关联文档：
  - `doc/产品需求说明书.md`
  - `doc/产品功能开发说明书.md`
  - `doc/数据库表设计.md`
  - `doc/服务端接口设计.md`

## 2. MySQL 5.5 适配说明

本草案按 MySQL 5.5 的兼容性优先编写，约束如下：

1. 不使用 `json` 数据类型，统一使用 `text / longtext`
2. 默认字符集使用 `utf8`
3. 不使用物理外键，保持与若依现有 SQL 风格一致，避免部署与迁移复杂度上升
4. `datetime` 字段不依赖高版本默认值能力，建议由应用层显式写入
5. 大文本字段仅用于日志、快照、扩展信息，不参与索引

补充说明：

- 如果你的 MySQL 5.5 版本高于 `5.5.3`，理论上可升级为 `utf8mb4`，但需要重新检查联合索引长度。
- 当前草案优先保证“能稳妥执行”，所以默认采用 `utf8`。

## 3. 推荐执行顺序

建议按以下顺序执行建表 SQL：

1. `hg_app_user`
2. `hg_auth_account`
3. `hg_user_device`
4. `hg_auth_sms_code`
5. `hg_auth_email_code`
6. `hg_auth_wechat_qrcode`
7. `hg_family_member`
8. `hg_product`
9. `hg_product_sku`
10. `hg_wallet_account`
11. `hg_wallet_ledger`
12. `hg_order`
13. `hg_payment_transaction`
14. `hg_refund_order`
15. `hg_usage_record`
16. `hg_ai_request_log`
17. `hg_risk_blacklist`
18. `hg_risk_limit_record`
19. `hg_risk_audit_log`

## 4. 初始化语句

```sql
SET NAMES utf8;
```

## 5. 用户与认证

### 5.1 前台业务用户表 `hg_app_user`

```sql
drop table if exists hg_app_user;
create table hg_app_user (
  id                      bigint(20)      not null auto_increment comment '主键ID',
  user_no                 varchar(32)     not null                comment '用户编号',
  user_name               varchar(64)     default null            comment '主用户名',
  email                   varchar(128)    default null            comment '主邮箱',
  phone                   varchar(20)     default null            comment '主手机号',
  nick_name               varchar(64)     default ''              comment '昵称',
  avatar_url              varchar(255)    default null            comment '头像地址',
  real_name               varchar(64)     default null            comment '真实姓名',
  gender                  varchar(16)     default null            comment '性别',
  birthday                date            default null            comment '生日',
  register_type           varchar(32)     default 'PHONE_CODE'    comment '注册方式',
  status                  varchar(16)     default 'ENABLED'       comment '用户状态',
  register_source         varchar(32)     default 'DESKTOP_APP'   comment '注册来源',
  password_login_enabled  char(1)         default 'N'             comment '是否启用密码登录（Y是 N否）',
  email_verified_flag     char(1)         default 'N'             comment '邮箱是否已验证（Y是 N否）',
  phone_verified_flag     char(1)         default 'N'             comment '手机是否已验证（Y是 N否）',
  wechat_bind_flag        char(1)         default 'N'             comment '是否已绑定微信（Y是 N否）',
  register_ip             varchar(64)     default null            comment '注册IP',
  last_login_type         varchar(32)     default null            comment '最近登录方式',
  last_login_ip           varchar(64)     default null            comment '最近登录IP',
  last_login_time         datetime        default null            comment '最近登录时间',
  del_flag                char(1)         default '0'             comment '删除标记（0存在 2删除）',
  create_by               varchar(64)     default ''              comment '创建者',
  create_time             datetime        default null            comment '创建时间',
  update_by               varchar(64)     default ''              comment '更新者',
  update_time             datetime        default null            comment '更新时间',
  remark                  varchar(500)    default null            comment '备注',
  primary key (id),
  unique key uk_user_no (user_no),
  unique key uk_user_name (user_name),
  unique key uk_email (email),
  unique key uk_phone (phone),
  key idx_status_create_time (status, create_time)
) engine=innodb default charset=utf8 comment='前台业务用户表';
```

### 5.2 认证账号表 `hg_auth_account`

```sql
drop table if exists hg_auth_account;
create table hg_auth_account (
  id               bigint(20)      not null auto_increment comment '主键ID',
  auth_no          varchar(32)     not null                comment '认证账号编号',
  user_id          bigint(20)      not null                comment '用户ID',
  auth_type        varchar(32)     not null                comment '认证方式',
  principal_value  varchar(128)    not null                comment '认证主体值',
  password_hash    varchar(255)    default null            comment '密码哈希',
  app_id           varchar(64)     default null            comment '微信应用ID',
  open_id          varchar(128)    default null            comment '微信OpenID',
  union_id         varchar(128)    default null            comment '微信UnionID',
  verified_flag    char(1)         default 'N'             comment '是否已验证（Y是 N否）',
  bind_status      varchar(16)     default 'BOUND'         comment '绑定状态',
  bind_time        datetime        default null            comment '绑定时间',
  last_login_ip    varchar(64)     default null            comment '最近登录IP',
  last_login_time  datetime        default null            comment '最近登录时间',
  ext_json         longtext                                comment '扩展信息',
  create_by        varchar(64)     default ''              comment '创建者',
  create_time      datetime        default null            comment '创建时间',
  update_by        varchar(64)     default ''              comment '更新者',
  update_time      datetime        default null            comment '更新时间',
  remark           varchar(500)    default null            comment '备注',
  primary key (id),
  unique key uk_auth_no (auth_no),
  unique key uk_auth_type_principal (auth_type, principal_value),
  unique key uk_app_openid (app_id, open_id),
  key idx_user_auth (user_id, auth_type)
) engine=innodb default charset=utf8 comment='用户认证账号表';
```

### 5.3 用户设备表 `hg_user_device`

```sql
drop table if exists hg_user_device;
create table hg_user_device (
  id                bigint(20)      not null auto_increment comment '主键ID',
  user_id           bigint(20)      not null                comment '用户ID',
  device_id         varchar(64)     not null                comment '设备ID',
  device_name       varchar(128)    default null            comment '设备名称',
  device_type       varchar(32)     default null            comment '设备类型',
  os_name           varchar(32)     default null            comment '操作系统',
  os_version        varchar(32)     default null            comment '系统版本',
  app_version       varchar(32)     default null            comment '客户端版本',
  login_ip          varchar(64)     default null            comment '最近登录IP',
  last_login_time   datetime        default null            comment '最近登录时间',
  last_active_time  datetime        default null            comment '最近活跃时间',
  status            varchar(16)     default 'ACTIVE'        comment '设备状态',
  create_by         varchar(64)     default ''              comment '创建者',
  create_time       datetime        default null            comment '创建时间',
  update_by         varchar(64)     default ''              comment '更新者',
  update_time       datetime        default null            comment '更新时间',
  remark            varchar(500)    default null            comment '备注',
  primary key (id),
  unique key uk_user_device (user_id, device_id),
  key idx_user_last_active (user_id, last_active_time)
) engine=innodb default charset=utf8 comment='用户设备表';
```

### 5.4 手机验证码表 `hg_auth_sms_code`

```sql
drop table if exists hg_auth_sms_code;
create table hg_auth_sms_code (
  id            bigint(20)      not null auto_increment comment '主键ID',
  biz_no        varchar(32)     not null                comment '业务流水号',
  biz_type      varchar(32)     not null                comment '业务类型',
  phone         varchar(20)     not null                comment '手机号',
  code_hash     varchar(128)    not null                comment '验证码摘要',
  send_channel  varchar(32)     default null            comment '短信渠道',
  request_ip    varchar(64)     default null            comment '请求IP',
  device_id     varchar(64)     default null            comment '设备ID',
  expire_time   datetime        default null            comment '过期时间',
  verify_time   datetime        default null            comment '校验时间',
  status        varchar(16)     default 'INIT'          comment '状态',
  fail_reason   varchar(255)    default null            comment '失败原因',
  create_time   datetime        default null            comment '创建时间',
  primary key (id),
  key idx_phone_biz_type (phone, biz_type, create_time),
  key idx_status_expire (status, expire_time)
) engine=innodb default charset=utf8 comment='手机验证码记录表';
```

### 5.5 邮箱验证码表 `hg_auth_email_code`

```sql
drop table if exists hg_auth_email_code;
create table hg_auth_email_code (
  id            bigint(20)      not null auto_increment comment '主键ID',
  biz_no        varchar(32)     not null                comment '业务流水号',
  biz_type      varchar(32)     not null                comment '业务类型',
  email         varchar(128)    not null                comment '邮箱地址',
  code_hash     varchar(128)    not null                comment '验证码摘要',
  subject       varchar(128)    default null            comment '邮件主题',
  request_ip    varchar(64)     default null            comment '请求IP',
  expire_time   datetime        default null            comment '过期时间',
  verify_time   datetime        default null            comment '校验时间',
  status        varchar(16)     default 'INIT'          comment '状态',
  fail_reason   varchar(255)    default null            comment '失败原因',
  create_time   datetime        default null            comment '创建时间',
  primary key (id),
  key idx_email_biz_type (email, biz_type, create_time),
  key idx_email_status_expire (status, expire_time)
) engine=innodb default charset=utf8 comment='邮箱验证码记录表';
```

### 5.6 微信扫码场景表 `hg_auth_wechat_qrcode`

```sql
drop table if exists hg_auth_wechat_qrcode;
create table hg_auth_wechat_qrcode (
  id                bigint(20)      not null auto_increment comment '主键ID',
  scene_no          varchar(32)     not null                comment '扫码场景号',
  scene_token       varchar(64)     not null                comment '轮询令牌',
  qr_type           varchar(16)     default 'LOGIN'         comment '二维码类型',
  app_id            varchar(64)     default null            comment '微信应用ID',
  qr_code_url       varchar(255)    default null            comment '二维码地址',
  scene_status      varchar(16)     default 'CREATED'       comment '二维码状态',
  expect_device_id  varchar(64)     default null            comment '发起端设备ID',
  request_ip        varchar(64)     default null            comment '请求IP',
  open_id           varchar(128)    default null            comment '微信OpenID',
  union_id          varchar(128)    default null            comment '微信UnionID',
  temp_nick_name    varchar(128)    default null            comment '微信昵称快照',
  callback_code     varchar(128)    default null            comment '微信回调Code',
  user_id           bigint(20)      default null            comment '关联用户ID',
  scanned_time      datetime        default null            comment '扫码时间',
  confirmed_time    datetime        default null            comment '确认时间',
  success_time      datetime        default null            comment '成功时间',
  expire_time       datetime        default null            comment '过期时间',
  create_time       datetime        default null            comment '创建时间',
  update_time       datetime        default null            comment '更新时间',
  primary key (id),
  unique key uk_scene_no (scene_no),
  unique key uk_scene_token (scene_token),
  key idx_scene_status_expire (scene_status, expire_time),
  key idx_user_scene (user_id, create_time)
) engine=innodb default charset=utf8 comment='微信扫码场景表';
```

## 6. 家庭成员

### 6.1 家庭成员表 `hg_family_member`

```sql
drop table if exists hg_family_member;
create table hg_family_member (
  id              bigint(20)      not null auto_increment comment '主键ID',
  member_no       varchar(32)     not null                comment '成员编号',
  user_id         bigint(20)      not null                comment '所属用户ID',
  member_name     varchar(64)     not null                comment '成员姓名',
  relation_code   varchar(32)     not null                comment '关系编码',
  gender          varchar(16)     default null            comment '性别',
  birthday        date            default null            comment '生日',
  mobile          varchar(20)     default null            comment '联系电话',
  id_card_mask    varchar(32)     default null            comment '脱敏证件号',
  health_note     varchar(500)    default null            comment '健康备注',
  is_default      char(1)         default 'N'             comment '是否默认（Y是 N否）',
  status          varchar(16)     default 'ENABLED'       comment '状态',
  del_flag        char(1)         default '0'             comment '删除标记（0存在 2删除）',
  create_by       varchar(64)     default ''              comment '创建者',
  create_time     datetime        default null            comment '创建时间',
  update_by       varchar(64)     default ''              comment '更新者',
  update_time     datetime        default null            comment '更新时间',
  remark          varchar(500)    default null            comment '备注',
  primary key (id),
  unique key uk_member_no (member_no),
  key idx_user_status (user_id, status),
  key idx_user_default (user_id, is_default)
) engine=innodb default charset=utf8 comment='家庭成员表';
```

## 7. 商品与 SKU

### 7.1 商品主表 `hg_product`

```sql
drop table if exists hg_product;
create table hg_product (
  id             bigint(20)      not null auto_increment comment '主键ID',
  product_no     varchar(32)     not null                comment '商品编号',
  product_name   varchar(128)    not null                comment '商品名称',
  product_type   varchar(32)     not null                comment '商品类型',
  cover_url      varchar(255)    default null            comment '封面图',
  product_desc   varchar(1000)   default null            comment '商品说明',
  sale_status    varchar(16)     default 'OFF_SALE'      comment '销售状态',
  sort_no        int(11)         default 0               comment '排序值',
  status         varchar(16)     default 'ENABLED'       comment '数据状态',
  del_flag       char(1)         default '0'             comment '删除标记（0存在 2删除）',
  create_by      varchar(64)     default ''              comment '创建者',
  create_time    datetime        default null            comment '创建时间',
  update_by      varchar(64)     default ''              comment '更新者',
  update_time    datetime        default null            comment '更新时间',
  remark         varchar(500)    default null            comment '备注',
  primary key (id),
  unique key uk_product_no (product_no),
  key idx_sale_status_sort (sale_status, sort_no)
) engine=innodb default charset=utf8 comment='商品主表';
```

### 7.2 商品 SKU 表 `hg_product_sku`

```sql
drop table if exists hg_product_sku;
create table hg_product_sku (
  id             bigint(20)      not null auto_increment comment '主键ID',
  sku_no         varchar(32)     not null                comment 'SKU编号',
  product_id     bigint(20)      not null                comment '所属商品ID',
  sku_name       varchar(128)    not null                comment 'SKU名称',
  sale_price     bigint(20)      default 0               comment '销售价，单位分',
  origin_price   bigint(20)      default 0               comment '原价，单位分',
  ocr_times      int(11)         default 0               comment 'OCR次数',
  analyze_times  int(11)         default 0               comment '分析次数',
  chat_times     int(11)         default 0               comment '对话次数',
  valid_days     int(11)         default null            comment '有效期天数',
  buy_limit      int(11)         default null            comment '每用户限购数',
  sale_status    varchar(16)     default 'OFF_SALE'      comment '销售状态',
  status         varchar(16)     default 'ENABLED'       comment '数据状态',
  del_flag       char(1)         default '0'             comment '删除标记（0存在 2删除）',
  create_by      varchar(64)     default ''              comment '创建者',
  create_time    datetime        default null            comment '创建时间',
  update_by      varchar(64)     default ''              comment '更新者',
  update_time    datetime        default null            comment '更新时间',
  remark         varchar(500)    default null            comment '备注',
  primary key (id),
  unique key uk_sku_no (sku_no),
  key idx_product_sale (product_id, sale_status)
) engine=innodb default charset=utf8 comment='商品SKU表';
```

## 8. 钱包

### 8.1 用户次数钱包账户表 `hg_wallet_account`

```sql
drop table if exists hg_wallet_account;
create table hg_wallet_account (
  id              bigint(20)      not null auto_increment comment '主键ID',
  account_no      varchar(32)     not null                comment '账户编号',
  user_id         bigint(20)      not null                comment '用户ID',
  credit_type     varchar(32)     not null                comment '次数类型',
  balance         int(11)         default 0               comment '当前余额',
  frozen_balance  int(11)         default 0               comment '冻结余额',
  total_grant     int(11)         default 0               comment '累计发放',
  total_used      int(11)         default 0               comment '累计消耗',
  total_refund    int(11)         default 0               comment '累计返还',
  status          varchar(16)     default 'ENABLED'       comment '状态',
  version_no      int(11)         default 0               comment '乐观锁版本号',
  create_by       varchar(64)     default ''              comment '创建者',
  create_time     datetime        default null            comment '创建时间',
  update_by       varchar(64)     default ''              comment '更新者',
  update_time     datetime        default null            comment '更新时间',
  remark          varchar(500)    default null            comment '备注',
  primary key (id),
  unique key uk_account_no (account_no),
  unique key uk_user_credit (user_id, credit_type)
) engine=innodb default charset=utf8 comment='用户次数钱包账户表';
```

### 8.2 钱包台账流水表 `hg_wallet_ledger`

```sql
drop table if exists hg_wallet_ledger;
create table hg_wallet_ledger (
  id                 bigint(20)      not null auto_increment comment '主键ID',
  ledger_no          varchar(32)     not null                comment '台账流水号',
  user_id            bigint(20)      not null                comment '用户ID',
  wallet_account_id  bigint(20)      not null                comment '钱包账户ID',
  credit_type        varchar(32)     not null                comment '次数类型',
  direction          varchar(8)      not null                comment '流水方向',
  change_amount      int(11)         not null                comment '变动次数',
  before_balance     int(11)         not null                comment '变动前余额',
  after_balance      int(11)         not null                comment '变动后余额',
  biz_type           varchar(32)     not null                comment '业务类型',
  biz_no             varchar(32)     not null                comment '业务编号',
  operator_type      varchar(16)     default 'SYSTEM'        comment '操作人类型',
  operator_id        bigint(20)      default null            comment '操作人ID',
  occurred_time      datetime        default null            comment '发生时间',
  remark             varchar(500)    default null            comment '备注',
  primary key (id),
  unique key uk_ledger_no (ledger_no),
  key idx_user_credit_time (user_id, credit_type, occurred_time),
  key idx_biz_type_no (biz_type, biz_no)
) engine=innodb default charset=utf8 comment='钱包台账流水表';
```

## 9. 订单与支付

### 9.1 订单主表 `hg_order`

```sql
drop table if exists hg_order;
create table hg_order (
  id                     bigint(20)      not null auto_increment comment '主键ID',
  order_no               varchar(32)     not null                comment '订单号',
  user_id                bigint(20)      not null                comment '用户ID',
  product_id             bigint(20)      not null                comment '商品ID',
  sku_id                 bigint(20)      not null                comment 'SKU ID',
  product_name_snapshot  varchar(128)    not null                comment '商品名称快照',
  sku_name_snapshot      varchar(128)    not null                comment 'SKU名称快照',
  rights_snapshot        text                                    comment '权益快照',
  order_status           varchar(32)     default 'CREATED'       comment '订单状态',
  pay_status             varchar(32)     default 'UNPAID'        comment '支付状态',
  refund_status          varchar(32)     default 'NONE'          comment '退款状态',
  total_amount           bigint(20)      default 0               comment '订单总额，单位分',
  payable_amount         bigint(20)      default 0               comment '应付金额，单位分',
  paid_amount            bigint(20)      default 0               comment '实付金额，单位分',
  refund_amount          bigint(20)      default 0               comment '已退款金额，单位分',
  pay_channel            varchar(32)     default null            comment '支付渠道',
  source_channel         varchar(32)     default 'DESKTOP_APP'   comment '下单来源',
  cancel_reason          varchar(255)    default null            comment '取消原因',
  pay_time               datetime        default null            comment '支付时间',
  cancel_time            datetime        default null            comment '取消时间',
  close_time             datetime        default null            comment '关闭时间',
  refund_time            datetime        default null            comment '退款完成时间',
  create_by              varchar(64)     default ''              comment '创建者',
  create_time            datetime        default null            comment '创建时间',
  update_by              varchar(64)     default ''              comment '更新者',
  update_time            datetime        default null            comment '更新时间',
  remark                 varchar(500)    default null            comment '备注',
  primary key (id),
  unique key uk_order_no (order_no),
  key idx_user_order (user_id, create_time),
  key idx_order_status (order_status, create_time)
) engine=innodb default charset=utf8 comment='订单主表';
```

### 9.2 支付交易表 `hg_payment_transaction`

```sql
drop table if exists hg_payment_transaction;
create table hg_payment_transaction (
  id                  bigint(20)      not null auto_increment comment '主键ID',
  trans_no            varchar(32)     not null                comment '支付交易号',
  order_no            varchar(32)     not null                comment '订单号',
  user_id             bigint(20)      not null                comment '用户ID',
  pay_channel         varchar(32)     not null                comment '支付渠道',
  trade_type          varchar(32)     default null            comment '交易类型',
  provider_trade_no   varchar(64)     default null            comment '三方支付单号',
  provider_prepay_id  varchar(128)    default null            comment '预支付ID',
  request_amount      bigint(20)      default 0               comment '请求金额',
  paid_amount         bigint(20)      default 0               comment '实付金额',
  transaction_status  varchar(32)     default 'INIT'          comment '交易状态',
  notify_status       varchar(32)     default 'NONE'          comment '回调状态',
  notify_count        int(11)         default 0               comment '回调次数',
  last_notify_time    datetime        default null            comment '最近回调时间',
  request_payload     longtext                                comment '请求报文',
  response_payload    longtext                                comment '响应报文',
  create_time         datetime        default null            comment '创建时间',
  update_time         datetime        default null            comment '更新时间',
  primary key (id),
  unique key uk_trans_no (trans_no),
  key idx_order_no (order_no),
  key idx_provider_trade_no (provider_trade_no)
) engine=innodb default charset=utf8 comment='支付交易表';
```

### 9.3 退款单表 `hg_refund_order`

```sql
drop table if exists hg_refund_order;
create table hg_refund_order (
  id                  bigint(20)      not null auto_increment comment '主键ID',
  refund_no           varchar(32)     not null                comment '退款单号',
  order_no            varchar(32)     not null                comment '原订单号',
  trans_no            varchar(32)     default null            comment '原支付交易号',
  user_id             bigint(20)      not null                comment '用户ID',
  refund_amount       bigint(20)      default 0               comment '退款金额，单位分',
  refund_reason       varchar(255)    default null            comment '退款原因',
  refund_status       varchar(32)     default 'INIT'          comment '退款状态',
  provider_refund_no  varchar(64)     default null            comment '三方退款单号',
  request_time        datetime        default null            comment '发起时间',
  success_time        datetime        default null            comment '成功时间',
  fail_reason         varchar(255)    default null            comment '失败原因',
  notify_payload      longtext                                comment '回调报文',
  create_by           varchar(64)     default ''              comment '创建者',
  create_time         datetime        default null            comment '创建时间',
  update_by           varchar(64)     default ''              comment '更新者',
  update_time         datetime        default null            comment '更新时间',
  remark              varchar(500)    default null            comment '备注',
  primary key (id),
  unique key uk_refund_no (refund_no),
  key idx_order_refund (order_no, refund_status)
) engine=innodb default charset=utf8 comment='退款单表';
```

## 10. AI 使用与日志

### 10.1 OCR / AI 调用主记录表 `hg_usage_record`

```sql
drop table if exists hg_usage_record;
create table hg_usage_record (
  id                bigint(20)      not null auto_increment comment '主键ID',
  usage_no          varchar(32)     not null                comment '使用流水号',
  user_id           bigint(20)      not null                comment '用户ID',
  member_id         bigint(20)      default null            comment '家庭成员ID',
  usage_type        varchar(32)     not null                comment '使用类型',
  scene_code        varchar(32)     default null            comment '场景编码',
  idempotency_key   varchar(64)     not null                comment '幂等键',
  provider_code     varchar(32)     default null            comment 'AI服务商',
  model_code        varchar(64)     default null            comment '模型编码',
  usage_status      varchar(16)     default 'INIT'          comment '使用状态',
  charge_status     varchar(16)     default 'INIT'          comment '扣费状态',
  credit_type       varchar(32)     default null            comment '扣减钱包类型',
  deduct_ledger_no  varchar(32)     default null            comment '扣费流水号',
  refund_ledger_no  varchar(32)     default null            comment '返还流水号',
  request_title     varchar(255)    default null            comment '请求摘要',
  result_summary    varchar(1000)   default null            comment '结果摘要',
  result_url        varchar(255)    default null            comment '结果地址',
  token_input       int(11)         default 0               comment '输入token数',
  token_output      int(11)         default 0               comment '输出token数',
  cost_ms           int(11)         default 0               comment '耗时毫秒',
  error_code        varchar(64)     default null            comment '失败码',
  error_msg         varchar(255)    default null            comment '失败描述',
  request_time      datetime        default null            comment '发起时间',
  finish_time       datetime        default null            comment '完成时间',
  create_by         varchar(64)     default ''              comment '创建者',
  create_time       datetime        default null            comment '创建时间',
  update_by         varchar(64)     default ''              comment '更新者',
  update_time       datetime        default null            comment '更新时间',
  remark            varchar(500)    default null            comment '备注',
  primary key (id),
  unique key uk_usage_no (usage_no),
  unique key uk_user_idempotent (user_id, usage_type, idempotency_key),
  key idx_user_usage_time (user_id, request_time),
  key idx_member_usage_time (member_id, request_time)
) engine=innodb default charset=utf8 comment='OCR和AI调用主记录表';
```

### 10.2 AI 请求响应日志表 `hg_ai_request_log`

```sql
drop table if exists hg_ai_request_log;
create table hg_ai_request_log (
  id                      bigint(20)      not null auto_increment comment '主键ID',
  log_no                  varchar(32)     not null                comment '日志号',
  usage_no                varchar(32)     not null                comment '使用流水号',
  user_id                 bigint(20)      default null            comment '用户ID',
  provider_code           varchar(32)     default null            comment '服务商编码',
  api_type                varchar(32)     default null            comment '接口类型',
  request_url             varchar(255)    default null            comment '请求地址',
  request_method          varchar(16)     default null            comment '请求方法',
  request_headers_masked  longtext                                comment '脱敏请求头',
  request_body            longtext                                comment '请求体',
  response_body           longtext                                comment '响应体',
  http_status             int(11)         default null            comment 'HTTP状态码',
  success_flag            char(1)         default 'N'             comment '是否成功（Y是 N否）',
  trace_id                varchar(64)     default null            comment '链路追踪ID',
  cost_ms                 int(11)         default 0               comment '耗时毫秒',
  create_time             datetime        default null            comment '创建时间',
  primary key (id),
  unique key uk_log_no (log_no),
  key idx_usage_no (usage_no),
  key idx_trace_id (trace_id)
) engine=innodb default charset=utf8 comment='AI请求响应日志表';
```

## 11. 风控

### 11.1 黑名单表 `hg_risk_blacklist`

```sql
drop table if exists hg_risk_blacklist;
create table hg_risk_blacklist (
  id             bigint(20)      not null auto_increment comment '主键ID',
  target_type    varchar(16)     not null                comment '目标类型',
  target_value   varchar(128)    not null                comment '命中值',
  user_id        bigint(20)      default null            comment '用户ID',
  risk_level     varchar(16)     default 'MEDIUM'        comment '风险等级',
  reason         varchar(255)    default null            comment '拉黑原因',
  source_type    varchar(16)     default 'MANUAL'        comment '来源类型',
  status         varchar(16)     default 'ACTIVE'        comment '状态',
  start_time     datetime        default null            comment '开始时间',
  end_time       datetime        default null            comment '结束时间',
  last_hit_time  datetime        default null            comment '最近命中时间',
  create_by      varchar(64)     default ''              comment '创建者',
  create_time    datetime        default null            comment '创建时间',
  update_by      varchar(64)     default ''              comment '更新者',
  update_time    datetime        default null            comment '更新时间',
  remark         varchar(500)    default null            comment '备注',
  primary key (id),
  key idx_target_status (target_type, target_value, status),
  key idx_user_black (user_id, status)
) engine=innodb default charset=utf8 comment='风险黑名单表';
```

### 11.2 限流命中记录表 `hg_risk_limit_record`

```sql
drop table if exists hg_risk_limit_record;
create table hg_risk_limit_record (
  id            bigint(20)      not null auto_increment comment '主键ID',
  record_no     varchar(32)     not null                comment '记录号',
  user_id       bigint(20)      default null            comment '用户ID',
  target_type   varchar(32)     not null                comment '目标类型',
  target_key    varchar(128)    not null                comment '限流目标键',
  rule_code     varchar(32)     not null                comment '规则编码',
  window_start  datetime        default null            comment '窗口开始',
  window_end    datetime        default null            comment '窗口结束',
  hit_count     int(11)         default 0               comment '命中次数',
  block_flag    char(1)         default 'Y'             comment '是否拦截（Y是 N否）',
  request_ip    varchar(64)     default null            comment '请求IP',
  device_id     varchar(64)     default null            comment '设备ID',
  create_time   datetime        default null            comment '创建时间',
  primary key (id),
  unique key uk_record_no (record_no),
  key idx_target_window (target_type, target_key, window_start),
  key idx_user_time (user_id, create_time)
) engine=innodb default charset=utf8 comment='限流命中记录表';
```

### 11.3 风控审计日志表 `hg_risk_audit_log`

```sql
drop table if exists hg_risk_audit_log;
create table hg_risk_audit_log (
  id            bigint(20)      not null auto_increment comment '主键ID',
  audit_no      varchar(32)     not null                comment '审计号',
  user_id       bigint(20)      default null            comment '用户ID',
  device_id     varchar(64)     default null            comment '设备ID',
  event_type    varchar(32)     not null                comment '事件类型',
  risk_result   varchar(16)     default 'PASS'          comment '风险结果',
  risk_level    varchar(16)     default null            comment '风险级别',
  request_ip    varchar(64)     default null            comment '请求IP',
  request_uri   varchar(255)    default null            comment '请求URI',
  trace_id      varchar(64)     default null            comment '链路追踪ID',
  biz_no        varchar(32)     default null            comment '关联业务号',
  detail_json   longtext                                comment '风险详情',
  create_time   datetime        default null            comment '创建时间',
  primary key (id),
  unique key uk_audit_no (audit_no),
  key idx_user_event_time (user_id, event_type, create_time),
  key idx_trace_id (trace_id)
) engine=innodb default charset=utf8 comment='风控审计日志表';
```

## 12. 执行与落库说明

### 12.1 建议补充的初始化数据

- 商品、SKU、支付渠道、AI 提供商、风控规则等字典项，建议继续落在若依字典体系中维护
- 如需后台菜单、权限、参数配置，请单独补充若依 `sys_menu / sys_dict_data / sys_config` 初始化 SQL

### 12.2 MySQL 5.5 注意事项

1. 导入前请确认数据库默认字符集为 `utf8`，避免和表级字符集不一致
2. 如果你当前库是 `utf8mb4`，本草案也能执行，但建议先确认联合索引长度是否仍在限制范围内
3. 所有 `longtext / text` 字段都不参与索引，适合保存请求报文、响应报文、快照和详情文本
4. `ext_json`、`rights_snapshot`、`request_payload`、`response_payload`、`notify_payload`、`detail_json` 等字段，存储的是 JSON 字符串内容，但字段类型仍然是 `text / longtext`，不是 MySQL `json`
5. 由于未建立物理外键，删除用户、商品、成员时需要由应用层控制数据完整性
6. 时间字段建议统一由服务端写入，避免 MySQL 5.5 在默认值能力上的版本差异

### 12.3 与设计文档对应关系

- 本文 SQL 与 `数据库表设计.md` 一一对应
- 若后续接口字段调整，应优先同步更新 `服务端接口设计.md` 和本文中的相关列定义

## 13. 结论

这份草案已经按 MySQL 5.5 做了保守兼容处理，核心策略是：

- 不使用 `json`
- 默认使用 `utf8`
- 不建立物理外键
- 以 `text / longtext` 承接快照、日志和扩展详情

如果你下一步要继续落地，我可以直接基于这份草案再产出一版“可导入执行”的 `sql/health_guard_init.sql`。
