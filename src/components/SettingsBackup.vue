<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NCard, NSwitch, NInputNumber, NPopover, NIcon } from "naive-ui"
import { HelpCircle } from "@lucide/vue"
import type { AppBootstrap } from "../types"

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
  <NCard :title="t('settings.backup.title')" size="small">
    <div class="flex items-center justify-between">
      <span class="text-sm">{{ t("settings.backup.autoKeepCount") }}</span>
      <NInputNumber :value="backupCount" :min="1" :max="20" size="small" style="width: 120px" @update:value="updateBackupCount" />
    </div>
    <div class="border-t border-c-default my-3"></div>
    <div class="flex items-center justify-between">
      <NPopover trigger="hover" placement="right" :width="240">
        <template #trigger>
          <span class="flex items-center gap-1 cursor-help text-sm text-c-primary" :style="{ lineHeight: '1' }">
            <span>{{ t("settings.backup.onPathSwitch") }}</span>
            <NIcon :size="14" class="text-c-muted"><HelpCircle /></NIcon>
          </span>
        </template>
        <span class="text-xs">{{ t("settings.backup.onPathSwitchDesc") }}</span>
      </NPopover>
      <NSwitch :value="backupOnPathSwitch" @update:value="toggleBackupOnPathSwitch" />
    </div>
  </NCard>
</template>
