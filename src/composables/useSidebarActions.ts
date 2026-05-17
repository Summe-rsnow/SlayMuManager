import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useMessage } from "naive-ui"
import { useI18n } from "vue-i18n"
import { useRouter } from "vue-router"
import type { CloudSaveStatus, ModProfile } from "../types"

// --- 模块级共享状态（SideNav + LibraryPage 共用同一实例）---
export const launchingGame = ref(false)
export const showLaunchMismatchDialog = ref(false)
export const launchMismatchStatus = ref<CloudSaveStatus | null>(null)
export const quickPresetId = ref<string | null>(null)
export const quickPresetOptions = ref<Array<{ label: string; value: string }>>([])
export const activePresetName = ref("")
export const activePresetId = ref<string | null>(null)
export const presetSnapshot = ref<Set<string>>(new Set())
export const presetAppliedTick = ref(0)
export const sidebarCollapsed = ref(true)

export function useSidebarActions() {
  const { t } = useI18n()
  const message = useMessage()
  const router = useRouter()

  async function doLaunchGame() {
    launchingGame.value = true
    try {
      await invoke("launch_game")
      message.success(t("library.success.gameLaunched"))
    } catch (e: any) {
      message.error(t("library.error.launchFailed", { e }))
    } finally {
      launchingGame.value = false
    }
  }

  async function handleLaunchGame() {
    launchingGame.value = true
    try {
      const cloudStatus = await invoke<CloudSaveStatus>("get_cloud_save_status")
      if (cloudStatus.isAvailable && cloudStatus.hasMismatch) {
        launchMismatchStatus.value = cloudStatus
        showLaunchMismatchDialog.value = true
        return
      }
      await doLaunchGame()
    } catch {
      await doLaunchGame()
    } finally {
      if (!showLaunchMismatchDialog.value) {
        launchingGame.value = false
      }
    }
  }

  function handleGoToSaves() {
    showLaunchMismatchDialog.value = false
    launchingGame.value = false
    router.push("/saves")
  }

  async function handleLaunchAnyway() {
    showLaunchMismatchDialog.value = false
    await doLaunchGame()
  }

  async function loadQuickPresets() {
    try {
      const profiles = await invoke<ModProfile[]>("list_profiles")
      quickPresetOptions.value = profiles.map((p) => ({ label: p.name, value: p.id }))
    } catch { /* ignore */ }
  }

  async function handleQuickPreset(presetId: string) {
    if (!presetId) return
    try {
      const label =
        quickPresetOptions.value.find((p) => p.value === presetId)?.label ?? presetId
      await invoke("apply_profile", { id: presetId })
      quickPresetId.value = presetId
      activePresetName.value = label
      activePresetId.value = presetId
      // 快照预设声明的 mod ID（用于脏检测）
      try {
        const profiles = await invoke<ModProfile[]>("list_profiles")
        const profile = profiles.find((p) => p.id === presetId)
        if (profile) presetSnapshot.value = new Set(profile.modIds)
      } catch { /* ignore */ }
      message.success(t("library.success.presetApplied", { name: label }))
      presetAppliedTick.value++
    } catch (e: any) {
      message.error(`${t("profiles.error.applyFailed")}: ${e}`)
    }
  }

  return {
    launchingGame,
    showLaunchMismatchDialog,
    launchMismatchStatus,
    handleLaunchGame,
    handleGoToSaves,
    handleLaunchAnyway,
    quickPresetId,
    quickPresetOptions,
    loadQuickPresets,
    handleQuickPreset,
    activePresetName,
    activePresetId,
    presetSnapshot,
    presetAppliedTick,
    sidebarCollapsed,
  }
}
