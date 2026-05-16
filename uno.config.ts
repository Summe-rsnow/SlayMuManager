import { defineConfig, presetUno, presetIcons, transformerDirectives } from "unocss"

export default defineConfig({
  presets: [
    presetUno(),
    presetIcons({
      scale: 1.2,
    }),
  ],
  transformers: [transformerDirectives()],
  shortcuts: {
    // 导航按钮 — Naive UI Menu 风格
    "nav-item":
      "flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-all duration-200 cursor-pointer select-none",
    "nav-item-active": "bg-primary/10 text-primary",
    "nav-item-inactive": "text-gray-500 hover:text-gray-700 hover:bg-gray-100",
    // 卡片
    "card-base": "bg-white rounded-xl border border-gray-100 shadow-sm",
  },
  theme: {
    colors: {
      // 与 Naive UI themeOverrides.common 对齐
      primary: {
        DEFAULT: "#6366f1", // primaryColor
        hover: "#818cf8", // primaryColorHover
        pressed: "#4f46e5", // primaryColorPressed
        suppl: "#818cf8", // primaryColorSuppl
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
      // Naive UI 语义色参考
      success: {
        DEFAULT: "#18a058",
      },
      warning: {
        DEFAULT: "#f0a020",
      },
      error: {
        DEFAULT: "#d03050",
      },
      info: {
        DEFAULT: "#2080f0",
      },
    },
  },
})
