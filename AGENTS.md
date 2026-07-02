# SlayMuManager — Agent 速查

**项目**：Tauri 2（Vue 3 前端 + Rust 后端），包管理器 `pnpm`。

## 命令
- `pnpm tauri dev`
- `pnpm tauri build`

## 结构（仅定位关键路径）
- 前端：`src/`（哈希路由，`@/` 别名），后端：`src-tauri/`（命令在 `app/`，业务在 `services/`，存储在 `repositories/` 即 JSON 文件）。

## 必须注意的特性/限制
- **窗口**：无系统边框 → 自定义标题栏，拖拽区需加 `data-tauri-drag-region`。
- **CSP**：未启用（`null`），无需处理 CSP 限制。
- **后端 IO**：
  - `rfd`（文件对话框）**必须在主线程**调用（Tauri 约束）。
  - `reqwest` 使用**阻塞模式**（Nexus API 调用）。
  - `notify` 监控 `mods/` + `mods_disabled/` 目录，**500ms 防抖**，触发事件 `slaymgr:mods-changed`。
- **IPC**：所有跨边界类型使用 `camelCase` 序列化（`serde` 约定）。
- **发布**：推送 `v*` 标签 → 仅创建源码 Release；生产构建目标为 **NSIS**（简中/英文）。