<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NSwitch, NRadioGroup, NRadio } from "naive-ui"
import type { AppBootstrap } from "../types"
import SettingsSection from "./SettingsSection.vue"
import SettingsRow from "./SettingsRow.vue"

const { t } = useI18n()

const launchMode = ref("steam")
const launchCheckCloudSave = ref(true)

onMounted(async () => {
  const b = await invoke<AppBootstrap>("get_app_bootstrap")
  launchMode.value = b.launchMode
  launchCheckCloudSave.value = b.launchCheckCloudSave
})

async function handleLaunchModeChange(val: string) {
  launchMode.value = val
  try { await invoke("update_launch_mode", { mode: val }) } catch { /* ignore */ }
}

async function handleLaunchCheckCloudSaveChange(val: boolean) {
  launchCheckCloudSave.value = val
  try { await invoke("update_launch_check_cloud_save", { check: val }) } catch { /* ignore */ }
}
</script>

<template>
  <SettingsSection :title="t('settings.launchGame.title')">
    <SettingsRow :label="t('settings.launchGame.mode')">
      <NRadioGroup :value="launchMode" size="small" @update:value="handleLaunchModeChange">
        <NRadio value="steam">{{ t("settings.launchGame.steam") }}</NRadio>
        <NRadio value="direct">{{ t("settings.launchGame.direct") }}</NRadio>
      </NRadioGroup>
    </SettingsRow>
    <div class="border-t border-c-default my-3"></div>
    <SettingsRow :label="t('settings.launchGame.checkCloudSave')">
      <NSwitch :value="launchCheckCloudSave" @update:value="handleLaunchCheckCloudSaveChange" />
    </SettingsRow>
  </SettingsSection>
</template>
