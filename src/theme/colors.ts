// ============================================================
// 预设主题色板（由 @miuui/tokens OKLCH 引擎生成）
// ============================================================

import { generateScale } from "@miuui/tokens"

export interface ColorPalette {
  DEFAULT: string
  hover: string
  pressed: string
  suppl: string
  50: string
  100: string
  200: string
  300: string
  400: string
  500: string
  600: string
  700: string
  800: string
  900: string
}

export type ThemeColorKey =
  | "indigo"
  | "blue"
  | "green"
  | "purple"
  | "rose"
  | "orange"
  | "cyan"
  | "pink"
  | "yellow"

const SEED_COLORS: Record<ThemeColorKey, string> = {
  indigo: "#6366f1",
  blue: "#3b82f6",
  green: "#22c55e",
  purple: "#a855f7",
  rose: "#f43f5e",
  orange: "#f97316",
  cyan: "#06b6d4",
  pink: "#ec4899",
  yellow: "#eab308",
}

function buildPalette(seedHex: string): ColorPalette {
  const scale = generateScale(seedHex)
  return {
    DEFAULT: seedHex,
    hover: scale[400],
    pressed: scale[600],
    suppl: scale[400],
    50: scale[50],
    100: scale[100],
    200: scale[200],
    300: scale[300],
    400: scale[400],
    500: scale[500],
    600: scale[600],
    700: scale[700],
    800: scale[800],
    900: scale[900],
  }
}

export const colorPalettes = Object.fromEntries(
  (Object.keys(SEED_COLORS) as ThemeColorKey[]).map((key) => [key, buildPalette(SEED_COLORS[key])]),
) as Record<ThemeColorKey, ColorPalette>
/** 所有主题色 key 的展示名称 */
export const themeColorLabels: Record<ThemeColorKey, string> = {
  indigo: "靛蓝",
  blue: "蓝色",
  green: "绿色",
  purple: "紫色",
  rose: "玫红",
  orange: "橙色",
  cyan: "青色",
  pink: "粉色",
  yellow: "黄色",
}
