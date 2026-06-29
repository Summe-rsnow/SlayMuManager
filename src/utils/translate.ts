import { invoke } from "@tauri-apps/api/core"
import { currentLocale } from "@/i18n"

const translationCache = new Map<string, string>()

function containsCJK(text: string): boolean {
  return /[\u4e00-\u9fff\u3400-\u4dbf\uf900-\ufaff]/.test(text)
}

export async function translateText(
  text: string | null | undefined
): Promise<{ ok: true; text: string } | { ok: false; error: string }> {
  if (!text) return { ok: false, error: "No text to translate" }
  const targetLang = currentLocale.value === "zh-CN" ? "zh-CN" : "en"
  if (targetLang === "zh-CN" && containsCJK(text)) return { ok: true, text }
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
