# 医疗健康资料 OCR 与 AI 智能分析系统

## 1. 文档信息

- 文档名称：MySQL 建表 SQL 草案
- 适用版本：V2.0
- 数据库版本：MySQL 8.0+
- 配套文档：`docs/产品需求说明书.md`、`docs/服务端接口设计.md`、`docs/数据库表设计.md`
- 文档目标：给出可直接落地改造的 MySQL DDL 草案，作为 Spring Boot + RuoYi-Vue 服务端初始化数据库的基础

## 2. 使用说明

- 本文档为业务库草案，不包含 RuoYi-Vue 自带 `sys_*`、`gen_*`、`qrtz_*` 表
- 主键统一采用 `bigint`，由服务端雪花算法、号段服务或其他 ID 生成策略生成
- 默认字符集建议使用 `utf8mb4`
- 默认存储引擎建议使用 `InnoDB`
- 本草案默认弱外键设计，优先通过应用层保障一致性；如需强约束可自行补充外键

## 3. 建库建议

```sql
CREATE DATABASE IF NOT EXISTS `health_guard_cloud`
  DEFAULT CHARACTER SET utf8mb4
  DEFAULT COLLATE utf8mb4_0900_ai_ci;
```

```sql
USE `health_guard_cloud`;
```

## 4. 表结构 SQL

## 4.1 前台用户表 `app_user`

