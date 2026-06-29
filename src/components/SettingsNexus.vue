<script setup lang="ts">
import { ref, onMounted } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NInput, NButton, NIcon } from "naive-ui"
import { Key, ChevronDown } from "@lucide/vue"
import type { AppBootstrap } from "../types"
import { useHighlightStore } from "@/stores/useHighlightStore"
import SettingsSection from "./SettingsSection.vue"

const { t } = useI18n()
const { highlightedSetting } = storeToRefs(useHighlightStore())

const nexusKey = ref("")
const showNexusHelp = ref(false)

onMounted(async () => {
  const b = await invoke<AppBootstrap>("get_app_bootstrap")
  nexusKey.value = b.nexusApiKey ?? ""
})

function openUrl(url: string) {
  invoke("open_url_in_browser", { url })
}

async function saveNexusKey() {
  try {
    await invoke("update_nexus_api_key", { apiKey: nexusKey.value })
  } catch (e: unknown) {
    // 忽略
  }
}
</script>

<template>
  <SettingsSection id="setting-nexus" :title="t('settings.nexus.title')" :highlighted="highlightedSetting === 'nexus'">
    <div class="flex gap-2">
      <NInput v-model:value="nexusKey" :placeholder="t('settings.nexus.apiKeyPlaceholder')" type="password" show-password-on="click">
        <template #prefix><NIcon :size="16"><Key /></NIcon></template>
      </NInput>
      <NButton secondary @click="saveNexusKey">{{ t("common.save") }}</NButton>
    </div>
    <div class="flex items-center gap-2">
      <NButton text size="tiny" class="text-primary-theme" @click="showNexusHelp = !showNexusHelp">
        {{ t("settings.nexus.howToGet") }}
        <NIcon :size="12" class="ml-0.5 transition-transform" :class="showNexusHelp ? '' : '-rotate-90'"><ChevronDown /></NIcon>
      </NButton>
    </div>

    <div v-if="showNexusHelp" class="bg-primary-10-theme border border-primary-theme rounded-lg p-3 space-y-2 text-xs">
      <div class="flex items-start gap-2">
        <span class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold">1</span>
        <span>{{ t("settings.nexus.help.step1") }} — </span>
        <NButton text size="tiny" class="text-primary-600-theme! underline! p-0!" @click="openUrl('https://www.nexusmods.com/')">nexusmods.com</NButton>
      </div>
      <div class="flex items-start gap-2">
        <span class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold">2</span>
        <span>{{ t("settings.nexus.help.step2") }}</span>
      </div>
      <div class="flex items-start gap-2">
        <span class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold">3</span>
        <span>{{ t("settings.nexus.help.step3") }} — </span>
        <NButton text size="tiny" class="text-primary-600-theme! underline! p-0!" @click="openUrl('https://www.nexusmods.com/users/myaccount?tab=api')">{{ t("settings.nexus.help.step3Btn") }}</NButton>
      </div>
      <div class="flex items-start gap-2">
        <span class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold">4</span>
        <span>{{ t("settings.nexus.help.step4") }}</span>
      </div>
      <div class="flex items-start gap-2">
        <span class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold">5</span>
        <span>{{ t("settings.nexus.help.step5") }}</span>
      </div>
      <div class="flex items-start gap-2">
        <span class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold">6</span>
        <span>{{ t("settings.nexus.help.step6") }}</span>
      </div>
    </div>
  </SettingsSection>
</template>
