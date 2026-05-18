import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useMessage } from "naive-ui"
import { useI18n } from "vue-i18n"
import { useRouter } from "vue-router"
import type { CloudSaveStatus, ModProfile, AppBootstrap, ApplyProfileResult } from "../types"

// --- 模块级共享状态（SideNav + LibraryPage 共用同一实例）---
const BUILTIN_VANILLA_ID = "__builtin__vanilla"
const launchingGame = ref(false)
const showLaunchMismatchDialog = ref(false)
const launchMismatchStatus = ref<CloudSaveStatus | null>(null)
const quickPresetId = ref<string | null>(null)
const quickPresetOptions = ref<Array<{ label: string; value: string }>>([])
const activePresetName = ref("")
const activePresetId = ref<string | null>(null)
const presetSnapshot = ref<Set<string>>(new Set())
const presetAppliedTick = ref(0)
const sidebarCollapsed = ref(true)

export function useSidebarActions() {
  const { t } = useI18n()
  const message = useMessage()
  const router = useRouter()

  async function doLaunchGame() {
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
    launchingGame.value = true
    try {
      // 从后端读取当前设置（避免模块级 ref 不同步）
      const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
      if (bootstrap.launchCheckCloudSave) {
        try {
          const cloudStatus = await invoke<CloudSaveStatus>("get_cloud_save_status")
          if (cloudStatus.isAvailable && cloudStatus.hasMismatch) {
            launchMismatchStatus.value = cloudStatus
            showLaunchMismatchDialog.value = true
            return
          }
        } catch {
          // 云存档检查失败，继续启动
        }
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
      quickPresetOptions.value = profiles.map((p) => ({
        label: p.id === BUILTIN_VANILLA_ID ? t("profiles.builtinVanilla") : p.name,
        value: p.id,
      }))
      // 默认选中第一个（始终是原版）
      if (!quickPresetId.value && profiles.length > 0) {
        quickPresetId.value = profiles[0].id
      }
    } catch { /* ignore */ }
  }

  async function handleQuickPreset(presetId: string) {
    if (!presetId) return
    try {
      const label =
        quickPresetOptions.value.find((p) => p.value === presetId)?.label ?? presetId
      const result = await invoke<ApplyProfileResult>("apply_profile", { id: presetId })
      quickPresetId.value = presetId
      activePresetName.value = label
      activePresetId.value = presetId
      // 快照预设声明的 mod ID（用于脏检测）—— 从 apply_profile 结果直接获取，避免重读 profiles.json
      presetSnapshot.value = new Set(result.profile.modIds)
      message.success(t("library.success.presetApplied", { name: label }))
      presetAppliedTick.value++
    } catch (e: unknown) {
      message.error(`${t("profiles.error.applyFailed")}: ${String(e)}`)
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
