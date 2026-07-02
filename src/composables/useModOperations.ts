import { ref } from "vue"
import { storeToRefs } from "pinia"
import { invoke } from "@tauri-apps/api/core"
import { useMessage } from "naive-ui"
import { useI18n } from "vue-i18n"
import type { InstalledMod, ModToggleResult } from "../types"
import { useSidebarStore } from "@/stores/useSidebarStore"
import { useModCacheStore } from "@/stores/useModCacheStore"
import { useIsActive } from "./useIsActive"

/**
 * 模组操作：启用/禁用/卸载/打开文件夹/批量操作
 */
export function useModOperations() {
  const message = useMessage()
  const { t } = useI18n()
  const modCacheStore = useModCacheStore()
  const { enabledMods, disabledMods } = storeToRefs(modCacheStore)
  const { fetchMods } = modCacheStore
  const sidebarStore = useSidebarStore()
  const { activePresetId, presetSnapshot } = storeToRefs(sidebarStore)
  const { isActive } = useIsActive()

  const busyId = ref<string | null>(null)
  const batchBusy = ref(false)
  const showSaveGuardDialog = ref(false)
  const saveGuardInfo = ref<ModToggleResult | null>(null)

  /** 切换单个 Mod 的启用/禁用状态 */
  async function handleToggle(mod: InstalledMod) {
    if (busyId.value) return
    const isEnabling = mod.state === "disabled"
    busyId.value = mod.id
    try {
      const result = await invoke<ModToggleResult>(
        isEnabling ? "enable_mod" : "disable_mod",
        { modId: mod.id },
      )
      if (result.saveGuard.pathSwitched) {
        saveGuardInfo.value = result
        showSaveGuardDialog.value = true
      } else {
        message.success(
          t(isEnabling ? "library.success.enabled" : "library.success.disabled", { name: mod.name }),
        )
      }
      await fetchMods()
      // 同步激活预设快照
      if (activePresetId.value) {
        const next = new Set(presetSnapshot.value)
        if (isEnabling) next.add(mod.id)
        else next.delete(mod.id)
        presetSnapshot.value = next
      }
    } catch (e: unknown) {
      message.error(t("library.error.operationFailed", { e }))
    } finally {
      busyId.value = null
    }
  }

  /** 卸载 Mod */
  async function handleUninstall(mod: InstalledMod) {
    if (busyId.value) return
    busyId.value = mod.id
    try {
      await invoke("uninstall_mod", { modId: mod.id })
      message.success(t("library.success.uninstalled", { name: mod.name }))
      await fetchMods()
    } catch (e: unknown) {
      message.error(t("library.error.uninstallFailed", { e }))
    } finally {
      busyId.value = null
    }
  }

  /** 打开 Mod 文件夹 */
  async function handleOpenFolder(mod: InstalledMod) {
    try {
      await invoke("open_mod_folder", { modId: mod.id })
    } catch (e: unknown) {
      message.error(t("library.error.openFailed", { e }))
    }
  }

  /** 打开 Mods 目录 */
  async function handleOpenModsDir() {
    try {
      await invoke("open_mods_directory")
    } catch (e: unknown) {
      message.error(t("library.error.openFailed", { e }))
    }
  }

  /** 全部禁用 → 启用 */
  async function enableAllMods() {
    const targets = disabledMods.value
    if (targets.length === 0) {
      message.info(t("library.info.allAlreadyEnabled"))
      return
    }
    batchBusy.value = true
    let success = 0
    for (const mod of targets) {
      try {
        await invoke<ModToggleResult>("enable_mod", { modId: mod.id })
        success++
      } catch { /* skip failed */ }
      if (!isActive.value) break
    }
    batchBusy.value = false
    if (!isActive.value) return
    message.success(t("library.success.batchEnabled", { n: success }))
    await fetchMods()
  }

  /** 全部启用 → 禁用 */
  async function disableAllMods() {
    const targets = enabledMods.value.filter(m => m.source !== "workshop")
    if (targets.length === 0) {
      message.info(t("library.info.allAlreadyDisabled"))
      return
    }
    batchBusy.value = true
    let success = 0
    for (const mod of targets) {
      try {
        await invoke<ModToggleResult>("disable_mod", { modId: mod.id })
        success++
      } catch { /* skip failed */ }
      if (!isActive.value) break
    }
    batchBusy.value = false
    if (!isActive.value) return
    message.success(t("library.success.batchDisabled", { n: success }))
    await fetchMods()
  }

  /** 关闭 Save Guard 弹窗 */
  function dismissSaveGuard() {
    showSaveGuardDialog.value = false
    if (saveGuardInfo.value) {
      message.success(
        t("library.success.toggle", {
          action:
            saveGuardInfo.value.modItem.state === "enabled"
              ? t("common.enabled")
              : t("common.disabled"),
          name: saveGuardInfo.value.modItem.name,
        }),
      )
    }
  }

  return {
    busyId,
    batchBusy,
    showSaveGuardDialog,
    saveGuardInfo,
    handleToggle,
    handleUninstall,
    handleOpenFolder,
    handleOpenModsDir,
    enableAllMods,
    disableAllMods,
    dismissSaveGuard,
  }
}
