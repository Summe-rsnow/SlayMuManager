<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { Button, FormRow, RadioGroup, Select, Slider, useToast } from "@xcmm1543/miuui-vue"
import { Image, RotateCcw, Crop } from "@lucide/vue"
import { setLocale } from "@/i18n"
import {
  displayMode,
  setDisplayMode,
  themeColorKey,
  setThemeColor,
  colorPalettes,
  type ThemeColorKey,
  type DisplayMode,
} from "@/theme"
import { useBackgroundStore } from "@/stores/useBackgroundStore"
import { storeToRefs } from "pinia"
import SettingsSection from "./SettingsSection.vue"
import WallpaperCropModal from "./WallpaperCropModal.vue"

const { t } = useI18n()
const { toast } = useToast()

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

async function updateLocale(val: string | number | undefined) {
  if (val == null) return
  const next = String(val)
  locale.value = next
  setLocale(next)
  try {
    await invoke("update_app_locale", { locale: next })
  } catch {
    /* ignore */
  }
}

async function handleDisplayModeChange(val: string | number) {
  const mode = val as DisplayMode
  setDisplayMode(mode)
  try {
    await invoke("update_theme_mode", { mode })
  } catch {
    /* ignore */
  }
}

async function handleThemeColorChange(val: ThemeColorKey) {
  setThemeColor(val)
  try {
    await invoke("update_theme_color", { color: val })
  } catch {
    /* ignore */
  }
}

async function handleCropApply(payload: { croppedUrl: string; originalUrl: string }) {
  await bgStore.applyCroppedWallpaper(payload.croppedUrl, payload.originalUrl)
  toast.success(t("settings.appearance.cropSuccess"))
}
</script>

<template>
  <SettingsSection :title="t('settings.appearance.title')">
    <FormRow :label="t('settings.appearance.language')">
      <Select
        :options="languageOptions"
        :model-value="locale"
        class="w-40"
        size="sm"
        @update:model-value="updateLocale"
      />
    </FormRow>

    <FormRow :label="t('settings.appearance.displayMode')">
      <RadioGroup
        :model-value="displayMode"
        :options="displayModeOptions"
        orientation="horizontal"
        name="displayMode"
        size="sm"
        @update:model-value="handleDisplayModeChange"
      />
    </FormRow>

    <FormRow :label="t('settings.appearance.themeColor')">
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
    </FormRow>

    <!-- 自定义背景图片 -->
    <FormRow :label="t('settings.appearance.customBackground')">
      <div class="flex items-center gap-2.5">
        <!-- 缩略图预览（点击直接打开裁切） -->
        <div
          v-if="customBgUrl"
          class="w-10 h-7 rounded-md border border-white/20 shadow-sm flex-shrink-0 bg-cover bg-center cursor-pointer hover:opacity-80 transition-opacity"
          :style="{ backgroundImage: `url(${customBgUrl})` }"
          :title="t('settings.appearance.cropBackground')"
          @click="bgStore.reopenCurrentCrop"
        />
        <!-- 裁切调整按钮（仅在已设置壁纸时出现） -->
        <Button
          v-if="customBgUrl"
          size="sm"
          variant="secondary"
          :loading="bgLoading"
          @click="bgStore.reopenCurrentCrop"
        >
          <template #icon><Crop :size="15" /></template>
          {{ t("settings.appearance.cropBackground") }}
        </Button>
        <Button
          size="sm"
          :loading="bgLoading"
          variant="secondary"
          @click="bgStore.chooseCustomBackground"
        >
          <template #icon><Image :size="15" /></template>
          {{
            customBgUrl
              ? t("settings.appearance.changeBackground")
              : t("settings.appearance.chooseBackground")
          }}
        </Button>
        <Button
          v-if="customBgUrl"
          size="sm"
          variant="danger"
          :loading="bgLoading"
          @click="bgStore.removeCustomBackground"
        >
          <template #icon><RotateCcw :size="15" /></template>
          {{ t("settings.appearance.clearBackground") }}
        </Button>
      </div>
    </FormRow>

    <!-- 背景模糊调节（仅在设置了自定义背景时生效） -->
    <FormRow v-if="customBgUrl" :label="t('settings.appearance.backgroundBlur')">
      <div class="flex items-center gap-3 w-56">
        <Slider v-model="customBgBlur" :min="0" :max="40" :step="1" size="sm" aria-label="Blur" />
        <span class="text-xs font-mono text-c-muted w-10 text-right">{{ customBgBlur }}px</span>
      </div>
    </FormRow>

    <!-- 背景遮罩暗度调节（仅在设置了自定义背景时生效） -->
    <FormRow v-if="customBgUrl" :label="t('settings.appearance.backgroundDim')">
      <div class="flex items-center gap-3 w-56">
        <Slider v-model="customBgDim" :min="0" :max="80" :step="5" size="sm" aria-label="Dim" />
        <span class="text-xs font-mono text-c-muted w-10 text-right">{{ customBgDim }}%</span>
      </div>
    </FormRow>
  </SettingsSection>

  <!-- 壁纸裁切调整弹窗 -->
  <WallpaperCropModal
    v-model:show="bgStore.showCropModal"
    :image-url="bgStore.cropSourceUrl"
    @apply="handleCropApply"
  />
</template>

<style scoped>
.color-btn {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition:
    transform 0.15s ease,
    border-color 0.15s ease;
}
.color-btn:hover {
  transform: scale(1.15);
}
.color-btn.active {
  border-color: #fff;
  transform: scale(1.15);
  box-shadow: 0 0 0 2px var(--primary-color);
}
</style>
