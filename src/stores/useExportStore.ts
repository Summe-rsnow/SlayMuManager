import { defineStore } from "pinia"
import { ref } from "vue"

export const useExportStore = defineStore("export", () => {
  const exportingId = ref<string | null>(null)
  return { exportingId }
})
