import { defineStore } from "pinia"
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useStorage } from "@/composables/useStorage"

export const useBackgroundStore = defineStore("background", () => {
  const customBgUrl = ref<string>("")
  const customBgBlur = useStorage<number>("slaymgr:custom-bg-blur", 0)
  const customBgDim = useStorage<number>("slaymgr:custom-bg-dim", 40)
  const loading = ref(false)

  async function loadCustomBackground() {
    loading.value = true
    try {
      const res = await invoke<string | null>("get_custom_background")
      customBgUrl.value = res ?? ""
    } catch {
      customBgUrl.value = ""
    } finally {
      loading.value = false
    }
  }

  async function chooseCustomBackground() {
    loading.value = true
    try {
      const res = await invoke<string | null>("pick_custom_background")
      if (res) {
        customBgUrl.value = res
      }
    } catch {
      // 忽略用户取消或选择错误
    } finally {
      loading.value = false
    }
  }

  async function removeCustomBackground() {
    loading.value = true
    try {
      await invoke("clear_custom_background")
      customBgUrl.value = ""
    } catch {
      // 忽略
    } finally {
      loading.value = false
    }
  }

  return {
    customBgUrl,
    customBgBlur,
    customBgDim,
    loading,
    loadCustomBackground,
    chooseCustomBackground,
    removeCustomBackground,
  }
})
