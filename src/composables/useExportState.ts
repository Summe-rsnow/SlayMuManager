import { ref } from "vue"

// --- 模块级共享状态（跨页面持久，导出时不因切换页面丢失 loading 状态）---
export const exportingId = ref<string | null>(null)

export function useExportState() {
  return {
    exportingId,
  }
}
