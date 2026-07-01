import { defineStore } from "pinia"
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useRouter } from "vue-router"
import type { CloudSaveStatus, ModProfile, AppBootstrap, ApplyProfileResult } from "../types"

export const useSidebarStore = defineStore("sidebar", () => {
  const launchingGame = ref(false)
  const showLaunchMismatchDialog = ref(false)
  const launchMismatchStatus = ref<CloudSaveStatus | null>(null)
  const vanillaLaunch = ref(false)
  const quickPresetId = ref<string | null>(null)
  const quickPresetOptions = ref<Array<{ label: string; value: string }>>([])
  const activePresetName = ref("")
  const activePresetId = ref<string | null>(null)
  const presetSnapshot = ref<Set<string>>(new Set())
  const presetAppliedTick = ref(0)
  const sidebarCollapsed = ref(true)

  /** 尝试启动游戏。返回结构化结果，由组件层处理 UI 反馈 */
  async function handleLaunchGame(): Promise<{
    ok: boolean
    error?: string
    needsGamePath?: boolean
    alreadyRunning?: boolean
    mismatch?: CloudSaveStatus
  }> {
    launchingGame.value = true
    try {
      const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
      if (!bootstrap.gameDirectory) {
        return { ok: false, needsGamePath: true }
      }

      if (bootstrap.launchCheckCloudSave) {
        try {
          const cloudStatus = await invoke<CloudSaveStatus>("get_cloud_save_status")
          if (cloudStatus.isAvailable && cloudStatus.hasMismatch) {
            launchMismatchStatus.value = cloudStatus
            showLaunchMismatchDialog.value = true
            return { ok: false, mismatch: cloudStatus }
          }
        } catch { /* ignore */ }
      }

      const running = await invoke<boolean>("is_game_running")
      if (running) {
        return { ok: false, alreadyRunning: true }
      }

      return await doLaunchGame()
    } catch (e) {
      return { ok: false, error: String(e) }
    } finally {
      if (!showLaunchMismatchDialog.value) launchingGame.value = false
    }
  }

  async function doLaunchGame(): Promise<{ ok: boolean; error?: string }> {
    try {
      await invoke("launch_game")
      return { ok: true }
    } catch (e: unknown) {
      return { ok: false, error: String(e) }
    }
  }

  function handleGoToSaves() {
    const router = useRouter()
    showLaunchMismatchDialog.value = false
    launchingGame.value = false
    router.push("/saves")
  }

  async function handleLaunchAnyway(): Promise<{ ok: boolean; error?: string }> {
    showLaunchMismatchDialog.value = false
    return await doLaunchGame()
  }

  async function handleToggleVanillaLaunch(val: boolean) {
    vanillaLaunch.value = val
    try { await invoke("update_vanilla_launch", { enabled: val }) } catch { /* ignore */ }
  }

  async function loadQuickPresets() {
    try {
      const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
      vanillaLaunch.value = bootstrap.vanillaLaunch
      const profiles = await invoke<ModProfile[]>("list_profiles")
      quickPresetOptions.value = profiles.map((p) => ({ label: p.name, value: p.id }))
      if (!quickPresetId.value && profiles.length > 0) quickPresetId.value = profiles[0].id
    } catch { /* ignore */ }
  }

  /** 快速切换预设。返回结构化结果，由组件层处理 UI 反馈 */
  async function handleQuickPreset(presetId: string): Promise<{ ok: boolean; error?: string; label?: string }> {
    if (!presetId) return { ok: false, error: "No preset selected" }
    try {
      const label = quickPresetOptions.value.find((p) => p.value === presetId)?.label ?? presetId
      const result = await invoke<ApplyProfileResult>("apply_profile", { id: presetId })
      quickPresetId.value = presetId
      activePresetName.value = label
      activePresetId.value = presetId
      presetSnapshot.value = new Set(result.profile.modIds)
      presetAppliedTick.value++
      return { ok: true, label }
    } catch (e: unknown) {
      return { ok: false, error: String(e) }
    }
  }

  return {
    launchingGame, showLaunchMismatchDialog, launchMismatchStatus,
    vanillaLaunch, quickPresetId, quickPresetOptions,
    activePresetName, activePresetId, presetSnapshot, presetAppliedTick,
    sidebarCollapsed,
    handleLaunchGame, handleGoToSaves, handleLaunchAnyway,
    handleToggleVanillaLaunch, loadQuickPresets, handleQuickPreset,
  }
})
