# 开发 Issue 进度清单

> 说明：本清单用于承接各 Epic 的 Issue 进度，完成一个 Issue 必须更新状态与测试结果。  
> 每完成一个 Issue，需同步更新 **状态**、**测试结果** 和 **备注**。

## 状态约定

| 状态 | 含义 |
|------|------|
| Todo | 待开发 |
| Skip | 本轮跳过 |
| In Progress | 开发中 |
| Test Passed | 测试通过 |
| Done | 已完成并合并 |

---

## Epic 1：基础设施搭建

| Issue | 描述 | 状态 | 依赖 | 测试 | 备注 |
|-------|------|------|------|------|------|
| ISS-001 | 安装 TailwindCSS 并配置 | Done | - | Passed | @tailwindcss/vite 插件 + main.css 入口 |
| ISS-002 | 安装 ECharts 及 vue-echarts | Done | ISS-001 | Passed | npm install echarts vue-echarts |
| ISS-003 | 配置 Vue Router 三模块路由 | Done | - | Passed | /upload, /trends, /settings 三路由 + keepAlive |
| ISS-004 | 创建全局中文化布局组件 | Done | ISS-001, ISS-003 | Passed | 左侧栏 + Logo + 导航 + 内容区，Material Symbols 图标 |
| ISS-005 | Rust 后端依赖配置 | Done | - | Passed | rusqlite(bundled), reqwest(socks), uuid, chrono, base64, futures-util |
| ISS-006 | SQLite 数据库初始化模块（8张表） | Done | ISS-005 | Passed | WAL 模式, 外键约束, 8 张表 DDL |
| ISS-007 | Tauri 状态管理（DB连接池注入） | Done | ISS-006 | Passed | Database + AppDir 注入 Tauri State |

---

## Epic 2：系统设置模块

| Issue | 描述 | 状态 | 依赖 | 测试 | 备注 |
|-------|------|------|------|------|------|
| ISS-008 | [后端] 检查项目 CRUD Commands | Done | ISS-007 | Passed | list/create/update/delete_project，含 pictures 子目录创建 |
| ISS-009 | [前端] 项目管理页面 UI | Done | ISS-008 | Passed | el-table 列表 + 新增/编辑弹窗 + 状态切换 |
| ISS-010 | [后端] 检查指标 CRUD Commands | Done | ISS-008 | Passed | list/create/update/delete_indicator，关联数据校验 |
| ISS-011 | [前端] 指标管理 UI | Done | ISS-009, ISS-010 | Passed | 抽屉面板 + 表格 + CRUD 弹窗 |
| ISS-012 | [后端] 通用配置 get/save Commands | Done | ISS-007 | Passed | UPSERT 语义的 system_config 操作 |
| ISS-013 | [后端] AI 连接测试 Command | Done | ISS-012 | Passed | test_ai_connection 调用真实 API 验证连接，支持代理 |
| ISS-014 | [前端] AI 设置页面 UI | Done | ISS-012, ISS-013 | Passed | API/Key/模型/代理全配置 + Prompt模板编辑 + 实际连接测试 |
| ISS-015 | [后端+前端] OCR/AI Prompt 模板设置 | Done | ISS-012 | Passed | 前端 Prompt 编辑 + 后端 config 存取 |

---

## Epic 3：检查数据上传与存档模块

