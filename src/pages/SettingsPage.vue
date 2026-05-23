<script setup lang="ts">
import { watch, nextTick } from "vue"
import { useI18n } from "vue-i18n"
import { useSettingsHighlight } from "../composables/useSettingsHighlight"
import SettingsGamePath from "../components/SettingsGamePath.vue"
import SettingsLaunch from "../components/SettingsLaunch.vue"
import SettingsNexus from "../components/SettingsNexus.vue"
import SettingsProxy from "../components/SettingsProxy.vue"
import SettingsAppearance from "../components/SettingsAppearance.vue"
import SettingsDiscover from "../components/SettingsDiscover.vue"
import SettingsBackup from "../components/SettingsBackup.vue"
import SettingsAbout from "../components/SettingsAbout.vue"
import SettingsDefaultPage from "../components/SettingsDefaultPage.vue"

const { t } = useI18n()
const { highlightedSetting, clearHighlight } = useSettingsHighlight()

// 监听到高亮信号时滚动到目标元素，3 秒后自动清除
watch(highlightedSetting, (val) => {
  if (!val) return
  nextTick(() => {
    const el = document.getElementById(`setting-${val}`)
    if (el) {
      el.scrollIntoView({ behavior: "smooth", block: "center" })
    }
  })
  setTimeout(() => clearHighlight(), 3000)
})
</script>

<template>
  <div>
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-c-primary">{{ t("settings.title") }}</h1>
    </div>

    <div class="max-w-2xl mx-auto flex flex-col gap-4">
      <SettingsGamePath />
      <SettingsLaunch />
      <SettingsNexus />
      <SettingsProxy />
      <SettingsAppearance />
      <SettingsDiscover />
      <SettingsDefaultPage />
      <SettingsBackup />
      <SettingsAbout />
    </div>
  </div>
</template>

<style scoped>
.setting-highlight {
  animation: highlight-flash 2.5s ease-in-out;
}
@keyframes highlight-flash {
  0% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--primary-color) 50%, transparent); }
  50% { box-shadow: 0 0 0 4px color-mix(in srgb, var(--primary-color) 30%, transparent); }
  100% { box-shadow: 0 0 0 0 color-mix(in srgb, var(--primary-color) 0%, transparent); }
}
</style>
