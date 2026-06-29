import { defineStore } from "pinia"
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useMessage, useDialog } from "naive-ui"
import { useI18n } from "vue-i18n"
import { useRouter } from "vue-router"
import type { CloudSaveStatus, ModProfile, AppBootstrap, ApplyProfileResult } from "../types"
import { useHighlightStore } from "./useHighlightStore"

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

  function showGamePathPrompt() {
    const { t } = useI18n()
    const dialog = useDialog()
    const router = useRouter()
    const { highlight } = useHighlightStore()
    launchingGame.value = false
    dialog.warning({
      title: t("settings.prompt.gamePathRequired"),
      content: t("settings.prompt.gamePathRequiredDesc"),
      positiveText: t("settings.prompt.goToSettings"),
      negativeText: t("common.cancel"),
      onPositiveClick: () => {
        highlight("game-path")
        router.push("/settings")
      },
      maskClosable: true,
    })
  }

  async function doLaunchGame() {
    const { t } = useI18n()
    const message = useMessage()
    launchingGame.value = true
    try {
      await invoke("launch_game")
      message.success(t("library.success.gameLaunched"))
    } catch (e: unknown) {
      message.error(t("library.error.launchFailed", { e }))
    } finally {
      launchingGame.value = false
    }
  }

  async function handleLaunchGame() {
    const { t } = useI18n()
    const message = useMessage()
    launchingGame.value = true
    try {
      const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
      if (!bootstrap.gameDirectory) { showGamePathPrompt(); return }

      if (bootstrap.launchCheckCloudSave) {
        try {
          const cloudStatus = await invoke<CloudSaveStatus>("get_cloud_save_status")
          if (cloudStatus.isAvailable && cloudStatus.hasMismatch) {
            launchMismatchStatus.value = cloudStatus
            showLaunchMismatchDialog.value = true
            return
          }
        } catch { /* ignore */ }
      }
      const running = await invoke<boolean>("is_game_running")
      if (running) {
        message.warning(t("library.info.gameAlreadyRunning"))
        launchingGame.value = false
        return
      }
      await doLaunchGame()
    } catch {
      await doLaunchGame()
    } finally {
      if (!showLaunchMismatchDialog.value) launchingGame.value = false
    }
  }

  function handleGoToSaves() {
    const router = useRouter()
    showLaunchMismatchDialog.value = false
    launchingGame.value = false
    router.push("/saves")
  }

  async function handleLaunchAnyway() {
    showLaunchMismatchDialog.value = false
    await doLaunchGame()
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

  async function handleQuickPreset(presetId: string) {
    const { t } = useI18n()
    const message = useMessage()
    if (!presetId) return
    try {
      const label = quickPresetOptions.value.find((p) => p.value === presetId)?.label ?? presetId
      const result = await invoke<ApplyProfileResult>("apply_profile", { id: presetId })
      quickPresetId.value = presetId
      activePresetName.value = label
      activePresetId.value = presetId
      presetSnapshot.value = new Set(result.profile.modIds)
      message.success(t("library.success.presetApplied", { name: label }))
      presetAppliedTick.value++
    } catch (e: unknown) {
      message.error(`${t("profiles.error.applyFailed")}: ${String(e)}`)
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
