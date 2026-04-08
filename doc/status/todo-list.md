# 健康管家用户体系与通用 AI 服务 · 开发 Issue 进度清单（todo-list）

> **说明：** 本清单基于 `doc/epic/` 下各 Epic 文件正文显式展开的 Issue 生成。
> **目录：** Epic 拆分文件位于 `doc/epic/`，状态文件位于 `doc/status/`。
> **口径：** PRD 负责业务范围，当前因缺少正式开发说明书，执行口径以 `docs/桌面客户端对接文档.md`、`stitch/*` 设计稿和 `需求描述.txt` 为补充。
> 完成一个 Issue 后**必须**更新状态与测试结果，再进入下一个。
> Issue 编号规则固定为：`E1-S1-I1`。

---

## 状态约定

| 状态 | 含义 |
|------|------|
| `Todo` | 待开发，条件未就绪或排队中 |
| `In Progress` | 正在开发 |
| `Test Passed` | 开发完成，本地测试全部通过 |
| `Done` | 已合并主干，联调验证通过 |
| `Skip` | 本版本跳过（标注原因） |

---

## 开发前更新规则

1. 开发开始前先将对应 Issue 状态改为 `In Progress`。
2. 本地验证通过后更新“测试”列，并将状态改为 `Test Passed`。
3. 合并主干或联调通过后，才能改为 `Done`。
4. 如需拆分新增任务，先更新对应 `doc/epic/*.md`，再补录到本表。
5. 如遇阻塞，不得继续下一个 Issue，必须先记录阻塞原因。

---

## 进度表

| Issue | 描述 | 状态 | 依赖 | 测试 | 备注 |
|------|------|------|------|------|------|
| E1-S1-I1 | 本地设备凭证与签名请求层 | Test Passed | 无 | `npm test`、`npm run build` 通过 | 已补设备激活、续期、签名头注入与兼容模式 |
| E1-S1-I2 | 认证 API 与会话存储封装 | Test Passed | E1-S1-I1 | `npm test`、`npm run build` 通过 | 含登录、注册、刷新、登出、访客态 |
| E1-S2-I1 | 登录/注册/临时访客弹框 | Test Passed | E1-S1-I2 | `npm test`、`npm run build` 通过 | 已实现认证弹框 Tab 切换、表单校验、访客入口与认证 API 对接 |
| E1-S2-I2 | 侧边栏用户区与会话状态联动 | Test Passed | E1-S1-I2, E1-S2-I1 | `npm test`、`npm run build` 通过 | 已完成未登录/访客/登录三态渲染、会话恢复与退出联动 |
| E2-S1-I1 | 账户资料、余额、默认成员与套餐状态层 | Test Passed | E1-S1-I2 | `npm test`、`npm run build` 通过 | 已建立统一状态层，支持登录态上下文加载、访客态商品加载与默认成员阻塞提示 |
| E2-S1-I2 | 用户区剩余次数、账号菜单与使用反馈 | Test Passed | E2-S1-I1, E1-S2-I2 | `npm test`、`npm run build` 通过 | 侧边栏与设置页共享同一账户状态源，余额刷新后多区域同步更新 |
| E2-S2-I1 | 购买次数包弹窗与订单创建 | Test Passed | E2-S1-I1 | `npm test`、`npm run build` 通过 | 已实现套餐选择、下单、订单摘要、微信/支付宝切换与二维码加载 |
| E2-S2-I2 | 订单轮询、到账刷新与注册赠送试用提示 | Test Passed | E2-S2-I1, E1-S1-I2 | `npm test`、`npm run build` 通过 | 已实现订单状态轮询、超时/失败/取消反馈、支付成功自动刷新余额并关闭购买弹窗 |
| E3-S1-I1 | 通用/自定义模式状态与设置页改造 | Test Passed | E2-S1-I2 | `npm test`、`npm run build` 通过 | 已完成模式状态持久化、设置页通用/自定义分区显隐，保留 Provider/Model/网络配置原有 CRUD 能力 |
| E3-S1-I2 | 自定义模式说明页与第三方平台注册引导 | Test Passed | E3-S1-I1 | `npm test`、`npm run build` 通过 | 已新增自定义模式说明抽屉、平台接入步骤与可配置外链占位，未配置链接时明确提示“暂未配置” |
| E3-S2-I1 | AI 问答入口的通用模式预检与购买拦截 | Test Passed | E2-S2-I2, E3-S1-I1 | `npm test`、`npm run build` 通过 | 已接入通用模式发送前 `usage/precheck`，余额不足自动拉起购买弹窗并在支付成功后自动续发原问题；自定义模式保持原 `chat_with_ai` 流程 |
| E3-S2-I2 | OCR 与 AI 分析入口复用同一预检/拦截器 | Test Passed | E3-S2-I1 | `npm test`、`npm run build` 通过 | 上传页 OCR/AI 分析前已接入通用模式预检，余额不足自动拉起购买并在支付成功后自动重试原操作；自定义模式不受影响 |
| E4-S1-I1 | 首登自动创建本人成员与业务准入控制 | Todo | E1-S1-I2, E2-S1-I1 | 待补 | V2 强制登录后初始化成员；无成员时自动创建 `SELF` 默认成员 |
| E4-S1-I2 | 家庭成员管理页与跨页面成员切换 | Todo | E4-S1-I1 | 待补 | 用正式成员切换替换当前“仅默认成员阻塞”的临时方案 |
| E4-S2-I1 | 本地数据库 schema 迁移与成员过滤基础层 | Todo | E4-S1-I1 | 待补 | 重点改造 `src-tauri/src/db.rs` 与各命令 SQL，补齐 `owner_user_id + member_id` 隔离，并为 `chat_logs` 增加 `conversation_id`；本期不做旧数据自动迁移 |
| E4-S2-I2 | 报告、OCR、分析、趋势与 AI 对话成员级隔离 | Todo | E4-S1-I2, E4-S2-I1, E3-S2-I2 | 待补 | 保证不同家庭成员的资料、分析结果和聊天上下文完全独立 |


