<script setup lang="ts">
import { computed } from "vue"
import {
  NConfigProvider,
  NMessageProvider,
  NNotificationProvider,
  NDialogProvider,
  NButton,
  NIcon,
  zhCN,
  dateZhCN,
  enUS,
  dateEnUS,
} from "naive-ui"
import { Minus, Square, X } from "lucide-vue-next"
import { currentLocale } from "./i18n"
import { naiveTheme, naiveThemeOverrides } from "./theme"
import { useWindow } from "./composables/useWindow"
import SideNav from "./components/SideNav.vue"

const naiveLocale = computed(() => (currentLocale.value === "zh-CN" ? zhCN : enUS))
const naiveDateLocale = computed(() => (currentLocale.value === "zh-CN" ? dateZhCN : dateEnUS))
const { minimize, toggleMaximize, close } = useWindow()
</script>

<template>
  <NConfigProvider
    :theme="naiveTheme"
    :theme-overrides="naiveThemeOverrides"
    :locale="naiveLocale"
    :date-locale="naiveDateLocale"
  >
    <NNotificationProvider>
      <NMessageProvider>
        <NDialogProvider>
          <!-- 根容器使用 CSS 变量实现动态主题 -->
          <div
            class="h-screen overflow-hidden relative"
            :style="{
              backgroundColor: 'var(--color-bg-primary)',
              color: 'var(--color-text-primary)',
            }"
          >
            <!-- 内容层（全高，标题栏叠加在上方） -->
            <div class="flex h-full pt-12">
              <SideNav />
              <main
                class="flex-1 overflow-auto px-12 pt-0 pb-6"
                :style="{
                  backgroundColor: 'var(--color-main-bg)',
                }"
              >
                <router-view />
              </main>
            </div>

            <!-- 沉浸标题栏（绝对定位叠加，毛玻璃效果） -->
            <div
              data-tauri-drag-region
              class="absolute top-0 left-0 right-0 h-12 z-30 flex items-center justify-between px-4 select-none"
              :style="{
                backdropFilter: 'blur(12px)',
                WebkitBackdropFilter: 'blur(12px)',
              }"
            >
              <div class="flex items-center gap-2.5">
                <svg viewBox="0 0 128 128" class="w-6 h-6 flex-shrink-0">
                  <defs>
                    <linearGradient id="titlelogo" x1="0%" y1="0%" x2="100%" y2="100%">
                      <stop offset="0%" stop-color="var(--primary-color)"/>
                      <stop offset="100%" style="stop-color: var(--primary-600)"/>
                    </linearGradient>
                  </defs>
                  <rect width="128" height="128" rx="28" fill="url(#titlelogo)"/>
                  <text x="64" y="84" font-family="system-ui,sans-serif" font-size="58" font-weight="800" fill="#fff" text-anchor="middle">S</text>
                </svg>
                <div class="flex items-baseline gap-1.5">
                  <span class="text-base font-semibold text-c-primary">SlayMuManager</span>
                  <span class="text-xs text-c-muted">v1.5.5</span>
                </div>
              </div>
              <div class="flex items-center -mr-2 h-full">
                <NButton text class="titlebar-btn h-full! w-11! rounded-none! text-c-muted" @click="minimize">
                  <template #icon><NIcon :size="13"><Minus /></NIcon></template>
                </NButton>
                <NButton text class="titlebar-btn h-full! w-11! rounded-none! text-c-muted" @click="toggleMaximize">
                  <template #icon><NIcon :size="13"><Square /></NIcon></template>
                </NButton>
                <NButton text class="titlebar-btn h-full! w-11! rounded-none! text-c-muted" @click="close">
                  <template #icon><NIcon :size="13"><X /></NIcon></template>
                </NButton>
              </div>
            </div>
          </div>
        </NDialogProvider>
      </NMessageProvider>
    </NNotificationProvider>
  </NConfigProvider>
</template>

<style scoped>
.titlebar-btn:hover {
  background-color: color-mix(in srgb, var(--color-text-primary) 10%, transparent) !important;
}
</style>
