[🇬🇧 English](./README.en.md) | [🇨🇳 中文](./README.md)

---

# SlayMuManager

> 杀戮尖塔 2 (Slay the Spire 2) 桌面端 Mod 管理器

基于 **Tauri 2.0 + Vue 3 + Rust** 构建的跨平台桌面应用，提供 Mod 的浏览、安装、管理、存档同步等功能。

## 功能

- **Mod 管理** — 启用/禁用/卸载 Mod，自定义标签与备注，快速筛选
- **预设系统** — 将 Mod 组合保存为预设，一键切换，便于管理不同配置
- **预设整合包** — 将预设及关联 Mod 导出为 `.7z` 整合包，方便分享与重装
- **发现页** — 对接 Nexus Mods API，搜索、浏览、安装社区 Mod（事件驱动后端搜索，不阻塞 UI）
- **Steam 创意工坊** — 搜索、订阅/取消订阅创意工坊 Mod（需 Steam 运行中）
- **存档管理** — 原版/Mod 版存档分离，配对同步，Steam 云存档检测与差异对比
- **存档备份** — 手动/自动备份存档，支持恢复到任意槽位，自动清理旧备份
- **导入 Mod** — 支持拖放 `.zip`/`.7z`/`.rar` 文件或文件夹导入，自动处理冲突
- **游戏启动** — 一键启动游戏（Steam/直连），支持 `--nomods` 纯净模式
- **Mod 更新检测** — 比对本地与在线文件哈希，识别可更新 Mod
- **Mod 简介翻译** — 通过后端翻译 API 将英文简介译为中文
- **文件系统监听** — 自动检测 `mods/` 目录的外部变更并同步
- **主题系统** — 浅色/深色/跟随系统，9 种主题色自由切换
- **应用自动更新** — 启动时检查新版本，支持 GitHub Release 与网盘下载
- **活动日志** — 记录 Mod/预设/存档操作历史
- **代理支持** — 为 Nexus Mods API 配置 HTTP/SOCKS 代理
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
| 状态管理 | Pinia (stores) + Composables |
| 路由 | Vue Router 5 |
| 国际化 | Vue I18n |
| 后端 | Rust (edition 2024, rust-version 1.94.1) |
| 包管理 | pnpm |

### 后端依赖

| 用途 | 库 |
|------|----|
| 序列化 | serde / serde_json |
| HTTP 客户端 | reqwest (blocking, json, socks) |
| 存档解压 | zip / sevenz-rust |
| 文件对话框 | rfd |
| Steam 集成 | steamworks |
| 文件监听 | notify |
| 注册表查询 | winreg |
| UUID | uuid (v4) |

## 开发

### 前置条件

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/) (edition 2024)

### 启动

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev

# 生产构建
pnpm tauri build
```

## 下载

GitHub Releases 或 [夸克网盘](https://pan.quark.cn/s/3bd89f2513a8)

## 致谢

- 灵感来源于 [SlaySP2Manger](https://github.com/wakaka6/SlaySP2Manger) — 感谢开源社区的启发

## 许可证

Apache License 2.0 - 详见 [LICENSE](./LICENSE)
