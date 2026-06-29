<script setup lang="ts">
import { useI18n } from "vue-i18n"
import { storeToRefs } from "pinia"
import { NSwitch, NRadioGroup, NRadio } from "naive-ui"
import { useDiscoverPrefStore } from "@/stores/useDiscoverPrefStore"
import SettingsSection from "./SettingsSection.vue"
import SettingsRow from "./SettingsRow.vue"
import TipIcon from "./TipIcon.vue"

const { t } = useI18n()
const discoverPrefStore = useDiscoverPrefStore()
const { discoverColumns, prefetchEnabled, showTranslateQuotaTip } = storeToRefs(discoverPrefStore)
const { setDiscoverColumns, setPrefetchEnabled, setShowTranslateQuotaTip } = discoverPrefStore
</script>

<template>
  <SettingsSection :title="t('settings.discover.title')">
    <SettingsRow :label="t('settings.discover.columnCount')">
      <NRadioGroup :value="discoverColumns" size="small" @update:value="setDiscoverColumns">
        <NRadio :value="1">1</NRadio>
        <NRadio :value="2">2</NRadio>
        <NRadio :value="3">3</NRadio>
        <NRadio :value="4">4</NRadio>
      </NRadioGroup>
    </SettingsRow>
    <div class="border-t border-c-default my-3"></div>
    <SettingsRow>
      <template #label>
        <TipIcon :label="t('settings.discover.prefetch')" :text="t('settings.discover.prefetchDesc')" placement="right" :width="240" />
      </template>
      <NSwitch :value="prefetchEnabled" @update:value="setPrefetchEnabled" />
    </SettingsRow>
    <div class="border-t border-c-default my-3"></div>
    <SettingsRow :label="t('settings.discover.translateQuotaTip')">
      <NSwitch :value="showTranslateQuotaTip" @update:value="setShowTranslateQuotaTip" />
    </SettingsRow>
  </SettingsSection>
</template>
