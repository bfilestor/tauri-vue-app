# 健康管理系统（Health Guard）— 功能需求文档

> **版本**: v1.0  
> **日期**: 2026-02-11  
> **技术栈**: Tauri 2 + Vue 3 + Element Plus + TailwindCSS + SQLite  
> **目标平台**: Windows 桌面端

---

## 1. 系统概述

### 1.1 背景

针对需要长期、持续关注自身健康状况的患者，定期前往医院进行多项目检查（如血常规、肝功能、尿肾功能等）。现有检查结果以纸质或图片形式分散存储，缺乏系统化管理和趋势分析能力。

### 1.2 目标

构建一个桌面端健康管理系统，实现：
- **检查数据数字化**：通过 OCR 将检查报告图片转化为结构化数据
- **AI 辅助分析**：将多次检查结果综合发送给 AI，获取治疗方案建议
- **趋势可视化**：以图表形式展示核心检查指标随时间的变化趋势
- **本地化存储**：所有数据（图片、OCR 结果、AI 分析）存储在本地，保护患者隐私

### 1.3 用户角色

| 角色 | 说明 |
|------|------|
| 患者用户 | 系统唯一使用者，上传检查报告、查看分析结果、管理系统设置 |

---

## 2. 系统架构

### 2.1 整体架构

```
┌─────────────────────────────────────────────────┐
│                   Vue 3 前端                      │
│  ┌─────────────┬──────────────┬────────────────┐ │
│  │ 检查数据上传  │   系统设置    │   趋势分析     │ │
│  │ 与存档模块   │   模块        │   模块         │ │
│  └─────┬───────┴──────┬───────┴───────┬────────┘ │
│        │              │               │           │
│  ┌─────┴──────────────┴───────────────┴────────┐ │
│  │          Tauri IPC 通信层                     │ │
│  └─────────────────┬───────────────────────────┘ │
└────────────────────┼─────────────────────────────┘
                     │
┌────────────────────┼─────────────────────────────┐
│                Rust 后端                          │
│  ┌─────────────────┴───────────────────────────┐ │
│  │              命令处理层 (Commands)            │ │
│  ├─────────────┬──────────────┬────────────────┤ │
│  │ 文件管理服务 │  OCR/AI 服务  │  数据查询服务  │ │
│  ├─────────────┴──────────────┴────────────────┤ │
│  │           SQLite 数据访问层 (DAL)            │ │
│  ├─────────────────────────────────────────────┤ │
│  │        文件系统 + SQLite 数据库               │ │
│  └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

### 2.2 领域模型（DDD）

```
┌─ 聚合根：检查记录 (CheckupRecord) ─────────────────┐
│  - id: UUID                                        │
│  - checkup_date: Date         # 检查日期            │
│  - created_at: DateTime                            │
│  - status: Enum               # 待OCR/已OCR/已分析  │
│                                                     │
│  ┌─ 实体：检查项文件 (CheckupFile) ──────────────┐  │
│  │  - id: UUID                                   │  │
│  │  - record_id: FK                              │  │
│  │  - project_id: FK          # 所属检查项目      │  │
│  │  - original_filename: String                  │  │
│  │  - stored_path: String     # 本地存储相对路径   │  │
│  │  - file_type: String       # image/pdf        │  │
│  │  - uploaded_at: DateTime                      │  │
│  └───────────────────────────────────────────────┘  │
│                                                     │
│  ┌─ 值对象：OCR 结果 (OcrResult) ────────────────┐  │
│  │  - id: UUID                                   │  │
│  │  - file_id: FK                                │  │
│  │  - project_id: FK                             │  │
│  │  - checkup_date: Date                         │  │
│  │  - raw_json: Text          # OCR 原始返回      │  │
│  │  - parsed_items: JSON      # 解析后的指标列表   │  │
│  │  - created_at: DateTime                       │  │
│  └───────────────────────────────────────────────┘  │
│                                                     │
│  ┌─ 值对象：AI 分析结果 (AiAnalysis) ────────────┐  │
│  │  - id: UUID                                   │  │
│  │  - record_id: FK                              │  │
│  │  - request_prompt: Text                       │  │
│  │  - response_content: Text  # AI 返回的建议     │  │
│  │  - model_used: String                         │  │
│  │  - created_at: DateTime                       │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘

