<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { Input, Button, useToast } from "@xcmm1543/miuui-vue"
import { invoke } from "@tauri-apps/api/core"
import { Globe } from "@lucide/vue"
import type { AppBootstrap } from "../types"
import SettingsSection from "./SettingsSection.vue"

const { t } = useI18n()
const { toast } = useToast()

const proxyUrl = ref("")
const proxyTesting = ref(false)

onMounted(async () => {
  const b = await invoke<AppBootstrap>("get_app_bootstrap")
  proxyUrl.value = b.proxyUrl ?? ""
})

async function saveProxy() {
  try {
    await invoke("update_proxy_url", { url: proxyUrl.value || null })
    toast({ title: t("settings.success.proxySaved"), variant: "success" })
  } catch (e: unknown) {
    toast({ title: t("settings.error.saveFailed", { e }), variant: "error" })
  }
}

async function testProxy() {
  if (!proxyUrl.value.trim()) {
    toast({ title: t("settings.warning.proxyUrlRequired"), variant: "warning" })
    return
  }
  proxyTesting.value = true
  try {
    const ok = await invoke<boolean>("test_proxy", { url: proxyUrl.value.trim() })
    toast({
      title: ok ? t("settings.success.proxyOk") : t("settings.error.proxyFail"),
      variant: ok ? "success" : "error",
    })
  } catch (e: unknown) {
    toast({ title: t("settings.error.testFailed", { e }), variant: "error" })
  } finally {
  }
}
</script>

<template>
  <SettingsSection :title="t('settings.proxy.title')">
    <div class="flex gap-2">
      <Input v-model="proxyUrl" placeholder="http://127.0.0.1:7890" clearable class="flex-1">
        <template #prefix><Globe :size="16" /></template>
      </Input>
      <Button variant="secondary" @click="saveProxy">{{ t("common.save") }}</Button>
    </div>
    <div class="flex items-center gap-2">
      <Button variant="secondary" size="sm" :loading="proxyTesting" @click="testProxy">
        {{ t("settings.proxy.testConnection") }}
      </Button>
    </div>
  </SettingsSection>
</template>
