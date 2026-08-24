<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { NSelect, NRadioGroup, NRadio, NButton, NIcon, NSlider } from "naive-ui"
import { Image, RotateCcw } from "@lucide/vue"
import { setLocale } from "@/i18n"
import { displayMode, setDisplayMode, themeColorKey, setThemeColor, colorPalettes, type ThemeColorKey, type DisplayMode } from "@/theme"
import { useBackgroundStore } from "@/stores/useBackgroundStore"
import { storeToRefs } from "pinia"
import SettingsSection from "./SettingsSection.vue"
import SettingsRow from "./SettingsRow.vue"

const { t } = useI18n()

const bgStore = useBackgroundStore()
const { customBgUrl, customBgBlur, customBgDim, loading: bgLoading } = storeToRefs(bgStore)

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

    <!-- 自定义背景图片 -->
    <SettingsRow :label="t('settings.appearance.customBackground')">
      <div class="flex items-center gap-2.5">
        <!-- 缩略图预览 -->
        <div
          v-if="customBgUrl"
          class="w-10 h-7 rounded-md border border-white/20 shadow-sm flex-shrink-0 bg-cover bg-center"
          :style="{ backgroundImage: `url(${customBgUrl})` }"
        />
        <NButton
          size="small"
          :loading="bgLoading"
          secondary
          @click="bgStore.chooseCustomBackground"
        >
          <template #icon>
            <NIcon :size="15"><Image /></NIcon>
          </template>
          {{ customBgUrl ? t("settings.appearance.changeBackground") : t("settings.appearance.chooseBackground") }}
        </NButton>
        <NButton
          v-if="customBgUrl"
          size="small"
          quaternary
          type="error"
          :loading="bgLoading"
          @click="bgStore.removeCustomBackground"
        >
          <template #icon>
            <NIcon :size="15"><RotateCcw /></NIcon>
          </template>
          {{ t("settings.appearance.clearBackground") }}
        </NButton>
      </div>
    </SettingsRow>

    <!-- 背景模糊调节（仅在设置了自定义背景时生效） -->
    <SettingsRow
      v-if="customBgUrl"
      :label="t('settings.appearance.backgroundBlur')"
    >
      <div class="flex items-center gap-3 w-56">
        <NSlider
          v-model:value="customBgBlur"
          :min="0"
          :max="40"
          :step="1"
          size="small"
        />
        <span class="text-xs font-mono text-c-muted w-10 text-right">{{ customBgBlur }}px</span>
      </div>
    </SettingsRow>

    <!-- 背景遮罩暗度调节（仅在设置了自定义背景时生效） -->
    <SettingsRow
      v-if="customBgUrl"
      :label="t('settings.appearance.backgroundDim')"
    >
      <div class="flex items-center gap-3 w-56">
        <NSlider
          v-model:value="customBgDim"
          :min="0"
          :max="80"
          :step="5"
          size="small"
        />
        <span class="text-xs font-mono text-c-muted w-10 text-right">{{ customBgDim }}%</span>
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