┌─ 聚合根：检查项目 (CheckupProject) ────────────────┐
│  - id: UUID                                        │
│  - name: String               # 如"血常规"          │
│  - description: String                             │
│  - sort_order: Integer                             │
│  - is_active: Boolean                              │
│  - created_at: DateTime                            │
│                                                     │
│  ┌─ 值对象：检查指标 (Indicator) ────────────────┐  │
│  │  - id: UUID                                   │  │
│  │  - project_id: FK                             │  │
│  │  - name: String            # 如"白细胞计数"     │  │
│  │  - unit: String            # 如"×10⁹/L"       │  │
│  │  - reference_range: String # 如"3.5-9.5"      │  │
│  │  - sort_order: Integer                        │  │
│  │  - is_core: Boolean        # 是否为核心指标     │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘

┌─ 值对象：系统配置 (SystemConfig) ──────────────────┐
│  - id: UUID                                        │
│  - config_key: String                              │
│  - config_value: Text (JSON)                       │
│  - updated_at: DateTime                            │
└─────────────────────────────────────────────────────┘
```

### 2.3 数据库设计（SQLite）

#### 表：`checkup_projects`（检查项目）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT (UUID) | 主键 |
| name | TEXT | 项目名称 |
| description | TEXT | 项目描述 |
| sort_order | INTEGER | 排序顺序 |
| is_active | INTEGER (0/1) | 是否启用 |
| created_at | TEXT (ISO8601) | 创建时间 |
| updated_at | TEXT (ISO8601) | 更新时间 |

#### 表：`indicators`（检查指标）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT (UUID) | 主键 |
| project_id | TEXT (FK) | 所属项目ID |
| name | TEXT | 指标名称 |
| unit | TEXT | 单位 |
| reference_range | TEXT | 参考范围 |
| sort_order | INTEGER | 排序顺序 |
| is_core | INTEGER (0/1) | 是否核心指标（用于趋势图） |
| created_at | TEXT (ISO8601) | 创建时间 |

#### 表：`checkup_records`（检查记录）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT (UUID) | 主键 |
| checkup_date | TEXT (YYYY-MM-DD) | 检查日期 |
| status | TEXT | 状态：pending_ocr / ocr_done / analyzed |
| notes | TEXT | 备注 |
| created_at | TEXT (ISO8601) | 创建时间 |
| updated_at | TEXT (ISO8601) | 更新时间 |

#### 表：`checkup_files`（检查文件）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT (UUID) | 主键 |
| record_id | TEXT (FK) | 关联检查记录 |
| project_id | TEXT (FK) | 所属检查项目 |
| original_filename | TEXT | 原始文件名 |
| stored_path | TEXT | 存储相对路径 |
| file_size | INTEGER | 文件大小（字节） |
| mime_type | TEXT | 文件MIME类型 |
| uploaded_at | TEXT (ISO8601) | 上传时间 |

#### 表：`ocr_results`（OCR 结果）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT (UUID) | 主键 |
| file_id | TEXT (FK) | 关联文件 |
| record_id | TEXT (FK) | 关联检查记录 |
| project_id | TEXT (FK) | 所属检查项目 |
| checkup_date | TEXT | 检查日期 |
| raw_json | TEXT | OCR 原始返回JSON |
| parsed_items | TEXT (JSON) | 解析后的指标KV列表 |
| status | TEXT | 状态：processing / success / failed |
| error_message | TEXT | 错误信息 |
| created_at | TEXT (ISO8601) | 创建时间 |

#### 表：`ai_analyses`（AI 分析记录）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT (UUID) | 主键 |
| record_id | TEXT (FK) | 关联检查记录 |
| request_prompt | TEXT | 发送给AI的完整提示词 |
| response_content | TEXT | AI 返回的建议内容 |
| model_used | TEXT | 使用的模型名称 |
| status | TEXT | 状态：processing / success / failed |
| error_message | TEXT | 错误信息 |
| created_at | TEXT (ISO8601) | 创建时间 |

#### 表：`indicator_values`（指标值，用于趋势分析）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT (UUID) | 主键 |
| ocr_result_id | TEXT (FK) | 关联OCR结果 |
| record_id | TEXT (FK) | 关联检查记录 |
| project_id | TEXT (FK) | 所属检查项目 |
| indicator_id | TEXT (FK) | 关联指标定义 |
| checkup_date | TEXT | 检查日期 |
| value | REAL | 指标数值 |
| value_text | TEXT | 原始文本值（可能含符号） |
| is_abnormal | INTEGER (0/1) | 是否异常 |
| created_at | TEXT (ISO8601) | 创建时间 |

#### 表：`system_config`（系统配置）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT (UUID) | 主键 |
| config_key | TEXT UNIQUE | 配置键 |
| config_value | TEXT | 配置值(JSON) |
| updated_at | TEXT (ISO8601) | 更新时间 |

---

## 3. 模块详细需求

### 3.1 模块一：检查数据上传与存档

#### 3.1.1 检查记录创建

- 用户点击"新建检查记录"按钮
- 选择检查日期（默认当天）
- 系统创建一条 `checkup_records` 记录，状态为 `pending_ocr`

#### 3.1.2 文件上传

- 在检查记录详情页，用户可分项目上传图片文件
- 界面展示所有启用的检查项目（来自 `checkup_projects` 表）
- 每个项目区域提供文件选择区域，支持：
  - 多文件选择（支持 jpg/png/bmp/pdf）
  - 拖拽上传
  - 文件预览（缩略图）
- 点击"一键上传"按钮，批量保存所有项目下的文件
- 文件存储路径规则：`{程序所在目录}/pictures/{项目名称}/{YYYY-MM-DD}/{原始文件名}`
- 同时写入 `checkup_files` 表记录元信息

#### 3.1.3 OCR 识别

- 上传完成后，用户可点击"OCR 识别"按钮
- 系统将该次检查记录下所有文件的图片，按项目分组，调用 AI 接口进行 OCR
- OCR 请求通过 Rust 后端发起 HTTP 请求（可配置 SOCKS 代理）
- OCR 结果保存到 `ocr_results` 表
- 同时解析出各指标值，写入 `indicator_values` 表
- 识别完成后，发送应用内通知（Element Plus Notification）
- 检查记录状态更新为 `ocr_done`

#### 3.1.4 AI 分析

- 用户收到 OCR 完成通知后，在检查记录页面可查看 OCR 结果
- 用户可选择"发起 AI 分析"
- 系统将该次检查的全部 OCR 结构化数据 + 历史检查数据，组装成 Prompt 发送给 AI
- AI 返回结果保存到 `ai_analyses` 表
- 支持流式返回，实时显示 AI 回复内容
- 检查记录状态更新为 `analyzed`
- AI 分析结果以 Markdown 格式渲染展示

#### 3.1.5 检查记录列表

- 时间倒序展示所有检查记录
- 显示：检查日期、包含的项目数、文件数、状态（待OCR / 已OCR / 已分析）
- 支持按日期范围筛选
- 支持点击查看详情、删除操作

---

### 3.2 模块二：系统设置

#### 3.2.1 AI 接口设置

| 配置项 | 说明 |
|--------|------|
| API URL | AI 服务接口地址 |
| API Key | 认证密钥 |
| 可用模型列表 | 支持添加多个模型名称，可选择默认模型 |
| SOCKS 代理开关 | 是否启用代理 |
| 代理地址 | 如 `socks5://127.0.0.1:1080` |
| 代理用户名 | 可选 |
| 代理密码 | 可选 |

