import { ref } from "vue"
import { useStorage } from "@/composables/useStorage"
import { version as APP_VERSION } from "@/../package.json"

const showReleaseNotes = ref(false)
const lastSeenVersion = useStorage<string>("slaymgr:last-seen-version", "")

export function useReleaseNotes() {
  function checkOnStartup() {
    const prev = lastSeenVersion.value
    if (prev && prev !== APP_VERSION) {
      // 从旧版本升级上来的，自动弹出更新日志
      showReleaseNotes.value = true
    }
    // 更新已阅版本号
    lastSeenVersion.value = APP_VERSION
  }

  function openReleaseNotes() {
    showReleaseNotes.value = true
  }

  function closeReleaseNotes() {
    showReleaseNotes.value = false
  }

  return {
    showReleaseNotes,
    checkOnStartup,
    openReleaseNotes,
    closeReleaseNotes,
  }
}
