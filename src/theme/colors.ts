// ============================================================
// 预设主题色板
// 每个色板定义完整的 primary 色链（50~900）
// 与 Naive UI + UnoCSS 对齐
// ============================================================

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

export type ThemeColorKey = "indigo" | "blue" | "green" | "purple" | "rose" | "orange" | "cyan"

export const colorPalettes: Record<ThemeColorKey, ColorPalette> = {
  indigo: {
    DEFAULT: "#6366f1",
    hover: "#818cf8",
    pressed: "#4f46e5",
    suppl: "#818cf8",
    50: "#eef2ff",
    100: "#e0e7ff",
    200: "#c7d2fe",
    300: "#a5b4fc",
    400: "#818cf8",
    500: "#6366f1",
    600: "#4f46e5",
    700: "#4338ca",
    800: "#3730a3",
    900: "#312e81",
  },
  blue: {
    DEFAULT: "#3b82f6",
    hover: "#60a5fa",
    pressed: "#2563eb",
    suppl: "#60a5fa",
    50: "#eff6ff",
    100: "#dbeafe",
    200: "#bfdbfe",
    300: "#93c5fd",
    400: "#60a5fa",
    500: "#3b82f6",
    600: "#2563eb",
    700: "#1d4ed8",
    800: "#1e40af",
    900: "#1e3a8a",
  },
  green: {
    DEFAULT: "#22c55e",
    hover: "#4ade80",
    pressed: "#16a34a",
    suppl: "#4ade80",
    50: "#f0fdf4",
    100: "#dcfce7",
    200: "#bbf7d0",
    300: "#86efac",
    400: "#4ade80",
    500: "#22c55e",
    600: "#16a34a",
    700: "#15803d",
    800: "#166534",
    900: "#14532d",
  },
  purple: {
    DEFAULT: "#a855f7",
    hover: "#c084fc",
    pressed: "#9333ea",
    suppl: "#c084fc",
    50: "#faf5ff",
    100: "#f3e8ff",
    200: "#e9d5ff",
    300: "#d8b4fe",
    400: "#c084fc",
    500: "#a855f7",
    600: "#9333ea",
    700: "#7e22ce",
    800: "#6b21a8",
    900: "#581c87",
  },
  rose: {
    DEFAULT: "#f43f5e",
    hover: "#fb7185",
    pressed: "#e11d48",
    suppl: "#fb7185",
    50: "#fff1f2",
    100: "#ffe4e6",
    200: "#fecdd3",
    300: "#fda4af",
    400: "#fb7185",
    500: "#f43f5e",
    600: "#e11d48",
    700: "#be123c",
    800: "#9f1239",
    900: "#881337",
  },
  orange: {
    DEFAULT: "#f97316",
    hover: "#fb923c",
    pressed: "#ea580c",
    suppl: "#fb923c",
    50: "#fff7ed",
    100: "#ffedd5",
    200: "#fed7aa",
    300: "#fdba74",
    400: "#fb923c",
    500: "#f97316",
    600: "#ea580c",
    700: "#c2410c",
    800: "#9a3412",
    900: "#7c2d12",
  },
  cyan: {
    DEFAULT: "#06b6d4",
    hover: "#22d3ee",
    pressed: "#0891b2",
    suppl: "#22d3ee",
    50: "#ecfeff",
    100: "#cffafe",
    200: "#a5f3fc",
    300: "#67e8f9",
    400: "#22d3ee",
    500: "#06b6d4",
    600: "#0891b2",
    700: "#0e7490",
    800: "#155e75",
    900: "#164e63",
  },
}

/** 所有主题色 key 的展示名称（由 i18n 提供） */
export const themeColorLabels: Record<ThemeColorKey, string> = {
  indigo: "靛蓝",
  blue: "蓝色",
  green: "绿色",
  purple: "紫色",
  rose: "玫红",
  orange: "橙色",
  cyan: "青色",
}
