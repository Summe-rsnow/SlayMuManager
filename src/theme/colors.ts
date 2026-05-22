// ============================================================
// 预设主题色板（由 @poupe/material-color-utilities 动态生成）
// 每个种子色 → MCU 算法 → 完整 tonal palette（50~900）
//
//   50-900 阶调用固定 tone 映射（保证跨色板明度一致）：
//     50→95, 100→90, 200→80, 300→70, 400→60,
//     500→50, 600→40, 700→30, 800→20, 900→10
//
//   DEFAULT/hover/pressed 则用种子自身的 L* 确定基调，
//   确保「蓝色按钮显示蓝色」、「绿色按钮显示绿色」。
// ============================================================

import {
  argbFromHex,
  Hct,
  themeFromSourceColor,
} from "@poupe/material-color-utilities"

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

// --- 种子色（与旧版 DEFAULT 一致，确保平滑过渡） ---
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

// Tailwind 50-900 → MCU tone 映射
const TONE_MAP: Record<number, number> = {
  50: 95,
  100: 90,
  200: 80,
  300: 70,
  400: 60,
  500: 50,
  600: 40,
  700: 30,
  800: 20,
  900: 10,
}

const TAILWIND_KEYS = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900] as const

function argbToHex(argb: number): string {
  const r = (argb >> 16) & 0xff
  const g = (argb >> 8) & 0xff
  const b = argb & 0xff
  return (
    "#" +
    r.toString(16).padStart(2, "0") +
    g.toString(16).padStart(2, "0") +
    b.toString(16).padStart(2, "0")
  )
}

function generatePalette(seedHex: string): ColorPalette {
  const seedArgb = argbFromHex(seedHex)
  const seedTone = Math.round(Hct.fromInt(seedArgb).tone)

  const theme = themeFromSourceColor(seedArgb)
  const primary = theme.palettes.primary

  const palette = {} as ColorPalette

  for (const tw of TAILWIND_KEYS) {
    palette[tw] = argbToHex(primary.tone(TONE_MAP[tw]))
  }

  // DEFAULT = 种子色自身的 L*，保证颜色与标签一致
  palette.DEFAULT = argbToHex(primary.tone(seedTone))
  palette.hover = argbToHex(primary.tone(Math.min(100, seedTone + 10)))
  palette.pressed = argbToHex(primary.tone(Math.max(0, seedTone - 10)))
  palette.suppl = argbToHex(primary.tone(Math.min(100, seedTone + 10)))

  return palette
}

// --- 运行时生成 7 套色板（模块加载时计算一次） ---
export const colorPalettes: Record<ThemeColorKey, ColorPalette> =
  {} as Record<ThemeColorKey, ColorPalette>

const keys: ThemeColorKey[] = [
  "indigo",
  "blue",
  "green",
  "purple",
  "rose",
  "orange",
  "cyan",
  "pink",
  "yellow",
]

for (const key of keys) {
  colorPalettes[key] = generatePalette(SEED_COLORS[key])
}

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
