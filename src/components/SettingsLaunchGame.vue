<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NCard, NSwitch, NRadioGroup, NRadio } from "naive-ui"
import type { AppBootstrap } from "../types"

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
  <NCard :title="t('settings.launchGame.title')" size="small">
    <div class="flex items-center justify-between">
      <span class="text-sm">{{ t("settings.launchGame.mode") }}</span>
      <NRadioGroup :value="launchMode" size="small" @update:value="handleLaunchModeChange">
        <NRadio value="steam">{{ t("settings.launchGame.steam") }}</NRadio>
        <NRadio value="direct">{{ t("settings.launchGame.direct") }}</NRadio>
      </NRadioGroup>
    </div>
    <div class="border-t border-c-default my-3"></div>
    <div class="flex items-center justify-between">
      <span class="text-sm">{{ t("settings.launchGame.checkCloudSave") }}</span>
      <NSwitch :value="launchCheckCloudSave" @update:value="handleLaunchCheckCloudSaveChange" />
    </div>
  </NCard>
</template>
