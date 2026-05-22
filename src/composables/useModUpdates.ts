import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { useMessage } from "naive-ui"
import { useI18n } from "vue-i18n"
import type { ModUpdateInfo, InstalledMod } from "../types"

// Module-level state — singleton cache shared across all components
const updateModsMap = ref<Map<string, ModUpdateInfo>>(new Map())
const checkingUpdates = ref(false)

// 请求计数器（模块级，匹配事件 reqId，忽略过期响应）
let searchReqId = 0
let listenerReady = false

interface ModUpdateCheckEvent {
  reqId: number
  success: boolean
  error: string | null
  results: ModUpdateInfo[]
  summary: {
    totalMods: number
    updatedMods: number
  }
}

export function useModUpdates() {
  const message = useMessage()
  const { t } = useI18n()

  // 设置事件监听（模块级单例，仅首次调用时注册）
  if (!listenerReady) {
    listenerReady = true
    listen<ModUpdateCheckEvent>("slaymgr:update-check-result", (event) => {
      const payload = event.payload
      if (payload.reqId !== searchReqId) return

      checkingUpdates.value = false

      if (payload.success) {
        const map = new Map<string, ModUpdateInfo>()
        for (const info of payload.results) {
          map.set(info.modId, info)
        }
        updateModsMap.value = map

        if (payload.summary.updatedMods === 0) {
          message.success(t("library.updateCheck.allUpToDate"))
        } else {
          message.success(t("library.updateCheck.foundUpdates", { n: payload.summary.updatedMods }))
        }
      } else {
        const err = payload.error ?? ""
        if (err.includes("API Key")) {
          message.warning(t("library.updateCheck.noApiKey"))
        } else if (err.includes("游戏目录")) {
          message.warning(t("library.updateCheck.noGamePath"))
        } else {
          message.error(t("library.updateCheck.error", { e: err }))
        }
      }
    }).catch(() => { /* listen 失败静默 */ })
  }

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

  /** 联网检查更新（不阻塞，结果由事件驱动） */
  function checkUpdates() {
    checkingUpdates.value = true
    searchReqId++
    const currentReqId = searchReqId

    invoke("start_mod_update_check", { reqId: currentReqId })
      .catch((e: unknown) => {
        checkingUpdates.value = false
        message.error(t("library.updateCheck.error", { e: String(e) }))
      })
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
