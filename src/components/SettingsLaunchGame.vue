<script setup lang="ts">
import { computed, ref, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { FormRow, RadioGroup, Switch } from "@miuui/vue"
import type { AppBootstrap } from "../types"
import SettingsSection from "./SettingsSection.vue"

const { t } = useI18n()

const launchMode = ref("steam")
const launchCheckCloudSave = ref(true)
const launchModeOptions = computed(() => [
  { value: "steam", label: t("settings.launchGame.steam") },
  { value: "direct", label: t("settings.launchGame.direct") },
])
onMounted(async () => {
  const b = await invoke<AppBootstrap>("get_app_bootstrap")
  launchMode.value = b.launchMode
  launchCheckCloudSave.value = b.launchCheckCloudSave
})

async function handleLaunchModeChange(val: string | number) {
  const mode = String(val)
  launchMode.value = mode
  try {
    await invoke("update_launch_mode", { mode })
  } catch {
    /* ignore */
  }
}

async function handleLaunchCheckCloudSaveChange(val: boolean) {
  launchCheckCloudSave.value = val
  try {
    await invoke("update_launch_check_cloud_save", { check: val })
  } catch {
    /* ignore */
  }
}
</script>

<template>
  <SettingsSection :title="t('settings.launchGame.title')">
    <FormRow :label="t('settings.launchGame.mode')">
      <RadioGroup
        :model-value="launchMode"
        :options="launchModeOptions"
        orientation="horizontal"
        size="sm"
        @update:model-value="handleLaunchModeChange"
      />
    </FormRow>
    <FormRow :label="t('settings.launchGame.checkCloudSave')">
      <Switch
        :model-value="launchCheckCloudSave"
        size="sm"
        @update:model-value="handleLaunchCheckCloudSaveChange"
      />
    </FormRow>
  </SettingsSection>
</template>
