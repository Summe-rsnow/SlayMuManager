import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import type { InstalledMod } from "../types"

// Module-level state — singleton cache shared across all components
const enabledMods = ref<InstalledMod[]>([])
const disabledMods = ref<InstalledMod[]>([])
const loading = ref(false)
const lastFetched = ref<number | null>(null)

export function useModCache() {
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

  return {
    enabledMods,
    disabledMods,
    loading,
    lastFetched,
    fetchMods,
    /** Refetch after any mutation (enable/disable/uninstall/import) */
    invalidate: fetchMods,
  }
}
