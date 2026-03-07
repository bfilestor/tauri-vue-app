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

#### 3.2.1 AI 接口设置（多提供商 · Cherry Studio 风格）

> **v2.0 重构** — 参考 [Cherry Studio](https://github.com/CherryHQ/cherry-studio) 的提供商管理架构

##### 整体布局（三栏式）

设置页面由原来的「AI 设置 + Prompt 模板」合并在一个 Tab 内，改为分离为**两个独立 Tab**：

| Tab 标签 | 内容 |
|----------|------|
| **模型服务** | 多提供商管理：左侧提供商列表 + 中间详情配置 + 右侧模型列表 |
| **Prompt 设置** | OCR Prompt 模板 + AI 分析 Prompt 模板 |

「模型服务」Tab 采用**三栏布局**（参考 Cherry Studio 截图）：

```
┌───────────────┬──────────────────────────────────────┐
│  提供商列表     │  提供商详情配置                         │
│ ┌───────────┐ │  ┌──────────────────────────────────┐│
│ │ GPT-Load ☑│ │  │ 名称: GPT-Load    ⚙ 编辑  🗑 删除 ││
│ │ Gemini   ☑│ │  │ 启用: [ON/OFF]                    ││
│ │ Ollama   ☐│ │  │                                   ││
│ │ ...       │ │  │ API 密钥: ●●●●●●●  [检测]          ││
│ └───────────┘ │  │ API 地址: http://...               ││
│               │  │                                   ││
│  [搜索框]      │  │ ─────────────────────────────────  ││
│  [+ 添加]     │  │ 模型列表 (4个)        [+ 添加模型]   ││
│               │  │  ▼ gpt-4.1                         ││
│               │  │    · gpt-4.1-nano  [⚙] [−]         ││
│               │  │    · gpt-4.1-nano-2025  [⚙] [−]    ││
│               │  │  ▼ gpt-4o                          ││
│               │  │    · gpt-4o-mini  ★默认  [⚙] [−]   ││
│               │  └──────────────────────────────────┘│
└───────────────┴──────────────────────────────────────┘
```

##### 数据模型

**新增数据库表 `ai_providers`：**

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT (UUID) | 主键 |
| name | TEXT | 提供商名称（如「GPT-Load」、「Gemini 官方」） |
| type | TEXT | 提供商类型：`openai` / `gemini` / `anthropic` / `azure-openai` / `ollama` / `custom` |
| api_key | TEXT | API 密钥 |
| api_url | TEXT | API 接口基础地址 |
| enabled | INTEGER (0/1) | 是否启用 |
| sort_order | INTEGER | 排序顺序 |
| created_at | TEXT (ISO8601) | 创建时间 |
| updated_at | TEXT (ISO8601) | 更新时间 |

**新增数据库表 `ai_models`：**

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT (UUID) | 主键 |
| provider_id | TEXT (FK) | 所属提供商 ID |
| model_id | TEXT | 模型标识（调用 API 时使用，如 `gpt-4o-mini`） |
| model_name | TEXT | 模型显示名称（可选，如「GPT-4o 旗舰版」） |
| group_name | TEXT | 分组名称（可选，如 `gpt-4o`、`gpt-4.1`） |
| is_default | INTEGER (0/1) | 是否为全局默认模型 |
| enabled | INTEGER (0/1) | 是否启用 |
| sort_order | INTEGER | 排序顺序 |
| created_at | TEXT (ISO8601) | 创建时间 |

##### 提供商类型与 API 兼容

| 提供商类型 | API 协议 | 备注 |
|-----------|---------|------|
| `openai` | OpenAI Chat Completions | 默认类型，大部分第三方兼容 |
| `gemini` | Google Gemini API | 需特殊 URL 拼接 |
| `anthropic` | Anthropic Messages API | 使用 `x-api-key` 头 |
| `azure-openai` | Azure OpenAI Service | 部署名 + API 版本参数 |
| `ollama` | Ollama API (OpenAI 兼容) | 通常无需 API Key |
| `custom` | 自定义 OpenAI 兼容 | 用户自定义的兼容接口 |

##### 提供商管理操作

| 操作 | 说明 |
|------|------|
| 添加提供商 | 弹窗：填写名称 + 选择类型 |
| 编辑提供商 | 弹窗：修改名称 / 类型 |
| 删除提供商 | 确认后级联删除所有关联模型 |
| 启用/停用 | 列表中 Switch 切换 |
| 配置 API | 右侧面板：输入 API Key + API URL |
| 测试连接 | 使用该提供商配置发送测试请求 |

##### 模型管理操作

| 操作 | 说明 |
|------|------|
| 添加模型 | 弹窗：模型 ID（必填）+ 显示名称（可选）+ 分组名称（可选） |
| 编辑模型 | 弹窗：修改模型 ID / 名称 / 分组 |
| 删除模型 | 从列表移除 |
| 设为全局默认 | 标记为系统默认模型，同一时间仅一个 |

##### 全局默认模型

系统各处（OCR / AI 分析 / AI 问答）调用 AI 时，使用**全局默认模型**对应的 Provider 的 `api_url` + `api_key` + `model_id`。

##### 全局网络设置（保持不变）

| 配置项 | 说明 |
|--------|------|
| SOCKS 代理开关 | 是否启用代理 |
| 代理地址 | 如 `socks5://127.0.0.1:1080` |
| 代理用户名 | 可选 |
| 代理密码 | 可选 |
| 请求超时时间 | 默认 120 秒 |

- `ai_providers` 和 `ai_models` 使用独立数据库表管理
- 网络代理设置继续存储在 `system_config` 表
- 提供"测试连接"按钮，验证配置有效性
- 配置变更实时生效
- **向前兼容**：初次加载时如果检测到旧 `system_config` 中存在 `ai_api_url`/`ai_api_key`/`ai_models`，自动迁移为默认提供商

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

#### 3.2.4 增强 Prompt 管理（v3.0 新增）

> **版本**: v3.0 — 保护核心 Prompt 防误删; 新增患者情况说明模板

##### 需求背景

目前 OCR 识别 Prompt 和 AI 分析 Prompt 完全暴露给用户，容易被误删或误改，从而导致系统功能异常。同时，用户（医生/患者）在每次 AI 问诊时需要手动输入患者基本信息（年龄、性别等），体验繁琐。

##### 功能描述

**A. 开发者模式保护**

- 默认"正常模式"：
  - **隐藏** OCR 识别 Prompt 模板
  - **隐藏** AI 问答 Prompt 模板（即 `ai_analysis_prompt_template`）
  - **仅显示** 用户自定义 Prompt 模板（患者情况说明）
- 进入"开发者模式"：用户在「Prompt 模板设置」标题区域**连续点击 7 次**触发
  - **显示**三个 Prompt（OCR 识别 Prompt、AI 问答 Prompt、用户自定义 Prompt）
  - 三个 Prompt 均**可编辑**
  - 显示**关闭开发者模式**按钮，点击后恢复正常模式
  - 开发者模式状态**不持久化**（刷新后自动退出）

| 模式 | OCR Prompt | AI 问答 Prompt | 用户自定义 Prompt |
|------|-----------|--------------|-----------------|
| 正常模式 | 隐藏，不可编辑 | 隐藏，不可编辑 | 显示，可编辑 |
| 开发者模式 | 显示，可编辑 | 显示，可编辑 | 显示，可编辑 |

**B. 用户自定义 Prompt 模板（患者情况说明）**

- **标题**：患者情况说明
- **作用**：让用户预先填写患者的基本信息（年龄、身高、体重、性别、病史等）
- **配置键**：`user_custom_prompt_template`
- **默认值**：
  ```
  患者基本信息：
  - 姓名：（患者姓名）
  - 年龄：（岁）
  - 性别：（男/女）
  - 身高：（cm）
  - 体重：（kg）
  - 主要病史：（现有疾病或既往病史）
  - 当前用药：（正在服用的药物）
  - 其他说明：（其他需要医生了解的情况）
  ```
- **使用场景**：在 AI 问答时，将「用户自定义 Prompt + AI 问答 Prompt」拼接后作为系统 Prompt 发送给 AI

**C. AI 工作流程（调用逻辑）**

```
OCR 识别时：
  发送给 AI → [OCR 识别 Prompt 模板] + [图片]

AI 问答时（chat_with_ai）：
  系统 Prompt = [用户自定义 Prompt 模板] + "\n\n" + [AI 问答 Prompt 模板]
  用户消息 = 用户输入的问题
```

##### 数据存储

新增 `system_config` 配置键：

| 配置键 | 默认值说明 |
|--------|----------|
| `user_custom_prompt_template` | 包含患者基本信息字段的模板文本 |

##### 接口变更

`chat_with_ai` 命令需要从数据库读取：
1. `user_custom_prompt_template` — 用户自定义 Prompt
2. `ai_analysis_prompt_template` — AI 分析 Prompt 模板（兼容旧名，继续沿用）

将两者拼接为系统 Prompt 发送给 AI，替换原有的硬编码系统 Prompt。

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
  - 旋转 (RotateLeft / RotateRight)
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
   - **脱敏工作台特殊逻辑**：
     - 若在脱敏工作台发起上传，接收到第一张图片后，自动将其加载到画布中（替换当前图片）。
     - 支持手机端"拍照"或"从相册选择"单张图片进行编辑。


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
