import { defineStore } from "pinia"
import { ref, shallowRef } from "vue"
import { invoke } from "@tauri-apps/api/core"
import type { InstalledMod } from "../types"

export const useModCacheStore = defineStore("mod-cache", () => {
  const enabledMods = shallowRef<InstalledMod[]>([])
  const disabledMods = shallowRef<InstalledMod[]>([])
  const loading = ref(false)
  const lastFetched = ref<number | null>(null)

  async function fetchMods() {
    loading.value = true
    try {
      const [enabled, disabled] = await Promise.all([
        invoke<InstalledMod[]>("list_installed_mods"),
        invoke<InstalledMod[]>("list_disabled_mods"),
      ])
      enabledMods.value = enabled
      disabledMods.value = disabled
      lastFetched.value = Date.now()
    } catch {
      enabledMods.value = []
      disabledMods.value = []
    } finally {
      loading.value = false
    }
  }

  return { enabledMods, disabledMods, loading, lastFetched, fetchMods }
})
