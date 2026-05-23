<script setup lang="ts">
import { computed } from "vue"
import { useRouter } from "vue-router"
import { useI18n } from "vue-i18n"
import { NCard, NSelect } from "naive-ui"
import { useStorage } from "../composables/useStorage"

const { t, te } = useI18n()
const router = useRouter()

const defaultPage = useStorage<string>("slaymgr:default-page", "")

// 动态生成页面选项，自动适配新增路由
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
  <NCard :title="t('settings.defaultPage.title')" size="small">
    <div class="flex items-center justify-between">
      <span class="text-sm">{{ t("settings.defaultPage.description") }}</span>
      <NSelect
        :value="defaultPage"
        :options="pageOptions"
        size="small"
        style="width: 200px"
        @update:value="(v: string) => { defaultPage = v }"
      />
    </div>
  </NCard>
</template>
