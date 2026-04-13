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
| E4-S1-I1 | 首登自动创建本人成员与业务准入控制 | Test Passed | E1-S1-I2, E2-S1-I1 | `npm test`、`npm run build` 通过 | 已实现登录后成员列表检查、无成员自动创建 `SELF` 本人成员、创建失败时阻塞业务入口 |
| E4-S1-I2 | 家庭成员管理页与跨页面成员切换 | Test Passed | E4-S1-I1 | `npm test`、`npm run build` 通过 | 已补齐成员管理页、侧边栏成员切换、当前成员持久化与上传/历史/趋势/AI 问答页联动刷新 |
| E4-S2-I1 | 本地数据库 schema 迁移与成员过滤基础层 | Test Passed | E4-S1-I1 | `cargo test db -- --nocapture`、`cargo check` 通过 | 已补齐 V2 schema、`PRAGMA user_version` 迁移机制、新表/新字段/索引与成员上下文读取 helper；未处理旧数据归属迁移 |
| E4-S2-I2 | 报告、OCR、分析、趋势与 AI 对话成员级隔离 | Test Passed | E4-S1-I2, E4-S2-I1, E3-S2-I2 | `npm test` 60 项通过、`npm run build`、`cargo check`、`cargo test commands:: -- --nocapture` 17 项通过 | 已完成前端 scope 透传与 Tauri 命令层 owner/member/conversation 隔离，并补齐 scope/record/ai/trend/file/ocr 的成员隔离回归测试，不同成员的记录、OCR、分析、趋势与聊天上下文互不影响 |
| E5-S1-I1 | Tauri 本地成员管理命令（CRUD + 设默认） | Test Passed | E4-S2-I1, E4-S2-I2 | `cargo test commands::member -- --nocapture`、`cargo check` 通过 | 已新增 `list/create/update/delete/set_default_family_member` 命令，覆盖首成员默认、跨用户隔离、删除默认重排、最后一个成员删除保护 |
| E5-S1-I2 | 前端成员管理接入本地成员仓储并修复登录态展示 | Test Passed | E5-S1-I1 | `npm test` 60 项通过、`npm run build` 通过 | 账户上下文成员操作改为本地命令优先（无本地命令环境回退远端接口），设置页成员管理区改为仅访客态阻断，登录后可直接新增/编辑/删除/设默认 |
| E6-S1-I1 | 成员仓储改为登录后云端权威源（禁用本地优先） | Test Passed | E5-S1-I2 | `npm test` 63 项通过 | `account-service` 成员仓储已移除本地优先路径，登录态成员列表/CRUD/设默认统一走 `/app-api/family-members` |
| E6-S2-I1 | 患者信息 Prompt 改为成员级配置 | Test Passed | E6-S1-I1 | `cargo test commands::config -- --nocapture`、`cargo test commands::ai -- --nocapture`、`npm test` 通过 | `save_config/get_config` 新增成员作用域（key=`user_custom_prompt_template`），AI 分析与问答读取当前成员患者说明 |
| E6-S2-I2 | 检查项目与指标成员化（schema + 命令 + 页面） | Test Passed | E6-S2-I1 | `cargo test commands:: -- --nocapture`、`cargo test db -- --nocapture`、`cargo check`、`npm run build` 通过 | `db.rs` 升级 V3 并为项目/指标加成员字段与索引；`project/indicator/ocr/trend/file/record` 与上传/趋势/设置页全部按当前成员作用域查询与写入 |
| E7-S1-I1 | 数据层增加场景化默认模型能力（OCR/分析双默认） | Test Passed | E6-S2-I2 | `cargo test db -- --nocapture`、`cargo test provider::tests -- --nocapture` 通过 | 已升级 DB schema 到 V4，新增 `is_default_ocr/is_default_analysis` 与场景化设默认命令（兼容旧 `set_default_model` 走 analysis） |
| E7-S1-I2 | Provider 连通性检测改为按场景测试 | Test Passed | E7-S1-I1 | `cargo test provider::tests -- --nocapture`、`cargo check` 通过 | `test_provider_connection` 已支持 `scene=ocr|analysis`，未传 scene 默认 analysis，按场景默认模型选路并在返回消息中包含场景信息 |
| E7-S2-I1 | 设置页模型列表增加 OCR 默认/分析默认标记与操作 | Test Passed | E7-S1-I1 | `npm run build`、`cargo check` 通过 | 模型列表已展示 `分析默认/OCR默认` 双标签，并通过 `set_default_model_for_scene` 支持双场景默认设置 |
| E7-S2-I2 | 设置页提供商配置区增加场景化检测入口与路由摘要 | Todo | E7-S1-I2 | 待补充 | 检测按钮拆分为“检测 OCR 接口/检测分析接口” |
| E7-S3-I1 | OCR 调用链切换为 OCR 场景路由 | Todo | E7-S1-I1 | 待补充 | `start_ocr/retry_ocr` 使用 OCR 场景配置与模型 |
| E7-S3-I2 | AI 分析与问答调用链切换为分析场景路由 | Todo | E7-S1-I1 | 待补充 | `start_ai_analysis/chat_with_ai` 使用分析场景配置与模型 |


