# Tauri + Vue 3 + Element Plus 版本

该源码可帮助你快速开始一个Tauri + Vue 3的项目，采用Vue 3 `<script setup>`风格。

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
npm install
# 或者
yarn install
pnpm install
```

### 3. 开发运行

启动开发服务器，这将同时启动 Vue 前端服务和 Tauri 应用窗口：

```bash
npm run tauri dev
# 或者
yarn tauri dev
pnpm tauri dev
```

### 4. 打包构建

构建生产环境应用程序。构建完成后，安装包将位于 `src-tauri/target/release/bundle` 目录下。

```bash
npm run tauri build
# 或者
yarn tauri build
pnpm tauri build
```
