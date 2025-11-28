# DiskSight - 目录文件大小查看器

DiskSight 是一个使用 Rust 和 egui 开发的图形化目录文件大小查看工具，可以帮助用户直观地查看和分析磁盘空间使用情况。

![DiskSight](https://img.shields.io/badge/Rust-1.60%2B-blue)
![License](https://img.shields.io/badge/License-MIT-green)

## 功能特点

- 📂 实时查看目录内容及文件大小
- 📊 按文件大小排序显示
- 🎨 支持暗黑/浅色主题切换
- ⚡ 异步加载，界面响应流畅
- 📱 友好的用户界面，操作简单

## 界面预览

```
╭──────┬──────┬─────────┬────────────╮
│ 类型 ┆ 权限 ┆ 大小    ┆ 路径       │
╞══════╪══════╪═════════╪════════════╡
│   -  ┆  rwx ┆ 8.0B    ┆ .gitignore │
├╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┤
│   -  ┆  rwx ┆ 124.0B  ┆ Cargo.toml │
├╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┤
│   d  ┆  rwx ┆ 5.5KB   ┆ src        │
├╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┤
│   d  ┆  rwx ┆ 25.7KB  ┆ .git       │
├╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┤
│   -  ┆  rwx ┆ 102.2KB ┆ Cargo.lock │
├╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┤
│   d  ┆  rwx ┆ 9.3MB   ┆ fonts      │
├╌╌╌╌╌╌┼╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┤
│   d  ┆  rwx ┆ 1.4GB   ┆ target     │
╰──────┴──────┴─────────┴────────────╯
┌─────────────────────────────────┐
│ 总数量:      7 │ 总大小: 1.4GB
└─────────────────────────────────┘
```

## 安装方法

### 前提条件

- Rust 1.60+ 和 Cargo
- 支持的操作系统：Windows, macOS, Linux

### 从源码构建

1. 克隆仓库：
```bash
git clone https://github.com/Sunrisies/DiskSight.git
cd DiskSight
```

2. 构建并运行：
```bash
cargo run --release
```

3. 仅构建：
```bash
cargo build --release
```

构建完成后，可执行文件将位于 `target/release/disk-sight`（或 Windows 上的 `target/release/disk-sight.exe`）。

## 使用方法

1. 启动程序后，界面会显示当前工作目录的内容
2. 点击"浏览..."按钮可以选择其他目录
3. 点击"刷新"按钮可以手动刷新目录内容
4. 使用主题切换按钮在暗黑和浅色模式之间切换

## 项目结构

```
disksight/
├── src/
│   └── main.rs          # 主程序入口
├── fonts/
│   └── SimHei.ttf       # 中文字体文件（可选）
├── Cargo.toml           # 项目依赖配置
└── README.md            # 项目说明文档
```

## 依赖项

- `eframe` - egui 框架应用基础
- `egui` - 即时模式 GUI 库
- `egui_extras` - egui 额外组件（表格等）
- `rfd` - 原生文件对话框
- 其他标准库依赖

详细依赖见 `Cargo.toml` 文件。


### 添加中文字体支持

要正确显示中文，需要提供中文字体文件（如 SimHei.ttf）并放置在 `fonts/` 目录下。

## 开发说明

### 代码架构

程序采用典型的 Rust GUI 应用结构：
- 主结构体 `FileSizeViewer` 管理应用状态
- 实现了 `eframe::App` trait 处理界面更新
- 使用多线程异步加载目录内容
- 原子操作保证线程安全的状态更新

### 性能优化

- 使用多线程处理文件系统操作，避免阻塞 UI
- 按文件大小排序，突出显示大文件
- 可配置的刷新间隔，减少不必要的磁盘访问

## 常见问题

### 程序启动慢

首次运行或打开包含大量文件的目录时，程序可能需要一些时间来计算文件大小。这是正常现象。

### 中文显示问题

如果遇到中文显示为方块，请确保已提供正确的中文字体文件。

### 权限问题

在某些系统上，访问受保护目录可能需要管理员权限。

## 贡献指南

欢迎提交 Issue 和 Pull Request 来改进 DiskSight！

1. Fork 本项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开 Pull Request

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 致谢

- 感谢 [egui](https://github.com/emilk/egui) 团队提供的优秀 GUI 框架
- 感谢所有贡献者和用户

## 联系方式

如有问题或建议，请通过以下方式联系：
- 提交 [GitHub Issue](https://github.com/Sunrisies/DiskSight.git/issues)
- 发送邮件至: 3266420686@qq.com

---

⭐ 如果这个项目对你有帮助，请给它一个星标！
