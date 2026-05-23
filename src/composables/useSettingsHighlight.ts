import { ref } from "vue"

/** 模块级共享状态 — 哪个设置项需要高亮 */
const highlightedSetting = ref<string | null>(null)

/**
 * 跨组件共享的设置高亮状态
 *
 * - 在操作入口（DiscoverPage / LibraryPage / SideNav）调用 `highlight(id)`
 *   然后 `router.push("/settings")`
 * - SettingsPage 监听到变化后自动滚动到目标元素
 * - 目标元素（SettingsNexus / SettingsGamePath）自动应用高亮动画
 * - 3 秒后自动清除高亮
 */
export function useSettingsHighlight() {
  function highlight(id: string) {
    highlightedSetting.value = id
  }

  function clearHighlight() {
    highlightedSetting.value = null
  }

  return { highlightedSetting, highlight, clearHighlight }
}
