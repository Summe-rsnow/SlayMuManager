<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useRouter } from "vue-router"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NCard, NSwitch, NSelect } from "naive-ui"
import { useStorage } from "../composables/useStorage"
import type { AppBootstrap } from "../types"

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
  <NCard :title="t('settings.launch.title')" size="small">
    <div class="flex items-center justify-between">
      <span class="text-sm">{{ t("settings.launch.defaultPageLabel") }}</span>
      <NSelect
        :value="defaultPage"
        :options="pageOptions"
        size="small"
        style="width: 200px"
        @update:value="(v: string) => { defaultPage = v }"
      />
    </div>
    <div class="border-t border-c-default my-3"></div>
    <div class="flex items-center justify-between">
      <span class="text-sm">{{ t("settings.launch.autoCheck") }}</span>
      <NSwitch :value="autoCheckUpdate" size="small" @update:value="toggleAutoCheck" />
    </div>
  </NCard>
</template>