- 所有配置存储在 `system_config` 表
- 提供"测试连接"按钮，验证配置有效性
- 配置变更实时生效

#### 3.2.2 检查项目管理

- 以列表/卡片形式展示所有检查项目
- 支持操作：
  - **新增项目**：输入名称、描述，自动创建对应文件夹
  - **编辑项目**：修改名称、描述
  - **启用/停用**：停用后在上传页面不再显示
  - **排序调整**：拖拽排序
  - **删除项目**：仅允许删除无关联数据的项目
- 每个项目下可管理检查指标（Indicators）：
  - 指标名称、单位、参考范围
  - 标记是否为核心指标（用于趋势图）

#### 3.2.3 OCR 设置

| 配置项 | 说明 |
|--------|------|
| OCR 引擎选择 | 使用 AI 视觉模型 / 专用OCR API |
| OCR Prompt 模板 | 可自定义发送给AI的OCR提示词 |
| AI 分析 Prompt 模板 | 可自定义 AI 分析时的提示词模板 |

---

### 3.3 模块三：趋势分析

#### 3.3.1 指标选择

- 左侧显示检查项目树形结构
- 展开项目可看到该项目下的所有指标
- 每个指标前有复选框，用户勾选要展示的指标
- 默认勾选所有标记为"核心指标"的项

