import { ref, computed } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { version as APP_VERSION } from "../../package.json"
import type { AppBootstrap } from "../types"

export type UpdateStatus = "idle" | "checking" | "uptodate" | "available"

/**
 * 应用版本语义化比较，同 SettingsAbout.vue 中的逻辑
 */
function compareVersions(a: string, b: string): number {
  const pa = a.split(".").map(Number)
  const pb = b.split(".").map(Number)
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const na = pa[i] || 0
    const nb = pb[i] || 0
    if (na > nb) return 1
    if (na < nb) return -1
  }
  return 0
}

/** GitHub 仓库信息 */
const GITHUB_REPO = "Summe-rsnow/SlayMuManager"
const NETDISK_URL = "https://pan.quark.cn/s/3bd89f2513a8"

export function useAppUpdateCheck() {
  const status = ref<UpdateStatus>("idle")
  const latestVersion = ref("")
  const updateUrl = ref("")
  const showDialog = ref(false)

  const isUpdateAvailable = computed(() => status.value === "available")

  /** 获取用户是否开启了自动检查 */
  async function isAutoCheckEnabled(): Promise<boolean> {
    try {
      const bootstrap = await invoke<AppBootstrap>("get_app_bootstrap")
      return bootstrap.autoCheckUpdate
    } catch {
      return true // 默认开启
    }
  }

  /** 检查更新，返回新版本是否可用 */
  async function checkForUpdate(): Promise<boolean> {
    status.value = "checking"
    try {
      const res = await fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`)
      if (!res.ok) throw new Error(`GitHub API: ${res.status}`)
      const data = await res.json()
      const tag: string = data.tag_name
      latestVersion.value = tag.replace(/^v/, "")
      updateUrl.value = data.html_url
      const hasUpdate = compareVersions(latestVersion.value, APP_VERSION) > 0
      status.value = hasUpdate ? "available" : "uptodate"
      return hasUpdate
    } catch {
      status.value = "idle"
      return false
    }
  }

  /** 打开 URL（调用 Rust 端能力） */
  function openUrl(url: string) {
    invoke("open_url_in_browser", { url })
  }

  /** 打开 Github 下载 */
  function downloadFromGithub() {
    if (updateUrl.value) openUrl(updateUrl.value)
  }

  /** 打开网盘下载 */
  function downloadFromNetdisk() {
    openUrl(NETDISK_URL)
  }

  /** 关闭对话框 */
  function closeDialog() {
    showDialog.value = false
  }

  /** 不再提醒：关闭自动检查 */
  async function disableAutoCheck() {
    try {
      await invoke("update_auto_check_update", { enabled: false })
    } catch {
      // 静默失败
    }
    showDialog.value = false
  }

  /** 启动时自动检查：如果开启则检查，有更新则弹出对话框 */
  async function autoCheckOnStartup() {
    const enabled = await isAutoCheckEnabled()
    if (!enabled) return
    const hasUpdate = await checkForUpdate()
    if (hasUpdate) {
      showDialog.value = true
    }
  }

  return {
    status,
    latestVersion,
    updateUrl,
    showDialog,
    isUpdateAvailable,
    checkForUpdate,
    downloadFromGithub,
    downloadFromNetdisk,
    closeDialog,
    disableAutoCheck,
    autoCheckOnStartup,
  }
}
