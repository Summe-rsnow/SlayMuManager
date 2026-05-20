import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useMessage } from "naive-ui"
import { useI18n } from "vue-i18n"
import type { ModUpdateInfo, InstalledMod } from "../types"

// Module-level state — singleton cache shared across all components
const updateModsMap = ref<Map<string, ModUpdateInfo>>(new Map())
const checkingUpdates = ref(false)

export function useModUpdates() {
  const message = useMessage()
  const { t } = useI18n()

  /** 从后端读取缓存，不发起网络请求 */
  async function loadCachedUpdates() {
    try {
      const cached = await invoke<ModUpdateInfo[]>("get_cached_mod_updates")
      const map = new Map<string, ModUpdateInfo>()
      for (const info of cached) {
        map.set(info.modId, info)
      }
      updateModsMap.value = map
    } catch { /* ignore */ }
  }

  /** 联网检查更新，结果自动写入后端缓存 */
  async function checkUpdates() {
    checkingUpdates.value = true
    try {
      const results = await invoke<ModUpdateInfo[]>("check_mod_updates")
      const map = new Map<string, ModUpdateInfo>()
      for (const info of results) {
        map.set(info.modId, info)
      }
      updateModsMap.value = map
      const updates = results.filter(r => r.hasUpdate)
      if (updates.length === 0) {
        message.success(t("library.updateCheck.allUpToDate"))
      } else {
        message.success(t("library.updateCheck.foundUpdates", { n: updates.length }))
      }
    } catch (e: unknown) {
      const err = String(e)
      if (err.includes("API Key")) {
        message.warning(t("library.updateCheck.noApiKey"))
      } else if (err.includes("游戏目录")) {
        message.warning(t("library.updateCheck.noGamePath"))
      } else {
        message.error(t("library.updateCheck.error", { e: err }))
      }
    } finally {
      checkingUpdates.value = false
    }
  }

  function hasUpdate(modId: string): boolean {
    return updateModsMap.value.get(modId)?.hasUpdate ?? false
  }

  function getUpdateInfo(modId: string): ModUpdateInfo | undefined {
    return updateModsMap.value.get(modId)
  }

  function openUpdateUrl(mod: InstalledMod) {
    const info = updateModsMap.value.get(mod.id)
    const url = info?.remoteMod?.detailUrl
    if (url) {
      invoke("open_url_in_browser", { url }).catch(() => {})
    }
  }

  return {
    updateModsMap,
    checkingUpdates,
    loadCachedUpdates,
    checkUpdates,
    hasUpdate,
    getUpdateInfo,
    openUpdateUrl,
  }
}