#### 3.3.2 趋势图展示

- 右侧为图表展示区域
- 每个选中的指标生成一个独立的趋势图
- 图表类型：折线图（默认）/ 柱状图（可切换）
- X 轴：检查日期
- Y 轴：指标数值
- 图表功能要求：
  - 显示参考范围区间（用浅色阴影）
  - 异常值高亮（红色标注）
  - 鼠标悬停显示详细数值
  - 支持日期范围筛选
  - 支持数据导出（图片）
- 使用 ECharts 图表库

#### 3.3.3 数据概览

- 在图表上方显示最新一次检查的概要信息
- 对比前次检查，显示升降趋势箭头
- 异常指标突出显示

---

### 3.4 模块四：AI 问答

#### 3.4.1 核心布局
- 采用左右分栏布局（参考 `stitch/AIQA.html`）
  - 左侧（30%）：历史 AI 分析建议展示区
  - 右侧（70%）：AI 问答对话框

#### 3.4.2 左侧：历史分析建议
- **数据源**：读取 `ai_analyses` 表（关联 `checkup_records` 获取日期）
- **展示形式**：时间轴倒序排列，显示检查日期与概要
- **交互功能**：
  - **折叠/展开**：默认折叠，点击展开查看完整分析内容
  - **分页加载**：支持滚动加载更多历史记录
  - **内容编辑**：展开后可点击"编辑"按钮修改 AI 的建议内容并保存
  - **引用发送**：点击"复制到输入框"按钮，将该段分析内容自动填充到右侧对话框的输入框中，方便用户针对该报告提问

#### 3.4.3 右侧：AI 问答对话框
- **功能定位**：用户与 AI 的自由问答，或基于左侧分析结果的深入咨询
- **对话交互**：
  - 输入框支持手动输入或从左侧引用
  - 发送后 AI 流式返回答案
  - 支持"重新生成"（可选）
  - 聊天记录本地持久化存储（`chat_logs` 表）
- **辅助功能**：
  - **复制答案**：每条 AI 回复均有复制按钮，点击复制到剪贴板
  - **清空记录**：一键清空当前对话历史


---

### 3.5 模块五：数据脱敏

#### 3.5.1 核心需求
- 针对展示报告中的敏感信息（如姓名、ID），提供简单的图片编辑功能
- 独立的功能 Tab，方便用户对本地任意图片进行处理

#### 3.5.2 图片编辑器
- **基础能力**：
  - 加载本地图片
  - 缩放查看 (Zoom In/Out)
  - 撤销 (Undo) / 重置 (Reset)
- **裁剪 (Crop)**：
  - 自由矩形框选
  - 确认裁剪后保留选区
- **马赛克/模糊 (Mosaic/Blur)**：
  - 框选指定区域自动应用马赛克效果
  - 用于遮挡敏感文字

#### 3.5.3 保存与导出
- **覆盖保存**：直接修改原文件
- **另存为**：保存为新文件

---

### 3.6 模块六：手机扫码上传

#### 3.6.1 核心需求
- 在局域网环境下，用户无需安装 App，直接使用手机系统相机或微信扫码。
- 扫码后打开一个网页，直接选择手机相册中的体检报告图片进行上传。
- 上传成功后，桌面端实时接收并显示图片，无需手动刷新或同步。

#### 3.6.2 交互流程
1. **启动服务**：用户在桌面端点击"手机上传"按钮。
2. **展示二维码**：
   - 系统自动获取本机局域网 IP。
   - 启动内置 HTTP 服务器（随机端口）。
   - 生成包含上传地址（如 `http://192.168.1.5:34567/`）的二维码弹窗展示。
3. **手机操作**：
   - 手机扫码访问网页。
   - 网页提供简洁的"选择图片"（支持多选）和"确认上传"按钮。
   - 上传进度实时反馈。
4. **桌面端反馈**：
   - 桌面端收到文件后，自动将文件加入当前的待上传列表。
   - 弹窗提示"收到来自手机的X个文件"。

---


## 4. 文件存储结构

