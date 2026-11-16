<script setup lang="ts">
import { ref, onMounted } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { Alert, Input, Button, Badge, useToast } from "@xcmm1543/miuui-vue"
import { Key, ChevronDown, RefreshCw, Crown, User } from "@lucide/vue"
import type { AppBootstrap, NexusUserInfo } from "../types"
import { useHighlightStore } from "@/stores/useHighlightStore"
import SettingsSection from "./SettingsSection.vue"

const { t } = useI18n()
const { highlightedSetting } = storeToRefs(useHighlightStore())
const { toast } = useToast()

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
      toast.success(t("settings.nexus.saveSuccessPremium", { user: info.name }))
    } else {
      toast.info(t("settings.nexus.saveSuccess", { user: info.name }))
    }
  } catch (e: unknown) {
    toast.error(String(e))
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
    toast.success(t("settings.nexus.saveSuccess", { user: info.name }))
  } catch (e: unknown) {
    toast.error(String(e))
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
      <Input
        v-model="nexusKey"
        :placeholder="t('settings.nexus.apiKeyPlaceholder')"
        type="password"
        password-toggle
        class="flex-1"
      >
        <template #prefix><Key :size="16" /></template>
      </Input>
      <Button variant="secondary" :loading="saving" @click="saveNexusKey">{{
        t("common.save")
      }}</Button>
      <Button v-if="nexusKey" variant="secondary" :loading="verifying" @click="verifyKey">
        <template #icon><RefreshCw :size="14" /></template>
        {{ t("settings.nexus.verify") }}
      </Button>
    </div>

    <!-- 账号与会员状态展示 -->
    <div v-if="userName" class="flex items-center gap-2 pt-1">
      <Badge v-if="isPremium" variant="warning" size="sm" class="font-medium">
        <Crown :size="13" />
        {{ t("settings.nexus.premium") }}: {{ userName }}
      </Badge>
      <Badge v-else variant="neutral" size="sm">
        <User :size="13" />
        {{ t("settings.nexus.nonPremium") }}: {{ userName }}
      </Badge>
      <span class="text-xs text-c-muted">
        {{ isPremium ? t("settings.nexus.premiumHint") : t("settings.nexus.nonPremiumHint") }}
      </span>
    </div>
    <div class="flex items-center gap-2">
      <Button
        variant="ghost"
        size="xs"
        class="text-primary-theme"
        @click="showNexusHelp = !showNexusHelp"
      >
        {{ t("settings.nexus.howToGet") }}
        <ChevronDown
          :size="12"
          class="ml-0.5 transition-transform"
          :class="showNexusHelp ? '' : '-rotate-90'"
        />
      </Button>
    </div>

    <Alert v-if="showNexusHelp" variant="info" material="glass" class="text-xs">
      <div class="space-y-2">
        <div class="flex items-start gap-2">
          <span
            class="flex-shrink-0 w-5 h-5 rounded-full bg-primary-theme text-white flex items-center justify-center text-[10px] font-bold"
            >1</span
          >
          <span>{{ t("settings.nexus.help.step1") }} — </span>
          <Button
            variant="ghost"
            size="xs"
            class="text-primary-600-theme! underline! p-0!"
            @click="openUrl('https://www.nexusmods.com/')"
            >nexusmods.com</Button
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
          <Button
            variant="ghost"
            size="xs"
            class="text-primary-600-theme! underline! p-0!"
            @click="openUrl('https://www.nexusmods.com/users/myaccount?tab=api')"
            >{{ t("settings.nexus.help.step3Btn") }}</Button
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
    </Alert>
  </SettingsSection>
</template>
