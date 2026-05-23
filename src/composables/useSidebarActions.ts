import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useMessage, useDialog } from "naive-ui"
import { useI18n } from "vue-i18n"
import { useRouter } from "vue-router"
import type { CloudSaveStatus, ModProfile, AppBootstrap, ApplyProfileResult, InstalledMod } from "../types"
import { useSettingsHighlight } from "./useSettingsHighlight"

// --- 模块级共享状态（SideNav + LibraryPage 共用同一实例）---
const BUILTIN_VANILLA_ID = "__builtin__vanilla"
const launchingGame = ref(false)
const showLaunchMismatchDialog = ref(false)
const launchMismatchStatus = ref<CloudSaveStatus | null>(null)
const showVanillaLaunchDialog = ref(false)
const vanillaLaunchEnabledCount = ref(0)
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
  const dialog = useDialog()
  const router = useRouter()
  const { highlight } = useSettingsHighlight()

  /** 弹窗引导前往设置游戏路径 */
  function showGamePathPrompt() {
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

      // 0. 游戏路径未设置则弹窗引导
      if (!bootstrap.gameDirectory) {
        showGamePathPrompt()
        return
      }

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
      // 2. 原版预设下已启用模组提示
      if (bootstrap.installedCount > 0 && bootstrap.activeProfileId === BUILTIN_VANILLA_ID) {
        vanillaLaunchEnabledCount.value = bootstrap.installedCount
        showVanillaLaunchDialog.value = true
        return
      }
      // 3. 检测游戏是否已在运行
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
      if (!showLaunchMismatchDialog.value && !showVanillaLaunchDialog.value) {
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

  /** 原版预设启动冲突：禁用所有模组并启动 */
  async function handleVanillaLaunchDisable() {
    showVanillaLaunchDialog.value = false
    try {
      const allMods = await invoke<InstalledMod[]>("list_installed_mods")
      const enabled = allMods.filter(m => m.state === "enabled")
      for (const mod of enabled) {
        await invoke("disable_mod", { modId: mod.id })
      }
    } catch { /* best effort */ }
    await doLaunchGame()
  }

  /** 原版预设启动冲突：取消 */
  function handleVanillaLaunchCancel() {
    showVanillaLaunchDialog.value = false
    launchingGame.value = false
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
    showVanillaLaunchDialog,
    vanillaLaunchEnabledCount,
    handleLaunchGame,
    handleGoToSaves,
    handleLaunchAnyway,
    handleVanillaLaunchDisable,
    handleVanillaLaunchCancel,
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
