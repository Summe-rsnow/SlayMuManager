<script setup lang="ts">
import { ref, onMounted } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NInput, NButton, NIcon, NTag, useMessage } from "naive-ui"
import { Key, ChevronDown, RefreshCw, Crown, User } from "@lucide/vue"
import type { AppBootstrap, NexusUserInfo } from "../types"
import { useHighlightStore } from "@/stores/useHighlightStore"
import SettingsSection from "./SettingsSection.vue"

const { t } = useI18n()
const { highlightedSetting } = storeToRefs(useHighlightStore())
const message = useMessage()

const nexusKey = ref("")
const userName = ref<string | null>(null)
const isPremium = ref(false)
const saving = ref(false)
const verifying = ref(false)
const showNexusHelp = ref(false)

onMounted(async () => {
  const b = await invoke<AppBootstrap>("get_app_bootstrap")
  nexusKey.value = b.nexusApiKey ?? ""
  userName.value = b.nexusUserName ?? null
  isPremium.value = b.nexusIsPremium ?? false
})

function openUrl(url: string) {
  invoke("open_url_in_browser", { url })
}

async function saveNexusKey() {
  saving.value = true
  try {
    const info = await invoke<NexusUserInfo>("update_nexus_api_key", { apiKey: nexusKey.value })
    userName.value = info.name
    isPremium.value = info.is_premium
    if (info.is_premium) {
      message.success(t("settings.nexus.saveSuccessPremium", { user: info.name }))
    } else {
      message.info(t("settings.nexus.saveSuccess", { user: info.name }))
    }
  } catch (e: unknown) {
    message.error(String(e))
  } finally {
    saving.value = false
  }
}

async function verifyKey() {
  verifying.value = true
  try {
    const info = await invoke<NexusUserInfo>("validate_nexus_api_key")
    userName.value = info.name
    isPremium.value = info.is_premium
    message.success(t("settings.nexus.saveSuccess", { user: info.name }))
  } catch (e: unknown) {
    message.error(String(e))
  } finally {
    verifying.value = false
  }
}
</script>

<template>
  <SettingsSection
    id="setting-nexus"
    :title="t('settings.nexus.title')"
    :highlighted="highlightedSetting === 'nexus'"
  >
    <div class="flex gap-2">
      <NInput
        v-model:value="nexusKey"
        :placeholder="t('settings.nexus.apiKeyPlaceholder')"
        type="password"
        show-password-on="click"
      >
        <template #prefix
          ><NIcon :size="16"><Key /></NIcon
        ></template>
      </NInput>
      <NButton secondary :loading="saving" @click="saveNexusKey">{{ t("common.save") }}</NButton>
      <NButton v-if="nexusKey" secondary :loading="verifying" @click="verifyKey">
        <template #icon
          ><NIcon :size="14"><RefreshCw /></NIcon
        ></template>
        {{ t("settings.nexus.verify") }}
      </NButton>
    </div>

    <!-- 账号与会员状态展示 -->
    <div v-if="userName" class="flex items-center gap-2 pt-1">
      <NTag v-if="isPremium" type="warning" size="small" :bordered="false" class="font-medium">
        <template #icon
          ><NIcon :size="13"><Crown /></NIcon
        ></template>
        {{ t("settings.nexus.premium") }}: {{ userName }}
      </NTag>
      <NTag v-else type="default" size="small" :bordered="false">
        <template #icon
          ><NIcon :size="13"><User /></NIcon
        ></template>
        {{ t("settings.nexus.nonPremium") }}: {{ userName }}
      </NTag>
      <span class="text-xs text-c-muted">
        {{ isPremium ? t("settings.nexus.premiumHint") : t("settings.nexus.nonPremiumHint") }}
      </span>
    </div>
    <div class="flex items-center gap-2">
      <NButton text size="tiny" class="text-primary-theme" @click="showNexusHelp = !showNexusHelp">
        {{ t("settings.nexus.howToGet") }}
        <NIcon
          :size="12"
          class="ml-0.5 transition-transform"
          :class="showNexusHelp ? '' : '-rotate-90'"
          ><ChevronDown
        /></NIcon>
      </NButton>
    </div>

    <div
      v-if="showNexusHelp"
      class="bg-primary-10-theme border border-primary-theme rounded-lg p-3 space-y-2 text-xs"
    >
      <div class="flex items-start gap-2">
        <span
          class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold"
          >1</span
        >
        <span>{{ t("settings.nexus.help.step1") }} — </span>
        <NButton
          text
          size="tiny"
          class="text-primary-600-theme! underline! p-0!"
          @click="openUrl('https://www.nexusmods.com/')"
          >nexusmods.com</NButton
        >
      </div>
      <div class="flex items-start gap-2">
        <span
          class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold"
          >2</span
        >
        <span>{{ t("settings.nexus.help.step2") }}</span>
      </div>
      <div class="flex items-start gap-2">
        <span
          class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold"
          >3</span
        >
        <span>{{ t("settings.nexus.help.step3") }} — </span>
        <NButton
          text
          size="tiny"
          class="text-primary-600-theme! underline! p-0!"
          @click="openUrl('https://www.nexusmods.com/users/myaccount?tab=api')"
          >{{ t("settings.nexus.help.step3Btn") }}</NButton
        >
      </div>
      <div class="flex items-start gap-2">
        <span
          class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold"
          >4</span
        >
        <span>{{ t("settings.nexus.help.step4") }}</span>
      </div>
      <div class="flex items-start gap-2">
        <span
          class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold"
          >5</span
        >
        <span>{{ t("settings.nexus.help.step5") }}</span>
      </div>
      <div class="flex items-start gap-2">
        <span
          class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold"
          >6</span
        >
        <span>{{ t("settings.nexus.help.step6") }}</span>
      </div>
    </div>
  </SettingsSection>
</template>
