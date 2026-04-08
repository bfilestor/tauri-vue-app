# 健康管家用户体系与通用 AI 服务 · Openclaw 开发协作规范

## 1. 项目输入

固定优先读取以下源文档作为本轮需求依据：
- `docs/产品需求说明书.md`
- `docs/V2版本功能需求说明.md`
- `docs/桌面客户端对接文档.md`
- `stitch/pics/设计说明.txt`
- `stitch/sidebar-user-area.html`
- `stitch/model-service.html`
- `需求描述.txt`

协作文档目录结构：
- `doc/README.md`
- `doc/epic/`
- `doc/status/todo-list.md`
- `doc/status/feature-done.md`
- `doc/status/E4-联调验收脚本.md`

补充说明：
- 仓库中不存在 `产品功能开发说明书.md`，本轮暂以 `docs/桌面客户端对接文档.md`、`stitch/*` 设计稿和 `需求描述.txt` 补齐开发执行口径。
- 设计稿图片文件已给出文件名，但缺少可直接解析的结构化标注；UI 细节以 `stitch/sidebar-user-area.html`、`stitch/model-service.html` 和 `stitch/pics/设计说明.txt` 的共同交集为准。
- E4 多用户多成员联调与手工验收口径统一记录在 `doc/status/E4-联调验收脚本.md`。

## 2. Epic 文件列表

| Epic ID | 文件 | 范围摘要 |
|---------|------|----------|
| E1 | `doc/epic/E1-auth-session-foundation.md` | 设备激活、认证接口封装、登录注册访客弹框、会话持久化 |
| E2 | `doc/epic/E2-account-payment-flow.md` | 账户资料、余额/次数、商品套餐、下单支付、到账刷新 |
| E3 | `doc/epic/E3-mode-gating-and-ui-integration.md` | 通用/自定义模式切换、设置页改造、AI 调用预检与支付拦截 |
| E4 | `doc/epic/E4-multi-user-member-isolation.md` | 首登默认本人成员、本地 SQLite 多用户多成员隔离、成员级报告与对话空间 |
| E5 | `doc/epic/E5-member-management-local-operations.md` | 登录后家庭成员管理能力补齐：本地成员命令、设置页成员操作可用性修复 |

> 仅在处理某个 Issue 时读取对应 Epic 文件，避免加载全部需求上下文。

## 3. 核心开发原则

1. 严格采用 TDD：先测试，后实现。
2. 一次只允许开发一个 Issue。
3. 每次开发前必须先检查依赖与验收条件。
4. 每次开发后必须更新 `doc/status/todo-list.md` 与 `doc/status/feature-done.md`。
5. 不得跳过阻塞项，不得默认补全缺失需求。
6. 不得跨 Epic 隐式扩展范围。
7. 开发当前 Issue 时，仅加载相关 Epic 文件与状态文件。
8. 必须保留现有“自定义模式”下的 AI Provider / Model / Prompt / OCR / AI 分析调用流程，不得在本轮重构或替换。
9. 通用模式的购买和次数校验只做在调用入口前的拦截层，不得直接改写现有自定义模式接口设置逻辑。

## 4. 编号规则

- Epic：`E{n}`
- Story：`E{n}-S{n}`
- Issue：`E{n}-S{n}-I{n}`

## 5. 开发顺序

当收到 `openclaw 开始开发` 指令时，按以下顺序执行：

1. 读取 `doc/status/todo-list.md`
2. 找到首个未完成 Issue
3. 打开该 Issue 所属 Epic 文件
4. 先编写测试设计与测试用例
5. 再进行最小实现
6. 验证测试结果
7. 更新文档

建议执行顺序：
1. E1-S1-I1
2. E1-S1-I2
3. E1-S2-I1
4. E1-S2-I2
5. E2-S1-I1
6. E2-S1-I2
7. E2-S2-I1
8. E2-S2-I2
9. E3-S1-I1
10. E3-S1-I2
11. E3-S2-I1
12. E3-S2-I2
13. E4-S1-I1
14. E4-S1-I2
15. E4-S2-I1
16. E4-S2-I2
17. E5-S1-I1
18. E5-S1-I2

## 6. 阻塞处理

出现以下任一情况必须停止并反馈：
- 需求缺失
- 接口/字段定义不明确
- 外部依赖不可用
- 验收标准冲突
- 上游 Issue 未完成

本轮已知阻塞或缺口：
- 缺少正式 `产品功能开发说明书.md`
- 登录注册 UI 与接口字段存在口径差异：设计稿偏手机号，接口文档同时提供邮箱注册、密码登录、验证码登录
- 当前桌面端成员切换与成员隔离已落地，E5 聚焦“登录后成员管理操作链路可执行性”补齐
- 次数包套餐为 `20/100/500`，但商品 `productId/skuId` 映射关系未在源文档中固定
- 临时访客的数据保留策略仅在设计稿中提到“数据不保存”，未定义对现有本地记录的影响范围
- `db.rs` 仍需补充版本化 schema 迁移脚本，但本期不处理旧版本业务数据自动迁移

## 7. 测试要求

每个 Issue 至少包含：
- 功能测试
- 边界测试

测试必须能够映射到验收标准。