```sql
DROP TABLE IF EXISTS `app_user`;
CREATE TABLE `app_user` (
  `id` bigint NOT NULL COMMENT '主键',
  `user_no` varchar(32) NOT NULL COMMENT '用户编号',
  `mobile` varchar(20) NOT NULL COMMENT '手机号',
  `nickname` varchar(64) DEFAULT NULL COMMENT '昵称',
  `avatar` varchar(255) DEFAULT NULL COMMENT '头像地址',
  `status` varchar(20) NOT NULL DEFAULT 'normal' COMMENT '用户状态',
  `source` varchar(20) NOT NULL DEFAULT 'mobile_code' COMMENT '注册来源',
  `default_member_id` bigint DEFAULT NULL COMMENT '默认成员ID',
  `last_login_ip` varchar(64) DEFAULT NULL COMMENT '最近登录IP',
  `last_login_time` datetime DEFAULT NULL COMMENT '最近登录时间',
  `deleted` tinyint(1) NOT NULL DEFAULT 0 COMMENT '逻辑删除',
  `remark` varchar(500) DEFAULT NULL COMMENT '备注',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_app_user_user_no` (`user_no`),
  UNIQUE KEY `uk_app_user_mobile` (`mobile`),
  KEY `idx_app_user_status` (`status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='前台用户表';
```

## 4.2 用户刷新令牌表 `app_user_token`

```sql
DROP TABLE IF EXISTS `app_user_token`;
CREATE TABLE `app_user_token` (
  `id` bigint NOT NULL COMMENT '主键',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `device_id` varchar(64) NOT NULL COMMENT '设备标识',
  `refresh_token` varchar(255) NOT NULL COMMENT '刷新令牌',
  `token_status` varchar(20) NOT NULL DEFAULT 'valid' COMMENT '令牌状态',
  `expire_time` datetime NOT NULL COMMENT '过期时间',
  `last_refresh_time` datetime DEFAULT NULL COMMENT '最近刷新时间',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_user_token_refresh_token` (`refresh_token`),
  KEY `idx_user_token_user_device` (`user_id`, `device_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='用户刷新令牌表';
```

## 4.3 用户设备表 `user_device`

```sql
DROP TABLE IF EXISTS `user_device`;
CREATE TABLE `user_device` (
  `id` bigint NOT NULL COMMENT '主键',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `device_id` varchar(64) NOT NULL COMMENT '设备唯一标识',
  `device_name` varchar(100) DEFAULT NULL COMMENT '设备名称',
  `os_type` varchar(50) DEFAULT NULL COMMENT '操作系统类型',
  `client_version` varchar(32) DEFAULT NULL COMMENT '客户端版本',
  `last_login_ip` varchar(64) DEFAULT NULL COMMENT '最近登录IP',
  `last_login_time` datetime DEFAULT NULL COMMENT '最近登录时间',
  `status` varchar(20) NOT NULL DEFAULT 'active' COMMENT '设备状态',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_user_device_user_device` (`user_id`, `device_id`),
  KEY `idx_user_device_status` (`status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='用户设备表';
```

## 4.4 验证码日志表 `sms_code_log`

```sql
DROP TABLE IF EXISTS `sms_code_log`;
CREATE TABLE `sms_code_log` (
  `id` bigint NOT NULL COMMENT '主键',
  `mobile` varchar(20) NOT NULL COMMENT '手机号',
  `scene` varchar(20) NOT NULL DEFAULT 'login' COMMENT '业务场景',
  `code_hash` varchar(128) NOT NULL COMMENT '验证码哈希',
  `status` varchar(20) NOT NULL DEFAULT 'sent' COMMENT '状态',
  `send_ip` varchar(64) DEFAULT NULL COMMENT '发送IP',
  `expire_time` datetime NOT NULL COMMENT '过期时间',
  `verify_time` datetime DEFAULT NULL COMMENT '校验时间',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  PRIMARY KEY (`id`),
  KEY `idx_sms_code_mobile_scene` (`mobile`, `scene`),
  KEY `idx_sms_code_expire` (`expire_time`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='验证码日志表';
```

## 4.5 家庭成员表 `family_member`

```sql
DROP TABLE IF EXISTS `family_member`;
CREATE TABLE `family_member` (
  `id` bigint NOT NULL COMMENT '主键',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `member_no` varchar(32) NOT NULL COMMENT '成员编号',
  `name` varchar(50) NOT NULL COMMENT '成员姓名',
  `gender` varchar(20) DEFAULT 'unknown' COMMENT '性别',
  `birthday` date DEFAULT NULL COMMENT '出生日期',
  `relation` varchar(30) DEFAULT 'self' COMMENT '与用户关系',
  `remark` varchar(500) DEFAULT NULL COMMENT '备注',
  `is_default` tinyint(1) NOT NULL DEFAULT 0 COMMENT '是否默认成员',
  `status` varchar(20) NOT NULL DEFAULT 'normal' COMMENT '状态',
  `deleted` tinyint(1) NOT NULL DEFAULT 0 COMMENT '逻辑删除',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_family_member_member_no` (`member_no`),
  KEY `idx_family_member_user_id` (`user_id`),
  KEY `idx_family_member_user_default` (`user_id`, `is_default`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='家庭成员表';
```

## 4.6 商品表 `product`

```sql
DROP TABLE IF EXISTS `product`;
CREATE TABLE `product` (
  `id` bigint NOT NULL COMMENT '主键',
  `product_no` varchar(32) NOT NULL COMMENT '商品编号',
  `product_name` varchar(100) NOT NULL COMMENT '商品名称',
  `product_type` varchar(30) NOT NULL COMMENT '商品类型',
  `scene` varchar(30) NOT NULL DEFAULT 'recharge' COMMENT '适用场景',
  `status` varchar(20) NOT NULL DEFAULT 'on' COMMENT '上下架状态',
  `tag` varchar(50) DEFAULT NULL COMMENT '标签',
  `sort` int NOT NULL DEFAULT 0 COMMENT '排序值',
  `remark` varchar(500) DEFAULT NULL COMMENT '备注',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_product_product_no` (`product_no`),
  KEY `idx_product_status_sort` (`status`, `sort`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='商品表';
```

## 4.7 商品规格表 `product_sku`

```sql
DROP TABLE IF EXISTS `product_sku`;
CREATE TABLE `product_sku` (
  `id` bigint NOT NULL COMMENT '主键',
  `product_id` bigint NOT NULL COMMENT '商品ID',
  `sku_no` varchar(32) NOT NULL COMMENT 'SKU编号',
  `sku_name` varchar(100) NOT NULL COMMENT 'SKU名称',
  `price_amount` decimal(10,2) NOT NULL DEFAULT 0.00 COMMENT '售价',
  `original_amount` decimal(10,2) DEFAULT 0.00 COMMENT '原价',
  `ocr_credits` int NOT NULL DEFAULT 0 COMMENT 'OCR次数',
  `qa_credits` int NOT NULL DEFAULT 0 COMMENT 'AI问答次数',
  `gift_ocr_credits` int NOT NULL DEFAULT 0 COMMENT '赠送OCR次数',
  `gift_qa_credits` int NOT NULL DEFAULT 0 COMMENT '赠送问答次数',
  `status` varchar(20) NOT NULL DEFAULT 'on' COMMENT '状态',
  `sort` int NOT NULL DEFAULT 0 COMMENT '排序',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_product_sku_sku_no` (`sku_no`),
  KEY `idx_product_sku_product_id` (`product_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='商品规格表';
```

## 4.8 订单表 `order_info`

```sql
DROP TABLE IF EXISTS `order_info`;
CREATE TABLE `order_info` (
  `id` bigint NOT NULL COMMENT '主键',
  `order_no` varchar(40) NOT NULL COMMENT '订单号',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `product_id` bigint NOT NULL COMMENT '商品ID',
  `sku_id` bigint DEFAULT NULL COMMENT 'SKU ID',
  `product_name` varchar(100) NOT NULL COMMENT '商品快照名称',
  `quantity` int NOT NULL DEFAULT 1 COMMENT '数量',
  `order_amount` decimal(10,2) NOT NULL DEFAULT 0.00 COMMENT '订单金额',
  `pay_amount` decimal(10,2) NOT NULL DEFAULT 0.00 COMMENT '实付金额',
  `pay_channel` varchar(30) DEFAULT NULL COMMENT '支付渠道',
  `order_status` varchar(20) NOT NULL DEFAULT 'pending' COMMENT '订单状态',
  `pay_status` varchar(20) NOT NULL DEFAULT 'unpaid' COMMENT '支付状态',
  `client_request_id` varchar(64) DEFAULT NULL COMMENT '客户端幂等请求号',
  `expire_time` datetime DEFAULT NULL COMMENT '订单过期时间',
  `paid_time` datetime DEFAULT NULL COMMENT '支付时间',
  `close_time` datetime DEFAULT NULL COMMENT '关闭时间',
  `deleted` tinyint(1) NOT NULL DEFAULT 0 COMMENT '逻辑删除',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_order_info_order_no` (`order_no`),
  UNIQUE KEY `uk_order_info_client_request_id` (`user_id`, `client_request_id`),
  KEY `idx_order_info_user_id` (`user_id`),
  KEY `idx_order_info_status` (`order_status`, `pay_status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='订单表';
```

## 4.9 支付流水表 `payment_transaction`

```sql
DROP TABLE IF EXISTS `payment_transaction`;
CREATE TABLE `payment_transaction` (
  `id` bigint NOT NULL COMMENT '主键',
  `order_no` varchar(40) NOT NULL COMMENT '订单号',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `pay_channel` varchar(30) NOT NULL COMMENT '支付渠道',
  `out_trade_no` varchar(64) DEFAULT NULL COMMENT '商户单号',
  `channel_trade_no` varchar(64) DEFAULT NULL COMMENT '渠道交易号',
  `transaction_status` varchar(20) NOT NULL DEFAULT 'created' COMMENT '支付状态',
  `request_amount` decimal(10,2) NOT NULL DEFAULT 0.00 COMMENT '请求金额',
  `success_amount` decimal(10,2) DEFAULT 0.00 COMMENT '成功金额',
  `callback_time` datetime DEFAULT NULL COMMENT '回调时间',
  `callback_content` text COMMENT '回调原文',
  `remark` varchar(500) DEFAULT NULL COMMENT '备注',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_payment_transaction_channel_trade_no` (`channel_trade_no`),
  KEY `idx_payment_transaction_order_no` (`order_no`),
  KEY `idx_payment_transaction_status` (`transaction_status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='支付流水表';
```

## 4.10 次数钱包表 `credit_wallet`

```sql
DROP TABLE IF EXISTS `credit_wallet`;
CREATE TABLE `credit_wallet` (
  `id` bigint NOT NULL COMMENT '主键',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `ocr_balance` int NOT NULL DEFAULT 0 COMMENT 'OCR可用次数',
  `qa_balance` int NOT NULL DEFAULT 0 COMMENT 'AI问答可用次数',
  `status` varchar(20) NOT NULL DEFAULT 'active' COMMENT '钱包状态',
  `version_no` int NOT NULL DEFAULT 0 COMMENT '乐观锁版本号',
  `updated_reason` varchar(100) DEFAULT NULL COMMENT '最近变更原因',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_credit_wallet_user_id` (`user_id`),
  CONSTRAINT `chk_credit_wallet_ocr_balance` CHECK (`ocr_balance` >= 0),
  CONSTRAINT `chk_credit_wallet_qa_balance` CHECK (`qa_balance` >= 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='次数钱包表';
```

## 4.11 次数流水表 `credit_ledger`

```sql
DROP TABLE IF EXISTS `credit_ledger`;
CREATE TABLE `credit_ledger` (
  `id` bigint NOT NULL COMMENT '主键',
  `ledger_no` varchar(40) NOT NULL COMMENT '流水编号',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `member_id` bigint DEFAULT NULL COMMENT '成员ID',
  `credit_type` varchar(20) NOT NULL COMMENT '次数类型',
  `biz_type` varchar(20) NOT NULL COMMENT '业务类型',
  `change_amount` int NOT NULL DEFAULT 0 COMMENT '变动值',
  `before_balance` int NOT NULL DEFAULT 0 COMMENT '变更前余额',
  `after_balance` int NOT NULL DEFAULT 0 COMMENT '变更后余额',
  `request_id` varchar(64) DEFAULT NULL COMMENT 'AI请求幂等号',
  `order_no` varchar(40) DEFAULT NULL COMMENT '订单号',
  `usage_record_id` bigint DEFAULT NULL COMMENT '调用记录ID',
  `operator_type` varchar(20) NOT NULL DEFAULT 'system' COMMENT '操作来源',
  `remark` varchar(500) DEFAULT NULL COMMENT '备注',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_credit_ledger_ledger_no` (`ledger_no`),
  KEY `idx_credit_ledger_user_credit` (`user_id`, `credit_type`),
  KEY `idx_credit_ledger_order_no` (`order_no`),
  KEY `idx_credit_ledger_request_id` (`request_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='次数流水表';
```

## 4.12 调用记录表 `usage_record`

```sql
DROP TABLE IF EXISTS `usage_record`;
CREATE TABLE `usage_record` (
  `id` bigint NOT NULL COMMENT '主键',
  `request_id` varchar(64) NOT NULL COMMENT '请求幂等号',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `member_id` bigint NOT NULL COMMENT '成员ID',
  `scene_type` varchar(20) NOT NULL COMMENT '场景类型',
  `credit_type` varchar(20) NOT NULL COMMENT '扣费类型',
  `need_credits` int NOT NULL DEFAULT 1 COMMENT '需扣次数',
  `charged_credits` int NOT NULL DEFAULT 0 COMMENT '实际扣减次数',
  `charge_status` varchar(20) NOT NULL DEFAULT 'none' COMMENT '扣费状态',
  `call_status` varchar(20) NOT NULL DEFAULT 'processing' COMMENT '调用状态',
  `provider_type` varchar(30) NOT NULL DEFAULT 'builtin' COMMENT '提供商类型',
  `model_name` varchar(100) DEFAULT NULL COMMENT '模型名称',
  `biz_no` varchar(64) DEFAULT NULL COMMENT '业务单号',
  `error_code` varchar(50) DEFAULT NULL COMMENT '错误码',
  `error_message` varchar(500) DEFAULT NULL COMMENT '错误摘要',
  `elapsed_ms` int DEFAULT 0 COMMENT '耗时',
  `request_summary` varchar(500) DEFAULT NULL COMMENT '请求摘要',
  `finished_time` datetime DEFAULT NULL COMMENT '完成时间',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_usage_record_request_id` (`request_id`),
  KEY `idx_usage_record_user_scene` (`user_id`, `scene_type`),
  KEY `idx_usage_record_member_id` (`member_id`),
  KEY `idx_usage_record_charge_status` (`charge_status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='调用记录表';
```

## 4.13 AI 请求日志表 `ai_request_log`

```sql
DROP TABLE IF EXISTS `ai_request_log`;
CREATE TABLE `ai_request_log` (
  `id` bigint NOT NULL COMMENT '主键',
  `usage_record_id` bigint NOT NULL COMMENT '调用记录ID',
  `request_id` varchar(64) NOT NULL COMMENT '请求幂等号',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `member_id` bigint NOT NULL COMMENT '成员ID',
  `provider_name` varchar(50) DEFAULT NULL COMMENT '提供商名称',
  `model_name` varchar(100) DEFAULT NULL COMMENT '模型名',
  `endpoint` varchar(255) DEFAULT NULL COMMENT '调用地址',
  `request_summary` varchar(1000) DEFAULT NULL COMMENT '请求摘要',
  `response_summary` varchar(1000) DEFAULT NULL COMMENT '响应摘要',
  `http_status` int DEFAULT NULL COMMENT 'HTTP状态码',
  `success_flag` tinyint(1) NOT NULL DEFAULT 0 COMMENT '是否成功',
  `error_code` varchar(50) DEFAULT NULL COMMENT '错误码',
  `error_message` varchar(500) DEFAULT NULL COMMENT '错误信息',
  `elapsed_ms` int DEFAULT 0 COMMENT '耗时',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  PRIMARY KEY (`id`),
  KEY `idx_ai_request_log_usage_record_id` (`usage_record_id`),
  KEY `idx_ai_request_log_request_id` (`request_id`),
  KEY `idx_ai_request_log_create_time` (`create_time`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='AI请求日志表';
```

## 4.14 退款记录表 `refund_record`

```sql
DROP TABLE IF EXISTS `refund_record`;
CREATE TABLE `refund_record` (
  `id` bigint NOT NULL COMMENT '主键',
  `refund_no` varchar(40) NOT NULL COMMENT '退款单号',
  `order_no` varchar(40) NOT NULL COMMENT '订单号',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `refund_amount` decimal(10,2) NOT NULL DEFAULT 0.00 COMMENT '退款金额',
  `refund_status` varchar(20) NOT NULL DEFAULT 'processing' COMMENT '退款状态',
  `refund_reason` varchar(255) DEFAULT NULL COMMENT '退款原因',
  `channel_refund_no` varchar(64) DEFAULT NULL COMMENT '渠道退款单号',
  `success_time` datetime DEFAULT NULL COMMENT '成功时间',
  `operator_id` bigint DEFAULT NULL COMMENT '操作人',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_refund_record_refund_no` (`refund_no`),
  KEY `idx_refund_record_order_no` (`order_no`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='退款记录表';
```

## 4.15 风控日志表 `risk_control_log`

```sql
DROP TABLE IF EXISTS `risk_control_log`;
CREATE TABLE `risk_control_log` (
  `id` bigint NOT NULL COMMENT '主键',
  `user_id` bigint DEFAULT NULL COMMENT '用户ID',
  `device_id` varchar(64) DEFAULT NULL COMMENT '设备ID',
  `event_type` varchar(30) NOT NULL COMMENT '风控事件类型',
  `risk_level` varchar(20) NOT NULL DEFAULT 'low' COMMENT '风险等级',
  `event_desc` varchar(500) DEFAULT NULL COMMENT '事件描述',
  `ip` varchar(64) DEFAULT NULL COMMENT 'IP',
  `status` varchar(20) NOT NULL DEFAULT 'hit' COMMENT '状态',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  PRIMARY KEY (`id`),
  KEY `idx_risk_control_log_user_id` (`user_id`),
  KEY `idx_risk_control_log_event_type` (`event_type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='风控日志表';
```

## 4.16 用户黑名单表 `user_blacklist`

```sql
DROP TABLE IF EXISTS `user_blacklist`;
CREATE TABLE `user_blacklist` (
  `id` bigint NOT NULL COMMENT '主键',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `block_type` varchar(30) NOT NULL DEFAULT 'ai_call' COMMENT '限制类型',
  `reason` varchar(500) DEFAULT NULL COMMENT '限制原因',
  `start_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '生效时间',
  `end_time` datetime DEFAULT NULL COMMENT '结束时间',
  `status` varchar(20) NOT NULL DEFAULT 'active' COMMENT '状态',
  `operator_id` bigint DEFAULT NULL COMMENT '操作人',
  `create_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `update_time` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  PRIMARY KEY (`id`),
  KEY `idx_user_blacklist_user_id` (`user_id`),
  KEY `idx_user_blacklist_status` (`status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='用户黑名单表';
```

## 5. 推荐初始化数据

## 5.1 商品初始化示例

```sql
INSERT INTO `product` (`id`, `product_no`, `product_name`, `product_type`, `scene`, `status`, `tag`, `sort`, `remark`)
VALUES
  (30001, 'P_OCR_50', 'OCR 50 次包', 'ocr_pack', 'recharge', 'on', '推荐', 10, 'OCR基础次数包'),
  (30002, 'P_QA_20', 'AI 问答 20 次包', 'qa_pack', 'recharge', 'on', '热销', 20, 'AI问答基础次数包'),
  (30003, 'P_COMBO_FAMILY', '家庭体检组合包', 'combo_pack', 'recharge', 'on', '家庭推荐', 30, 'OCR+问答组合包');
```

```sql
INSERT INTO `product_sku` (`id`, `product_id`, `sku_no`, `sku_name`, `price_amount`, `original_amount`, `ocr_credits`, `qa_credits`, `gift_ocr_credits`, `gift_qa_credits`, `status`, `sort`)
VALUES
  (31001, 30001, 'SKU_OCR_50', 'OCR 50 次包', 19.90, 29.90, 50, 0, 0, 0, 'on', 10),
  (31002, 30002, 'SKU_QA_20', 'AI 问答 20 次包', 9.90, 19.90, 0, 20, 0, 0, 'on', 20),
  (31003, 30003, 'SKU_COMBO_FAMILY', '家庭体检组合包', 29.90, 49.90, 50, 30, 0, 0, 'on', 30);
```

## 6. 建表顺序建议

建议按以下顺序执行：

1. `app_user`
2. `app_user_token`
3. `user_device`
4. `sms_code_log`
5. `family_member`
6. `product`
7. `product_sku`
8. `order_info`
9. `payment_transaction`
10. `credit_wallet`
11. `credit_ledger`
12. `usage_record`
13. `ai_request_log`
14. `refund_record`
15. `risk_control_log`
16. `user_blacklist`

## 7. 落地注意事项

- 如果使用 MySQL 8.0 之前版本，`CHECK` 约束支持不稳定，可改为应用层校验
- `uk_order_info_client_request_id(user_id, client_request_id)` 在 `client_request_id` 允许为空时，需统一客户端传值策略，避免幂等失效
- `callback_content` 建议限制存储周期，避免支付原文长期堆积
- `request_summary`、`response_summary` 应做脱敏处理，不记录完整医疗正文
- 高并发扣费时，`credit_wallet` 建议结合乐观锁或 `where balance >= need_credits` 条件更新

## 8. 结论

这份 SQL 草案已经覆盖服务端业务闭环所需的核心数据表，可直接作为数据库初始化蓝本。下一步可以继续细化为：

- Flyway / Liquibase 迁移脚本版本文件
- Spring Boot 实体类、Mapper、Service 草案
- RuoYi-Vue 后台菜单与页面字段配置说明