```
{程序所在目录}/
├── pictures/                      # 图片存储根目录
│   ├── 血常规/                    # 检查项目文件夹
│   │   ├── 2026-01-15/           # 按日期归类
│   │   │   ├── report_page1.jpg
│   │   │   └── report_page2.jpg
│   │   └── 2026-02-11/
│   │       └── blood_test.png
│   ├── 肝功能/
│   │   └── 2026-02-11/
│   │       └── liver_test.jpg
│   └── 尿肾功能/
│       └── ...
└── health_guard.db                # SQLite 数据库文件
```

---

## 5. 接口设计（Tauri Commands）

### 5.1 检查记录相关

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `create_checkup_record` | `{ checkup_date }` | `Record` | 创建检查记录 |
| `list_checkup_records` | `{ page, size, date_range? }` | `PagedList<Record>` | 分页查询记录 |
| `get_checkup_record` | `{ id }` | `RecordDetail` | 获取记录详情 |
| `delete_checkup_record` | `{ id }` | `bool` | 删除记录 |

### 5.2 文件上传相关

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `upload_checkup_files` | `{ record_id, files: [{project_id, file_paths}] }` | `UploadResult` | 批量上传文件 |
| `get_record_files` | `{ record_id }` | `Vec<FileInfo>` | 获取记录下所有文件 |
| `delete_checkup_file` | `{ file_id }` | `bool` | 删除单个文件 |
| `get_file_preview` | `{ file_id }` | `Base64Image` | 获取文件预览 |

### 5.3 OCR 相关

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `start_ocr` | `{ record_id }` | `TaskId` | 发起OCR识别 |
| `get_ocr_status` | `{ record_id }` | `OcrStatus` | 查询OCR状态 |
| `get_ocr_results` | `{ record_id }` | `Vec<OcrResult>` | 获取OCR结果 |

### 5.4 AI 分析相关

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `start_ai_analysis` | `{ record_id }` | `TaskId` | 发起AI分析 |
| `get_ai_analysis` | `{ record_id }` | `AiAnalysis` | 获取分析结果 |

### 5.5 趋势分析相关

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_indicator_trend` | `{ indicator_id, date_range? }` | `Vec<TrendPoint>` | 获取指标趋势数据 |
| `get_latest_comparison` | `{ project_id }` | `Comparison` | 获取最近两次对比 |

### 5.6 系统设置相关

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_config` | `{ key }` | `ConfigValue` | 获取配置 |
| `save_config` | `{ key, value }` | `bool` | 保存配置 |
| `test_ai_connection` | `{}` | `TestResult` | 测试AI连接 |
| `list_projects` | `{}` | `Vec<Project>` | 获取项目列表 |
| `create_project` | `{ name, description }` | `Project` | 创建项目 |
| `update_project` | `{ id, name?, description?, is_active? }` | `Project` | 更新项目 |
| `delete_project` | `{ id }` | `bool` | 删除项目 |
| `list_indicators` | `{ project_id }` | `Vec<Indicator>` | 获取指标列表 |
| `create_indicator` | `{ project_id, name, unit, reference_range, is_core }` | `Indicator` | 创建指标 |
| `update_indicator` | `{ id, ... }` | `Indicator` | 更新指标 |
| `delete_indicator` | `{ id }` | `bool` | 删除指标 |
| `get_ai_analyses_history` | `{ page, size }` | `PagedList<Analysis>` | 获取所有历史分析 |
| `update_ai_analysis` | `{ id, content }` | `bool` | 更新分析内容 |
| `chat_with_ai` | `{ message, history_context? }` | `Stream<String>` | 与AI对话 |
| `get_chat_history` | `{ page, size }` | `PagedList<ChatMessage>` | 获取对话历史 |
| `clear_chat_history` | `{}` | `bool` | 清空对话历史 |

---

## 6. 非功能需求

| 项目 | 要求 |
|------|------|
| 语言 | 界面所有文字使用中文 |
| 性能 | 文件上传支持超过 10 张图片并发传输；OCR 和 AI 分析异步执行不阻塞 UI |
| 安全 | API Key 在 Rust 端管理，不暴露给前端；数据本地存储 |
| 可用性 | 断网状态下可浏览历史数据和趋势图；仅 OCR 和 AI 分析需要网络 |
| 错误处理 | 所有网络请求需要超时控制和重试机制；错误信息以中文友好提示 |
| 数据备份 | 提供数据库导出/导入功能 |
