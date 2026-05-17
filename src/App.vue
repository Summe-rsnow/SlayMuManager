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
            class="h-screen flex flex-col overflow-hidden"
            :style="{
              backgroundColor: 'var(--color-bg-primary)',
              color: 'var(--color-text-primary)',
            }"
          >
            <!-- 虚拟标题栏 -->
            <div
              data-tauri-drag-region
              class="flex-shrink-0 h-9 flex items-center justify-between px-3 select-none"
              :style="{
                backgroundColor: 'var(--color-titlebar-bg)',
                borderBottom: '1px solid var(--color-titlebar-border)',
              }"
            >
              <div class="flex items-center gap-2">
                <svg viewBox="0 0 128 128" class="w-4 h-4 flex-shrink-0">
                  <defs>
                    <linearGradient id="titlelogo" x1="0%" y1="0%" x2="100%" y2="100%">
                      <stop offset="0%" stop-color="var(--primary-color)"/>
                      <stop offset="100%" style="stop-color: var(--primary-600)"/>
                    </linearGradient>
                  </defs>
                  <rect width="128" height="128" rx="28" fill="url(#titlelogo)"/>
                  <text x="64" y="84" font-family="system-ui,sans-serif" font-size="58" font-weight="800" fill="#fff" text-anchor="middle">S</text>
                </svg>
                <span class="text-sm font-semibold" :style="{ color: 'var(--color-text-primary)' }">SlayMuManager</span>
                <span class="text-xs" :style="{ color: 'var(--color-text-muted)' }">v1.3.3</span>
              </div>
              <div class="flex items-center -mr-2 h-full">
                <NButton text class="h-full! w-11! rounded-none!" :style="{ color: 'var(--color-text-muted)' }" @click="minimize">
                  <template #icon><NIcon :size="13"><Minus /></NIcon></template>
                </NButton>
                <NButton text class="h-full! w-11! rounded-none!" :style="{ color: 'var(--color-text-muted)' }" @click="toggleMaximize">
                  <template #icon><NIcon :size="11"><Square /></NIcon></template>
                </NButton>
                <NButton text class="h-full! w-11! rounded-none! hover:bg-red-500!" :style="{ color: 'var(--color-text-muted)' }" @click="close">
                  <template #icon><NIcon :size="14"><X /></NIcon></template>
                </NButton>
              </div>
            </div>
            <div class="flex flex-1 overflow-hidden relative">
              <SideNav />
              <main
                class="flex-1 overflow-auto px-12 py-6"
                :style="{
                  backgroundColor: 'var(--color-main-bg)',
                }"
              >
                <router-view />
              </main>
            </div>
          </div>
        </NDialogProvider>
      </NMessageProvider>
    </NNotificationProvider>
  </NConfigProvider>
</template>
