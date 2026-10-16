import { defineStore } from "pinia"
import { useStorage } from "@/composables/useStorage"

export const useDiscoverPrefStore = defineStore("discover-pref", () => {
  const discoverColumns = useStorage<number>("discover-column-count", 3)
  const prefetchEnabled = useStorage<boolean>("discover-prefetch", true)
  const showTranslateQuotaTip = useStorage<boolean>("slaymgr:translate-quota-tip", true)
  const discoverViewMode = useStorage<"grid" | "split">("discover-view-mode", "grid")
  const discoverSourceTab = useStorage<"nexus" | "workshop">("discover-source-tab", "nexus")
  const parseWorkshopBBCode = useStorage<boolean>("discover-parse-bbcode", false)
  const discoverPageSize = useStorage<number>("discover-page-size", 12)

  function setDiscoverColumns(val: number) {
    discoverColumns.value = val
  }
  function setPrefetchEnabled(v: boolean) {
    prefetchEnabled.value = v
  }
  function setShowTranslateQuotaTip(val: boolean) {
    showTranslateQuotaTip.value = val
  }
  function setDiscoverViewMode(val: "grid" | "split") {
    discoverViewMode.value = val
  }
  function setDiscoverSourceTab(val: "nexus" | "workshop") {
    discoverSourceTab.value = val
  }
  function setParseWorkshopBBCode(val: boolean) {
    parseWorkshopBBCode.value = val
  }
  function setDiscoverPageSize(val: number) {
    discoverPageSize.value = val
  }
  return {
    discoverColumns,
    prefetchEnabled,
    showTranslateQuotaTip,
    discoverViewMode,
    discoverSourceTab,
    parseWorkshopBBCode,
    setDiscoverColumns,
    setPrefetchEnabled,
    setShowTranslateQuotaTip,
    setDiscoverViewMode,
    setDiscoverSourceTab,
    setParseWorkshopBBCode,
    discoverPageSize,
    setDiscoverPageSize,
  }
})
