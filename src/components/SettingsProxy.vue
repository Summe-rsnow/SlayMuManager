<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { useMessage } from "naive-ui"
import { invoke } from "@tauri-apps/api/core"
import { NCard, NSpace, NInput, NButton, NIcon } from "naive-ui"
import { Globe } from "lucide-vue-next"
import type { AppBootstrap } from "../types"

const { t } = useI18n()
const message = useMessage()

const proxyUrl = ref("")
const proxyTesting = ref(false)

onMounted(async () => {
  const b = await invoke<AppBootstrap>("get_app_bootstrap")
  proxyUrl.value = b.proxyUrl ?? ""
})

async function saveProxy() {
  try {
    await invoke("update_proxy_url", { url: proxyUrl.value || null })
    message.success(t("settings.success.proxySaved"))
  } catch (e: unknown) {
    message.error(t("settings.error.saveFailed", { e }))
  }
}

async function testProxy() {
  if (!proxyUrl.value.trim()) {
    message.warning(t("settings.warning.proxyUrlRequired"))
    return
  }
  proxyTesting.value = true
  try {
    const ok = await invoke<boolean>("test_proxy", { url: proxyUrl.value.trim() })
    message[ok ? "success" : "error"](ok ? t("settings.success.proxyOk") : t("settings.error.proxyFail"))
  } catch (e: unknown) {
    message.error(t("settings.error.testFailed", { e }))
  } finally {
    proxyTesting.value = false
  }
}
</script>

<template>
  <NCard :title="t('settings.proxy.title')" size="small">
    <NSpace vertical>
      <div class="flex gap-2">
        <NInput v-model:value="proxyUrl" placeholder="http://127.0.0.1:7890" clearable>
          <template #prefix><NIcon :size="16"><Globe /></NIcon></template>
        </NInput>
        <NButton secondary @click="saveProxy">{{ t("common.save") }}</NButton>
      </div>
      <NSpace>
        <NButton secondary size="small" :loading="proxyTesting" @click="testProxy">
          {{ t("settings.proxy.testConnection") }}
        </NButton>
      </NSpace>
    </NSpace>
  </NCard>
</template>