| Issue | 描述 | 状态 | 依赖 | 测试 | 备注 |
|-------|------|------|------|------|------|
| ISS-016 | [后端] 检查记录 CRUD Commands | Done | ISS-007 | Passed | list/create/update/delete/get_record，含文件数与项目名称 |
| ISS-017 | [前端] 检查记录列表页面 | Done | ISS-016 | Passed | 卡片列表 + 展开详情 + 创建弹窗 + 状态标签 |
| ISS-018 | [后端] 文件批量上传 Command | Done | ISS-008, ISS-016 | Passed | base64 解码 + 结构化存储 + DB 记录 |
| ISS-019 | [前端] 文件上传区域 UI | Done | ISS-018 | Passed | 项目选择 + 文件预览 + 批量上传 |
| ISS-020 | [后端] 文件预览与删除 Commands | Done | ISS-018 | Passed | read_file_base64 + delete_file |
| ISS-021 | [前端] 图片预览大图弹窗 | Done | ISS-020 | Passed | 全尺寸预览 + 文件缩略图 |
| ISS-022 | [后端] OCR 识别 Command | Done | ISS-010, ISS-012, ISS-018 | Passed | 异步 OCR + 视觉 API + JSON 解析 + indicator_values 写入 |
| ISS-023 | [后端] OCR 状态与结果查询 | Done | ISS-022 | Passed | get_ocr_status + get_ocr_results |
| ISS-024 | [前端] OCR 操作与结果展示 UI | Done | ISS-022, ISS-023 | Passed | 进度条 + 结果弹窗表格 + Event 监听 |
| ISS-025 | [后端] AI 分析 Command（流式） | Done | ISS-012, ISS-022 | Passed | SSE 流式请求 + 历史数据汇集 + ai_stream_chunk 事件 |
| ISS-026 | [后端] AI 分析结果查询 | Done | ISS-025 | Passed | get_ai_analysis |
| ISS-027 | [前端] AI 分析操作与 Markdown 展示 | Done | ISS-025, ISS-026 | Passed | 实时流式渲染 + Markdown 解析 + 结果弹窗 |

---

## Epic 4：趋势分析模块

| Issue | 描述 | 状态 | 依赖 | 测试 | 备注 |
|-------|------|------|------|------|------|
| ISS-028 | [后端] 趋势数据查询与对比 Commands | Done | ISS-010, ISS-022 | Passed | get_project_trends + get_all_trends |
| ISS-029 | [前端] 指标选择树组件 | Done | ISS-010 | Passed | 项目下拉选择 + 全项目查看按钮 |
| ISS-030 | [前端] ECharts 趋势图渲染 | Done | ISS-002, ISS-028, ISS-029 | Passed | 渐变线形图 + 参考范围标线 + 异常点高亮 |
| ISS-031 | [前端] 最新检查概览卡片 | Done | ISS-028 | Passed | 最新值展示 + 异常/正常状态颜色 |

---

## Epic 5：通知与体验优化

| Issue | 描述 | 状态 | 依赖 | 测试 | 备注 |
|-------|------|------|------|------|------|
| ISS-032 | 全局通知中心（Tauri Event） | Done | ISS-022, ISS-025 | Passed | ocr_progress/complete/error + ai_stream_chunk/done/error |
| ISS-033 | 统一错误处理与中文提示 | Done | ISS-007 | Passed | 全链路中文错误提示 + ElMessage/ElNotification |

---

## Epic 6：AI 问答模块

| Issue | 描述 | 状态 | 依赖 | 测试 | 备注 |
|-------|------|------|------|------|------|
| ISS-036 | [前端] AI 问答路由与入口 | Done | - | Passed | /aiqa 路由 + 侧边栏 Material Symbol |
| ISS-037 | [后端] 历史分析查询与更新 | Done | ISS-025 | Passed | JOIN checkup_records + content update |
| ISS-038 | [前端] 左侧历史分析时间轴 | Done | ISS-037 | Passed | 展开/编辑/引用功能 |
| ISS-039 | [后端] 问答系统后端（DB+Command） | Done | ISS-006 | Passed | chat_logs 表 + 流式 chat_with_ai |
| ISS-040 | [前端] 问答对话框 UI | Done | ISS-039 | Passed | 流式渲染 + 历史记录 + 清空/复制 |

---

---

## Epic 7：数据脱敏模块

| Issue | 描述 | 状态 | 依赖 | 测试 | 备注 |
|-------|------|------|------|------|------|
| ISS-041 | [前端] Canvas 图片编辑器组件基础 | Done | - | Passed | 基础 Canvas 封装 + 缩放/平移 |
| ISS-042 | [前端] 裁剪功能实现 | Done | ISS-041 | Passed | 矩形裁剪 + 确认/取消 |
| ISS-043 | [前端] 区域马赛克功能 | Done | ISS-041 | Passed | 像素化模糊 |
| ISS-052 | [前端] 图片旋转功能实现 | Done | ISS-041 | Passed | 左转/右转 90 度 |
| ISS-044 | [前端] 脱敏工作台页面 (`/desensitize`) | Done | ISS-041 | Passed | 布局/路由/工具栏 |
| ISS-045 | [前端] 文件 IO 集成 (Open/Save) | Done | ISS-044 | Passed | fs/dialog 插件集成 |

