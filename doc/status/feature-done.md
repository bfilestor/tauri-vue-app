# 健康管家用户体系与通用 AI 服务 · 已完成功能记录（feature-done）

---

## 记录说明

每条记录仅在 Issue 达到完成条件后追加，字段必须完整。

| 完成时间 | Issue 编号 | 功能摘要 | 测试结果 | 风险说明 |
|----------|------------|----------|----------|----------|
| 2026-04-02 16:34:55 | E1-S1-I1 | 新增设备凭证存储、激活/续期能力、统一签名请求客户端，并在应用启动时自动初始化 | `npm test` 4 项通过；`npm run build` 通过 | 首次激活签名材料仍使用可注入占位口径，待服务端确认正式 `secretProof/activate signature` 方案后切换 |
| 2026-04-02 17:31:32 | E1-S1-I2 | 新增认证 API（登录/注册/刷新/登出）与会话存储容器，接入请求层 401 自动刷新一次并支持访客态标记 | `npm test` 10 项通过；`npm run build` 通过 | 自动刷新依赖 `/app-api/auth/refresh-token` 返回新令牌；刷新失败会清理登录态并要求重新登录 |
| 2026-04-03 09:19:23 | E1-S2-I1 | 新增登录/注册/临时访客认证弹框，完成 Tab 切换、表单校验、注册赠送提示、条款占位链接及认证 API 调用接入 | `npm test` 17 项通过；`npm run build` 通过 | 验证码发送能力当前为前端占位提示，待后续 Issue 联调 `/app-api/auth/send-*` 接口后替换为真实发送流程 |
| 2026-04-03 09:22:47 | E1-S2-I2 | 侧边栏用户区改为会话驱动渲染，支持未登录/访客/已登录三态展示，接入会话恢复与退出登录联动 | `npm test` 22 项通过；`npm run build` 通过 | 剩余次数仍依赖 `userInfo` 字段口径，账户余额与套餐实时同步将在 E2 阶段补齐 |
| 2026-04-03 09:30:04 | E2-S1-I1 | 新增账户上下文状态层，封装 profile/balance/wallet/members/products 拉取与缓存策略，支持默认成员阻塞和 `20/100/500` 套餐卡映射 | `npm test` 26 项通过；`npm run build` 通过 | 商品档位映射依赖服务端返回可识别次数字段（如 `callTimes`）；若字段口径变更需在映射规则中补充别名 |
| 2026-04-03 09:42:37 | E2-S1-I2 | 新增共享账户上下文 store 与次数视图模型，侧边栏用户区和设置页通用模式概览共用同一余额状态，支持账号菜单入口与刷新失败缓存提示 | `npm test` 30 项通过；`npm run build` 通过 | 当前“账号菜单”为快捷入口提示，完整菜单交互与购买弹窗将在 E2-S2 阶段落地 |
| 2026-04-03 09:59:31 | E2-S2-I1 | 新增购买次数包弹窗与下单流程，支持套餐选择、订单创建、订单号/金额摘要展示、微信与支付宝切换并拉取支付二维码 | `npm test` 34 项通过；`npm run build` 通过 | 二维码展示依赖后端返回可直连图片 URL；若返回内容需二次签名或短时 token，后续需补充图片代理/续签逻辑 |
| 2026-04-03 10:15:45 | E2-S2-I2 | 新增订单状态轮询与支付结果反馈，支持超时/失败/取消状态提示，支付成功后自动刷新余额并关闭购买弹窗；注册赠送试用支持启动后自动消费提示并刷新余额 | `npm test` 37 项通过；`npm run build` 通过 | 轮询周期与上限当前为前端默认策略（3 秒/30 次），待联调稳定后可改为服务端可配置下发 |
| 2026-04-03 10:24:22 | E3-S1-I1 | 新增通用/自定义模式状态服务并接入设置页模式化改造，默认通用模式，支持重启恢复；自定义模式完整复用现有 Provider/Model/网络配置流程 | `npm test` 41 项通过；`npm run build` 通过 | 模式持久化当前存储在本地 `health.ai.mode`，后续如需多端同步可在账号配置接口就绪后迁移到服务端配置 |
| 2026-04-03 10:28:36 | E3-S1-I2 | 新增自定义模式“接入说明”抽屉，提供 API Key 准备步骤、智谱/文心/豆包平台类型说明与注册链接占位；未配置链接时提示“暂未配置”避免误导 | `npm test` 44 项通过；`npm run build` 通过 | 平台链接当前为配置占位未内置来源 URL；待产品确认官方地址后仅需更新配置表即可生效 |
| 2026-04-03 10:34:38 | E3-S2-I1 | AI 问答页新增通用模式调用前预检：发送前执行 `usage/precheck`，余额不足自动打开购买弹窗；支付成功后自动恢复并续发刚才的问题，自定义模式继续沿用原 `chat_with_ai` 链路 | `npm test` 46 项通过；`npm run build` 通过 | 访客态当前引导跳转系统设置页完成登录/购买，后续可在全局认证弹框支持跨页面唤起后改为一步直达登录 |
| 2026-04-03 10:41:20 | E3-S2-I2 | 上传页 OCR/AI 分析入口复用通用模式预检拦截：点击前执行 `usage/precheck`，余额不足弹购买并在支付成功后自动继续原动作；自定义模式维持现有调用链路 | `npm test` 46 项通过；`npm run build` 通过 | 访客态当前通过跳转系统设置页完成登录与购买，后续可接入全局登录弹框提升转化路径 |
| 2026-04-08 10:00:59 | E4-S1-I1 | 账户上下文新增首登成员初始化能力：登录后拉取成员列表，无成员时自动创建 `SELF` 本人成员并恢复默认成员上下文；若自动创建失败则保持成员阻塞，禁止后续业务放行 | `npm test` 53 项通过；`npm run build` 通过 | 目前仅补齐“默认本人成员自动创建”，尚未提供正式成员切换 UI；当前成员仍与默认成员保持一致，跨页面手动切换在 E4-S1-I2 落地 |
| 2026-04-08 10:46:00 | E4-S1-I2 | 账户上下文补齐当前成员持久化与成员 CRUD 接口，设置页新增家庭成员管理页，侧边栏支持全局成员切换；上传、历史、趋势与 AI 问答统一按当前成员联动刷新 | `npm test` 55 项通过；`npm run build` 通过 | 当前成员切换已落地在前端共享状态层，但本地 SQLite 与 Tauri 命令层尚未完成真实成员级隔离，需在 E4-S2-I1/E4-S2-I2 继续补齐 |
| 2026-04-08 15:05:00 | E4-S2-I1 | `db.rs` 改为版本化 schema 迁移结构，使用 `PRAGMA user_version` 支持 V1 基线到 V2 多用户多成员 schema 升级；新增 `local_users`、`family_members`、`chat_conversations`，并为业务表补齐 `owner_user_id`、`member_id`、`conversation_id` 与隔离索引 | `cargo test db -- --nocapture` 通过；`cargo check` 通过 | 本 issue 仅完成 SQLite 结构与迁移基础，不处理旧业务数据归属，也尚未把成员上下文真正接入记录/OCR/分析/趋势/聊天命令链路 |
| 2026-04-08 16:20:00 | E4-S2-I2 | 上传、历史、趋势、OCR、AI 分析与 AI 问答全部改为按 `owner_user_id + member_id` 查询与写入；聊天链路接入 `chat_conversations` 和 `conversation_id`，每个成员自动使用自己的独立会话空间 | `npm test` 55 项通过；`npm run build` 通过；`cargo check` 通过 | 当前会话模型为“每成员一个默认会话容器”，尚未提供多会话管理 UI；若后续要支持成员内多会话列表，可直接复用现有 `chat_conversations` 结构扩展 |
| 2026-04-08 22:10:00 | E4-S2-I2 | 补齐多成员隔离回归测试与命令层数据库隔离测试，覆盖前端 scope 构造、会话作用域解析，以及 `record/ai/trend/file/ocr` 命令在多成员场景下的查询、删除、状态统计与结果重建链路 | `npm test` 60 项通过；`cargo test commands:: -- --nocapture` 17 项通过；`cargo check` 通过 | 当前覆盖以本地单测和命令层集成为主，尚未包含真实后端接口联调和完整桌面端手工验收链路，后续联调阶段仍需按注册/成员切换/上传/OCR/AI 全流程复核 |
| 2026-04-08 23:05:00 | E5-S1-I1 | 新增 Tauri 本地家庭成员命令：`list/create/update/delete/set_default_family_member`，并完成默认成员自动分配、删除默认成员重排、最后一个成员删除保护与按 owner 隔离 | `cargo test commands::member -- --nocapture` 4 项通过；`cargo check` 通过 | 当前成员数据为本地优先写入，暂未增加云端同步冲突处理策略；后续若接入多端同步需补齐版本冲突解决 |
| 2026-04-08 23:18:00 | E5-S1-I2 | 账户上下文成员仓储改为本地命令优先（非 Tauri 环境回退远端接口），设置页“家庭成员管理”改为仅访客态阻断，登录后可执行新增/编辑/删除/设默认完整操作 | `npm test` 60 项通过；`npm run build` 通过 | 本地命令与远端接口存在双轨兜底路径，若后续远端字段口径变更需同步更新归一化映射，避免出现成员字段展示差异 |
| 2026-04-08 15:45:41 | E6-S1-I1 | 成员仓储回退策略调整为“后端权威源”：移除 `account-service` 本地优先调用，登录态成员列表与 CRUD/设默认统一通过 `/app-api/family-members` 系列接口执行 | `npm test` 63 项通过（新增 `account-service-member-repository` 测试） | 依赖后端接口可用性；离线状态下不再使用本地成员表兜底 |
| 2026-04-08 15:45:41 | E6-S2-I1 | 患者信息 Prompt 改为成员级配置：`save_config/get_config` 为 `user_custom_prompt_template` 增加成员作用域 key，设置页按当前成员读写，AI 分析与问答链路改为读取当前成员患者说明 | `cargo test commands::config -- --nocapture`、`cargo test commands::ai -- --nocapture`、`npm test` 通过 | 历史全局患者说明仅作为回退读取，不自动批量迁移为成员专属配置 |
| 2026-04-08 15:45:41 | E6-S2-I2 | 检查项目与指标成员化：`db.rs` 升级 V3 并新增项目/指标成员字段与索引，`project/indicator/ocr/trend/file/record` 命令及上传/趋势/设置页全部按 `owner_user_id + member_id` 作用域运行 | `cargo test commands:: -- --nocapture` 28 项通过；`cargo test db -- --nocapture` 3 项通过；`cargo check` 通过；`npm run build` 通过 | 旧全局项目未做自动归属迁移，需在对应成员下重新维护项目/指标 |
| 2026-04-13 14:52:52 | E7-S1-I1 | 数据层增加场景化默认模型能力：SQLite 升级 V4，`ai_models` 新增 `is_default_ocr/is_default_analysis`，Provider 新增场景化设默认命令并兼容旧 `set_default_model` | `cargo test db -- --nocapture`、`cargo test provider::tests -- --nocapture` 通过 | 旧链路仍依赖 `is_default`；当前已与 analysis 场景保持同步，后续接口分流完成后再逐步收敛到场景字段 |
| 2026-04-13 15:01:12 | E7-S1-I2 | Provider 连通性检测支持场景参数：`test_provider_connection` 新增 `scene=ocr|analysis`（默认 analysis），按场景默认模型检测并输出场景化结果 | `cargo test provider::tests -- --nocapture`、`cargo check` 通过 | 前端尚未接入“双检测按钮”，当前仍走默认 analysis；将于 E7-S2-I2 完成界面接入 |
| 2026-04-13 15:03:44 | E7-S2-I1 | 设置页模型列表支持双场景默认：新增 `分析默认/OCR默认` 标签与“设分析默认/设OCR默认”操作，调用 `set_default_model_for_scene` 命令落库 | `npm run build`、`cargo check` 通过 | 旧“设默认”按钮已替换为双按钮，需在后续 issue 补齐双场景检测入口与场景路由摘要 |
| 2026-04-13 15:06:13 | E7-S2-I2 | 设置页提供商配置区拆分场景化检测入口：新增“检测分析接口/检测OCR接口”双按钮，调用 `test_provider_connection(scene)` 并展示 OCR/分析路由摘要 | `npm run build`、`cargo check` 通过 | 当前路由摘要基于当前 Provider 模型默认标记；若未来支持跨 Provider 场景路由，摘要展示需升级为全局视图 |
| 2026-04-13 15:17:27 | E7-S3-I1 | OCR 调用链切换为 OCR 场景路由：`http_client` 新增场景化配置/模型解析函数，`start_ocr/retry_ocr` 改为使用 OCR 场景默认 provider/model 并输出 scene 路由日志 | `cargo test http_client::tests -- --nocapture`、`cargo test ocr::tests -- --nocapture`、`cargo check` 通过 | 目前 OCR/分析仍共享 provider 级 API URL 字段，若后续要“同 provider 双 URL”需继续扩展 provider 数据结构 |
| 2026-04-13 15:26:01 | E7-S3-I2 | AI 分析与问答调用链切换为分析场景路由：`start_ai_analysis` 与 `chat_with_ai` 改为 `load_ai_config_for_analysis + get_default_model_for_analysis`，并增加 scene=analysis 路由日志 | `cargo test ai::tests -- --nocapture`、`cargo check` 通过 | 目前分析与问答共享同一 analysis 场景；若后续产品要求再拆“分析/问答双路由”，可在 scene 维度继续扩展 |
