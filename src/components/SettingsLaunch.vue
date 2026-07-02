<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useRouter } from "vue-router"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NSwitch, NSelect } from "naive-ui"
import { useStorage } from "@/composables/useStorage"
import type { AppBootstrap } from "../types"
import SettingsSection from "./SettingsSection.vue"
import SettingsRow from "./SettingsRow.vue"

const { t, te } = useI18n()
const router = useRouter()

const autoCheckUpdate = ref(true)
const defaultPage = useStorage<string>("slaymgr:default-page", "")

onMounted(async () => {
  const b = await invoke<AppBootstrap>("get_app_bootstrap")
  autoCheckUpdate.value = b.autoCheckUpdate
  if (!defaultPage.value && pageOptions.value.length > 0) {
    defaultPage.value = pageOptions.value[0].value
  }
})

async function toggleAutoCheck(val: boolean) {
  autoCheckUpdate.value = val
  try { await invoke("update_auto_check_update", { enabled: val }) } catch { /* ignore */ }
}

const pageOptions = computed(() => {
  const options: Array<{ label: string; value: string }> = []
  for (const route of router.getRoutes()) {
    if (route.name && typeof route.name === "string") {
      const i18nKey = `nav.${route.name}`
      const label = te(i18nKey) ? t(i18nKey) : route.name
      options.push({ label, value: route.path })
    }
  }
  return options
})
</script>

<template>
  <SettingsSection :title="t('settings.launch.title')">
    <SettingsRow :label="t('settings.launch.defaultPageLabel')">
      <NSelect
        :value="defaultPage"
        :options="pageOptions"
        size="small"
        style="width: 200px"
        @update:value="(v: string) => { defaultPage = v }"
      />
    </SettingsRow>
    <SettingsRow :label="t('settings.launch.autoCheck')">
      <NSwitch :value="autoCheckUpdate" size="small" @update:value="toggleAutoCheck" />
    </SettingsRow>
  </SettingsSection>
</template>
