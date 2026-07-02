// ============================================================
// 主题系统 — 管理显示模式 + 主题色
// ============================================================

import { ref, watch, computed, type Ref } from "vue"
import { darkTheme, type GlobalTheme, type GlobalThemeOverrides } from "naive-ui"
import { colorPalettes, type ThemeColorKey } from "./colors"
import { useStorage } from "@/composables/useStorage"

// 重新导出供外部使用
export { colorPalettes } from "./colors"
export type { ThemeColorKey } from "./colors"

// --- 持久化 key ---
const STORAGE_MODE = "slaymgr:theme-mode"
const STORAGE_COLOR = "slaymgr:theme-color"

// --- 类型 ---
export type DisplayMode = "system" | "light" | "dark"

// --- 响应式状态（useStorage 自动同步 localStorage） ---
export const displayMode = useStorage<DisplayMode>(STORAGE_MODE, "system")
export const themeColorKey = useStorage<ThemeColorKey>(STORAGE_COLOR, "indigo")

// 当前是否为暗色模式（由 displayMode + prefers-color-scheme 决定）
const isDark = ref(false)

function evaluateIsDark(): boolean {
  if (displayMode.value === "dark") return true
  if (displayMode.value === "light") return false
  // system — 跟随系统
  return window.matchMedia("(prefers-color-scheme: dark)").matches
}

function applyHtmlClass(dark: boolean) {
  if (dark) {
    document.documentElement.classList.add("dark")
  } else {
    document.documentElement.classList.remove("dark")
  }
}

// 计算当前有效暗色状态
export const effectiveIsDark: Ref<boolean> = computed(() => isDark.value)

// Naive UI 主题
export const naiveTheme: Ref<GlobalTheme | null> = computed(() =>
  isDark.value ? darkTheme : null,
)

// 动态 themeOverrides — 随主题色 + 暗色模式变化
export const naiveThemeOverrides: Ref<GlobalThemeOverrides> = computed(() => {
  const p = colorPalettes[themeColorKey.value]
  return {
    common: {
      primaryColor: p.DEFAULT,
      primaryColorHover: p.hover,
      primaryColorPressed: p.pressed,
      primaryColorSuppl: p.suppl,
      borderRadius: "8px",
      fontSize: "14px",
      fontFamily: `"OPPO Sans", "PingFang SC", "Microsoft YaHei", sans-serif`,
    },
    Layout: {
      siderBorderColor: "transparent",
    },
    Menu: {
      itemTextColor: isDark.value ? "#d1d5db" : "#4b5563",
      itemTextColorHover: p.DEFAULT,
      itemTextColorActive: p.DEFAULT,
      itemColorActive: isDark.value ? `color-mix(in srgb, ${p.DEFAULT} 10%, transparent)` : "#eef2ff",
      itemColorActiveHover: isDark.value ? `color-mix(in srgb, ${p.DEFAULT} 15%, transparent)` : "#e0e7ff",
      itemIconColor: isDark.value ? "#9ca3af" : "#9ca3af",
      itemIconColorHover: p.DEFAULT,
      itemIconColorActive: p.DEFAULT,
      borderRadius: "8px",
    },
    Button: {
      borderRadiusSmall: "8px",
      borderRadiusMedium: "10px",
      borderRadiusLarge: "12px",
    },
    Card: {
      borderRadius: "16px",
      paddingMedium: "20px",
      titleFontSizeMedium: "16px",
    },
    Tag: {
      borderRadius: "6px",
      padding: "0 8px",
      fontSizeSmall: "11px",
      heightSmall: "22px",
    },
    Switch: {
      railColor: isDark.value ? "#2a2a35" : "#d1d5db",
      railColorActive: p.DEFAULT,
    },
    InternalSelection: {
      borderRadius: "10px",
      borderFocus: `1px solid ${p.DEFAULT}`,
      boxShadowFocus: `0 0 0 2px color-mix(in srgb, ${p.DEFAULT} 15%, transparent)`,
    },
    Input: {
      borderRadius: "10px",
      borderFocus: `1px solid ${p.DEFAULT}`,
      boxShadowFocus: `0 0 0 2px color-mix(in srgb, ${p.DEFAULT} 15%, transparent)`,
    },
    InternalSelectMenu: {
      borderRadius: "12px",
    },
    Popover: {
      borderRadius: "12px",
      padding: "8px 12px",
    },
    Tooltip: {
      borderRadius: "8px",
    },
    Tabs: {
      tabGap: "24px",
      tabFontSizeMedium: "14px",
      tabFontWeightActive: "600",
      barColor: p.DEFAULT,
    },
    Pagination: {
      itemBorderRadius: "9999px",
      itemColorActive: p.DEFAULT,
      itemTextColorActive: "#fff",
    },
    Checkbox: {
      borderRadius: "4px",
      boxShadowFocus: `0 0 0 2px color-mix(in srgb, ${p.DEFAULT} 15%, transparent)`,
    },
    Message: {
      borderRadius: "10px",
      padding: "10px 16px",
    },
    Notification: {
      borderRadius: "16px",
      padding: "16px 20px",
    },
    Modal: {
      borderRadius: "20px",
    },
  }
})

// --- 应用到 CSS 变量 ---
function applyCssVariables() {
  const p = colorPalettes[themeColorKey.value]
  const root = document.documentElement
  root.style.setProperty("--primary-color", p.DEFAULT)
  root.style.setProperty("--primary-color-hover", p.hover)
  root.style.setProperty("--primary-color-pressed", p.pressed)
  root.style.setProperty("--primary-color-suppl", p.suppl)
  root.style.setProperty("--primary-50", p[50])
  root.style.setProperty("--primary-100", p[100])
  root.style.setProperty("--primary-200", p[200])
  root.style.setProperty("--primary-300", p[300])
  root.style.setProperty("--primary-400", p[400])
  root.style.setProperty("--primary-500", p[500])
  root.style.setProperty("--primary-600", p[600])
  root.style.setProperty("--primary-700", p[700])
  root.style.setProperty("--primary-800", p[800])
  root.style.setProperty("--primary-900", p[900])
}

// --- 设置函数 ---
export function setDisplayMode(mode: DisplayMode) {
  displayMode.value = mode
  isDark.value = evaluateIsDark()
  applyHtmlClass(isDark.value)
}

export function setThemeColor(key: ThemeColorKey) {
  themeColorKey.value = key
  applyCssVariables()
}

// --- 监听系统主题变化 ---
let mediaQuery: MediaQueryList | null = null

function setupSystemListener() {
  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)")
  const handler = () => {
    if (displayMode.value === "system") {
      isDark.value = evaluateIsDark()
      applyHtmlClass(isDark.value)
    }
  }
  mediaQuery.addEventListener("change", handler)
}

// --- 初始化 ---
export function initTheme() {
  isDark.value = evaluateIsDark()
  applyHtmlClass(isDark.value)
  applyCssVariables()
  setupSystemListener()

  // 监听 displayMode / themeColorKey 变化
  watch(displayMode, () => {
    isDark.value = evaluateIsDark()
    applyHtmlClass(isDark.value)
  })
}
