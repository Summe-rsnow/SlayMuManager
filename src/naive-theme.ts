/**
 * Naive UI 主题覆盖 — 已迁移到 src/theme/index.ts
 *
 * 此文件仅保留为兜底导入兼容，实际逻辑由 theme composable 接管。
 * 如无组件直接 import 此文件，可安全删除。
 */

import { naiveThemeOverrides } from "./theme"

/** @deprecated 使用 theme/index.ts 中的 naiveThemeOverrides */
export const themeOverrides = naiveThemeOverrides
