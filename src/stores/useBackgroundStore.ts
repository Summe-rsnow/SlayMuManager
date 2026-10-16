import { defineStore } from "pinia"
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { useStorage } from "@/composables/useStorage"
import {
  getWallpaperFromIDB,
  saveWallpaperToIDB,
  clearWallpaperFromIDB,
} from "@/utils/wallpaperStorage"

export const useBackgroundStore = defineStore("background", () => {
  const customBgUrl = ref<string>("")
  const originalBgUrl = ref<string>("")
  const customBgBlur = useStorage<number>("slaymgr:custom-bg-blur", 0)
  const customBgDim = useStorage<number>("slaymgr:custom-bg-dim", 40)
  const customBgFillColor = useStorage<string>("slaymgr:custom-bg-fill-color", "#0A0A12")
  const loading = ref(false)

  // 裁切弹窗控制
  const showCropModal = ref(false)
  const cropSourceUrl = ref<string>("")

  async function loadCustomBackground() {
    loading.value = true
    try {
      const cached = await getWallpaperFromIDB()
      if (cached.active) {
        customBgUrl.value = cached.active
        originalBgUrl.value = cached.original ?? cached.active
        return
      }
      const res = await invoke<string | null>("get_custom_background")
      if (res) {
        customBgUrl.value = res
        originalBgUrl.value = res
      } else {
        customBgUrl.value = ""
        originalBgUrl.value = ""
      }
    } catch {
      customBgUrl.value = ""
      originalBgUrl.value = ""
    } finally {
      loading.value = false
    }
  }

  function openCropWithImage(imgUrl: string) {
    cropSourceUrl.value = imgUrl
    showCropModal.value = true
  }

  function reopenCurrentCrop() {
    const target = originalBgUrl.value || customBgUrl.value
    if (target) {
      openCropWithImage(target)
    }
  }

  async function chooseCustomBackground() {
    loading.value = true
    try {
      const res = await invoke<string | null>("pick_custom_background")
      if (res) {
        openCropWithImage(res)
      }
    } catch {
      // 忽略用户取消或选择错误
    } finally {
      loading.value = false
    }
  }

  async function applyCroppedWallpaper(croppedUrl: string, originalUrl: string, fillColor: string) {
    loading.value = true
    try {
      customBgUrl.value = croppedUrl
      originalBgUrl.value = originalUrl
      customBgFillColor.value = fillColor
      await saveWallpaperToIDB(croppedUrl, originalUrl)
    } finally {
      loading.value = false
      showCropModal.value = false
    }
  }

  async function removeCustomBackground() {
    loading.value = true
    try {
      await clearWallpaperFromIDB()
      await invoke("clear_custom_background")
      customBgUrl.value = ""
      originalBgUrl.value = ""
    } catch {
      // 忽略
    } finally {
      loading.value = false
    }
  }

  return {
    customBgUrl,
    originalBgUrl,
    customBgBlur,
    customBgDim,
    customBgFillColor,
    loading,
    showCropModal,
    cropSourceUrl,
    loadCustomBackground,
    openCropWithImage,
    reopenCurrentCrop,
    chooseCustomBackground,
    applyCroppedWallpaper,
    removeCustomBackground,
  }
})
