import { defineStore } from "pinia"
import { ref, shallowRef, computed } from "vue"
import { invoke } from "@tauri-apps/api/core"
import type { InstalledMod } from "../types"

export interface ModConflictGroup {
  modId: string
  name: string
  instances: InstalledMod[]
}

export const useModCacheStore = defineStore("mod-cache", () => {
  const enabledMods = shallowRef<InstalledMod[]>([])
  const disabledMods = shallowRef<InstalledMod[]>([])
  const loading = ref(false)
  const lastFetched = ref<number | null>(null)

  const allMods = computed(() => [...enabledMods.value, ...disabledMods.value])

  // 检测多版本/多文件夹冲突组（同一个 mod.id 存在多个物理实例）
  const conflictGroups = computed<Map<string, ModConflictGroup>>(() => {
    const map = new Map<string, ModConflictGroup>()
    for (const mod of allMods.value) {
      const key = mod.id.toLowerCase()
      if (!map.has(key)) {
        map.set(key, { modId: mod.id, name: mod.name, instances: [] })
      }
      map.get(key)!.instances.push(mod)
    }

    const conflicts = new Map<string, ModConflictGroup>()
    for (const [key, group] of map.entries()) {
      if (group.instances.length > 1) {
        conflicts.set(key, group)
      }
    }
    return conflicts
  })

  const conflictCount = computed(() => conflictGroups.value.size)

  function hasConflict(modId: string): boolean {
    return conflictGroups.value.has(modId.toLowerCase())
  }

  function getConflictGroup(modId: string): ModConflictGroup | undefined {
    return conflictGroups.value.get(modId.toLowerCase())
  }

  async function fetchMods(silent = false) {
    const hasExistingData = enabledMods.value.length > 0 || disabledMods.value.length > 0
    if (!silent && !hasExistingData) {
      loading.value = true
    }
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
    conflictGroups,
    conflictCount,
    hasConflict,
    getConflictGroup,
    fetchMods,
  }
})
