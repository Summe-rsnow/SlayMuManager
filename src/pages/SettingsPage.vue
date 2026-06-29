<script setup lang="ts">
import { watch, nextTick } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import { useHighlightStore } from "@/stores/useHighlightStore"
import SettingsGamePath from "@/components/SettingsGamePath.vue"
import SettingsLaunch from "@/components/SettingsLaunch.vue"
import SettingsNexus from "@/components/SettingsNexus.vue"
import SettingsProxy from "@/components/SettingsProxy.vue"
import SettingsAppearance from "@/components/SettingsAppearance.vue"
import SettingsDiscover from "@/components/SettingsDiscover.vue"
import SettingsBackup from "@/components/SettingsBackup.vue"
import SettingsAbout from "@/components/SettingsAbout.vue"
import SettingsLaunchGame from "@/components/SettingsLaunchGame.vue"
import PageHeader from "@/components/PageHeader.vue"

const { t } = useI18n()
const highlightStore = useHighlightStore()
const { highlightedSetting } = storeToRefs(highlightStore)
const { clearHighlight } = highlightStore

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
    <PageHeader :title="t('settings.title')" :subtitle="t('settings.subtitle')" />

    <div class="max-w-2xl mx-auto flex flex-col gap-4">
      <SettingsGamePath />
      <SettingsNexus />
      <SettingsLaunch />
      <SettingsLaunchGame />
      <SettingsProxy />
      <SettingsAppearance />
      <SettingsDiscover />
      <SettingsBackup />
      <SettingsAbout />
    </div>
  </div>
</template>
