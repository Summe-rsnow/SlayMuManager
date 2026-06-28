import { invoke } from "@tauri-apps/api/core"
import { currentLocale } from "@/i18n"
import { useStorage } from "./useStorage"

// 翻译缓存: `${targetLang}:${text}` -> translatedText
const translationCache = new Map<string, string>()

/** 翻译配额提示开关（localStorage 持久化） */
const QUOTA_TIP_KEY = "slaymgr:translate-quota-tip"
export const showTranslateQuotaTip = useStorage(QUOTA_TIP_KEY, true)

export function setShowTranslateQuotaTip(val: boolean) {
  showTranslateQuotaTip.value = val
}

/** 检测文本是否已包含 CJK 字符（中文/日文/韩文） */
function containsCJK(text: string): boolean {
  return /[\u4e00-\u9fff\u3400-\u4dbf\uf900-\ufaff]/.test(text)
}

/**
 * 通过 Rust 后端调用 MyMemory API 翻译文本
 * @returns 成功返回 { ok: true, text: string }，失败返回 { ok: false, error: string }
 */
export async function translateText(
  text: string | null | undefined
): Promise<{ ok: true; text: string } | { ok: false; error: string }> {
  if (!text) return { ok: false, error: "No text to translate" }

  const targetLang = currentLocale.value === "zh-CN" ? "zh-CN" : "en"

  // 如果文本已经包含目标语言字符，无需翻译
  if (targetLang === "zh-CN" && containsCJK(text)) {
    return { ok: true, text }
  }

  // 检查缓存
  const cacheKey = `${targetLang}:${text}`
  const cached = translationCache.get(cacheKey)
  if (cached !== undefined) return { ok: true, text: cached }

  try {
    const result = await invoke<string>("translate_text", { text })
    translationCache.set(cacheKey, result)
    return { ok: true, text: result }
  } catch (e: unknown) {
    console.error("[translate] invoke failed:", e)
    return { ok: false, error: String(e) }
  }
}
