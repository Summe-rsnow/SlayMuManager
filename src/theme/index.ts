// ============================================================
// 主题系统 — 管理显示模式 + 主题色
// ============================================================

import { ref, watch, computed, type Ref } from "vue"
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
  document.documentElement.dataset.theme = dark ? "dark" : "light"
  if (dark) {
    document.documentElement.classList.add("dark")
  } else {
    document.documentElement.classList.remove("dark")
  }
}

// 计算当前有效暗色状态
export const effectiveIsDark: Ref<boolean> = computed(() => isDark.value)

export const activeSeedColor = computed(() => colorPalettes[themeColorKey.value].DEFAULT)

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
