import { defineStore } from "pinia"
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { useMessage } from "naive-ui"
import { useI18n } from "vue-i18n"
import type { ModUpdateInfo, InstalledMod } from "../types"

interface ModUpdateCheckEvent {
  reqId: number; success: boolean; error: string | null
  results: ModUpdateInfo[]
  summary: { totalMods: number; updatedMods: number }
}

export const useUpdateStore = defineStore("updates", () => {
  const updateModsMap = ref<Map<string, ModUpdateInfo>>(new Map())
  const checkingUpdates = ref(false)
  let searchReqId = 0
  let listenerReady = false

  if (!listenerReady) {
    listenerReady = true
    listen<ModUpdateCheckEvent>("slaymgr:update-check-result", (event) => {
      const { t } = useI18n()
      const message = useMessage()
      const payload = event.payload
      if (payload.reqId !== searchReqId) return
      checkingUpdates.value = false
      if (payload.success) {
        const map = new Map<string, ModUpdateInfo>()
        for (const info of payload.results) map.set(info.modId, info)
        updateModsMap.value = map
        if (payload.summary.updatedMods === 0) message.success(t("library.updateCheck.allUpToDate"))
        else message.success(t("library.updateCheck.foundUpdates", { n: payload.summary.updatedMods }))
      } else {
        const err = payload.error ?? ""
        if (err.includes("API Key")) message.warning(t("library.updateCheck.noApiKey"))
        else if (err.includes("游戏目录")) message.warning(t("library.updateCheck.noGamePath"))
        else message.error(t("library.updateCheck.error", { e: err }))
      }
    }).catch(() => { /* ignore */ })
  }

  async function loadCachedUpdates() {
    try {
      const cached = await invoke<ModUpdateInfo[]>("get_cached_mod_updates")
      const map = new Map<string, ModUpdateInfo>()
      for (const info of cached) map.set(info.modId, info)
      updateModsMap.value = map
    } catch { /* ignore */ }
  }

  function checkUpdates() {
    const { t } = useI18n()
    const message = useMessage()
    checkingUpdates.value = true
    searchReqId++
    const currentReqId = searchReqId
    invoke("start_mod_update_check", { reqId: currentReqId })
      .catch((e: unknown) => {
        checkingUpdates.value = false
        message.error(t("library.updateCheck.error", { e: String(e) }))
      })
  }

  function hasUpdate(modId: string): boolean { return updateModsMap.value.get(modId)?.hasUpdate ?? false }
  function getUpdateInfo(modId: string): ModUpdateInfo | undefined { return updateModsMap.value.get(modId) }

  function openUpdateUrl(mod: InstalledMod) {
    const url = updateModsMap.value.get(mod.id)?.remoteMod?.detailUrl
    if (url) invoke("open_url_in_browser", { url }).catch(() => {})
  }

  return { updateModsMap, checkingUpdates, loadCachedUpdates, checkUpdates, hasUpdate, getUpdateInfo, openUpdateUrl }
})
