<script setup lang="ts">
import { useI18n } from "vue-i18n"
import { storeToRefs } from "pinia"
import { computed } from "vue"
import { FormRow, RadioGroup, Switch } from "@miuui/vue"
import { useDiscoverPrefStore } from "@/stores/useDiscoverPrefStore"
import SettingsSection from "./SettingsSection.vue"
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

const viewModeOptions = computed(() => [
  { value: "grid", label: t("settings.discover.viewModeGrid") },
  { value: "split", label: t("settings.discover.viewModeSplit") },
])

const columnOptions = [
  { value: 1, label: "1" },
  { value: 2, label: "2" },
  { value: 3, label: "3" },
  { value: 4, label: "4" },
]
</script>
<template>
  <SettingsSection :title="t('settings.discover.title')">
    <FormRow :label="t('settings.discover.viewMode')">
      <RadioGroup
        :model-value="discoverViewMode"
        :options="viewModeOptions"
        orientation="horizontal"
        size="sm"
        @update:model-value="(v: string | number) => setDiscoverViewMode(v as 'grid' | 'split')"
      />
    </FormRow>
    <FormRow v-if="discoverViewMode === 'grid'" :label="t('settings.discover.columnCount')">
      <RadioGroup
        :model-value="discoverColumns"
        :options="columnOptions"
        orientation="horizontal"
        size="sm"
        @update:model-value="(v: string | number) => setDiscoverColumns(Number(v) as 1 | 2 | 3 | 4)"
      />
    </FormRow>
    <FormRow>
      <template #lead>
        <FloatingTip
          :label="t('settings.discover.prefetch')"
          :text="t('settings.discover.prefetchDesc')"
          placement="right"
          :width="240"
        />
      </template>
      <Switch :model-value="prefetchEnabled" size="sm" @update:model-value="setPrefetchEnabled" />
    </FormRow>
    <FormRow>
      <template #lead>
        <FloatingTip
          :label="t('settings.discover.parseBBCode')"
          :text="t('settings.discover.parseBBCodeDesc')"
          placement="right"
          :width="280"
        />
      </template>
      <Switch
        :model-value="parseWorkshopBBCode"
        size="sm"
        @update:model-value="setParseWorkshopBBCode"
      />
    </FormRow>
    <FormRow :label="t('settings.discover.translateQuotaTip')">
      <Switch
        :model-value="showTranslateQuotaTip"
        size="sm"
        @update:model-value="setShowTranslateQuotaTip"
      />
    </FormRow>
  </SettingsSection>
</template>
