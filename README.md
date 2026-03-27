# Tauri + Vue 3 + Element Plus 版本

## 项目功能
当前项目是一个医疗健康资料ocr与ai智能分析系统，实现了对用户上传的图片通过AI进行ocr解析，获取json化结果，并结合历史分析数据，综合后发送给AI进行分析，寻求AI给出建议，同时对关注的报告数据按照时间段进行直线图展示，显示数据变化趋势。
- 支持一次上传多个图片，排队进行orc解析，获取结构化返回结果（json格式）
- 设置自己关注的数据字段
- 对关注的数据字段进行折线图展示，显示变化趋势
- 支持多个ai api接入
- 支持对用户上传图标进行模糊，旋转，剪切等操作
- 支持独立AI对话，引入历史分析内容并和AI进行对话
## 项目目录结构

- `src`: 前端源码 (Vue 3)
  - `components`: Vue 组件目录
  - `views`: 页面视图目录
  - `router`: 路由配置
  - `assets`: 静态资源 (图片, 样式等)
  - `App.vue`: 应用根组件
  - `main.js`: 前端入口文件
- `src-tauri`: 后端源码 (Rust)
  - `tauri.conf.json`: Tauri 核心配置文件
  - `Cargo.toml`: Rust 项目依赖配置
  - `capabilities`: Tauri 权限与 capabilities 配置
  - `icons`: 应用程序图标资源
  - `src`: Rust 源代码目录
- `public`: 公共静态资源目录
- `package.json`: Node.js 项目依赖与脚本配置
- `vite.config.js`: Vite 构建工具配置

## 开发与运行步骤

### 1. 环境准备

确保你的开发环境中已安装以下工具：
- **Node.js**: 建议使用 LTS 版本。
- **Rust**: 请按照 [Tauri 官方文档](https://tauri.app/v1/guides/getting-started/prerequisites) 安装 Rust 和相关构建工具。
- **包管理器**: npm, yarn 或 pnpm。

### 2. 安装依赖

进入项目根目录，安装前端依赖：

```bash
pnpm install
```

### 3. 开发运行

启动开发服务器，这将同时启动 Vue 前端服务和 Tauri 应用窗口：

```bash
pnpm tauri dev
```

### 4. 打包构建

构建生产环境应用程序。构建完成后，安装包将位于 `src-tauri/target/release/bundle` 目录下。

```bash
pnpm tauri build
```

### 5. 开发步骤

拿到用户需求后，仔细分析需求，并给出详细的实现方案，补充到plan/requirements.md中，
并采用 Epic → Feature → Story → Issue 层级划分，标注依赖关系与优先级，详细划分issue，
划分好依赖关系图和开发迭代计划，创建DDD测试用例和e2e测试用户，补充完善plan/todo-list.md 
接着开始补充 plan/progress_list.md，更新准备开发计划，
完成上述工作后，开始编码

首先阅读 [需求文档](plan/requirements.md)，了解需求，及 [架构设计](plan/architecture.md)，然后阅读 [开发 Issue 进度清单](plan/progress_list.md)，最后按照清单中的 Issue 进行开发。

开发完成后要更新，[text](plan/features-done.md)和plan/progress_list.md，记录开发进度

