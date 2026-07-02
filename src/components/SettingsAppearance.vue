<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NSelect, NRadioGroup, NRadio } from "naive-ui"
import { setLocale } from "@/i18n"
import { displayMode, setDisplayMode, themeColorKey, setThemeColor, colorPalettes, type ThemeColorKey, type DisplayMode } from "@/theme"
import SettingsSection from "./SettingsSection.vue"
import SettingsRow from "./SettingsRow.vue"

const { t } = useI18n()

const locale = ref("zh-CN")

onMounted(async () => {
  const b = await invoke<{ locale: string }>("get_app_bootstrap")
  locale.value = b.locale
})

const languageOptions = computed(() => [
  { label: t("settings.language.zhCN"), value: "zh-CN" },
  { label: t("settings.language.en"), value: "en" },
])

const displayModeOptions = computed<{ label: string; value: DisplayMode }[]>(() => [
  { label: t("settings.appearance.displayModeSystem"), value: "system" },
  { label: t("settings.appearance.displayModeLight"), value: "light" },
  { label: t("settings.appearance.displayModeDark"), value: "dark" },
])

const themeColorOptions = computed<{ label: string; value: ThemeColorKey }[]>(() => [
  { label: t("settings.appearance.themeColors.indigo"), value: "indigo" },
  { label: t("settings.appearance.themeColors.blue"), value: "blue" },
  { label: t("settings.appearance.themeColors.green"), value: "green" },
  { label: t("settings.appearance.themeColors.purple"), value: "purple" },
  { label: t("settings.appearance.themeColors.rose"), value: "rose" },
  { label: t("settings.appearance.themeColors.orange"), value: "orange" },
  { label: t("settings.appearance.themeColors.cyan"), value: "cyan" },
  { label: t("settings.appearance.themeColors.pink"), value: "pink" },
  { label: t("settings.appearance.themeColors.yellow"), value: "yellow" },
])

async function updateLocale(val: string) {
  locale.value = val
  setLocale(val)
  try { await invoke("update_app_locale", { locale: val }) } catch { /* ignore */ }
}

async function handleDisplayModeChange(val: DisplayMode) {
  setDisplayMode(val)
  try { await invoke("update_theme_mode", { mode: val }) } catch { /* ignore */ }
}

async function handleThemeColorChange(val: ThemeColorKey) {
  setThemeColor(val)
  try { await invoke("update_theme_color", { color: val }) } catch { /* ignore */ }
}
</script>

<template>
  <SettingsSection :title="t('settings.appearance.title')">
    <SettingsRow :label="t('settings.appearance.language')">
      <NSelect
        :options="languageOptions"
        :value="locale"
        style="width: 160px"
        size="small"
        @update:value="updateLocale"
      />
    </SettingsRow>
    <SettingsRow :label="t('settings.appearance.displayMode')">
      <NRadioGroup :value="displayMode" size="small" @update:value="handleDisplayModeChange">
        <NRadio v-for="opt in displayModeOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </NRadio>
      </NRadioGroup>
    </SettingsRow>
    <SettingsRow :label="t('settings.appearance.themeColor')">
      <div class="flex gap-1.5">
        <button
          v-for="opt in themeColorOptions"
          :key="opt.value"
          :title="opt.label"
          class="color-btn"
          :class="{ active: themeColorKey === opt.value }"
          :style="[{ backgroundColor: colorPalettes[opt.value].DEFAULT }]"
          @click="handleThemeColorChange(opt.value)"
        />
      </div>
    </SettingsRow>
  </SettingsSection>
</template>

<style scoped>
.color-btn {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
}
.color-btn:hover { transform: scale(1.1); }
.color-btn.active {
  border-color: white;
  box-shadow: 0 0 0 1.5px var(--primary-color);
  transform: scale(1.15);
}
</style>
