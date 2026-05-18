import { ref } from "vue"
import { createI18n, type I18n } from "vue-i18n"
import zhCN from "./zh-CN.json"
import en from "./en.json"

// 共享的响应式 locale —— 驱动 vue-i18n + naive-ui + 所有组件
export const currentLocale = ref<string>("zh-CN")

let i18n: I18n | null = null

export function setupI18n() {
  i18n = createI18n({
    legacy: false,
    locale: currentLocale.value,
    fallbackLocale: "en",
    messages: {
      "zh-CN": zhCN,
      en,
    },
  })
  return i18n
}

/** 切换语言（同时更新 vue-i18n 全局 locale） */
export function setLocale(locale: string) {
  currentLocale.value = locale
  if (i18n) {
    ;(i18n.global.locale as { value: string }).value = locale
  }
}
