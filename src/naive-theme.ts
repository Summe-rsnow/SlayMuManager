import { type GlobalThemeOverrides } from "naive-ui"

// Naive UI 主题定制 — 与 UnoCSS primary 色板对齐
export const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: "#6366f1",
    primaryColorHover: "#818cf8",
    primaryColorPressed: "#4f46e5",
    primaryColorSuppl: "#818cf8",
    borderRadius: "8px",
    fontSize: "14px",
    fontFamily: `"OPPO Sans", "PingFang SC", "Microsoft YaHei", sans-serif`,
  },
  Layout: {
    siderBorderColor: "transparent",
  },
  Menu: {
    itemTextColor: "#4b5563",
    itemTextColorHover: "#6366f1",
    itemTextColorActive: "#6366f1",
    itemColorActive: "#eef2ff",
    itemColorActiveHover: "#e0e7ff",
    itemIconColor: "#9ca3af",
    itemIconColorHover: "#6366f1",
    itemIconColorActive: "#6366f1",
    borderRadius: "8px",
  },
  Button: {
    borderRadiusSmall: "6px",
    borderRadiusMedium: "8px",
    borderRadiusLarge: "10px",
  },
  Card: {
    borderRadius: "12px",
    paddingMedium: "20px",
    titleFontSizeMedium: "16px",
  },
  Tag: {
    borderRadius: "4px",
  },
  Input: {
    borderRadius: "8px",
  },
  Select: {
    peers: {
      InternalSelection: {
        borderRadius: "8px",
      },
    },
  },
  Modal: {
    borderRadius: "16px",
  },
}
