<script setup lang="ts">
import { useI18n } from "vue-i18n"
import { NCard, NSwitch, NRadioGroup, NRadio, NPopover, NIcon } from "naive-ui"
import { HelpCircle } from "@lucide/vue"
import { discoverColumns, setDiscoverColumns } from "../composables/useDiscoverColumns"
import { prefetchEnabled, setPrefetchEnabled } from "../composables/usePageCache"
import { showTranslateQuotaTip, setShowTranslateQuotaTip } from "../composables/useTranslation"

const { t } = useI18n()
</script>

<template>
  <NCard :title="t('settings.discover.title')" size="small">
    <div class="flex items-center justify-between">
      <span class="text-sm">{{ t("settings.discover.columnCount") }}</span>
      <NRadioGroup :value="discoverColumns" size="small" @update:value="setDiscoverColumns">
        <NRadio :value="1">1</NRadio>
        <NRadio :value="2">2</NRadio>
        <NRadio :value="3">3</NRadio>
        <NRadio :value="4">4</NRadio>
      </NRadioGroup>
    </div>
    <div class="border-t border-c-default my-3"></div>
    <div class="flex items-center justify-between">
      <NPopover trigger="hover" placement="right" :width="240">
        <template #trigger>
          <span class="flex items-center gap-1 cursor-help text-sm text-c-primary" :style="{ lineHeight: '1' }">
            <span>{{ t("settings.discover.prefetch") }}</span>
            <NIcon :size="14" class="text-c-muted"><HelpCircle /></NIcon>
          </span>
        </template>
        <span class="text-xs">{{ t("settings.discover.prefetchDesc") }}</span>
      </NPopover>
      <NSwitch :value="prefetchEnabled" @update:value="setPrefetchEnabled" />
    </div>
    <div class="border-t border-c-default my-3"></div>
    <div class="flex items-center justify-between">
      <span class="text-sm">{{ t("settings.discover.translateQuotaTip") }}</span>
      <NSwitch :value="showTranslateQuotaTip" @update:value="setShowTranslateQuotaTip" />
    </div>
  </NCard>
</template>
