import { defineStore } from "pinia"
import { useStorage } from "@/composables/useStorage"

export const useDiscoverPrefStore = defineStore("discover-pref", () => {
  const discoverColumns = useStorage<number>("discover-column-count", 3)
  const prefetchEnabled = useStorage<boolean>("discover-prefetch", true)
  const showTranslateQuotaTip = useStorage<boolean>("slaymgr:translate-quota-tip", true)

  function setDiscoverColumns(val: number) { discoverColumns.value = val }
  function setPrefetchEnabled(v: boolean) { prefetchEnabled.value = v }
  function setShowTranslateQuotaTip(val: boolean) { showTranslateQuotaTip.value = val }

  return {
    discoverColumns, prefetchEnabled, showTranslateQuotaTip,
    setDiscoverColumns, setPrefetchEnabled, setShowTranslateQuotaTip,
  }
})
