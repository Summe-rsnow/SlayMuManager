<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NSwitch, NInputNumber } from "naive-ui"
import type { AppBootstrap } from "../types"
import SettingsSection from "./SettingsSection.vue"
import SettingsRow from "./SettingsRow.vue"
import TipIcon from "./TipIcon.vue"

const { t } = useI18n()

const backupCount = ref(5)
const backupOnPathSwitch = ref(true)

onMounted(async () => {
  const b = await invoke<AppBootstrap>("get_app_bootstrap")
  backupCount.value = b.autoBackupKeepCount
  backupOnPathSwitch.value = b.backupOnPathSwitch
})

async function updateBackupCount(val: number | null) {
  if (val == null) return
  backupCount.value = val
  try { await invoke("update_auto_backup_keep_count", { count: val }) } catch { /* ignore */ }
}

async function toggleBackupOnPathSwitch(val: boolean) {
  backupOnPathSwitch.value = val
  try { await invoke("update_backup_on_path_switch", { enabled: val }) } catch { /* ignore */ }
}
</script>

<template>
  <SettingsSection :title="t('settings.backup.title')">
    <SettingsRow :label="t('settings.backup.autoKeepCount')">
      <NInputNumber :value="backupCount" :min="1" :max="20" size="small" style="width: 120px" @update:value="updateBackupCount" />
    </SettingsRow>
    <div class="border-t border-c-default my-3"></div>
    <SettingsRow>
      <template #label>
        <TipIcon :label="t('settings.backup.onPathSwitch')" :text="t('settings.backup.onPathSwitchDesc')" placement="right" :width="240" />
      </template>
      <NSwitch :value="backupOnPathSwitch" @update:value="toggleBackupOnPathSwitch" />
    </SettingsRow>
  </SettingsSection>
</template>
