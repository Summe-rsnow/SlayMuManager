<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { FormRow, NumberInput, Switch } from "@miuui/vue"
import type { AppBootstrap } from "../types"
import SettingsSection from "./SettingsSection.vue"
import FloatingTip from "./FloatingTip.vue"

const { t } = useI18n()

const backupCount = ref(5)
const backupOnPathSwitch = ref(true)

onMounted(async () => {
  const b = await invoke<AppBootstrap>("get_app_bootstrap")
  backupCount.value = b.autoBackupKeepCount
  backupOnPathSwitch.value = b.backupOnPathSwitch
})

async function updateBackupCount(val: number | undefined) {
  if (val == null) return
  backupCount.value = val
  try {
    await invoke("update_auto_backup_keep_count", { count: val })
  } catch {
    /* ignore */
  }
}

async function toggleBackupOnPathSwitch(val: boolean) {
  backupOnPathSwitch.value = val
  try {
    await invoke("update_backup_on_path_switch", { enabled: val })
  } catch {
    /* ignore */
  }
}
</script>

<template>
  <SettingsSection :title="t('settings.backup.title')">
    <FormRow :label="t('settings.backup.autoKeepCount')">
      <NumberInput
        :model-value="backupCount"
        :min="1"
        :max="20"
        size="sm"
        class="w-30"
        @update:model-value="updateBackupCount"
      />
    </FormRow>
    <FormRow>
      <template #lead>
        <FloatingTip
          :label="t('settings.backup.onPathSwitch')"
          :text="t('settings.backup.onPathSwitchDesc')"
          placement="right"
          :width="240"
        />
      </template>
      <Switch
        :model-value="backupOnPathSwitch"
        size="sm"
        @update:model-value="toggleBackupOnPathSwitch"
      />
    </FormRow>
  </SettingsSection>
</template>
