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
    // 玻璃质感辅助
    "glass-panel": "bg-[var(--glass-bg)] backdrop-blur-[var(--glass-blur)] border-[var(--glass-border)] rounded-[var(--glass-radius)]",
    // 阴影快捷类
    "shadow-card": "shadow-[var(--shadow-card)]",
    "shadow-glow": "shadow-[var(--shadow-glow)]",
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
