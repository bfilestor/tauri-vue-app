# 已完成能力清单

> 每完成一个 Issue，必须在此记录新增能力、接口、表结构或关键组件。  
> 按完成时间倒序排列，最新的记录在最前面。

## 记录格式说明

| 字段 | 说明 |
|------|------|
| 能力名 | 简要概括新增的能力 |
| 归属模块 | 所属业务模块 |
| 关联 Issue | 对应的 Issue 编号 |
| 涉及文件 | 修改或新增的文件列表 |
| 备注 | 补充说明 |

---

## 能力记录

### 2026-02-14

| 能力名 | 归属模块 | 关联 Issue | 涉及文件 | 备注 |
|--------|----------|-----------|----------|------|
| AI 问答完整模块 | AI 问答 | ISS-036, ISS-038, ISS-040 | `src/views/aiqa/index.vue` | 左右分栏布局 + 历史时间轴 + 流式对话框 |
| AI 历史分析查询与更新 | AI 问答 | ISS-037 | `src-tauri/src/commands/ai.rs` | 历史记录按时间倒序 + 内容编辑保存 |
| 聊天记录持久化 | AI 问答 | ISS-039 | `src-tauri/src/db.rs`, `src-tauri/src/commands/ai.rs` | chat_logs 表 + 历史消息加载 |
| 全局通知中心 | 通知与优化 | ISS-032 | `src/App.vue` | 监听 Tauri 事件弹出 ElNotification |
| 趋势分析图表 | 趋势分析 | ISS-030, ISS-031 | `src/views/trends/index.vue` | ECharts 折线/柱状图 + 异常高亮 + 最新概览 |
| 趋势数据查询 | 趋势分析 | ISS-028 | `src-tauri/src/commands/trend.rs` | 按指标 ID 查询历史值 + 对比计算 |
| AI 分析流式生成 | AI 分析 | ISS-025, ISS-027 | `src-tauri/src/commands/ai.rs`, `src/views/history/index.vue` | SSE 流式请求 + Markdown 渲染 |
| OCR 识别与结果展示 | OCR | ISS-022, ISS-024 | `src-tauri/src/commands/ocr.rs`, `src/views/history/index.vue` | 视觉模型调用 + JSON 解析 + 结果表格 |
| 文件上传与预览 | 上传与存档 | ISS-018, ISS-019, ISS-021 | `src-tauri/src/commands/file.rs`, `src/views/upload/index.vue` | 批量上传 + 图片预览 + 缩略图 |
| 检查记录管理 | 上传与存档 | ISS-016, ISS-017 | `src-tauri/src/commands/record.rs`, `src/views/history/index.vue` | 记录 CRUD + 状态流转 |

### 2026-02-13

| 能力名 | 归属模块 | 关联 Issue | 涉及文件 | 备注 |
|--------|----------|-----------|----------|------|
| 系统设置完整页面 | 系统设置 | ISS-009, ISS-011, ISS-014, ISS-015 | `src/views/settings/index.vue` | AI配置（URL/Key/模型/代理）+ Prompt模板 + 项目CRUD表格 + 指标管理抽屉 |
| OCR/AI Prompt 模板存取 | 系统设置 | ISS-015 | `src/views/settings/index.vue`, `src-tauri/src/commands/config.rs` | 前端编辑 + 后端 config 键值存储 |
| AI 接口设置 UI | 系统设置 | ISS-014 | `src/views/settings/index.vue` | 含连接测试占位、代理开关、模型Tag管理 |
| 通用配置 get/save 接口 | 系统设置 | ISS-012 | `src-tauri/src/commands/config.rs` | UPSERT 语义，支持任意 key-value 配置 |
| 检查指标 CRUD 接口 | 系统设置 | ISS-010, ISS-011 | `src-tauri/src/commands/indicator.rs`, `src/views/settings/index.vue` | list/create/update/delete，关联数据校验 |
| 检查项目 CRUD 接口 | 系统设置 | ISS-008, ISS-009 | `src-tauri/src/commands/project.rs`, `src/views/settings/index.vue` | 含 pictures 子目录自动创建、级联删除指标 |
| Tauri 状态注入 | 基础设施 | ISS-007 | `src-tauri/src/lib.rs`, `src-tauri/src/commands/mod.rs` | Database + AppDir 注入 Tauri State |
| SQLite 8 张表 DDL | 基础设施 | ISS-006 | `src-tauri/src/db.rs` | checkup_projects, indicators, checkup_records, checkup_files, ocr_results, ai_analyses, indicator_values, system_config |
| Rust 后端依赖 | 基础设施 | ISS-005 | `src-tauri/Cargo.toml` | rusqlite(bundled), reqwest(socks), uuid, chrono, base64, tokio |
| 中文化布局组件 | 基础设施 | ISS-004 | `src/components/layout/index.vue`, `src/App.vue`, `index.html` | 左侧固定导航栏 + Material Symbols 图标 + Noto Sans SC 字体 |
| Vue Router 三模块路由 | 基础设施 | ISS-003 | `src/router/index.js`, `src/views/upload/index.vue`, `src/views/trends/index.vue`, `src/views/settings/index.vue` | /upload, /trends, /settings + keepAlive |
| ECharts 安装 | 基础设施 | ISS-002 | `package.json` | echarts + vue-echarts |
| TailwindCSS 集成 | 基础设施 | ISS-001 | `src/assets/main.css`, `vite.config.js`, `src/main.js` | @tailwindcss/vite 插件方式 |
