import { defineStore } from "pinia"
import { ref } from "vue"

export const useHighlightStore = defineStore("highlight", () => {
  const highlightedSetting = ref<string | null>(null)

  function highlight(id: string) { highlightedSetting.value = id }
  function clearHighlight() { highlightedSetting.value = null }

  return { highlightedSetting, highlight, clearHighlight }
})
