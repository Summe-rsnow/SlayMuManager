<script setup lang="ts">
import { useI18n } from "vue-i18n"
import { storeToRefs } from "pinia"
import { NSwitch, NRadioGroup, NRadio } from "naive-ui"
import { useDiscoverPrefStore } from "@/stores/useDiscoverPrefStore"
import SettingsSection from "./SettingsSection.vue"
import SettingsRow from "./SettingsRow.vue"
import FloatingTip from "./FloatingTip.vue"

const { t } = useI18n()
const discoverPrefStore = useDiscoverPrefStore()
const {
  discoverColumns,
  prefetchEnabled,
  showTranslateQuotaTip,
  discoverViewMode,
  parseWorkshopBBCode,
} = storeToRefs(discoverPrefStore)
const {
  setDiscoverColumns,
  setPrefetchEnabled,
  setShowTranslateQuotaTip,
  setDiscoverViewMode,
  setParseWorkshopBBCode,
} = discoverPrefStore
</script>

<template>
  <SettingsSection :title="t('settings.discover.title')">
    <SettingsRow :label="t('settings.discover.viewMode')">
      <NRadioGroup :value="discoverViewMode" size="small" @update:value="setDiscoverViewMode">
        <NRadio value="grid">{{ t("settings.discover.viewModeGrid") }}</NRadio>
        <NRadio value="split">{{ t("settings.discover.viewModeSplit") }}</NRadio>
      </NRadioGroup>
    </SettingsRow>
    <SettingsRow v-if="discoverViewMode === 'grid'" :label="t('settings.discover.columnCount')">
      <NRadioGroup :value="discoverColumns" size="small" @update:value="setDiscoverColumns">
        <NRadio :value="1">1</NRadio>
        <NRadio :value="2">2</NRadio>
        <NRadio :value="3">3</NRadio>
        <NRadio :value="4">4</NRadio>
      </NRadioGroup>
    </SettingsRow>
    <SettingsRow>
      <template #label>
        <FloatingTip
          :label="t('settings.discover.prefetch')"
          :text="t('settings.discover.prefetchDesc')"
          placement="right"
          :width="240"
        />
      </template>
      <NSwitch :value="prefetchEnabled" @update:value="setPrefetchEnabled" />
    </SettingsRow>
    <SettingsRow>
      <template #label>
        <FloatingTip
          :label="t('settings.discover.parseBBCode')"
          :text="t('settings.discover.parseBBCodeDesc')"
          placement="right"
          :width="280"
        />
      </template>
      <NSwitch :value="parseWorkshopBBCode" @update:value="setParseWorkshopBBCode" />
    </SettingsRow>
    <SettingsRow :label="t('settings.discover.translateQuotaTip')">
      <NSwitch :value="showTranslateQuotaTip" @update:value="setShowTranslateQuotaTip" />
    </SettingsRow>
  </SettingsSection>
</template>