---

## Epic 8：手机扫码上传
| Issue | 描述 | 状态 | 依赖 | 测试 | 备注 |
|-------|------|------|------|------|------|
| ISS-046 | [后端] Axum HTTP 服务与 IP 获取 | Done | ISS-005 | - | Implemented with Axum 0.8 & local-ip-address |
| ISS-047 | [后端] 文件接收与二维码生成 | Done | ISS-046 | - | Mobile upload logic & Event emitting |
| ISS-048 | [前端] 手机上传二维码弹窗 | Done | ISS-047 | - | Vue component implemented |
| ISS-049 | [前端] 实时接收文件处理 | Done | ISS-048 | - | Event listener & read_temp_file command |



---

## Epic 9：脱敏工作台手机上传
| Issue | 描述 | 状态 | 依赖 | 测试 | 备注 |
|-------|------|------|------|------|------|
| ISS-050 | [前端] 提取手机上传弹窗组件 | Done | ISS-048 | - | Shared component |
| ISS-051 | [前端] 脱敏工作台集成手机上传 | Done | ISS-050 | - | With replacement content |

---


## Epic 10：多 AI 提供商接入

| Issue | 描述 | 状态 | 依赖 | 测试 | 备注 |
|-------|------|------|------|------|------|
| ISS-053 | [后端] 新增 ai_providers 和 ai_models 表 | Done | ISS-006 | Build ✅ | db.rs 新增两张表 |
| ISS-054 | [后端] 旧数据向前兼容迁移 | Done | ISS-053 | Build ✅ | migrate_legacy_config() |
| ISS-055 | [后端] Provider CRUD Commands | Done | ISS-053 | Build ✅ | commands/provider.rs |
| ISS-056 | [后端] Model CRUD Commands | Done | ISS-055 | Build ✅ | commands/provider.rs |
| ISS-057 | [后端] Provider 连接测试 Command | Done | ISS-055, ISS-013 | Build ✅ | test_provider_connection |
| ISS-058 | [后端] 重构 http_client 获取默认提供商 | Done | ISS-053, ISS-056 | Build ✅ | load_default_provider_config |
| ISS-059 | [后端] 更新 OCR 和 AI 使用新配置 | Done | ISS-058 | Build ✅ | load_ai_config 自动 fallback |
| ISS-060 | [前端] Tab 拆分（模型服务 + Prompt 设置） | Done | ISS-014 | Build ✅ | 两个独立 Tab |
| ISS-061 | [前端] 提供商列表侧边栏 UI | Done | ISS-055, ISS-060 | Build ✅ | 搜索+Switch+高亮 |
| ISS-062 | [前端] 提供商详情配置面板 | Done | ISS-061, ISS-057 | Build ✅ | API Key/URL+检测 |
| ISS-063 | [前端] 模型列表与管理 UI | Done | ISS-062, ISS-056 | Build ✅ | 分组+默认标签+操作 |
| ISS-064 | [前端] 添加/编辑提供商弹窗 | Done | ISS-061 | Build ✅ | 含首字母头像预览 |
| ISS-065 | [前端] 添加/编辑模型弹窗 | Done | ISS-063 | Build ✅ | model_id+name+group |
| ISS-066 | [前端] 全局网络与高级设置面板 | Done | ISS-060 | Build ✅ | 代理+超时设置 |



## Epic 11：增强 Prompt 管理

| Issue | 描述 | 状态 | 依赖 | 测试 | 备注 |
|-------|------|------|------|------|------|
| ISS-067 | [前端] Prompt 设置页面开发者模式（7次点击触发） | Done | ISS-060 | Build ✅ | 标题栏点击计数 + 隐藏/显示保护 Prompt |
| ISS-068 | [前端] 新增患者情况说明 Prompt（用户自定义） | Done | ISS-060 | Build ✅ | user_custom_prompt_template 配置键 |
| ISS-069 | [后端] chat_with_ai 整合用户自定义 + AI 分析 Prompt | Done | ISS-068 | Build ✅ | 系统 Prompt 拼接逻辑 |

---



## 汇总统计

| 状态 | 数量 |
|------|------|
| Todo | 0 |
| Skip | 0 |
| In Progress | 0 |
| Test Passed | 0 |
| Done | 67 |
| **合计** | **67** |


