# DiskSight - 高效磁盘空间分析工具

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows-blue.svg)](https://tauri.app/)
[![Version](https://img.shields.io/badge/version-1.0.0-green.svg)](https://github.com/yourusername/disk-sight/releases)

DiskSight 是一款基于 Tauri + React + TypeScript 构建的高效磁盘空间分析工具，专为快速扫描和分析磁盘使用情况而设计。通过直观的界面和强大的后端性能，它可以帮助您轻松识别占用空间最大的文件和文件夹，从而有效管理存储资源。

## 主要特性

- 🚀 **高性能扫描**：利用 Rust 的并行处理能力，快速计算目录大小
- 📊 **直观展示**：以表格形式清晰展示文件大小、权限和创建时间
- 🎨 **现代化界面**：基于 React + Tailwind CSS 构建的美观用户界面
- 🌙 **深色模式**：支持明暗主题切换，适应不同使用环境
- ⚙️ **灵活配置**：多种显示选项和扫描设置，满足不同需求
- 📁 **实时进度**：显示扫描进度和当前处理的文件/目录
- 🗂️ **文件操作**：支持文件删除等操作功能
- 🔍 **搜索功能**：可按名称搜索特定目录

## 技术栈

### 前端
- **React 19** - 用户界面框架
- **TypeScript** - 类型安全的 JavaScript
- **Tailwind CSS** - 实用优先的 CSS 框架
- **Radix UI** - 无样式、可访问的 UI 组件
- **Lucide React** - 美观的图标库

### 后端
- **Rust** - 高性能系统编程语言
- **Tauri** - 构建跨平台桌面应用的框架
- **Rayon** - Rust 的数据并行库
- **Serde** - 序列化和反序列化框架

## 快速开始

### 环境要求

- Node.js 18+
- Rust 1.70+
- pnpm (推荐) 或 npm/yarn

### 安装步骤

1. 克隆仓库
   ```bash
   git clone https://github.com/yourusername/disk-sight.git
   cd disk-sight
   ```

2. 安装依赖
   ```bash
   # 使用 pnpm (推荐)
   pnpm install

   # 或使用 npm
   npm install
   ```

3. 开发模式运行
   ```bash
   pnpm tauri dev
   ```

4. 构建应用
   ```bash
   pnpm tauri build
   ```

## 使用指南

1. **选择目录**：点击"浏览"按钮或直接输入目录路径
2. **配置选项**：根据需要调整显示格式、显示内容和处理选项
3. **开始扫描**：点击"刷新"按钮开始分析所选目录
4. **查看结果**：扫描完成后，查看文件列表和大小统计
5. **文件操作**：使用操作列中的按钮执行文件操作

## 功能详解

### 显示选项

- **人性化大小**：以易读格式显示文件大小（如 1.2MB）
- **隐藏文件**：显示或隐藏以点开头的隐藏文件
- **时间信息**：显示文件的创建/修改时间
- **完整路径**：显示文件的完整路径而非仅文件名

### 处理选项

- **并行处理**：启用多线程处理以提高扫描速度
- **大小排序**：按文件大小降序排列结果
- **显示扫描详情**：显示实时扫描进度和当前处理项

## 贡献指南

我们欢迎社区贡献！如果您想为 DiskSight 做出贡献，请遵循以下步骤：

1. Fork 本仓库
2. 创建您的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交您的更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启一个 Pull Request

## 许可证

本项目采用 MIT 许可证。详情请参阅 [LICENSE](LICENSE) 文件。

## 致谢

- [Tauri](https://tauri.app/) - 提供强大的跨平台应用框架
- [React](https://reactjs.org/) - 构建用户界面的 JavaScript 库
- [Tailwind CSS](https://tailwindcss.com/) - 实用优先的 CSS 框架

## 联系我们

如果您有任何问题或建议，欢迎通过以下方式联系我们：

- 提交 [Issue](https://github.com/yourusername/disk-sight/issues)
- 发送邮件至 your.email@example.com

---

© 2025 DiskSight. All rights reserved.
