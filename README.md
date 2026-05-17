# SlayMuManager

> 杀戮尖塔 2 (Slay the Spire 2) 桌面端 Mod 管理器

基于 **Tauri 2.0 + Vue 3 + Rust** 构建的跨平台桌面应用，提供 Mod 的浏览、安装、管理、存档同步等功能。

## 功能

- **Mod 管理** — 启用/禁用/卸载 Mod，自定义标签与备注，快速筛选
- **预设系统** — 将 Mod 组合保存为预设，一键切换，便捷管理不同配置
- **发现页** — 对接 Nexus Mods API，搜索、浏览、安装社区 Mod
- **存档管理** — 原版/Mod 版存档分离，配对同步，Steam 云存档检测
- **导入 Mod** — 支持拖放 .zip/.7z 文件或文件夹导入，自动处理冲突
- **主题系统** — 浅色/深色/跟随系统，7 种主题色自由切换
- **多语言** — 简体中文 / English

## 截图

<!-- TODO: 待补充截图 -->

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.0 |
| 前端 | Vue 3 (Composition API) + Vite 8 |
| 样式 | UnoCSS + Naive UI |
| 图标 | Lucide Vue Next |
| 状态管理 | Pinia |
| 国际化 | Vue I18n |
| 后端 | Rust (edition 2021) |
| 包管理 | pnpm |

## 开发

### 前置条件

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/) (edition 2021)

### 启动

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev

# 生产构建
pnpm tauri build
```

## 致谢

- 灵感来源于 [SlaySP2Manger](https://github.com/wakaka6/SlaySP2Manger) — 感谢开源社区的启发

## 版本

当前版本: **1.3.1**

## 许可证

Apache License 2.0 - 详见 [LICENSE](./LICENSE)
